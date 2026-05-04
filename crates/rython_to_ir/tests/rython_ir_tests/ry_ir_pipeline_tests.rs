use std::panic::{self, AssertUnwindSafe};

use rython_to_ir::codegen::{
    generate_code, CodegenError, ConstValue, IrInstruction, IrModule, IrType, Terminator,
};
use rython_to_ir::lexer::{Lexer, TokenKind};
use rython_to_ir::parser::Parser;

fn parse_source(source: &str) -> Vec<rython_to_ir::ast::Item> {
    let tokens = Lexer::create_tokens(source.to_string()).expect("lexing failed");
    let mut parser = Parser::new(tokens);
    parser.parse().unwrap()
}

fn unwrap_codegen(result: Result<IrModule, CodegenError>) -> IrModule {
    match result {
        Ok(module) => module,
        Err(_) => panic!("codegen failed"),
    }
}

#[test]
fn supported_function_source_program_generates_expected_ir_shape() {
    let items = parse_source(
        r#"
        fn answer() int { return 42; }
        fn empty() { return; }
        "#,
    );

    let module = unwrap_codegen(generate_code(&items));

    assert!(module.globals.is_empty());
    assert!(module.constants.is_empty());

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
fn top_level_global_and_const_parse_but_codegen_returns_error() {
    let items = parse_source(
        r#"
        global counter: int = 1;
        const enabled: bool = true;
        "#,
    );

    assert!(generate_code(&items).is_err());
}

#[test]
fn parser_rejects_trailing_garbage_after_function() {
    let tokens = Lexer::create_tokens("fn main() { return; } 123".to_string()).expect("lexing failed");
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
        let tokens = Lexer::create_tokens(sample.to_string()).expect("lexing failed");
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

        if let Ok(Ok(tokens)) = result {
            assert_eq!(tokens.last().unwrap().kind, TokenKind::Eof, "byte: {byte}");
        }
    }
}

#[test]
fn codegen_panics_for_parsed_but_unsupported_control_flow() {
    let items = parse_source("fn main() int { if true { return 1; } return 0; }");

    assert!(generate_code(&items).is_err());
}
