use assert_cmd::Command;
use assert_fs::prelude::*;
use color_eyre::eyre::Result;
use predicates::prelude::*;

#[test]
/// make sure help runs. This indicates the binary works
fn test_help() -> Result<()> {
    let mut cmd = Command::cargo_bin("garden")?;
    let assert = cmd.arg("--help").assert();
    assert.success().stderr("");
    Ok(())
}

#[test]
/// make sure we have a write command by running `garden write --help`
fn test_write_help() -> Result<()> {
    let mut cmd = Command::cargo_bin("garden")?;
    let assert = cmd.arg("write").arg("--help").assert();
    assert.success().stderr("");
    Ok(())
}

#[test]
fn test_write_with_title() {
    let temp_dir = assert_fs::TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("garden").unwrap();
    let fake_editor_path = std::env::current_dir()
        .expect("expect to be in a dir")
        .join("tests")
        .join("fake-editor.sh");
    if !fake_editor_path.exists() {
        panic!(
            "fake editor shell script could not be found"
        )
    }

    let assert = cmd
        .env("EDITOR", fake_editor_path.into_os_string())
        .env("GARDEN_PATH", temp_dir.path())
        .arg("write")
        .arg("-t")
        .arg("atitle")
        .write_stdin("N\n".as_bytes())
        .assert();

    assert.success();

    temp_dir
        .child("atitle.md")
        .assert(predicate::path::exists());
}
