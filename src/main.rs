use clap::{Parser, Subcommand};
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

fn main() {
    let args = Args::parse();
    dbg!(args);
}
