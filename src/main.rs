use color_eyre::{
    eyre::{eyre, Report, Result, WrapErr},
    Section,
};
use directories::UserDirs;
use edit::Builder;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;
use tracing::{info, instrument};

#[derive(Default, Debug, Serialize, Deserialize)]
struct Garden {
    path: PathBuf,
}
impl fmt::Display for Garden {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "  {} {}", "garden_path:".bold(), self.path.display())
    }
}

type GardenName = String;

#[derive(Debug, Serialize, Deserialize)]
struct GardenConfig {
    aliases: HashMap<GardenName, Garden>,
    default_garden: GardenName,
}

impl GardenConfig {
    fn get_default_garden_path(&self) -> Result<PathBuf> {
        self.aliases
            .get(&self.default_garden)
            .map(|v| v.path.clone())
            .ok_or(eyre!("garden path was not set in the environment, passed as a flag, or discoverable in the default garden config"))
    }
}

impl ::std::default::Default for GardenConfig {
    fn default() -> Self {
        let mut aliases = HashMap::new();
        let homedir = UserDirs::new()
            .and_then(|user_dirs| Some(user_dirs.home_dir().join(".garden")))
            .unwrap();
        aliases.insert(String::from("default"), Garden { path: homedir });
        Self {
            aliases,
            default_garden: String::from("default"),
        }
    }
}

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

    let mut cfg: GardenConfig = confy::load("garden").wrap_err("Failed to load configuration")?;

    let opt = Opt::from_args();
    println!("{:?}", opt);
    let garden_path: PathBuf = match opt.garden_path {
        Some(p) => p,
        None => cfg.get_default_garden_path()?,
    };
    match opt.cmd {
        Command::Init => {
            println!("{} garden", "Initializing".purple());
            if cfg.aliases.contains_key("default") {
                let garden = cfg.aliases.get("default").unwrap();
                println!(
                    "Garden with name `default` already exists with config:
{}",
                    garden
                );
            } else {
                cfg.aliases.insert(
                    String::from("default"),
                    Garden {
                        path: garden_path.clone(),
                    },
                );
                confy::store("garden", cfg).wrap_err("Failed to write garden config to disk")?
            };
            if garden_path.exists() {
                println!(
                    "{} garden path {} {}",
                    "✔".bold().green(),
                    "already exists:".bold().green(),
                    garden_path.display(),
                );
            } else {
                fs::create_dir_all(&garden_path).wrap_err("Failed to create garden directory")?;
                println!(
                    "{} {} {}",
                    "✔".bold().green(),
                    "created:".bold().green(),
                    garden_path.display(),
                );
            }

            Ok(())
        }
        Command::Write => {
            // file template
            let template = "# ";
            let mut builder = Builder::new();
            let named_tempfile = builder.suffix(".md").rand_bytes(5);
            // create a temp file so that we can use the name for our own purposes
            let a_temp_file = named_tempfile
                .tempfile()
                .wrap_err("Failed to create wip file")?;
            let garden_wip_file_name = a_temp_file.path().file_name().ok_or(eyre!(""))?;
            // let the user write whatever they want in their favorite editor
            // before returning to the cli and finishing up
            let edited = edit::edit_with_builder(template, &named_tempfile)?;
            // write the file out to a temporary directory in the garden
            // just in case anything goes wrong, so we don't lose the content
            fs::write(garden_path.join(garden_wip_file_name), &edited)?;
            // get a heading, if there is one, to use for the filename
            // this could be done in a more robust way by using a markdown
            // parser and using the AST to find headings
            let heading = edited
                .lines()
                .find(|v| v.starts_with("# "))
                .map(|maybe_line| maybe_line.trim_start_matches("# "));
            // get the filename to use for the file
            let filename = match heading {
                Some(raw_title) => {
                    // if we found a heading in the file, slugify it
                    // and ask the user if it's good enough
                    let title = slug::slugify(raw_title);
                    let file_slug = loop {
                        // prompt defaults to uppercase character in question
                        // this is a convention, not a requirement enforced by
                        // the code
                        let result = rprompt::prompt_reply_stderr(
                            format!(
                                "  current title: `{}`
  resulting filename: {}.md
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
                            "n" | "N" | "" => {
                                // the capital N in the prompt means "default",
                                // so we handle "" as input here
                                break title;
                            }
                            _ => {
                                // ask again because something went wrong
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
            // otherwise move tmpfile into new location
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
