#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use anyhow::anyhow;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, Default)]
pub enum Executor {
    #[default]
    Sync,
    Tokio,
    AsyncStd,
}

impl Executor {
    pub(crate) fn main(&self) -> String {
        match self {
            Executor::Sync => "fn main()".into(),
            Executor::Tokio => "#[tokio::main]async fn main()".into(),
            Executor::AsyncStd => "#[async_std::main]async fn main()".into(),
        }
    }
    /// Invokation that can be used with cargo-add
    /// The first argument is the crate name, it should be used with cargo-rm
    pub(crate) fn dependecy(&self) -> Option<Vec<String>> {
        match self {
            Executor::Sync => None,
            Executor::Tokio => Some(vec![
                "tokio".into(),
                "--features".into(),
                "macros rt-multi-thread".into(),
            ]),
            Executor::AsyncStd => Some(vec![
                "async_std".into(),
                "--features".into(),
                "attributes".into(),
            ]),
        }
    }
}
impl FromStr for Executor {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "sync" => Ok(Executor::Sync),
            "tokio" => Ok(Executor::Tokio),
            "async_std" => Ok(Executor::AsyncStd),
            _ => Err(anyhow!("Unknown executor"))
        }
    }
}

impl std::fmt::Display for Executor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Executor::Sync => write!(f, "sync"),
            Executor::Tokio => write!(f, "tokio"),
            Executor::AsyncStd => write!(f, "async_std"),
        }
    }
}
