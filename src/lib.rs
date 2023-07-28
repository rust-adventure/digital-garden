use edit::{edit_file, Builder};
use miette::Diagnostic;
use owo_colors::OwoColorize;
use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
pub enum GardenVarietyError {
    #[error(transparent)]
    #[diagnostic(code(garden::io_error))]
    IoError(#[from] std::io::Error),

    #[error("failed to create tempfile: {0}")]
    #[diagnostic(code(garden::tempfile_create_error))]
    TempfileCreationError(std::io::Error),

    #[error("failed to keep tempfile: {0}")]
    #[diagnostic(code(garden::tempfile_keep_error))]
    TempfileKeepError(#[from] tempfile::PersistError),

    #[error("Unable to read tempfile after passing edit control to user:\ntempfile: {filepath}\n{io_error}")]
    #[diagnostic(
        code(garden::tempfile_read_error),
        help("Make sure your editor isn't moving the file away from the temporary location")
    )]
    TempfileReadError {
        filepath: PathBuf,
        io_error: std::io::Error,
    },
}

pub fn write(
    garden_path: PathBuf,
    title: Option<String>,
) -> miette::Result<(), GardenVarietyError> {
    let (mut file, filepath) = Builder::new()
        .suffix(".md")
        .rand_bytes(5)
        .tempfile_in(&garden_path)
        .map_err(|e| {
            GardenVarietyError::TempfileCreationError(e)
        })?
        .keep()?;
    let template =
        format!("# {}", title.as_deref().unwrap_or(""));
    file.write_all(template.as_bytes())?;
    edit_file(&filepath)?;
    let contents =
        fs::read_to_string(&filepath).map_err(|e| {
            GardenVarietyError::TempfileReadError {
                filepath: filepath.clone(),
                io_error: e,
            }
        })?;

    let document_title = title.or_else(|| {
        contents.lines().find(|v| v.starts_with("# ")).map(
            |line| {
                line.trim_start_matches("# ").to_string()
            },
        )
    });

    let filename = match document_title {
        Some(raw_title) => confirm_filename(&raw_title),
        None => ask_for_filename(),
    }
    .map(|title| slug::slugify(title))?;

    for attempt in 0.. {
        let mut dest = garden_path.join(if attempt == 0 {
            filename.clone()
        } else {
            format!("{filename}{:03}", -attempt)
        });
        dest.set_extension("md");
        if dest.exists() {
            continue;
        }
        fs::rename(filepath, &dest)?;
        break;
    }

    Ok(())
}

fn ask_for_filename() -> io::Result<String> {
    rprompt::prompt_reply(
        "Enter filename
> "
        .blue()
        .bold(),
    )
}

fn confirm_filename(raw_title: &str) -> io::Result<String> {
    loop {
        // prompt defaults to uppercase character in question
        // this is a convention, not a requirement enforced by
        // the code
        let result = rprompt::prompt_reply(&format!(
            "current title: {}
Do you want a different title? (y/{}): ",
            &raw_title.bold().green(),
            "N".bold(),
        ))?;

        match result.as_str() {
            "y" | "Y" => break ask_for_filename(),
            "n" | "N" | "" => {
                // the capital N in the prompt means "default",
                // so we handle "" as input here
                break Ok(raw_title.to_string());
            }
            _ => {
                // ask again because something went wrong
            }
        };
    }
}
