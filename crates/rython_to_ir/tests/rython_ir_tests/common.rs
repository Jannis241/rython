#![allow(dead_code)]

use std::panic::{self, AssertUnwindSafe};

use rython_to_ir::ast::Item;
use rython_to_ir::codegen::{generate_code, CodegenError, IrInstruction, IrModule, Terminator};
use rython_to_ir::lexer::{Lexer, Token, TokenKind};
use rython_to_ir::parser::{ParseError, Parser};

pub fn lex(source: &str) -> Result<Vec<Token>, rython_to_ir::lexer::LexingError> {
    Lexer::create_tokens(source.to_string())
}

pub fn token_pairs(source: &str) -> Vec<(TokenKind, String)> {
    lex(source)
        .expect("lexing failed")
        .into_iter()
        .map(|token| (token.kind, token.value))
        .collect()
}

pub fn assert_tokens(source: &str, expected: &[(TokenKind, &str)]) {
    let actual = token_pairs(source);
    let expected: Vec<(TokenKind, String)> = expected
        .iter()
        .map(|(kind, value)| (kind.clone(), (*value).to_string()))
        .collect();
    assert_eq!(actual, expected, "source: {source:?}");
}

pub fn parse_items(source: &str) -> Result<Vec<Item>, ParseError> {
    let tokens = lex(source).expect("lexing failed");
    Parser::new(tokens).parse()
}

pub fn parse_items_no_panic(source: &str) -> Result<Result<Vec<Item>, ParseError>, Box<dyn std::any::Any + Send>> {
    panic::catch_unwind(AssertUnwindSafe(|| parse_items(source)))
}

pub fn compile(source: &str) -> Result<IrModule, CodegenError> {
    let items = parse_items(source).expect("parsing failed");
    generate_code(&items)
}

pub fn compile_ok(source: &str) -> IrModule {
    match compile(source) {
        Ok(module) => module,
        Err(err) => panic!("expected IR generation to succeed, got {err:?}"),
    }
}

pub fn compile_err(source: &str) -> CodegenError {
    match compile(source) {
        Ok(module) => panic!("expected IR generation to fail, got {module:#?}"),
        Err(err) => err,
    }
}

pub fn function<'a>(module: &'a IrModule, name: &str) -> &'a rython_to_ir::ir::IrFunction {
    module
        .functions
        .iter()
        .find(|function| function.name == name)
        .unwrap_or_else(|| panic!("missing function {name}; module functions: {:#?}", module.functions))
}

pub fn all_instructions(module: &IrModule) -> Vec<&IrInstruction> {
    module
        .functions
        .iter()
        .flat_map(|function| function.blocks.iter())
        .flat_map(|block| block.instructions.iter())
        .collect()
}

pub fn assert_all_blocks_terminated(module: &IrModule) {
    for function in &module.functions {
        for block in &function.blocks {
            match &block.terminator {
                Terminator::Ret(_) | Terminator::Jump { .. } | Terminator::Branch { .. } => {}
            }
        }
    }
}

pub fn assert_branches_target_existing_blocks(module: &IrModule) {
    for function in &module.functions {
        let labels: Vec<&str> = function
            .blocks
            .iter()
            .map(|block| block.label.trim_end_matches(':'))
            .collect();

        for block in &function.blocks {
            match &block.terminator {
                Terminator::Jump { target } => {
                    assert!(labels.contains(&target.as_str()), "unknown jump target {target:?}");
                }
                Terminator::Branch {
                    then_block,
                    else_block,
                    ..
                } => {
                    assert!(
                        labels.contains(&then_block.as_str()),
                        "unknown then target {then_block:?}"
                    );
                    assert!(
                        labels.contains(&else_block.as_str()),
                        "unknown else target {else_block:?}"
                    );
                }
                Terminator::Ret(_) => {}
            }
        }
    }
}

pub fn assert_no_duplicate_names(names: &[String]) {
    let mut seen = std::collections::HashSet::new();
    for name in names {
        assert!(seen.insert(name), "duplicate name {name:?} in {names:?}");
    }
}
