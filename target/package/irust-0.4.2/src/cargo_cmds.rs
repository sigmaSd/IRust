use std::env::temp_dir;
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::utils::stdout_and_stderr;

#[derive(Clone)]
pub struct CargoCmds {
    tmp_dir: PathBuf,
    irust_dir: PathBuf,
    pub main_file: PathBuf,
}
impl Default for CargoCmds {
    fn default() -> Self {
        let tmp_dir = temp_dir();
        let irust_dir = {
            let mut dir = tmp_dir.clone();
            dir.push("irust");
            dir
        };
        let main_file = {
            let mut dir = irust_dir.clone();
            dir.push("src/main.rs");
            dir
        };
        Self {
            tmp_dir,
            irust_dir,
            main_file,
        }
    }
}
impl CargoCmds {
    pub fn cargo_new(&self) -> Result<(), io::Error> {
        self.clean_toml();
        if Path::new(&self.irust_dir).exists() {
            std::fs::remove_dir_all(&self.irust_dir)?;
        }
        let _ = Command::new("cargo")
            .current_dir(&*self.tmp_dir)
            .args(&["new", "irust"])
            .output();
        self.cargo_build()?.wait()?;
        Ok(())
    }

    pub fn cargo_run(&self) -> Result<String, io::Error> {
        Ok(stdout_and_stderr(
            Command::new("cargo")
                .current_dir(&*self.irust_dir)
                .args(&["run", "--color", "always"])
                .output()?,
        ))
    }

    pub fn cargo_add(&self, dep: &[String]) -> io::Result<std::process::Child> {
        self.soft_clean()?;

        Ok(Command::new("cargo")
            .current_dir(&*self.irust_dir)
            .arg("add")
            .args(dep)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?)
    }

    pub fn cargo_build(&self) -> Result<std::process::Child, io::Error> {
        Ok(Command::new("cargo")
            .current_dir(&*self.irust_dir)
            .arg("build")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?)
    }

    fn clean_toml(&self) {
        use std::fs::File;
        use std::io::Read;

        let mut clean = String::new();

        let toml_file = {
            let mut f = self.irust_dir.clone();
            f.push("Cargo.toml");
            f
        };

        if !Path::exists(&toml_file) {
            return;
        }

        let mut toml_read = File::open(&toml_file).unwrap();

        let toml_contents = {
            let mut c = String::new();
            toml_read.read_to_string(&mut c).unwrap();
            c
        };

        for line in toml_contents.lines() {
            clean.push_str(line);
            if line.contains("[dependencies]") {
                break;
            }
            clean.push('\n')
        }

        let mut toml_write = File::create(&toml_file).unwrap();
        write!(toml_write, "{}", clean).unwrap();
    }

    fn soft_clean(&self) -> io::Result<()> {
        let mut main = std::fs::File::create(&self.main_file)?;
        let main_src = "fn main() {}";
        write!(main, "{}", main_src)?;
        Ok(())
    }
}
