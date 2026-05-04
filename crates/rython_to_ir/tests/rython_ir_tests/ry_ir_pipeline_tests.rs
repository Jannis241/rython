use std::panic::{self, AssertUnwindSafe};

use rython_to_ir::codegen::{generate_code, ConstValue, IrInstruction, IrType, Terminator};
use rython_to_ir::lexer::{Lexer, TokenKind};
use rython_to_ir::parser::Parser;

fn parse_source(source: &str) -> Vec<rython_to_ir::ast::Item> {
    let tokens = Lexer::create_tokens(source.to_string());
    let mut parser = Parser::new(tokens);
    parser.parse().unwrap()
}

#[test]
fn supported_source_program_generates_expected_ir_shape() {
    let items = parse_source(
        r#"
        global counter: int = 1;
        const enabled: bool = true;
        fn answer() int { return 42; }
        fn empty() { return; }
        "#,
    );

    let module = generate_code(&items);

    assert_eq!(module.globals.len(), 1);
    assert_eq!(module.globals[0].name, "counter");
    assert!(matches!(module.globals[0].ty, IrType::I64));
    assert!(matches!(module.globals[0].value, ConstValue::Int(1)));

    assert_eq!(module.constants.len(), 1);
    assert_eq!(module.constants[0].name, "enabled");
    assert!(matches!(module.constants[0].ty, IrType::Bool));
    assert!(matches!(module.constants[0].value, ConstValue::Bool(true)));

    assert_eq!(module.functions.len(), 2);
    assert_eq!(module.functions[0].name, "answer");
    assert!(matches!(module.functions[0].return_type, IrType::I64));
    assert_eq!(module.functions[0].blocks.len(), 1);
    assert_eq!(module.functions[0].blocks[0].instructions.len(), 1);
    assert!(matches!(
        module.functions[0].blocks[0].instructions[0],
        IrInstruction::Const {
            ty: IrType::I64,
            value: ConstValue::Int(42),
            ..
        }
    ));
    assert!(matches!(
        module.functions[0].blocks[0].terminator,
        Terminator::Ret(Some(_))
    ));

    assert_eq!(module.functions[1].name, "empty");
    assert!(matches!(module.functions[1].return_type, IrType::Void));
    assert!(matches!(
        module.functions[1].blocks[0].terminator,
        Terminator::Ret(None)
    ));
}

#[test]
fn parser_rejects_trailing_garbage_after_function() {
    let tokens = Lexer::create_tokens("fn main() { return; } 123".to_string());
    let mut parser = Parser::new(tokens);

    assert!(parser.parse().is_err());
}

#[test]
fn lexer_always_appends_exactly_one_eof_for_valid_ascii_starts() {
    let samples = [
        "",
        " \n\t\r",
        "abc",
        "abc_123",
        "123",
        "1.25",
        "\"text\"",
        "'x'",
        "+ - * / % = == != < <= << > >= >> & | ^ ~",
        "( ) { } [ ] , ; : .",
        "// comment without newline",
        "// comment\nfn",
    ];

    for sample in samples {
        let tokens = Lexer::create_tokens(sample.to_string());
        assert_eq!(tokens.last().unwrap().kind, TokenKind::Eof, "sample: {sample:?}");
        assert_eq!(
            tokens
                .iter()
                .filter(|token| token.kind == TokenKind::Eof)
                .count(),
            1,
            "sample: {sample:?}"
        );
    }
}

#[test]
fn every_single_ascii_byte_either_lexes_to_eof_or_panics_without_looping() {
    for byte in 0_u8..=127 {
        let input = char::from(byte).to_string();
        let result = panic::catch_unwind(AssertUnwindSafe(|| Lexer::create_tokens(input)));

        if let Ok(tokens) = result {
            assert_eq!(tokens.last().unwrap().kind, TokenKind::Eof, "byte: {byte}");
        }
    }
}

#[test]
fn codegen_panics_for_parsed_but_unsupported_control_flow() {
    let items = parse_source("fn main() int { if true { return 1; } return 0; }");

    assert!(panic::catch_unwind(AssertUnwindSafe(|| generate_code(&items))).is_err());
}
