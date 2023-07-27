use edit::Builder;
use std::path::PathBuf;

pub fn write(
    garden_path: PathBuf,
    _title: Option<String>,
) -> Result<(), std::io::Error> {
    let (_file, filepath) = Builder::new()
        .suffix(".md")
        .rand_bytes(5)
        .tempfile_in(garden_path)?
        .keep()?;
    dbg!(filepath);
    Ok(())
}
