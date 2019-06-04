use std::io;

pub enum IRustError {
    IoError(io::Error),
    CrosstermError(crossterm::ErrorKind),
}

impl From<io::Error> for IRustError {
    fn from(error: io::Error) -> Self {
        IRustError::IoError(error)
    }
}

impl From<&io::Error> for IRustError {
    fn from(error: &io::Error) -> Self {
        IRustError::IoError(*error)
    }
}

impl From<&mut io::Error> for IRustError {
    fn from(error: &mut io::Error) -> Self {
        IRustError::IoError(*error)
    }
}

impl From<crossterm::ErrorKind> for IRustError {
    fn from(error: crossterm::ErrorKind) -> Self {
        IRustError::CrosstermError(error)
    }
}
