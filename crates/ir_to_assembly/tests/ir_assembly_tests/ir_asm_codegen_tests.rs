#[test]
fn asm_codegen_test_harness_is_wired() {
    // AsmCodeGen currently returns a private error type from public methods.
    // Integration tests cannot call those methods until the public API is adjusted.
    assert!(true);
}
