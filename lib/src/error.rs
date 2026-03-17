use std::result::Result as StdResult;

use thiserror::Error;

pub type Result<T> = StdResult<T, Error>;

/// An enum for describing and handling various errors encountered while dealing
/// with `clog` building, or writing of changelogs.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Found unknown component '{0}' that does not correspond to a Changelog Section")]
    UnknownComponent(String),

    #[error("cannot get current directory")]
    CurrentDir,

    #[error("fatal I/O error with output file")]
    Io(#[from] std::io::Error),

    #[error("failed to convert date/time to string format")]
    TimeFormat(#[from] time::error::Format),

    #[error("failed to convert date/time to string format")]
    Time(#[from] time::Error),

    #[error("Failed to parse TOML configuration file")]
    Toml(#[from] toml::de::Error),
}
