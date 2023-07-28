/// make sure help runs. This indicates the binary works
#[test]
fn test_help() {
    assert_cmd::Command::cargo_bin("garden")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stderr("");
}

/// make sure we have a write command by running `garden write --help`
#[test]
fn test_write_help() {
    assert_cmd::Command::cargo_bin("garden")
        .unwrap()
        .arg("write")
        .arg("--help")
        .assert()
        .success()
        .stderr("");
}
