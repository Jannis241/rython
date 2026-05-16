use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rython_cli"))
}

fn temp_path(name: &str, extension: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("rython_cli_test_{name}_{nanos}.{extension}"))
}

fn temp_source(name: &str, contents: &str) -> PathBuf {
    let path = temp_path(name, "ry");
    fs::write(&path, contents).unwrap();
    path
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

#[test]
fn no_args_and_help_print_usage_successfully() {
    for args in [Vec::<&str>::new(), vec!["--help"], vec!["-h"]] {
        let output = cli().args(args).output().unwrap();

        assert!(output.status.success());
        let stdout = stdout(&output);
        assert!(stdout.contains("usage: rython_cli [OPTIONS] <your_program.ry>"));
        assert!(stdout.contains("--emit-tokens"));
        assert!(stdout.contains("--no-run"));
    }
}

#[test]
fn unknown_flag_missing_output_arg_and_multiple_inputs_are_usage_errors() {
    let unknown = cli().arg("--definitely-not-a-flag").output().unwrap();
    assert_eq!(unknown.status.code(), Some(2));
    assert!(stderr(&unknown).contains("unknown option"));

    let missing_output = cli().arg("-o").output().unwrap();
    assert_eq!(missing_output.status.code(), Some(2));
    assert!(stderr(&missing_output).contains("-o requires an argument"));

    let first = temp_source("multi_first", "fn main() {}\n");
    let second = temp_source("multi_second", "fn main() {}\n");
    let multiple = cli().arg(&first).arg(&second).output().unwrap();
    assert_eq!(multiple.status.code(), Some(2));
    assert!(stderr(&multiple).contains("multiple input files"));
    fs::remove_file(first).unwrap();
    fs::remove_file(second).unwrap();
}

#[test]
fn valid_file_compiles_to_ir_scope_and_reports_zero_exit_code() {
    let path = temp_source("valid", "fn main() int { return 7; }\n");

    let output = cli().arg(&path).output().unwrap();

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    assert!(stdout(&output).contains("exit code: 0"));
    fs::remove_file(path).unwrap();
}

#[test]
fn no_run_is_accepted_but_is_not_required_for_current_ir_only_pipeline() {
    let path = temp_source("no_run", "fn main() int { return 7; }\n");

    let output = cli().args(["--no-run"]).arg(&path).output().unwrap();

    assert!(output.status.success());
    assert!(stdout(&output).contains("exit code: 0"));
    fs::remove_file(path).unwrap();
}

#[test]
fn emit_tokens_ast_and_ir_print_stable_markers_to_stderr() {
    let path = temp_source("emit_all", "fn main() int { return 0; }\n");

    let output = cli()
        .args(["--emit-tokens", "--emit-ast", "--emit-ir", "--no-run"])
        .arg(&path)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stderr = stderr(&output);
    assert!(stderr.contains("[token] Fn"));
    assert!(stderr.contains("[token] EOF"));
    assert!(stderr.contains("[ast]"));
    assert!(stderr.contains("Function"));
    assert!(stderr.contains("==== IR Module ===="));
    assert!(stderr.contains("IrFunction { name: \"main\""));
    fs::remove_file(path).unwrap();
}

#[test]
fn output_path_option_is_parsed_but_does_not_assert_backend_binary_creation() {
    let path = temp_source("output_path", "fn main() int { return 0; }\n");
    let binary = temp_path("not_created", "bin");

    let output = cli()
        .args(["--no-run", "-o"])
        .arg(&binary)
        .arg(&path)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(!binary.exists());
    fs::remove_file(path).unwrap();
}

#[test]
fn rejects_non_ry_extension_and_missing_files_with_exit_code_one() {
    let wrong_extension = temp_path("wrong_extension", "txt");
    fs::write(&wrong_extension, "fn main() int { return 0; }\n").unwrap();

    let wrong = cli().arg(&wrong_extension).output().unwrap();
    assert_eq!(wrong.status.code(), Some(1));
    assert!(stderr(&wrong).contains("expected .ry file"));
    assert!(stdout(&wrong).contains("exit code: 1"));
    fs::remove_file(wrong_extension).unwrap();

    let missing = temp_path("missing", "ry");
    let missing_output = cli().arg(&missing).output().unwrap();
    assert_eq!(missing_output.status.code(), Some(1));
    let stderr = stderr(&missing_output);
    assert!(stderr.contains("No such file") || stderr.contains("not found"));
}

#[test]
fn lexer_parser_and_ir_errors_are_reported_on_stderr() {
    let lex = temp_source("lex", "fn main() { @ }\n");
    let parse = temp_source("parse", "fn main( { return; }\n");
    let ir = temp_source("ir", "fn main() int { return true; }\n");

    let lex_output = cli().arg(&lex).output().unwrap();
    assert_eq!(lex_output.status.code(), Some(1));
    assert!(stderr(&lex_output).contains("[lexer]"));

    let parse_output = cli().arg(&parse).output().unwrap();
    assert_eq!(parse_output.status.code(), Some(1));
    assert!(stderr(&parse_output).contains("[parser]"));

    let ir_output = cli().arg(&ir).output().unwrap();
    assert_eq!(ir_output.status.code(), Some(1));
    assert!(stderr(&ir_output).contains("[ir]"));

    fs::remove_file(lex).unwrap();
    fs::remove_file(parse).unwrap();
    fs::remove_file(ir).unwrap();
}

#[test]
fn release_keep_and_emit_asm_flags_are_accepted_without_backend_side_effect_assertions() {
    let path = temp_source("accepted_flags", "fn main() int { return 0; }\n");

    let output = cli()
        .args(["--release", "--keep", "--emit-asm", "--no-run"])
        .arg(&path)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(stdout(&output).contains("exit code: 0"));
    fs::remove_file(path).unwrap();
}
