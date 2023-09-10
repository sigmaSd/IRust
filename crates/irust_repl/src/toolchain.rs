#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};
use anyhow::anyhow;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, Default)]
pub enum ToolChain {
    Stable,
    Beta,
    Nightly,
    // cargo with no +argument, it can be different from the above
    #[default]
    Default,
}

impl FromStr for ToolChain {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "stable" => Ok(ToolChain::Stable),
            "beta" => Ok(ToolChain::Beta),
            "nightly" => Ok(ToolChain::Nightly),
            "default" => Ok(ToolChain::Default),
            _ => Err(anyhow!("Unknown toolchain")),
        }
    }
}

impl ToolChain {
    pub(crate) fn as_arg(&self) -> &str {
        match self {
            ToolChain::Stable => "+stable",
            ToolChain::Beta => "+beta",
            ToolChain::Nightly => "+nightly",
            // The caller should not call as_arg for the default toolchain
            ToolChain::Default => unreachable!(),
        }
    }
}

impl Display for ToolChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolChain::Stable => write!(f, "stable"),
            ToolChain::Beta => write!(f, "beta"),
            ToolChain::Nightly => write!(f, "nightly"),
            ToolChain::Default => write!(f, "default"),
        }
    }
}
