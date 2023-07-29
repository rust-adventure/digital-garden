use assert_fs::prelude::*;
use predicates::prelude::*;
#[cfg(not(target_os = "windows"))]
use rexpect::session::spawn_command;
use std::{error::Error, process::Command};

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

#[cfg(not(target_os = "windows"))]
#[test]
fn test_write_with_title() -> Result<(), Box<dyn Error>> {
    let temp_dir = assert_fs::TempDir::new()?;

    let bin_path = assert_cmd::cargo::cargo_bin("garden");
    let fake_editor_path = std::env::current_dir()?
        .join("tests")
        .join("fake-editor.sh");
    if !fake_editor_path.exists() {
        panic!(
            "fake editor shell script could not be found"
        )
    }

    let mut cmd = Command::new(bin_path);
    cmd.env(
        "EDITOR",
        fake_editor_path.into_os_string(),
    )
    .env("GARDEN_PATH", temp_dir.path())
    .env("NO_COLOR", "true")
    .arg("write")
    .arg("-t")
    .arg("atitle");

    let mut process = spawn_command(cmd, None)?;

    process.exp_string("current title: ")?;
    process.exp_string("atitle")?;
    process.exp_regex("\\s*")?;
    process.exp_string(
        "Do you want a different title? (y/N): ",
    )?;
    process.send_line("N")?;
    process.exp_eof()?;

    temp_dir
        .child("atitle.md")
        .assert(predicate::path::exists());
    Ok(())
}
