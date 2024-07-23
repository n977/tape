use clap::{ColorChoice, Parser};
use tape::Request;

#[derive(Parser)]
#[command(color = ColorChoice::Never)]
#[clap(about = "A terminal audio player")]
pub struct Cli {
    #[command(subcommand)]
    pub req: Request,
}
