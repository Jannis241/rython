#[path = "rython_ir_tests/common.rs"]
mod common;

#[path = "rython_ir_tests/lexer_semantics_tests.rs"]
mod lexer_semantics_tests;

#[path = "rython_ir_tests/parser_ast_semantics_tests.rs"]
mod parser_ast_semantics_tests;

#[path = "rython_ir_tests/ir_codegen_semantics_tests.rs"]
mod ir_codegen_semantics_tests;

#[path = "rython_ir_tests/pipeline_regression_tests.rs"]
mod pipeline_regression_tests;
