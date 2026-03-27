use std::path::Path;

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
pub enum SmolError {
    #[error("An unhandled error occurred:\n\t:: {0:?}")]
    Unknown(#[from] anyhow::Error),

    #[error("Configuration error:\n\t:: {0}")]
    Configuration(ConfigError),
}

impl<T: Into<ConfigError>> From<T> for SmolError {
    fn from(value: T) -> Self {
        Self::Configuration(value.into())
    }
}

pub type Result<T> = std::result::Result<T, SmolError>;
