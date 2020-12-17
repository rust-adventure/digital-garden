use color_eyre::eyre::{eyre, Report, Result, WrapErr};
use directories::UserDirs;
use edit::Builder;
use owo_colors::OwoColorize;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use structopt::StructOpt;
use tracing::{info, instrument};

use digital_garden::write;

/// maintain your garden
///
/// With digital-garden, you can take notes, publish, link documents and more
#[derive(StructOpt, Debug)]
#[structopt(name = "garden")]
struct Opt {
    #[structopt(short = "p", long, env)]
    garden_path: Option<PathBuf>,

    #[structopt(subcommand)] // Note that we mark a field as a subcommand
    cmd: Command,
}
#[derive(StructOpt, Debug)]
enum Command {
    /// Search for files
    Search {
        #[structopt(short)]
        tags: Vec<String>,
    },
    /// Write something
    Write {
        #[structopt(short, long)]
        title: Option<String>,
    },
    /// allows writing to a sparkfile.
    ///
    /// A sparkfile is a date-oriented markdown file that collects thoughts you
    /// have now.
    ///
    /// Put something in a sparkfile to get it out of your head now and work on it later.
    ///
    /// This command can also be useful to integrate with Alfred and other
    /// application launches
    Spark {
        #[structopt(short)]
        message: String,
    },
    Publish {
        #[structopt(short, long)]
        output: PathBuf,
    },
}

#[instrument]
fn get_default_garden_dir() -> PathBuf {
    let user_dirs = UserDirs::new().expect("expected dir");
    user_dirs.home_dir().join(".garden")
}

#[instrument]
fn main() -> Result<(), Report> {
    #[cfg(feature = "capture-spantrace")]
    install_tracing();
    color_eyre::install()?;

    let opt = Opt::from_args();
    let garden_path: PathBuf = match opt.garden_path {
        Some(p) => p,
        None => get_default_garden_dir(),
    };
    match opt.cmd {
        Command::Write { title } => write(garden_path, title),
        Command::Spark { message } => {
            // write to spark file
            let mut sparkfile_path = garden_path.join("_spark");
            sparkfile_path.set_extension("md");
            let mut sparkfile = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(sparkfile_path)?;
            write!(&mut sparkfile, "\n{}", message)?;

            // fs::write(sparkfile, &edited)?;
            Ok(())
        }
        Command::Search { tags } => Ok(()),
        Command::Publish { output } => Ok(()),
    }
}

#[cfg(feature = "capture-spantrace")]
fn install_tracing() {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};

    let fmt_layer = fmt::layer().with_target(false);
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();
}
