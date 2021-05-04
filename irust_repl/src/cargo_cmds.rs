use crate::utils::{stdout_and_stderr, ProcessUtils};
use crate::Result;
use once_cell::sync::Lazy;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::{env::temp_dir, process::Stdio};
use std::{fs, process};
use std::{io, str::FromStr};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub static TMP_DIR: Lazy<PathBuf> = Lazy::new(temp_dir);
pub static IRUST_DIR: Lazy<PathBuf> = Lazy::new(|| TMP_DIR.join("irust_host_repl"));
pub static IRUST_TARGET_DIR: Lazy<PathBuf> = Lazy::new(|| {
    if let Ok(p) = std::env::var("CARGO_TARGET_DIR") {
        if !p.is_empty() {
            return Path::new(&p).to_path_buf();
        }
    }
    IRUST_DIR.join("target")
});
pub static CARGO_TOML_FILE: Lazy<PathBuf> = Lazy::new(|| IRUST_DIR.join("Cargo.toml"));
pub static IRUST_SRC_DIR: Lazy<PathBuf> = Lazy::new(|| IRUST_DIR.join("src"));
pub static MAIN_FILE: Lazy<PathBuf> = Lazy::new(|| IRUST_SRC_DIR.join("main.rs"));
pub static MAIN_FILE_EXTERN: Lazy<PathBuf> = Lazy::new(|| IRUST_SRC_DIR.join("main_extern.rs"));
pub static LIB_FILE: Lazy<PathBuf> = Lazy::new(|| IRUST_SRC_DIR.join("lib.rs"));
#[cfg(windows)]
pub static EXE_PATH: Lazy<PathBuf> =
    Lazy::new(|| IRUST_TARGET_DIR.join("debug/irust_host_repl.exe"));
#[cfg(windows)]
pub static RELEASE_EXE_PATH: Lazy<PathBuf> =
    Lazy::new(|| IRUST_TARGET_DIR.join("release/irust_host_repl.exe"));
#[cfg(not(windows))]
pub static EXE_PATH: Lazy<PathBuf> = Lazy::new(|| IRUST_TARGET_DIR.join("debug/irust_host_repl"));
#[cfg(not(windows))]
pub static RELEASE_EXE_PATH: Lazy<PathBuf> =
    Lazy::new(|| IRUST_TARGET_DIR.join("release/irust_host_repl"));

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
            _ => Err("Unkown toolchain".into()),
        }
    }
}

impl ToolChain {
    fn as_arg(&self) -> String {
        use ToolChain::*;
        match self {
            Stable => "+stable".to_string(),
            Beta => "+beta".to_string(),
            Nightly => "+nightly".to_string(),
        }
    }
}

pub fn cargo_new(toolchain: ToolChain) -> std::result::Result<(), io::Error> {
    // Ignore directory exists error
    let _ = std::fs::create_dir_all(&*IRUST_SRC_DIR);
    clean_cargo_toml()?;
    clean_files()?;

    cargo_build(toolchain)?.wait()?;
    Ok(())
}

pub fn cargo_run(
    color: bool,
    release: bool,
    toolchain: ToolChain,
    interactive_function: Option<fn(&mut process::Child) -> Result<()>>,
) -> Result<(ExitStatus, String)> {
    let (status, output) = cargo_build_output(color, release, toolchain)?;

    if !status.success() {
        Ok((status, output))
    } else {
        // Run the exexcutable directly instead of cargo run
        // This allows to run it without modifying the current working directory
        // example: std::process::Commmand::new("pwd") will output the expected path instead of `/tmp/irust_host_repl`
        if !release {
            Ok((
                status,
                stdout_and_stderr(
                    std::process::Command::new(&*EXE_PATH)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn()?
                        .interactive_output(interactive_function)?,
                ),
            ))
        } else {
            Ok((
                status,
                stdout_and_stderr(
                    std::process::Command::new(&*RELEASE_EXE_PATH)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn()?
                        .interactive_output(interactive_function)?,
                ),
            ))
        }
    }
}

pub fn cargo_add(dep: &[String]) -> io::Result<std::process::Child> {
    Command::new("cargo-add")
        .current_dir(&*IRUST_DIR)
        .arg("add")
        .args(dep)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
}

macro_rules! cargo_common {
    // The difference in env flags makes cargo recompiles again!!!
    // => make  sure all build env flags are the same
    // Or even better dont use any
    ($cmd: literal, $toolchain: ident) => {
        Command::new("cargo")
            .arg($toolchain.as_arg())
            .arg($cmd)
            .env("CARGO_TARGET_DIR", &*IRUST_TARGET_DIR)
            //.env("RUSTFLAGS", "-Awarnings") // Not required anymore
            .current_dir(&*IRUST_DIR)
    };
}

pub fn cargo_check(toolchain: ToolChain) -> std::result::Result<std::process::Child, io::Error> {
    cargo_common!("check", toolchain)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
}

pub fn cargo_check_output(
    toolchain: ToolChain,
) -> std::result::Result<(ExitStatus, String), io::Error> {
    let output = cargo_common!("check", toolchain)
        .args(&["--color", "always"])
        .output()?;

    let status = output.status;
    Ok((status, stdout_and_stderr(output)))
}

pub fn cargo_build(toolchain: ToolChain) -> std::result::Result<std::process::Child, io::Error> {
    cargo_common!("build", toolchain)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
}

pub fn cargo_build_output(
    color: bool,
    release: bool,
    toolchain: ToolChain,
) -> std::result::Result<(ExitStatus, String), io::Error> {
    let color = if color { "always" } else { "never" };

    let output = if !release {
        cargo_common!("build", toolchain)
            .args(&["--color", color])
            .output()?
    } else {
        cargo_common!("build", toolchain)
            .arg("--release")
            .args(&["--color", color])
            .output()?
    };
    let status = output.status;

    Ok((status, stdout_and_stderr(output)))
}

pub fn cargo_bench(toolchain: ToolChain) -> std::result::Result<String, io::Error> {
    Ok(stdout_and_stderr(
        cargo_common!("bench", toolchain)
            .args(&["--color", "always"])
            .output()?,
    ))
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

pub fn cargo_asm(fnn: &str, toolchain: ToolChain) -> Result<String> {
    Ok(stdout_and_stderr(
        cargo_common!("asm", toolchain)
            .arg("--lib")
            .arg(format!("irust_host_repl::{}", fnn))
            .output()?,
    ))
}

pub fn cargo_fmt_file(file: &Path) {
    // Cargo fmt is optional
    let _ = try_cargo_fmt_file(file);
}

fn try_cargo_fmt_file(file: &Path) -> io::Result<()> {
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

fn clean_cargo_toml() -> io::Result<()> {
    // edition needs to be specified or racer will not be able to autocomplete dependencies
    // bug maybe?
    const CARGO_TOML: &str = r#"[package]
name = "irust_host_repl"
version = "0.1.0"
edition = "2018""#;
    let mut cargo_toml_file = fs::File::create(&*CARGO_TOML_FILE)?;
    write!(cargo_toml_file, "{}", CARGO_TOML)?;
    Ok(())
}

fn clean_files() -> io::Result<()> {
    const MAIN_SRC: &str = "fn main() {\n\n}";
    let mut main = fs::File::create(&*MAIN_FILE)?;
    write!(main, "{}", MAIN_SRC)?;
    std::fs::copy(&*MAIN_FILE, &*MAIN_FILE_EXTERN)?;
    let _ = std::fs::remove_file(&*LIB_FILE);
    Ok(())
}
