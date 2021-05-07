pub mod buffer;
pub mod printer;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
