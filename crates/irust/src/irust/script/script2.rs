use irust_api::GlobalVariables;
use std::{
    collections::HashMap,
    path::PathBuf,
    process::{self, Child, Stdio},
};

pub struct ScriptManager2 {
    map: HashMap<String, Child>,
    script_path: PathBuf,
}

impl ScriptManager2 {
    pub fn new() -> Self {
        let script_path = dirs_next::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("irust")
            .join("script2");
        Self {
            map: HashMap::new(),
            script_path,
        }
    }
}
impl super::Script for ScriptManager2 {
    fn input_prompt(&self, global_variables: &GlobalVariables) -> Option<String> {
        if !self.script_path.join("input_prompt").exists() {
            return None;
        }
        let mut script = process::Command::new(self.script_path.join("input_prompt"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;

        let stdin = script.stdin.as_mut().expect("stdin is piped");
        bincode::serialize_into(stdin, global_variables).ok()?;

        let stdout = script.stdout.as_mut().expect("stdout is piped");
        bincode::deserialize_from(stdout).ok()
    }

    fn get_output_prompt(&self, global_variables: &GlobalVariables) -> Option<String> {
        if !self.script_path.join("output_prompt").exists() {
            return None;
        }
        let mut script = process::Command::new(self.script_path.join("input_prompt"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;

        let stdin = script.stdin.as_mut().expect("stdin is piped");
        bincode::serialize_into(stdin, global_variables).ok()?;

        let stdout = script.stdout.as_mut().expect("stdout is piped");
        bincode::deserialize_from(stdout).ok()
    }
    fn while_compiling(&mut self, global_variables: &GlobalVariables) -> Option<()> {
        if !self.script_path.join("while_compiling").exists() {
            return None;
        }
        let mut script = process::Command::new(self.script_path.join("while_compiling"))
            .stdin(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;
        let stdin = script.stdin.as_mut().expect("stdin is piped");
        bincode::serialize_into(stdin, global_variables).ok()?;

        self.map.insert("while_compiling".into(), script);
        None
    }
    fn after_compiling(&mut self) -> Option<()> {
        self.map.get_mut("while_compiling")?.kill().ok()
    }
}
