use super::Edition;
use crate::Result;
use crate::{
    utils::{stdout_and_stderr, ProcessUtils},
    ToolChain,
};
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::process::{Command, ExitStatus};
use std::sync::OnceLock;
use std::{fs, process};

static NO_COLOR: OnceLock<bool> = OnceLock::new();
/// Have the top precedence
fn no_color() -> bool {
    *NO_COLOR.get_or_init(|| std::env::var("NO_COLOR").is_ok())
}

const WRITE_LIB_LIMIT: &str = concat!(
    "\nAlso your code in this repl session needs to only consist of top level statements",
    "\nSo if you have a `let a = 4;` it will not work",
    "\nUse :reset to reset the repl in that case"
);

#[derive(Debug, Clone)]
pub struct Cargo {
    pub name: String,
    pub paths: CargoPaths,
}
impl Default for Cargo {
    fn default() -> Self {
        let name = "irust_host_repl_".to_string() + &uuid::Uuid::new_v4().simple().to_string();
        let paths = CargoPaths::new(&name);
        Self { name, paths }
    }
}

#[derive(Debug, Clone)]
pub struct CargoPaths {
    pub tmp_dir: PathBuf,
    pub common_root: PathBuf,
    pub irust_dir: PathBuf,
    pub irust_target_dir: PathBuf,
    pub cargo_toml_file: PathBuf,
    pub irust_src_dir: PathBuf,
    pub main_file: PathBuf,
    pub main_file_extern: PathBuf,
    pub lib_file: PathBuf,
    pub exe_path: PathBuf,
    pub release_exe_path: PathBuf,
}

impl CargoPaths {
    fn new(name: &str) -> Self {
        let tmp_dir = if let Ok(dir) = std::env::var("IRUST_TEMP_DIR") {
            dir.into()
        } else {
            std::env::temp_dir()
        };
        let common_root = tmp_dir.join("irust_repls");
        let irust_dir = common_root.join(name);
        let irust_target_dir = (|| {
            if let Ok(p) = std::env::var("CARGO_TARGET_DIR") {
                if !p.is_empty() {
                    return Path::new(&p).to_path_buf();
                }
            }
            // CARGO_TARGET_DIR is not set, default to one common target location for all repls
            common_root.join("target")
        })();
        let cargo_toml_file = irust_dir.join("Cargo.toml");
        let irust_src_dir = irust_dir.join("src");
        let main_file = irust_src_dir.join("main.rs");
        let main_file_extern = irust_src_dir.join("main_extern.rs");
        let lib_file = irust_src_dir.join("lib.rs");
        let exe_path = if cfg!(windows) {
            irust_target_dir.join(format!("debug/{}.exe", &name))
        } else {
            irust_target_dir.join(format!("debug/{}", &name))
        };
        let release_exe_path = if cfg!(windows) {
            irust_target_dir.join(format!("release/{}.exe", &name))
        } else {
            irust_target_dir.join(format!("release/{}", &name))
        };

        Self {
            tmp_dir,
            irust_dir,
            irust_target_dir,
            cargo_toml_file,
            irust_src_dir,
            main_file,
            main_file_extern,
            lib_file,
            exe_path,
            release_exe_path,
            common_root,
        }
    }
}
impl Cargo {
    pub fn cargo_new(&self, edition: Edition) -> std::result::Result<(), io::Error> {
        // Ignore directory exists error
        let _ = std::fs::create_dir_all(&self.paths.irust_src_dir);
        self.clean_cargo_toml(edition)?;
        self.clean_files()?;

        Ok(())
    }

    pub fn cargo_new_lib_simple(
        &self,
        path: &Path,
        name: &'static str,
    ) -> std::result::Result<(), io::Error> {
        let lib_path = path.join(name);
        let _ = std::fs::create_dir_all(lib_path.join("src"));
        let create_if_not_exist = |path: PathBuf, contents| -> std::io::Result<()> {
            if !path.exists() {
                std::fs::write(path, contents)?;
            }
            Ok(())
        };
        let cargo_toml = format!(
            "\
[package]
name = \"{name}\"
version = \"0.1.0\"
edition = \"2021\""
        );
        create_if_not_exist(lib_path.join("src/lib.rs"), "")?;
        create_if_not_exist(lib_path.join("Cargo.toml"), &cargo_toml)?;

        Ok(())
    }

