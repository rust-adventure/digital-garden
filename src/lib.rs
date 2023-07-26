use edit::{edit_file, Builder};
use miette::Diagnostic;
use owo_colors::OwoColorize;
use std::{fs, io::Write};
use std::{io, path::PathBuf};
use thiserror::Error;

const TEMPLATE: &[u8; 2] = b"# ";

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
) -> Result<(), GardenVarietyError> {
    let (mut file, filepath) = Builder::new()
        .suffix(".md")
        .rand_bytes(5)
        .tempfile_in(&garden_path)
        .map_err(|e| {
            GardenVarietyError::TempfileCreationError(e)
        })?
        .keep()?;
    file.write_all(TEMPLATE)?;
    // let the user write whatever they want in their favorite editor
    // before returning to the cli and finishing up
    edit_file(&filepath)?;
    // Read the user's changes back from the file into a string
    // some editors like vim or emacs have pathological cases
    // when dealing with files, and will create brand new files
    // we don't know about before saving back to the original file
    // path, so we read_to_string here instead of using the original
    // file from the builder.
    let contents =
        fs::read_to_string(&filepath).map_err(|e| {
            GardenVarietyError::TempfileReadError {
                filepath: filepath.clone(),
                io_error: e,
            }
        })?;

    // use `title` if the user passed it in,
    // otherwise try to find a heading in the markdown
    let document_title = title.or_else(|| {
        contents
            .lines()
            .find(|v| v.starts_with("# "))
            // markdown headings are required to have `# ` with
            // at least one space
            .map(|maybe_line| {
                maybe_line
                    .trim_start_matches("# ")
                    .to_string()
            })
    });

    // get the filename to use for the file
    let filename = match document_title {
        Some(raw_title) => confirm_filename(&raw_title),
        None => ask_for_filename(),
    }?;

    let mut i: usize = 0;
    loop {
        let dest_filename = format!(
            "{}{}",
            filename,
            if i == 0 {
                "".to_string()
            } else {
                i.to_string()
            }
        );
        let mut dest = garden_path.join(dest_filename);
        dest.set_extension("md");
        if dest.exists() {
            i = i + 1;
        } else {
            fs::rename(filepath, &dest)?;
            break;
        }
    }

    Ok(())
}

fn ask_for_filename() -> io::Result<String> {
    rprompt::prompt_reply(&format!(
        "{}",
        "\
Enter filename
> "
        .blue()
        .bold(),
    ))
    .map(|title| slug::slugify(title))
}

fn confirm_filename(raw_title: &str) -> io::Result<String> {
    loop {
        // prompt defaults to uppercase character in question
        // this is a convention, not a requirement enforced by
        // the code
        let result = rprompt::prompt_reply(&format!(
            "\
{} {}
Do you want a different title? (y/N): ",
            "current title:".green().bold(),
            raw_title,
        ))?;

        match result.as_str() {
            "y" | "Y" => break ask_for_filename(),
            "n" | "N" | "" => {
                // the capital N in the prompt means "default",
                // so we handle "" as input here
                break Ok(slug::slugify(raw_title));
            }
            _ => {
                // ask again because something went wrong
            }
        };
    }
}
