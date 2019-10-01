use crate::utils::stdout_and_stderr;
use once_cell::sync::Lazy;
use std::env::temp_dir;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;

pub static TMP_DIR: Lazy<PathBuf> = Lazy::new(temp_dir);
pub static IRUST_DIR: Lazy<PathBuf> = Lazy::new(|| TMP_DIR.join("irust"));
pub static MAIN_FILE: Lazy<PathBuf> = Lazy::new(|| IRUST_DIR.join("src/main.rs"));

pub fn cargo_new() -> Result<(), io::Error> {
    let _ = std::fs::remove_dir_all(&*IRUST_DIR);

    Command::new("cargo")
        .current_dir(&*TMP_DIR)
        .args(&["new", "irust"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?
        .wait()?;

    clean_main_file()?;
    cargo_build()?.wait()?;
    Ok(())
}

pub fn cargo_run(color: bool) -> Result<String, io::Error> {
    let color = if color { "always" } else { "never" };

    Ok(stdout_and_stderr(
        Command::new("cargo")
            .current_dir(&*IRUST_DIR)
            .args(&["run", "--color", color])
            .env("RUSTFLAGS", "-Awarnings")
            .output()?,
    ))
}

pub fn cargo_add(dep: &[String]) -> io::Result<std::process::Child> {
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
        .current_dir(&*IRUST_DIR)
        .arg("build")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?)
}

fn clean_main_file() -> io::Result<()> {
    let mut main = fs::File::create(&*MAIN_FILE)?;
    let main_src = "fn main() {\n\n}";
    write!(main, "{}", main_src)?;
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