    pub fn cargo_run(
        &self,
        color: bool,
        release: bool,
        toolchain: ToolChain,
        interactive_function: Option<fn(&mut process::Child) -> Result<()>>,
    ) -> Result<(ExitStatus, String)> {
        let (status, output) = self.cargo_build_output(color, release, toolchain)?;

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
                        std::process::Command::new(&self.paths.exe_path)
                            .stdin(Stdio::piped())
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
                        std::process::Command::new(&self.paths.release_exe_path)
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .spawn()?
                            .interactive_output(interactive_function)?,
                    ),
                ))
            }
        }
    }

    pub fn cargo_add(&self, dep: &[String]) -> io::Result<std::process::Child> {
        Command::new("cargo")
            .arg("add")
            .args(dep)
            .args([
                "--manifest-path",
                &self.paths.cargo_toml_file.display().to_string(),
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .spawn()
    }

    pub fn cargo_add_prelude(&self, path: PathBuf, name: &'static str) -> io::Result<()> {
        let mut f = std::fs::OpenOptions::new()
            .append(true)
            .open(&self.paths.cargo_toml_file)?;

        let path = if !cfg!(windows) {
            path.display().to_string()
        } else {
            path.display().to_string().replace('\\', "\\\\")
        };

        writeln!(
            f,
            "
[dependencies]
{name} = {{ path = \"{path}\" }}
"
        )
    }

    pub fn cargo_add_sync(&self, dep: &[String]) -> Result<()> {
        let process = Command::new("cargo")
            .current_dir(&self.paths.irust_dir)
            .arg("add")
            .args(dep)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .spawn()?
            .wait()?;
        if process.success() {
            Ok(())
        } else {
            Err(format!("Failed to add dependency: {:?}", &dep).into())
        }
    }

    pub fn cargo_rm_sync(&self, dep: &str) -> Result<()> {
        // Ignore error if dependency doesn't exist
        Command::new("cargo-rm")
            .current_dir(&self.paths.irust_dir)
            .arg("rm")
            .arg(dep)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .spawn()?
            .wait()?;
        Ok(())
    }

    // The difference in env flags makes cargo recompiles again!!!
    // => make  sure all build env flags are the same
    // Or even better dont use any
    fn cargo_common<'a>(
        &self,
        cargo: &'a mut process::Command,
        cmd: &str,
        toolchain: ToolChain,
    ) -> &'a mut Command {
        match toolchain {
            ToolChain::Default => cargo,
            _ => cargo.arg(toolchain.as_arg()),
        }
        .arg(cmd)
        .env("CARGO_TARGET_DIR", &self.paths.irust_target_dir)
        .current_dir(&self.paths.irust_dir)
    }

    pub fn cargo_check(
        &self,
        toolchain: ToolChain,
    ) -> std::result::Result<std::process::Child, io::Error> {
        let mut cmd = Command::new("cargo");
        self.cargo_common(&mut cmd, "check", toolchain)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
    }

    pub fn cargo_check_output(
        &self,
        toolchain: ToolChain,
    ) -> std::result::Result<(ExitStatus, String), io::Error> {
        let color = if no_color() { "never" } else { "always" };
        let mut cmd = Command::new("cargo");
        let output = self
            .cargo_common(&mut cmd, "check", toolchain)
            .args(["--color", color])
            .output()?;

        let status = output.status;
        Ok((status, stdout_and_stderr(output)))
    }

    pub fn cargo_build(
        &self,
        toolchain: ToolChain,
    ) -> std::result::Result<std::process::Child, io::Error> {
        let mut cmd = Command::new("cargo");
        self.cargo_common(&mut cmd, "build", toolchain)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
    }

    pub fn cargo_build_output(
        &self,
        color: bool,
        release: bool,
        toolchain: ToolChain,
    ) -> std::result::Result<(ExitStatus, String), io::Error> {
        let color = if no_color() {
            "never"
        } else if color {
            "always"
        } else {
            "never"
        };
        let mut cmd = Command::new("cargo");

        let output = if !release {
            self.cargo_common(&mut cmd, "build", toolchain)
                .args(["--color", color])
                .output()?
        } else {
            self.cargo_common(&mut cmd, "build", toolchain)
                .arg("--release")
                .args(["--color", color])
                .output()?
        };
        let status = output.status;

        Ok((status, stdout_and_stderr(output)))
    }

    pub fn cargo_bench(&self, toolchain: ToolChain) -> std::result::Result<String, io::Error> {
        let color = if no_color() { "never" } else { "always" };
        let mut cmd = Command::new("cargo");
        Ok(stdout_and_stderr(
            self.cargo_common(&mut cmd, "bench", toolchain)
                .args(["--color", color])
                .output()?,
        ))
    }

    pub fn cargo_fmt(&self, c: &str) -> std::io::Result<String> {
        let fmt_path = self.paths.irust_dir.join("fmt_file");

        let mut fmt_file = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&fmt_path)?;

        write!(fmt_file, "{c}")?;

        self.cargo_fmt_file(&fmt_path);

        let mut fmt_c = String::new();
        fmt_file.rewind()?;
        fmt_file.read_to_string(&mut fmt_c)?;

        Ok(fmt_c)
    }

    pub fn cargo_asm(&self, fnn: &str, toolchain: ToolChain) -> Result<String> {
        // 0 doesn't activate FORCE_COLOR (tested)
        let force_color = if no_color() { "0" } else { "1" };
        let mut cmd = Command::new("cargo");
        let output = self
            .cargo_common(&mut cmd, "asm", toolchain)
            .arg("--lib")
            .arg(format!("{}::{fnn}", &self.name))
            .arg("--rust")
            .env("FORCE_COLOR", force_color)
            .output()?;
        if !output.status.success() {
            return Err(
            (stdout_and_stderr(output)
            + "\nMaybe you should make the function `pub`, see https://github.com/pacak/cargo-show-asm#my-function-isnt-there" +
                      WRITE_LIB_LIMIT).into());
        }
        Ok(stdout_and_stderr(output))
    }

    pub fn cargo_expand(&self, fnn: Option<&str>, toolchain: ToolChain) -> Result<String> {
        let color = if no_color() { "never" } else { "always" };
        let mut cmd = Command::new("cargo");
        let output = if let Some(fnn) = fnn {
            self.cargo_common(&mut cmd, "expand", toolchain)
                // For cargo expand, color needs to be specified here
                .args(["--color", color])
                .arg("--lib")
                .arg(fnn)
                .output()?
        } else {
            self.cargo_common(&mut cmd, "expand", toolchain)
                // For cargo expand, color needs to be specified here
                .args(["--color", color])
                .args(["--bin", &self.name])
                .output()?
        };
        if !output.status.success() {
            return Err((stdout_and_stderr(output) + WRITE_LIB_LIMIT).into());
        }
        Ok(stdout_and_stderr(output).trim().to_owned())
    }

    pub fn cargo_fmt_file(&self, file: &Path) {
        // Cargo fmt is optional
        let _ = self.try_cargo_fmt_file(file);
    }

    fn try_cargo_fmt_file(&self, file: &Path) -> io::Result<()> {
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

    fn clean_cargo_toml(&self, edition: Edition) -> io::Result<()> {
        // edition needs to be specified or racer will not be able to autocomplete dependencies
        // bug maybe?
        let cargo_toml = format!(
            "\
[package]
name = \"{}\"
version = \"0.1.0\"
edition = \"{edition}\"",
            self.name
        );
        let mut cargo_toml_file = fs::File::create(&self.paths.cargo_toml_file)?;
        write!(cargo_toml_file, "{cargo_toml}")?;
        Ok(())
    }

    fn clean_files(&self) -> io::Result<()> {
        const MAIN_SRC: &str = "fn main() {\n\n}";
        let mut main = fs::File::create(&self.paths.main_file)?;
        write!(main, "{MAIN_SRC}")?;
        std::fs::copy(&self.paths.main_file, &self.paths.main_file_extern)?;
        let _ = std::fs::remove_file(&self.paths.lib_file);
        Ok(())
    }

    /// Delete this repl specific folder, so for example `/tmp/irust_repls/irust_host_repl_$id` will
    /// be deleted
    pub fn delete_project(&self) -> io::Result<()> {
        std::fs::remove_dir_all(&self.paths.irust_dir)
    }
}
