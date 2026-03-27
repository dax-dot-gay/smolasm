use std::{num::ParseIntError, path::Path};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error(
        "Config file not found ({0}). Specify the -c/--config flag to specify an alternate config file."
    )]
    NotFound(String),

    #[error("Configuration syntax invalid: {0}")]
    Syntax(#[from] kdl::KdlError),

    #[error("Invalid configuration content: {0}")]
    InvalidContent(String),
}

impl ConfigError {
    pub fn not_found(file: impl AsRef<Path>) -> Self {
        Self::NotFound(file.as_ref().to_string_lossy().to_string())
    }

    pub fn invalid_content(reason: impl Into<String>) -> Self {
        Self::InvalidContent(reason.into())
    }
}

#[derive(Error, Debug)]
pub enum ParsingError {
    #[error("Error parsing numerical value: {0:?}")]
    ParseInt(#[from] ParseIntError),

    #[error("Error parsing hex value: {0:?}")]
    ParseHex(#[from] hex::FromHexError),

    #[error("Syntax error parsing assembly input: {0}")]
    Syntax(String),

    #[error("Unknown instruction \"{data}\": {reason}")]
    UnknownInstruction {data: String, reason: String},

    #[error("Address {0:#x} has already been allocated!")]
    Allocation(u64)

}

impl ParsingError {
    pub fn syntax(err: impl Into<String>) -> Self {
        Self::Syntax(err.into())
    }

    pub fn unknown(inst: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::UnknownInstruction { data: inst.into(), reason: reason.into() }
    }
}

#[derive(Error, Debug)]
pub enum SmolError {
    #[error("An unhandled error occurred:\n\t:: {0:?}")]
    Unknown(#[from] anyhow::Error),

    #[error("Configuration error:\n\t:: {0}")]
    Configuration(ConfigError),

    #[error("Assembly error on line {line}:\n\t:: {err}")]
    AssemblyError {
        line: u64,
        err: ParsingError
    },

    #[error("Input file {0} not found!")]
    NotFound(String)
}

impl<T: Into<ConfigError>> From<T> for SmolError {
    fn from(value: T) -> Self {
        Self::Configuration(value.into())
    }
}

pub type Result<T> = std::result::Result<T, SmolError>;
