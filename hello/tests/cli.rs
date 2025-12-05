use assert_cmd::Command;

#[test]
fn runs() {
    // Rust vars are immutable by default
    // so we have to explicitly set it
    // Pick the binary and execute it (must be in the same dir)
    // unwrap() returns the contained (in Result type) Ok value or panic if Err (sum type)
    let mut cmd = Command::cargo_bin("hello").unwrap();
    cmd.assert().success().stdout("Hello, world!\n");
}

#[test]
fn true_ok() {
    let mut cmd = Command::cargo_bin("true").unwrap();
    cmd.assert().success();
}
