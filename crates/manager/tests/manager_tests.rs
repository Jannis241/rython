use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_file(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("rython_manager_test_{name}_{nanos}.ry"))
}

#[test]
fn manager_test_harness_is_wired() {
    let _ = manager::read_file::read_file as fn(&str) -> Result<String, std::io::Error>;
}

#[test]
fn read_file_returns_exact_file_contents() {
    let path = temp_file("exact_contents");
    let content = "fn main() int {\n    return 42;\n}\n";
    fs::write(&path, content).unwrap();

    let actual = manager::read_file::read_file(path.to_str().unwrap()).unwrap();

    assert_eq!(actual, content);
    fs::remove_file(path).unwrap();
}

#[test]
fn read_file_reports_missing_files() {
    let path = temp_file("missing");

    let err = manager::read_file::read_file(path.to_str().unwrap()).unwrap_err();

    assert_eq!(err.kind(), ErrorKind::NotFound);
}
