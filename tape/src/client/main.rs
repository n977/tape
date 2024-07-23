mod cli;

use crate::cli::Cli;
use anyhow::{Context, Result};
use clap::Parser;
use std::io::Write;
use std::os::unix::net::UnixStream;
use tracing::error;

fn main() {
    if let Err(e) = run() {
        error!("{:#}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    tape::logger::init()?;

    let cli = Cli::parse();
    let path = tape::socket_path()?;
    let mut con = UnixStream::connect(&path)
        .with_context(|| format!("failed to connect to socket at {}", path.display()))?;

    let req = serde_json::to_string(&cli.req)?;
    con.write_all(req.as_bytes())?;

    Ok(())
}
