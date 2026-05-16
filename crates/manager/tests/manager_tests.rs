use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use manager::claude_print_ir::format_ir;
use manager::run::{run, BuildError, BuildOptions};
use rython_to_ir::ir::{
    IrBlock, IrField, IrFunction, IrInstruction, IrModule, IrType, PrimitiveValue, TempId,
    Terminator,
};

fn temp_path(name: &str, extension: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("rython_manager_test_{name}_{nanos}.{extension}"))
}

fn temp_source(name: &str, contents: &str) -> PathBuf {
    let path = temp_path(name, "ry");
    fs::write(&path, contents).unwrap();
    path
}

#[test]
fn build_options_default_matches_source_to_ir_scope() {
    let options = BuildOptions::default();

    assert!(!options.keep_intermediates);
    assert!(!options.release);
    assert!(options.run_after_build);
    assert!(options.output_path.is_none());
    assert!(!options.emit_tokens);
    assert!(!options.emit_ast);
    assert!(!options.emit_ir);
    assert!(!options.emit_asm);
}

#[test]
fn read_file_returns_exact_file_contents() {
    let path = temp_source("exact_contents", "fn main() int {\n    return 42;\n}\n");

    let actual = manager::read_file::read_file(path.to_str().unwrap()).unwrap();

    assert_eq!(actual, "fn main() int {\n    return 42;\n}\n");
    fs::remove_file(path).unwrap();
}

#[test]
fn read_file_reports_missing_files() {
    let path = temp_path("missing", "ry");

    let err = manager::read_file::read_file(path.to_str().unwrap()).unwrap_err();

    assert_eq!(err.kind(), ErrorKind::NotFound);
}

#[test]
fn run_accepts_valid_rython_source_and_returns_zero_without_backend_execution() {
    let path = temp_source("valid", "fn main() int { return 0; }\n");

    let code = run(path.to_str().unwrap(), &BuildOptions::default()).unwrap();

    assert_eq!(code, 0);
    fs::remove_file(path).unwrap();
}

#[test]
fn run_rejects_non_ry_extension_before_reading() {
    let path = temp_path("wrong_extension", "txt");
    fs::write(&path, "fn main() int { return 0; }\n").unwrap();

    let err = run(path.to_str().unwrap(), &BuildOptions::default()).unwrap_err();

    assert!(matches!(err, BuildError::InvalidExtension { .. }));
    fs::remove_file(path).unwrap();
}

#[test]
fn run_wraps_missing_files_as_read_errors() {
    let path = temp_path("missing", "ry");

    let err = run(path.to_str().unwrap(), &BuildOptions::default()).unwrap_err();

    assert!(matches!(
        err,
        BuildError::Read {
            source,
            ..
        } if source.kind() == ErrorKind::NotFound
    ));
}

#[test]
fn run_reports_lexer_parser_and_ir_codegen_errors_by_phase() {
    let lex = temp_source("lex_error", "fn main() { @ }\n");
    let parse = temp_source("parse_error", "fn main( { return; }\n");
    let ir = temp_source("ir_error", "fn main() int { return true; }\n");

    assert!(matches!(
        run(lex.to_str().unwrap(), &BuildOptions::default()).unwrap_err(),
        BuildError::Lex(_)
    ));
    assert!(matches!(
        run(parse.to_str().unwrap(), &BuildOptions::default()).unwrap_err(),
        BuildError::Parse(_)
    ));
    assert!(matches!(
        run(ir.to_str().unwrap(), &BuildOptions::default()).unwrap_err(),
        BuildError::IrCodegen(_)
    ));

    fs::remove_file(lex).unwrap();
    fs::remove_file(parse).unwrap();
    fs::remove_file(ir).unwrap();
}

#[test]
fn output_path_no_run_and_emit_asm_are_accepted_but_do_not_require_backend_artifacts() {
    let path = temp_source("backend_options", "fn main() int { return 0; }\n");
    let output = temp_path("ignored_output", "bin");
    let options = BuildOptions {
        run_after_build: false,
        output_path: Some(output.clone()),
        emit_asm: true,
        keep_intermediates: true,
        release: true,
        ..BuildOptions::default()
    };

    let code = run(path.to_str().unwrap(), &options).unwrap();

    assert_eq!(code, 0);
    assert!(
        !output.exists(),
        "Rython->IR manager run must not assert backend binary creation"
    );
    fs::remove_file(path).unwrap();
}

#[test]
fn format_ir_is_deterministic_for_empty_modules() {
    let output = format_ir(&IrModule::new());

    assert!(output.contains("==== IR Module ===="));
    assert!(output.contains("-- types --\n(none)"));
    assert!(output.contains("-- functions --\n(none)"));
}

#[test]
fn format_ir_prints_structured_function_instruction_and_terminator_data() {
    let mut module = IrModule::new();
    module.functions.push(IrFunction {
        name: "main".to_string(),
        parameter: vec![IrField {
            name: "argc".to_string(),
            ty: IrType::I64,
        }],
        return_type: IrType::I64,
        blocks: vec![IrBlock {
            label: "entry:".to_string(),
            instructions: vec![IrInstruction::PrimitiveConst {
                temp_id: TempId(0),
                ty: IrType::I64,
                value: PrimitiveValue::Int(42),
            }],
            terminator: Terminator::Ret(Some(TempId(0))),
        }],
    });

    let output = format_ir(&module);

    assert!(output.contains("IrFunction { name: \"main\""));
    assert!(output.contains("IrField { name: \"argc\", ty: I64 }"));
    assert!(output.contains("PrimitiveConst { temp_id: %0, ty: I64, value: Int(42) }"));
    assert!(output.contains("terminator: Ret(Some(%0))"));
}
