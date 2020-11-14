use super::IRustError;
use crate::utils::stdout_and_stderr;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::env::temp_dir;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;

// TODO:
// Move these paths to KnownPaths struct
pub static TMP_DIR: Lazy<PathBuf> = Lazy::new(|| dirs_next::cache_dir().unwrap_or_else(temp_dir));
pub static IRUST_DIR: Lazy<PathBuf> = Lazy::new(|| TMP_DIR.join("irust"));
pub static IRUST_TARGET_DIR: Lazy<PathBuf> = Lazy::new(|| IRUST_DIR.join("target"));
pub static CARGO_TOML_FILE: Lazy<PathBuf> = Lazy::new(|| IRUST_DIR.join("Cargo.toml"));
pub static IRUST_SRC_DIR: Lazy<PathBuf> = Lazy::new(|| IRUST_DIR.join("src"));
pub static MAIN_FILE: Lazy<PathBuf> = Lazy::new(|| IRUST_SRC_DIR.join("main.rs"));
pub static MAIN_FILE_EXTERN: Lazy<PathBuf> = Lazy::new(|| IRUST_SRC_DIR.join("main_extern.rs"));
#[cfg(windows)]
pub static EXE_PATH: Lazy<PathBuf> = Lazy::new(|| IRUST_DIR.join("target/debug/irust.exe"));
#[cfg(not(windows))]
pub static EXE_PATH: Lazy<PathBuf> = Lazy::new(|| IRUST_DIR.join("target/debug/irust"));

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub enum ToolChain {
    Stable,
    Beta,
    Nightly,
}

impl ToolChain {
    pub fn from_str(s: &str) -> Result<Self, IRustError> {
        use ToolChain::*;
        match s.to_lowercase().as_str() {
            "stable" => Ok(Stable),
            "beta" => Ok(Beta),
            "nightly" => Ok(Nightly),
            _ => Err("Unkown toolchain".into()),
        }
    }

    fn as_arg(&self) -> String {
        use ToolChain::*;
        match self {
            Stable => "+stable".to_string(),
            Beta => "+beta".to_string(),
            Nightly => "+nightly".to_string(),
        }
    }
}

pub fn cargo_new(toolchain: ToolChain) -> Result<(), io::Error> {
    // Ignore directory exists error
    let _ = std::fs::create_dir_all(&*IRUST_SRC_DIR);
    clean_cargo_toml()?;
    clean_main_file()?;

    cargo_build(toolchain)?.wait()?;
    Ok(())
}

pub fn cargo_run(color: bool, toolchain: ToolChain) -> Result<String, io::Error> {
    let output = cargo_build_output(color, toolchain)?;

    if super::format::output_is_err(&output) {
        Ok(output)
    } else {
        // Run the exexcutable directly instead of cargo run
        // This allows to run it without modifying the current working directory
        // example: std::process::Commmand::new("pwd") will output the expected path instead of `/tmp/irust`
        Ok(stdout_and_stderr(
            std::process::Command::new(&*EXE_PATH).output()?,
        ))
    }
}

pub fn cargo_add(dep: &[String]) -> io::Result<std::process::Child> {
    //TODO is this required?
    clean_main_file()?;

    Ok(Command::new("cargo-add")
        .current_dir(&*IRUST_DIR)
        .arg("add")
        .args(dep)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()?)
}

macro_rules! cargo_common {
    // The difference in env flags makes cargo recompiles again!!!
    // => make  sure all build env flags are the same
    //
    // Make sure to specify CARGO_TARGET_DIR to overwrite custom user one (in case it's set)
    ($cmd: literal, $toolchain: ident) => {
        Command::new("cargo")
            .arg($toolchain.as_arg())
            .arg($cmd)
            .env("CARGO_TARGET_DIR", &*IRUST_TARGET_DIR)
            .env("RUSTFLAGS", "-Awarnings")
            .current_dir(&*IRUST_DIR)
    };
}

pub fn cargo_check(toolchain: ToolChain) -> Result<std::process::Child, io::Error> {
    Ok(cargo_common!("check", toolchain)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?)
}

pub fn cargo_check_output(toolchain: ToolChain) -> Result<String, io::Error> {
    #[cfg(not(windows))]
    let color = "always";
    #[cfg(windows)]
    let color = if crossterm::ansi_support::supports_ansi() {
        "always"
    } else {
        "never"
    };

    Ok(stdout_and_stderr(
        cargo_common!("check", toolchain)
            .args(&["--color", color])
            .output()?,
    ))
}

pub fn cargo_build(toolchain: ToolChain) -> Result<std::process::Child, io::Error> {
    Ok(cargo_common!("build", toolchain)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?)
}

pub fn cargo_build_output(color: bool, toolchain: ToolChain) -> Result<String, io::Error> {
    #[cfg(not(windows))]
    let color = if color { "always" } else { "never" };
    #[cfg(windows)]
    let color = if crossterm::ansi_support::supports_ansi() {
        if color {
            "always"
        } else {
            "never"
        }
    } else {
        "never"
    };

    Ok(stdout_and_stderr(
        cargo_common!("build", toolchain)
            .args(&["--color", color])
            .output()?,
    ))
}

fn clean_cargo_toml() -> io::Result<()> {
    // edition needs to be specified or racer will not be able to autocomplete dependencies
    // bug maybe?
    const CARGO_TOML: &str = r#"[package]
name = "irust"
version = "0.1.0"
edition = "2018""#;
    let mut cargo_toml_file = fs::File::create(&*CARGO_TOML_FILE)?;
    write!(cargo_toml_file, "{}", CARGO_TOML)?;
    Ok(())
}

fn clean_main_file() -> io::Result<()> {
    const MAIN_SRC: &str = "fn main() {\n\n}";
    let mut main = fs::File::create(&*MAIN_FILE)?;
    write!(main, "{}", MAIN_SRC)?;
    std::fs::copy(&*MAIN_FILE, &*MAIN_FILE_EXTERN)?;
    Ok(())
}

pub fn cargo_fmt(c: &str) -> std::io::Result<String> {
    let fmt_path = IRUST_DIR.join("fmt_file");
    // Ignore file doesn't exist error
    let _ = fs::remove_file(&fmt_path);

    let mut fmt_file = fs::OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(&fmt_path)?;

    write!(fmt_file, "{}", c)?;

    cargo_fmt_file(&fmt_path);

    let mut fmt_c = String::new();
    fmt_file.seek(std::io::SeekFrom::Start(0))?;
    fmt_file.read_to_string(&mut fmt_c)?;

    Ok(fmt_c)
}

pub fn cargo_fmt_file(file: &PathBuf) {
    // Cargo fmt is optional
    let _ = try_cargo_fmt_file(file);
}

fn try_cargo_fmt_file(file: &PathBuf) -> io::Result<()> {
    std::process::Command::new("rustfmt")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        // ensure that main is always spread on two lines
        // this is needed for inserting the input correctly in the repl
        // fn main() {
        // }
        .arg("--config")
        .arg("empty_item_single_line=false")
        .arg(file)
        .spawn()?
        .wait()?;
    Ok(())
}
