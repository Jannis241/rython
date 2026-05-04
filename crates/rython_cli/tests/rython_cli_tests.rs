use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rython_cli"))
}

fn temp_source(contents: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("rython_cli_test_{nanos}.ry"));
    fs::write(&path, contents).unwrap();
    path
}

#[test]
fn no_args_prints_usage_and_exits_successfully() {
    let output = cli().output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("usage:"));
    assert!(stdout.contains("<your_program.ry>"));
}

#[test]
fn valid_file_is_lexed_and_parsed_by_cli() {
    let path = temp_source("fn main() int { return 42; }\n");

    let output = cli().arg(&path).output().unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Token"));
    assert!(stderr.contains("Function"));
    fs::remove_file(path).unwrap();
}

#[test]
fn missing_file_exits_with_failure() {
    let path = std::env::temp_dir().join("rython_cli_missing_file_for_test.ry");

    let output = cli().arg(path).output().unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("No such file") || stderr.contains("not found"));
}
