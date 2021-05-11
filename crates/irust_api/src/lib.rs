use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptInfo {
    pub name: String,
    pub path: PathBuf,
    pub hooks: Vec<Hook>,
    pub is_daemon: bool,
}
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum Hook {
    InputEvent,
    OutputEvent,
    SetInputPrompt,
    SetOutputPrompt,
    WhileCompiling,
    AfterCompile,
}

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum Message {
    Greeting,
    Hook,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Command {
    AcceptSuggestion,
    Continue,
    DeleteNextWord,
    DeleteUntilNewLine(bool),
    Multiple(Vec<Command>),
    SetThinCursor,
    SetWideCursor,
    HandleCharacter(char),
    HandleEnter(bool),
    HandleAltEnter,
    HandleTab,
    HandleBackTab,
    HandleRight,
    HandleLeft,
    HandleBackSpace,
    HandleDelete,
    HandleCtrlC,
    HandleCtrlD,
    HandleCtrlE,
    HandleCtrlL,
    HandleCtrlR,
    HandleCtrlZ,
    HandleUp,
    HandleDown,
    HandleCtrlRight,
    HandleCtrlLeft,
    HandleHome,
    HandleEnd,
    RemoveRacerSugesstion,
    Exit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalVariables {
    current_working_dir: PathBuf,
    previous_working_dir: PathBuf,
    last_loaded_code_path: Option<PathBuf>,
    /// last successful output
    last_output: Option<String>,
    pub operation_number: usize,

    pub prompt_position: (usize, usize), // (row, col)
    pub cursor_position: (usize, usize), // (row, col)
    pub prompt_len: usize,
    pub pid: u32,
    pub is_racer_suggestion_active: bool,
}

impl Default for GlobalVariables {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalVariables {
    pub fn new() -> Self {
        let cwd = std::env::current_dir().expect("Error getting current working directory");

        Self {
            current_working_dir: cwd.clone(),
            previous_working_dir: cwd,
            last_loaded_code_path: None,
            last_output: None,
            operation_number: 1,
            prompt_position: (0, 0), // (row, col)
            cursor_position: (0, 0), // (row, col)
            prompt_len: 0,
            pid: std::process::id(),
            is_racer_suggestion_active: false,
        }
    }

    pub fn update_cwd(&mut self, cwd: PathBuf) {
        self.previous_working_dir = self.current_working_dir.clone();
        self.current_working_dir = cwd;
    }

    pub fn get_cwd(&self) -> PathBuf {
        self.current_working_dir.clone()
    }

    pub fn get_pwd(&self) -> PathBuf {
        self.previous_working_dir.clone()
    }

    pub fn set_last_loaded_coded_path(&mut self, path: PathBuf) {
        self.last_loaded_code_path = Some(path);
    }

    pub fn get_last_loaded_coded_path(&self) -> Option<PathBuf> {
        self.last_loaded_code_path.clone()
    }

    pub fn get_last_output(&self) -> Option<&String> {
        self.last_output.as_ref()
    }

    pub fn set_last_output(&mut self, out: String) {
        self.last_output = Some(out);
    }
}
