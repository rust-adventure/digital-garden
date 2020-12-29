use color_eyre::eyre::Result;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(subcommand)]
    cmd: Command,
}
#[derive(StructOpt, Debug)]
enum Command {
    Write {
        #[structopt(short, long)]
        title: Option<String>,
    },
}
fn main() -> Result<()> {
    color_eyre::install()?;

    let opt = Opt::from_args();
    dbg!(opt);
    todo!()
}
