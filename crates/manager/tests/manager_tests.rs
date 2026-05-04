#[test]
fn manager_test_harness_is_wired() {
    let _ = manager::read_file::read_file as fn(&str) -> Result<String, std::io::Error>;
}
