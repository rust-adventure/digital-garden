use color_eyre::{
    eyre::{eyre, Report, Result, WrapErr},
    Section,
};
use directories::UserDirs;
use edit::Builder;
use owo_colors::OwoColorize;
use std::path::PathBuf;
use structopt::StructOpt;
use tracing::{info, instrument};

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
    /// initialize a garden
    ///
    /// Creates a new garden at the filepath
    /// initializes it with git
    Init,
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
    Spark {
        #[structopt(short)]
        message: String,
    },
    /// Search for files
    Search {
        #[structopt(short)]
        tags: Vec<String>,
    },
    Write,
}

fn ask_for_filename() -> String {
    loop {
        let filename = rprompt::prompt_reply_stderr(
            format!(
                "{} (will be slugified)
    > ",
                "Enter filename".bold().green()
            )
            .as_str(),
        )
        .wrap_err("Failed to get new filename");
        match filename {
            Ok(value) => {
                break value;
            }
            Err(_) => {
                // maybe invalid utf-8
                // let the user try again
            }
        };
    }
}

#[instrument]
fn main() -> Result<(), Report> {
    #[cfg(feature = "capture-spantrace")]
    install_tracing();
    color_eyre::install()?;

    let opt = Opt::from_args();
    println!("{:?}", opt);
    let garden_path = {
        opt.garden_path.ok_or_else(|| {
            if let Some(user_dirs) = UserDirs::new() {
                let home = user_dirs.home_dir();
                println!("{}", "initializing your garden!".purple());
            }
        })
    };
    match opt.cmd {
        Command::Init => Ok(()),
        Command::Write => {
            let template = "# ";
            let mut builder = Builder::new();
            let named_tempfile = builder.suffix(".md").rand_bytes(5);
            let edited = edit::edit_with_builder(template, &named_tempfile)?;
            // TODO: write file out in case anything after this fails
            // special tempdir?
            let heading = edited
                .lines()
                .find(|v| v.starts_with("# "))
                .map(|maybe_line| maybe_line.trim_start_matches("# "));
            let filename = match heading {
                Some(raw_title) => {
                    let title = slug::slugify(raw_title);
                    let file_slug = loop {
                        let result = rprompt::prompt_reply_stderr(
                            format!(
                                "> Current Title: {}
> filename: {}.md
{} (y/N): ",
                                raw_title,
                                title,
                                "Want different filename".bold().green()
                            )
                            .as_str(),
                        )
                        .wrap_err("Failed to get input for y/n question")?;
                        match result.to_lowercase().as_str() {
                            "y" | "Y" => {
                                break ask_for_filename();
                            }
                            "n" | "N" => {
                                break title;
                            }
                            _ => {
                                // ask again
                            }
                        };
                    };
                    file_slug
                }
                None => {
                    let result = ask_for_filename();
                    format!("{}.md", slug::slugify(result))
                }
            };
            // TODO: does file already exist?
            // if so, do what?
            println!(
                "after editing:
filename: {}
content: {}
",
                filename, edited
            );
            //
            Ok(())
        }
        Command::Spark { message } => Ok(()),
        Command::Search { tags } => Ok(()),
        Command::Sync {
            interactive,
            all,
            files,
        } => Ok(()),
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
