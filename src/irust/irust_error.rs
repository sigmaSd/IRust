use std::io;

use IRustError::*;

#[derive(Debug)]
pub enum IRustError {
    IoError(io::Error),
    CrosstermError(crossterm::ErrorKind),
    Custom(String),
    Ignore,
}

impl From<io::Error> for IRustError {
    fn from(error: io::Error) -> Self {
        IRustError::IoError(error)
    }
}

impl From<&io::Error> for IRustError {
    fn from(error: &io::Error) -> Self {
        IRustError::IoError(io::Error::new(error.kind(), error.to_string()))
    }
}

impl From<&mut io::Error> for IRustError {
    fn from(error: &mut io::Error) -> Self {
        IRustError::IoError(io::Error::new(error.kind(), error.to_string()))
    }
}

impl From<crossterm::ErrorKind> for IRustError {
    fn from(error: crossterm::ErrorKind) -> Self {
        IRustError::CrosstermError(error)
    }
}

impl ToString for IRustError {
    fn to_string(&self) -> String {
        match self {
            IoError(e) => e.to_string(),
            CrosstermError(e) => e.to_string(),
            Custom(e) => e.to_string(),
            Ignore => "".to_string(),
        }
    }
}
