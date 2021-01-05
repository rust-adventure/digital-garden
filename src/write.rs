use color_eyre::{eyre::WrapErr, Result};
use edit::{edit_file, Builder};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

const TEMPLATE: &[u8; 2] = b"# ";

pub fn write(garden_path: PathBuf, title: Option<String>) -> Result<()> {
    let (mut file, filepath) = Builder::new()
        .suffix(".md")
        .rand_bytes(5)
        .tempfile_in(&garden_path)
        .wrap_err("Failed to create wip file")?
        .keep()
        .wrap_err("Failed to keep tempfile")?;
    file.write_all(TEMPLATE)?;
    // let the user write whatever they want in their favorite editor
    // before returning to the cli and finishing up
    edit_file(filepath)?;
    // Read the user's changes back from the file into a string
    let mut contents = String::new();
    file.seek(SeekFrom::Start(0))?;
    file.read_to_string(&mut contents)?;

    // use `title` if the user passed it in,
    // otherwise try to find a heading in the markdown
    let document_title = title.or_else(|| {
        dbg!("here");
        contents
            .lines()
            .find(|v| dbg!(v).starts_with("# "))
            // markdown headings are required to have `# ` with
            // at least one space
            .map(|maybe_line| maybe_line.trim_start_matches("# ").to_string())
    });

    dbg!(contents, document_title);
    todo!()
}
