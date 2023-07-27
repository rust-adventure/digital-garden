use edit::{edit_file, Builder};
use std::{fs, io::Write, path::PathBuf};

pub fn write(
    garden_path: PathBuf,
    title: Option<String>,
) -> Result<(), std::io::Error> {
    let (mut file, filepath) = Builder::new()
        .suffix(".md")
        .rand_bytes(5)
        .tempfile_in(&garden_path)?
        .keep()?;
    dbg!(&filepath);
    let template =
        format!("# {}", title.as_deref().unwrap_or(""));
    file.write_all(template.as_bytes())?;
    edit_file(&filepath)?;
    let contents = fs::read_to_string(&filepath)?;

    let document_title = title.or_else(|| {
        contents.lines().find(|v| v.starts_with("# ")).map(
            |line| {
                line.trim_start_matches("# ").to_string()
            },
        )
    });

    let filename = match document_title {
        Some(raw_title) => slug::slugify(raw_title),
        None => {
            todo!("ask for filename");
        }
    };

    let mut dest = garden_path.join(filename);
    dest.set_extension("md");
    fs::rename(filepath, &dest)?;
    dbg!(dest);

    Ok(())
}
