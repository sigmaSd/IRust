#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use anyhow::anyhow;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CompileMode {
    Debug,
    Release,
}
impl CompileMode {
    pub fn is_release(&self) -> bool {
        matches!(self, Self::Release)
    }
}

impl FromStr for CompileMode {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "debug" => Ok(CompileMode::Debug),
            "release" => Ok(CompileMode::Release),
            _ => Err(anyhow!("Unknown compile mode"))
        }
    }
}

impl std::fmt::Display for CompileMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileMode::Debug => write!(f, "Debug"),
            CompileMode::Release => write!(f, "Release"),
        }
    }
}
