use super::global_variables::GlobalVariables;
use crossterm::style::Colorize;
use libloading::{Library, Symbol};
use std::{ffi::CString, io::Write};
use std::{fs::File, os::raw::c_char};
use std::{path::Path, process::Command};

pub struct ScriptManager {
    lib: Library,
}

impl ScriptManager {
    pub fn new() -> Option<Self> {
        let script_path = dirs_next::config_dir()?.join("irust").join("script");
        create_script_dir_with_src(&script_path)?;

        let script_lib_file_path = script_path.join("src/lib.rs");
        if !Path::exists(&script_lib_file_path) {
            create_template_script(&script_lib_file_path)?;
        }

        let script_target_dir = script_path.join("target");
        #[cfg(unix)]
        let compiled_script_lib_path = script_target_dir.join("debug/libirustscript.so");
        #[cfg(windows)]
        let compiled_script_lib_path = script_target_dir.join("debug/libirustscript.dll");

        let last_modified = std::fs::File::open(&script_lib_file_path)
            .ok()?
            .metadata()
            .ok()?
            .modified()
            .ok()?
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_secs();

        let script_timestamp_path = dirs_next::cache_dir()?
            .join("irust_repl")
            .join("script_timestamp");

        if let Some(last_timestamp) = (|| {
            std::fs::read_to_string(&script_timestamp_path)
                .ok()?
                .parse::<u64>()
                .ok()
        })() {
            if last_modified <= last_timestamp && Path::exists(&compiled_script_lib_path) {
                // library already compiled and no modification have occurred since last compilation
                return unsafe {
                    Some(Self {
                        lib: Library::new(compiled_script_lib_path).unwrap(),
                    })
                };
            }
        }

        println!(
            "{}",
            format!(
                "Found script file at {}\nStarting compilation..",
                script_lib_file_path.display()
            )
            .cyan()
        );
        println!();

        let compilation = (|| {
            Command::new("cargo")
                .arg("build")
                .args(&["--target-dir", &script_target_dir.display().to_string()])
                .current_dir(script_path)
                .spawn()
                .ok()?
                .wait()
                .ok()
        })();
        // safe unwrap
        if compilation.is_none() || compilation.map(|command| !command.success()).unwrap() {
            println!("{}", "Failed to compile script".red());
            return None;
        }

        println!(
            "{}",
            format!(
                "Compiled script successfully to {}",
                &compiled_script_lib_path.display()
            )
            .green()
        );
        println!();

        // write the new timestamp only after a successful compilation
        std::fs::write(&script_timestamp_path, last_modified.to_string()).ok()?;

        unsafe {
            Some(Self {
                lib: Library::new(compiled_script_lib_path).unwrap(),
            })
        }
    }

    pub fn input_prompt(&self, global_variables: &GlobalVariables) -> Option<String> {
        unsafe {
            let script: PromptFn = self.lib.get(b"input_prompt").ok()?;
            Some(
                CString::from_raw(script(global_variables))
                    .to_str()
                    .ok()?
                    .to_string(),
            )
        }
    }

    pub fn get_output_prompt(&self, global_variables: &GlobalVariables) -> Option<String> {
        unsafe {
            let script: PromptFn = self.lib.get(b"output_prompt").ok()?;
            Some(
                CString::from_raw(script(global_variables))
                    .to_str()
                    .ok()?
                    .to_string(),
            )
        }
    }
}

type PromptFn<'lib> = Symbol<'lib, unsafe extern "C" fn(&GlobalVariables) -> &mut c_char>;

fn create_script_dir_with_src(script_path: &Path) -> Option<()> {
    let _ = std::fs::create_dir_all(&script_path.join("src"));

    let cargo_toml_file = script_path.join("Cargo.toml");

    if Path::exists(&cargo_toml_file) {
        return Some(());
    }

    let mut cargo_toml_file = File::create(cargo_toml_file).ok()?;

    const CARGO_TOML: &str = r#"[package]
name = "irustscript"
version = "0.1.0"
edition = "2018"
[lib]
crate-type = ["dylib"]"#;
    write!(cargo_toml_file, "{}", CARGO_TOML).ok()
}

fn create_template_script(script_lib_file_path: &Path) -> Option<()> {
    const TEMPLATE: &str = r##"/// This script prints an input/output prompt with the number of the
/// evaluation prefixed to it
use std::{ffi::CString, os::raw::c_char, path::PathBuf};

// the signature must be this
pub struct GlobalVariables {
    // Current directory that IRust is in
    _current_working_dir: PathBuf,
    // Previous directory that IRust was in, this current directory can change if the user uses the `:cd` command
    _previous_working_dir: PathBuf,
    // Last path to a rust file loaded with `:load` command
    _last_loaded_code_path: Option<PathBuf>,
    /// Last successful printed output
    _last_output: Option<String>,
    /// A variable that increases with each input/output cycle
    operation_number: usize,
}

#[no_mangle]
// the signature must be this
pub extern "C" fn input_prompt(global_varibales: &GlobalVariables) -> *mut c_char {
    // Default script
    CString::new(format!("In [{}]: ", global_varibales.operation_number))
        .unwrap()
        .into_raw()
}

#[no_mangle]
// the signature must be this
pub extern "C" fn output_prompt(global_varibales: &GlobalVariables) -> *mut c_char {
    // Default script
    CString::new(format!("Out[{}]: ", global_varibales.operation_number))
        .unwrap()
        .into_raw()
}
"##;
    std::fs::write(script_lib_file_path, TEMPLATE).ok()
}
