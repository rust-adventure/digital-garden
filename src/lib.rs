use edit::{edit_file, Builder};
use std::{io::Write, path::PathBuf};

pub fn write(
    garden_path: PathBuf,
    title: Option<String>,
) -> Result<(), std::io::Error> {
    let (mut file, filepath) = Builder::new()
        .suffix(".md")
        .rand_bytes(5)
        .tempfile_in(garden_path)?
        .keep()?;
    dbg!(&filepath);
    let template =
        format!("# {}", title.unwrap_or("".to_string()));
    file.write_all(template.as_bytes())?;
    edit_file(&filepath)?;
    Ok(())
}
