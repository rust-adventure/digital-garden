use color_eyre::{
    eyre::{eyre, Report, WrapErr},
    Section,
};
use directories::UserDirs;
use std::path::PathBuf;
use structopt::StructOpt;
use tracing::{info, instrument};

#[derive(StructOpt, Debug)]
#[structopt(name = "garden")]
/// maintain your garden
///
/// With digital-garden, you can take notes, publish, link documents and more
enum Opt {
    /// initialize a garden
    ///
    /// Creates a new garden at the filepath
    /// initializes it with git
    Init {
        #[structopt(default_value = "", short = "p", long, env)]
        garden_path: PathBuf,
    },
    /// Sync files to git
    ///
    /// commits changes and attempts to push
    Sync {
        #[structopt(short, long)]
        interactive: bool,
        #[structopt(short)]
        all: bool,
        files: Vec<String>,
    },
    /// Write to a sparkfile.
    ///
    /// A sparkfile is a date-oriented markdown file that collects thoughts you
    /// have now.
    ///
    /// Put something here to get it out of your head now and work on it later.
    ///
    /// This command can also be useful to integrate with Alfred and other
    /// application launches
    Spark { content: String },
    /// Search for files
    Search {
        #[structopt(short)]
        tags: Vec<String>,
    },
}

#[instrument]
fn main() -> Result<(), Report> {
    #[cfg(feature = "capture-spantrace")]
    install_tracing();
    color_eyre::install()?;

    let opt = Opt::from_args();
    println!("{:?}", opt);

    if let Some(user_dirs) = UserDirs::new() {
        let home = user_dirs.home_dir();
        println!("initializing your garden!");
    }
    Ok(())
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
