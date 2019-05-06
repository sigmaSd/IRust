use std::env::temp_dir;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone)]
pub struct CargoCmds {
    tmp_dir: PathBuf,
    rust_repl_playground_dir: PathBuf,
    main_file: PathBuf,
}
impl Default for CargoCmds {
    fn default() -> Self {
        let tmp_dir = temp_dir();
        let rust_repl_playground_dir = {
            let mut dir = tmp_dir.clone();
            dir.push("rust_repl_playground");
            dir
        };
        let main_file = {
            let mut dir = rust_repl_playground_dir.clone();
            dir.push("src/main.rs");
            dir
        };
        Self {
            tmp_dir,
            rust_repl_playground_dir,
            main_file,
        }
    }
}
impl CargoCmds {
    pub fn cargo_new(&self) -> Result<(), io::Error> {
        self.clean_toml();
        if Path::new(&self.rust_repl_playground_dir).exists() {
            std::fs::remove_dir_all(&self.rust_repl_playground_dir)?;
        }
        Command::new("cargo")
            .current_dir(&*self.tmp_dir)
            .args(&["new", "rust_repl_playground"])
            .spawn()?
            .wait()?;
        self.cargo_build()?;
        Ok(())
    }

    pub fn cargo_run(&self, code: String) -> Result<String, io::Error> {
        let mut main = File::create(&*self.main_file)?;
        write!(main, "{}", code)?;
        let out = Command::new("cargo")
            .current_dir(&*self.rust_repl_playground_dir)
            .arg("run")
            .output()?;

        let out = if !out.stdout.is_empty() {
            out.stdout
        } else {
            out.stderr
        };

        Ok(String::from_utf8(out).unwrap_or_default())
    }

    pub fn cargo_add(&self, dep: &[String]) -> io::Result<()> {
        self.soft_clean()?;

        Command::new("cargo")
            .current_dir(&*self.rust_repl_playground_dir)
            .arg("add")
            .args(dep)
            .spawn()?
            .wait()?;

        Command::new("cargo")
            .current_dir(&*self.rust_repl_playground_dir)
            .arg("build")
            .spawn()?
            .wait()?;
        Ok(())
    }

    fn cargo_build(&self) -> Result<(), io::Error> {
        Command::new("cargo")
            .current_dir(&*self.rust_repl_playground_dir)
            .arg("build")
            .spawn()?
            .wait()?;
        Ok(())
    }

    fn clean_toml(&self) {
        use std::fs::File;
        use std::io::Read;

        let mut clean = String::new();

        let toml_file = {
            let mut f = self.rust_repl_playground_dir.clone();
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

    // associated fns
    pub fn remove_main(script: &mut String) {
        let main_start = match script.find("fn main() {") {
            Some(idx) => idx,
            None => return,
        };

        let open_tag = main_start + 11;
        // script == fn main() {
        if script.len() == 11 {
            return;
        }

        let mut close_tag = None;

        // look for closing tag
        let mut tag_score = 1;
        for (idx, character) in script[open_tag + 1..].chars().enumerate() {
            if character == '{' {
                tag_score += 1;
            }
            if character == '}' {
                tag_score -= 1;
                if tag_score == 0 {
                    close_tag = Some(idx);
                }
            }
        }
        if let Some(close_tag) = close_tag {
            script.remove(open_tag + close_tag + 1);
            script.replace_range(main_start..=open_tag, "");
        }
    }
}
