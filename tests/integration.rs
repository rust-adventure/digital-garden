mod test_utils;

#[test]
fn test_add() {
    let temp = assert_fs::TempDir::new().unwrap();
    let bin = escargot::CargoBuild::new()
        .bin("garden")
        .current_release()
        .current_target()
        .manifest_path("Cargo.toml")
        .target_dir(temp.path())
        .run()
        .unwrap();

    let mut cmd = assert_cmd::Command::new(bin.path());
    let assert = cmd.arg("init").env("GARDEN_PATH", temp.path()).assert();
    assert.success();

    // assert_eq!(3, 5);
}
