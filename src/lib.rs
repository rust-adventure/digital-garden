use color_eyre::eyre::{eyre, Result, WrapErr};
use edit::Builder;
use owo_colors::OwoColorize;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;
use tracing::instrument;

const DEFAULT_TEMPLATE: &str = "# ";

#[instrument]
fn ask_for_filename() -> String {
    let prompt = format!(
        "\
{} (will be slugified)
> ",
        "Enter filename".bold().green()
    );

    loop {
        if let Ok(filename) = rprompt::prompt_reply_stderr(&prompt) {
            break filename;
        }
    }
}

#[instrument]
fn let_user_edit<'a>(garden_path: PathBuf) -> Result<(String, PathBuf)> {
    let mut builder = Builder::new();
    let named_tempfile = builder.suffix(".md").rand_bytes(5);
    // create a temp file so that we can use the name for our own purposes
    let a_temp_file = named_tempfile
        .tempfile()
        .wrap_err("Failed to create wip file")?;
    let garden_wip_filename = a_temp_file
        .path()
        .file_name()
        .ok_or(eyre!("failed to create tmpfile"))?;
    // let the user write whatever they want in their favorite editor
    // before returning to the cli and finishing up
    let edited = edit::edit_with_builder(DEFAULT_TEMPLATE, &named_tempfile)?;
    // write the file out to a temporary directory in the garden
    // just in case anything goes wrong, so we don't lose the content
    let filepath = garden_path.join(garden_wip_filename);
    fs::write(&filepath, &edited)?;
    Ok((edited, filepath))
}

/// find_heading takes a piece of markdown and uses some
/// heuristics to find a potential title for the written
/// content
/// ## test
/// get a heading, if there is one, to use for the filename
/// this could be done in a more robust way by using a markdown
/// parser and using the AST to find headings
#[instrument]
fn find_heading(edited: &str) -> Option<&str> {
    edited
        .lines()
        .find(|v| v.starts_with("# "))
        // markdown headings are required to have `# ` with
        // at least one space
        .map(|maybe_line| maybe_line.trim_start_matches("# "))
}

fn confirm_filename<'a>(raw_title: &'a str, slug: String) -> Result<String> {
    Ok(loop {
        // prompt defaults to uppercase character in question
        // this is a convention, not a requirement enforced by
        // the code
        let result = rprompt::prompt_reply_stderr(&format!(
            "\
current title: `{}`
resulting filename: {}.md
{} (y/N): ",
            raw_title,
            slug,
            "Do you want a different filename?".bold().green()
        ))
        .wrap_err("Failed to get input for y/n question")?;

        match result.as_str() {
            "y" | "Y" => {
                break ask_for_filename();
            }
            "n" | "N" | "" => {
                // the capital N in the prompt means "default",
                // so we handle "" as input here
                break slug;
            }
            _ => {
                // ask again because something went wrong
            }
        };
    })
}
#[instrument]
pub fn write(garden_path: PathBuf, title: Option<String>) -> Result<()> {
    let (edited, garden_tmpfile): (String, _) = let_user_edit(garden_path.clone())?;

    let heading = find_heading(&edited);
    // get the filename to use for the file
    let filename = match heading {
        Some(raw_title) => {
            // if we found a heading in the file, slugify it
            // and ask the user if it's good enough
            let title_slug = slug::slugify(raw_title);
            confirm_filename(raw_title, title_slug)?
        }
        None => slug::slugify(ask_for_filename()),
    };

    // move tempfile into garden with a name
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
            fs::rename(garden_tmpfile, &dest)?;
            break;
        }
    }

    Ok(())
}
