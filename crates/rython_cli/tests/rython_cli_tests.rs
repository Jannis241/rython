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
    let path = temp_source("fn main() int { return 0; }\n");

    let output = cli()
        .args(["--emit-tokens", "--emit-ast"])
        .arg(&path)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Token"));
    assert!(stderr.contains("Function"));
    fs::remove_file(path).unwrap();
}

#[test]
fn rejects_non_ry_extension() {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("rython_cli_test_{nanos}.txt"));
    fs::write(&path, "fn main() int { return 0; }\n").unwrap();

    let output = cli().arg(&path).output().unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("expected .ry file"));
    fs::remove_file(path).unwrap();
}

#[test]
fn no_run_skips_program_execution_and_returns_zero() {
    let path = temp_source("fn main() int { return 7; }\n");

    let output = cli().args(["--no-run"]).arg(&path).output().unwrap();

    assert!(output.status.success());
    fs::remove_file(path).unwrap();
}

#[test]
fn output_path_overrides_default_binary_location() {
    let path = temp_source("fn main() int { return 0; }\n");
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let bin = std::env::temp_dir().join(format!("rython_cli_test_bin_{nanos}"));

    let output = cli()
        .args(["--no-run", "-o"])
        .arg(&bin)
        .arg(&path)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(bin.exists(), "expected binary at {}", bin.display());
    fs::remove_file(&path).unwrap();
    fs::remove_file(&bin).unwrap();
}

#[test]
fn program_exit_code_is_propagated() {
    let path = temp_source("fn main() int { return 7; }\n");

    let output = cli().arg(&path).output().unwrap();

    assert_eq!(output.status.code(), Some(7));
    fs::remove_file(path).unwrap();
}

#[test]
fn unknown_flag_exits_with_usage_error() {
    let output = cli().arg("--definitely-not-a-flag").output().unwrap();

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("unknown option"));
}

#[test]
fn missing_file_exits_with_failure() {
    let path = std::env::temp_dir().join("rython_cli_missing_file_for_test.ry");

    let output = cli().arg(path).output().unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("No such file") || stderr.contains("not found"));
}
