use assert_fs::prelude::*;
use predicates::prelude::*;
use rexpect::{self, session::PtySession};
use std::process::Command;

trait GardenExpectations {
    fn exp_title(
        &mut self,
        title: &str,
    ) -> Result<(), rexpect::error::Error>;
}
impl GardenExpectations for PtySession {
    fn exp_title(
        &mut self,
        title: &str,
    ) -> Result<(), rexpect::error::Error> {
        self.exp_string("current title: ")?;
        self.exp_string(title)?;
        self.exp_regex("\\s*")?;
        self.exp_string(
            "Do you want a different title? (y/N): ",
        )?;
        Ok(())
    }
}

fn setup_command() -> (Command, assert_fs::TempDir) {
    let temp_dir = assert_fs::TempDir::new().unwrap();

    let bin_path = assert_cmd::cargo::cargo_bin("garden");
    let fake_editor_path = std::env::current_dir()
        .expect("expect to be in a dir")
        .join("tests")
        .join("fake-editor.sh");
    if !fake_editor_path.exists() {
        panic!(
            "fake editor shell script could not be found"
        )
    }

    let mut cmd = std::process::Command::new(bin_path);
    cmd.env(
        "EDITOR",
        fake_editor_path.into_os_string(),
    )
    .env("GARDEN_PATH", temp_dir.path())
    .env("NO_COLOR", "true");
    (cmd, temp_dir)
}

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

#[test]
fn test_write_with_title(
) -> Result<(), rexpect::error::Error> {
    let (mut cmd, temp_dir) = setup_command();

    cmd.arg("write").arg("-t").arg("atitle");

    let mut process =
        rexpect::session::spawn_command(cmd, None).unwrap();

    process.exp_title("atitle")?;
    process.send_line("N")?;
    process.exp_eof()?;

    temp_dir
        .child("atitle.md")
        .assert(predicate::path::exists());
    Ok(())
}

#[test]
fn test_write_with_written_title(
) -> Result<(), rexpect::error::Error> {
    let (mut cmd, temp_dir) = setup_command();
    cmd.arg("write");

    let mut process =
        rexpect::session::spawn_command(cmd, None).unwrap();

    process.exp_title("testing")?;
    process.send_line("N")?;
    process.exp_eof()?;

    temp_dir
        .child("testing.md")
        .assert(predicate::path::exists());
    Ok(())
}
