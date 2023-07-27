use clap::{
    error::ErrorKind, CommandFactory, Parser, Subcommand,
};
use directories::UserDirs;
use std::path::PathBuf;

/// A CLI for the growing and curation of a digital garden
///
/// Visit https://www.rustadventure.dev for more!
#[derive(Parser, Debug)]
#[clap(version)]
struct Args {
    #[clap(short = 'p', long, env)]
    garden_path: Option<PathBuf>,

    #[command(subcommand)]
    cmd: Commands,
}
#[derive(Subcommand, Debug)]
enum Commands {
    /// write something in your garden
    ///
    /// This command will open your $EDITOR, wait for you
    /// to write something, and then save the file to your
    /// garden
    Write {
        /// Optionally set a title for what you are going to write about
        #[clap(short, long)]
        title: Option<String>,
    },
}

/// Get the user's garden directory, which by default
/// is placed in their home directory
fn get_default_garden_dir() -> Option<PathBuf> {
    UserDirs::new()
        .map(|dirs| dirs.home_dir().join("garden"))
}

fn main() {
    let args = Args::parse();

    let Some(garden_path) =
        args.garden_path.or_else(get_default_garden_dir)
    else {
        let mut cmd = Args::command();
        cmd.error(
            ErrorKind::ValueValidation,
            format!(
                "garden directory not provided and home directory unavailable for default garden directory"
            ),
        )
        .exit()
    };
    if !garden_path.exists() {
        let mut cmd = Args::command();
        cmd.error(
            ErrorKind::ValueValidation,
            format!(
                "garden directory `{}` doesn't exist, or is inaccessible",
                garden_path.display()
            ),
        )
        .exit()
    };

    match args.cmd {
        Commands::Write { title } => {
            garden::write(garden_path, title)
        }
    }
}
