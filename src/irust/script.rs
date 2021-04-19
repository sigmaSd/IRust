use super::global_variables::GlobalVariables;
use crossterm::style::Colorize;
use libloading::{Library, Symbol};
use std::{env::temp_dir, path::Path, process::Command};

pub struct ScriptManager {
    lib: Library,
}

impl ScriptManager {
    pub fn new() -> Option<Self> {
        let script_path = dirs_next::config_dir()?.join("irust").join("script.rs");
        let script_timestamp_path = dirs_next::cache_dir()?
            .join("irust_repl")
            .join("script_timestamp");

        let script_lib_name = "libirustscript";
        let script_lib_path = temp_dir().join(script_lib_name);

        if !Path::exists(&script_path) {
            // No user script
            return None;
        }

        let last_modified = std::fs::File::open(&script_path)
            .ok()?
            .metadata()
            .ok()?
            .modified()
            .ok()?
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_secs();

        if let Some(last_timestamp) = (|| {
            std::fs::read_to_string(&script_timestamp_path)
                .ok()?
                .parse::<u64>()
                .ok()
        })() {
            if last_modified <= last_timestamp && Path::exists(&script_lib_path) {
                // library already compiled and no modification have occurred since last compilation
                return unsafe {
                    Some(Self {
                        lib: Library::new(script_lib_path).unwrap(),
                    })
                };
            }
        }

        println!(
            "{}",
            format!(
                "Found script file at {}\nStarting compilation..",
                script_path.display()
            )
            .cyan()
        );
        println!();

        let compilation = (|| {
            Command::new("rustc")
                .args(&["--crate-type", "dylib"])
                .arg(script_path)
                .args(&["-o", &script_lib_path.display().to_string()])
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
                &script_lib_path.display()
            )
            .green()
        );
        println!();

        // write the new timestamp only after a successful compilation
        std::fs::write(&script_timestamp_path, last_modified.to_string()).ok()?;

        unsafe {
            Some(Self {
                lib: Library::new(script_lib_path).unwrap(),
            })
        }
    }

    pub fn input_prompt(&self, global_variables: &GlobalVariables) -> Option<String> {
        unsafe {
            let script: PromptFn = self.lib.get(b"input_prompt").ok()?;
            Some(script(global_variables))
        }
    }

    pub fn get_output_prompt(&self, global_variables: &GlobalVariables) -> Option<String> {
        unsafe {
            let script: PromptFn = self.lib.get(b"output_prompt").ok()?;
            Some(script(global_variables))
        }
    }
}

type PromptFn<'lib> = Symbol<'lib, unsafe extern "C" fn(&GlobalVariables) -> String>;
