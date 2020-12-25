#![feature(bool_to_option)]
use color_eyre::eyre::{eyre, Report, Result, WrapErr};
use console::Term;
use dialoguer::{theme::ColorfulTheme, Select};
use directories::UserDirs;
use edit::Builder;
use owo_colors::OwoColorize;
use pulldown_cmark::{html, CodeBlockKind, CowStr, Event, Options, Parser, Tag};
use std::fs;
use std::io::BufRead;
use std::io::Write;
use std::path::PathBuf;
use structopt::StructOpt;
use syntect::{
    easy::HighlightFile,
    highlighting::{Color, Style, ThemeSet},
    html::highlighted_html_for_string,
    parsing::SyntaxSet,
    util::as_24_bit_terminal_escaped,
};
use tracing::{info, instrument};
use walkdir::WalkDir;

use digital_garden::write;

const HTML_TEMPLATE_START: &str = r#"<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <link rel="stylesheet" href="https://unpkg.com/tailwindcss@^2/dist/base.min.css" />
    <link rel="stylesheet" href="https://unpkg.com/tailwindcss@^2/dist/components.min.css" />
    <link rel="stylesheet" href="https://unpkg.com/@tailwindcss/typography@0.2.x/dist/typography.min.css" />
    <link rel="stylesheet" href="https://unpkg.com/tailwindcss@^2/dist/utilities.min.css" />
  </head>
  <body>
  <article class="max-w-prose mx-auto prose lg:prose-xl mt-12">"#;
const HTML_TEMPLATE_END: &str = r#"</article></body></html>"#;

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
    Read {
        #[structopt(short, long)]
        preview: bool,
    },
}

#[instrument]
fn get_default_garden_dir() -> PathBuf {
    let user_dirs = UserDirs::new().expect("expected dir");
    user_dirs.home_dir().join(".garden")
}

fn highlight(text: &str, lang: &str) -> String {
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["Solarized (light)"];
    highlighted_html_for_string(&text, &ss, ss.find_syntax_by_token(lang).unwrap(), theme)
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
            Ok(())
        }
        Command::Search { tags } => {
            // dbg!(tags);
            for entry in WalkDir::new(garden_path)
                .into_iter()
                .filter_map(|e| e.ok().and_then(|e2| e2.path().is_file().then_some(e2)))
            {
                // println!("{}", &entry.path().display());
                let content = fs::read_to_string(&entry.path())?;
                if tags
                    .iter()
                    .any(|tag| content.contains(&format!("#{}", tag)))
                {
                    println!("{}", entry.path().display())
                }
                // TODO ideas:
                // print the paths without the user prefix
                // optionally use a pager (minus) to display them all
                // how would this output pipe into another program?
            }
            Ok(())
        }
        Command::Publish { output } => {
            for entry in WalkDir::new(&garden_path)
                .into_iter()
                .filter_map(|e| e.ok().and_then(|e2| e2.path().is_file().then_some(e2)))
            {
                let file_name = &entry.path().strip_prefix(&garden_path).unwrap();
                let content = fs::read_to_string(&entry.path())?;

                // Set up options and parser. Strikethroughs are not part of the CommonMark standard
                // and we therefore must enable it explicitly.
                let mut options = Options::empty();
                options.insert(Options::ENABLE_STRIKETHROUGH);
                let parser = Parser::new_ext(&content, options)
                    .scan(None, |state: &mut Option<String>, v| match v {
                        Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(block))) => {
                            *state = Some(block.to_string());
                            Some(None)
                        }
                        Event::End(Tag::CodeBlock(CodeBlockKind::Fenced(block))) => {
                            *state = None;
                            Some(None)
                        }
                        Event::Text(text) => match state {
                            Some(lang) => {
                                let html = highlight(&*text, lang);
                                Some(Some(Event::Html(CowStr::Boxed(Box::from(html)))))
                            }
                            None => Some(Some(Event::Text(text))),
                        },
                        _ => Some(Some(v)),
                    })
                    .filter_map(|v| v);

                // Write to String buffer.
                let mut html_output = String::new();
                html::push_html(&mut html_output, parser);
                let mut output_file = output.join(file_name);
                output_file.set_extension("html");
                fs::write(
                    output_file,
                    format!(
                        "{}{}{}",
                        HTML_TEMPLATE_START, html_output, HTML_TEMPLATE_END
                    ),
                );
            }
            Ok(())
        }
        Command::Read { preview } => {
            let files = WalkDir::new(&garden_path)
                .into_iter()
                .filter_map(|e| e.ok().and_then(|e2| e2.path().is_file().then_some(e2)))
                .map(|e| e.path().to_path_buf())
                .collect::<Vec<std::path::PathBuf>>();
            let items_for_display = files
                .iter()
                .map(|p| p.file_name().unwrap().to_string_lossy())
                .collect::<Vec<std::borrow::Cow<'_, str>>>();
            let selection = Select::with_theme(&ColorfulTheme::default())
                .items(&items_for_display)
                .default(0)
                .interact_on_opt(&Term::stderr())?;

            match (selection, preview) {
                (Some(index), false) => {
                    let ss = SyntaxSet::load_defaults_newlines();
                    let ts = ThemeSet::load_defaults();

                    let mut highlighter =
                        HighlightFile::new(&files[index], &ss, &ts.themes["Solarized (dark)"])
                            .unwrap();
                    let mut line = String::new();
                    while highlighter.reader.read_line(&mut line)? > 0 {
                        {
                            let regions: Vec<(Style, &str)> =
                                highlighter.highlight_lines.highlight(&line, &ss);
                            print!("{}", as_24_bit_terminal_escaped(&regions[..], false));
                        } // until NLL this scope is needed so we can clear the buffer after
                        line.clear(); // read_line appends so we need to clear between lines
                    }
                }
                (Some(index), true) => {}
                _ => println!("User did not select anything"),
            }
            Ok(())
        }
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
