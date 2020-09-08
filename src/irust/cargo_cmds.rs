use crate::utils::stdout_and_stderr;
use once_cell::sync::Lazy;
use std::env::temp_dir;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;

// TODO:
// Move these paths to KnownPaths struct
pub static TMP_DIR: Lazy<PathBuf> = Lazy::new(temp_dir);
pub static IRUST_DIR: Lazy<PathBuf> = Lazy::new(|| TMP_DIR.join("irust"));
pub static CARGO_TOML_FILE: Lazy<PathBuf> = Lazy::new(|| IRUST_DIR.join("Cargo.toml"));
pub static IRUST_SRC_DIR: Lazy<PathBuf> = Lazy::new(|| IRUST_DIR.join("src"));
pub static MAIN_FILE: Lazy<PathBuf> = Lazy::new(|| IRUST_SRC_DIR.join("main.rs"));
#[cfg(windows)]
pub static EXE_PATH: Lazy<PathBuf> = Lazy::new(|| IRUST_DIR.join("target/debug/irust.exe"));
#[cfg(not(windows))]
pub static EXE_PATH: Lazy<PathBuf> = Lazy::new(|| IRUST_DIR.join("target/debug/irust"));

pub fn cargo_new() -> Result<(), io::Error> {
    let _ = std::fs::create_dir_all(&*IRUST_SRC_DIR);
    clean_cargo_toml()?;
    clean_main_file()?;

    cargo_build()?.wait()?;
    Ok(())
}

pub fn cargo_run(color: bool) -> Result<String, io::Error> {
    let output = cargo_build_output(color)?;

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

pub fn cargo_build() -> Result<std::process::Child, io::Error> {
    Ok(Command::new("cargo")
        // the difference in env flags makes cargo recompiles again!!!
        // => make  sure all build env flags are the same
        .env("RUSTFLAGS", "-Awarnings")
        .current_dir(&*IRUST_DIR)
        .arg("build")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?)
}

pub fn cargo_build_output(color: bool) -> Result<String, io::Error> {
    let color = if color { "always" } else { "never" };

    Ok(stdout_and_stderr(
        Command::new("cargo")
            .current_dir(&*IRUST_DIR)
            .arg("build")
            .args(&["--color", color])
            .env("RUSTFLAGS", "-Awarnings")
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
    Ok(())
}

pub fn cargo_fmt(c: &str) -> std::io::Result<String> {
    let fmt_path = IRUST_DIR.join("fmt_file");
    let _ = fs::remove_file(&fmt_path);

    let mut fmt_file = fs::OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(&fmt_path)?;

    write!(fmt_file, "{}", c)?;

    std::process::Command::new("rustfmt")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .arg(&fmt_path)
        .spawn()?
        .wait()?;

    let mut fmt_c = String::new();
    fmt_file.seek(std::io::SeekFrom::Start(0))?;
    fmt_file.read_to_string(&mut fmt_c)?;

    Ok(fmt_c)
}

pub fn cargo_fmt_file(file: &PathBuf) -> io::Result<()> {
    std::process::Command::new("rustfmt")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .arg(file)
        .spawn()?
        .wait()?;
    Ok(())
}
