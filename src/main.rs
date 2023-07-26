use clap::{Parser, Subcommand};
use directories::UserDirs;
use garden::write;
use miette::{miette, Context, IntoDiagnostic, Result};
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
fn main() -> Result<()> {
    let args = Args::parse();

    let garden_path = args
        .garden_path
        .or_else(get_default_garden_dir)
        .ok_or(miette!("Could not find home directory"))?;

    match args.cmd {
        Commands::Write { title } => {
            write(garden_path, title)
                .into_diagnostic()
                .wrap_err("garden::write")
        }
    }
}
