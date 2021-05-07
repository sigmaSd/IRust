#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub enum ToolChain {
    Stable,
    Beta,
    Nightly,
}

impl FromStr for ToolChain {
    type Err = Box<dyn std::error::Error>;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        use ToolChain::*;
        match s.to_lowercase().as_str() {
            "stable" => Ok(Stable),
            "beta" => Ok(Beta),
            "nightly" => Ok(Nightly),
            _ => Err("Unknown toolchain".into()),
        }
    }
}

impl ToolChain {
    pub(crate) fn as_arg(&self) -> String {
        use ToolChain::*;
        match self {
            Stable => "+stable".to_string(),
            Beta => "+beta".to_string(),
            Nightly => "+nightly".to_string(),
        }
    }
}

impl Display for ToolChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ToolChain::*;
        match self {
            Stable => write!(f, "stable"),
            Beta => write!(f, "beta"),
            Nightly => write!(f, "nightly"),
        }
    }
}
