use std::{fmt::Display, str::FromStr};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
#[derive(Default)]
pub enum MainResult {
    /// fn main() -> () {()}
    #[default]
    Unit,
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {Ok(())}
    /// allows using `?` with no boilerplate
    Result,
}

impl MainResult {
    pub(crate) fn ttype(&self) -> &'static str {
        match self {
            Self::Unit => "()",
            Self::Result => "Result<(), Box<dyn std::error::Error>>",
        }
    }
    pub(crate) fn instance(&self) -> &'static str {
        match self {
            Self::Unit => "()",
            Self::Result => "Ok(())",
        }
    }
}


impl FromStr for MainResult {
    type Err = Box<dyn std::error::Error>;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "unit" => Ok(MainResult::Unit),
            "result" => Ok(MainResult::Result),
            _ => Err("Unknown main result type".into()),
        }
    }
}

impl Display for MainResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MainResult::Unit => write!(f, "Unit"),
            MainResult::Result => write!(f, "Result<(), Box<dyn std::error::Error>>"),
        }
    }
}
