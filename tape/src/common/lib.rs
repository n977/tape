mod cli;
pub mod logger;

pub use tape_core as core;
pub use tape_core::*;

use anyhow::{Context, Result};
use clap::{Subcommand, ValueHint};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub fn runtime_dir() -> Result<PathBuf> {
    let mut path = dirs::runtime_dir().context("failed to determine runtime directory")?;
    path.push("tape");
    Ok(path)
}

pub fn socket_path() -> Result<PathBuf> {
    let mut path = runtime_dir()?;
    path.push("tape");
    path.set_extension("sock");
    Ok(path)
}

#[derive(Subcommand, Serialize, Deserialize)]
pub enum Request {
    /// Add track(s) to queue
    Add {
        /// Paths of the track(s) to add
        #[arg(value_hint = ValueHint::AnyPath, value_parser=cli::expand_path)]
        paths: Vec<PathBuf>,
    },
    /// Remove track(s) from queue
    Remove {
        /// Indice(s) of the track(s) to remove
        ids: Vec<usize>,
    },
    /// Configure playback at runtime
    Config {
        /// Key-value property pairs separated by the '=' sign
        ///
        /// Possible properties are:
        /// repeat-mode=[disabled, track, playlist]     Should player repeat track(s) and how
        #[arg(value_name = "PROPERTY", short = 'p', long = "property", value_parser = cli::parse_prop, verbatim_doc_comment)]
        props: Vec<(String, String)>,
    },
    /// Seek track currently playing
    Seek {
        /// Timestamp in seconds passed since the beginning of a track
        #[arg(value_name = "TIMESTAMP")]
        t: u64,
    },
    /// Select track from current playlist
    Jump {
        /// Index of the track to select
        #[arg(value_name = "POSITION", allow_negative_numbers = true)]
        pos: isize,
        /// Interpret the value as relative to the currently playing track position
        #[arg(short = 'r', long = "relative")]
        relative: bool,
    },
    /// Continue playback
    Play,
    /// Stop playback
    Pause,
}
