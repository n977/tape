use clap::error::ErrorKind;
use clap::error::Result;
use clap::Error;
use std::path::PathBuf;

pub fn expand_path(path: &str) -> std::io::Result<PathBuf> {
    std::fs::canonicalize(path)
}

pub fn parse_prop(s: &str) -> Result<(String, String)> {
    let prop = s.split_once('=');

    if let Some((key, value)) = prop {
        Ok((key.into(), value.into()))
    } else {
        Err(Error::new(ErrorKind::TooFewValues))
    }
}
