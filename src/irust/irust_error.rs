use std::io;

use IRustError::*;

#[derive(Debug)]
pub enum IRustError {
    IoError(io::Error),
    CrosstermError(crossterm::ErrorKind),
    Custom(String),
    RacerDisabled,
    ParsingError(String),
}

impl From<io::Error> for IRustError {
    fn from(error: io::Error) -> Self {
        IRustError::IoError(error)
    }
}

impl From<&Self> for IRustError {
    fn from(error: &Self) -> Self {
        match error {
            RacerDisabled => RacerDisabled,
            _ => Custom(error.to_string()),
        }
    }
}

impl From<&mut Self> for IRustError {
    fn from(error: &mut Self) -> Self {
        match error {
            RacerDisabled => RacerDisabled,
            _ => Custom(error.to_string()),
        }
    }
}

impl From<&str> for IRustError {
    fn from(error: &str) -> Self {
        Custom(error.to_string())
    }
}

impl From<crossterm::ErrorKind> for IRustError {
    fn from(error: crossterm::ErrorKind) -> Self {
        IRustError::CrosstermError(error)
    }
}

impl From<toml::de::Error> for IRustError {
    fn from(error: toml::de::Error) -> Self {
        IRustError::ParsingError(error.to_string())
    }
}

impl From<toml::ser::Error> for IRustError {
    fn from(error: toml::ser::Error) -> Self {
        IRustError::ParsingError(error.to_string())
    }
}

impl ToString for IRustError {
    fn to_string(&self) -> String {
        match self {
            IoError(e) => e.to_string(),
            CrosstermError(e) => e.to_string(),
            Custom(e) => e.to_string(),
            RacerDisabled => "Racer is disabled".to_string(),
            ParsingError(e) => e.to_string(),
        }
    }
}
