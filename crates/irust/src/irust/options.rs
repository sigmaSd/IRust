use crate::irust::{IRust, Result};
use crossterm::style::Color;
use irust_repl::{Edition, Executor, MainResult, ToolChain, DEFAULT_EVALUATOR};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Options {
    add_irust_cmd_to_history: bool,
    add_shell_cmd_to_history: bool,
    pub ok_color: Color,
    pub eval_color: Color,
    pub irust_color: Color,
    pub irust_warn_color: Color,
    pub out_color: Color,
    pub shell_color: Color,
    pub err_color: Color,
    pub input_color: Color,
    pub insert_color: Color,
    pub welcome_msg: String,
    pub welcome_color: Color,
    pub racer_inline_suggestion_color: Color,
    pub racer_suggestions_table_color: Color,
    pub racer_selected_suggestion_color: Color,
    pub racer_max_suggestions: usize,
    pub first_irust_run: bool,
    pub enable_racer: bool,
    pub toolchain: ToolChain,
    pub check_statements: bool,
    pub auto_insert_semicolon: bool,
    pub replace_marker: String,
    pub replace_output_with_marker: bool,
    pub input_prompt: String,
    pub output_prompt: String,
    pub activate_scripting: bool,
    pub executor: Executor,
    pub evaluator: Vec<String>,
    pub compile_time: bool,
    pub main_result: MainResult,
    pub show_warnings: bool,
    pub edition: Edition,
    pub debugger: Debugger,
    pub shell_interpolate: bool,
    pub local_server: bool,
    pub local_server_adress: std::net::SocketAddrV4,
    pub highlight_engine: String,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            // [Histroy]
            add_irust_cmd_to_history: true,
            add_shell_cmd_to_history: false,

            // [Colors]
            ok_color: Color::Blue,
            eval_color: Color::White,
            irust_color: Color::DarkBlue,
            irust_warn_color: Color::Cyan,
            out_color: Color::Red,
            shell_color: Color::DarkYellow,
            err_color: Color::DarkRed,
            input_color: Color::Yellow,
            insert_color: Color::White,

            // [Welcome]
            welcome_msg: String::new(),
            welcome_color: Color::DarkBlue,

            // [Racer]
            enable_racer: true,
            racer_inline_suggestion_color: Color::Cyan,
            racer_suggestions_table_color: Color::Green,
            racer_selected_suggestion_color: Color::DarkRed,
            racer_max_suggestions: 5,

            //other
            first_irust_run: true,
            toolchain: ToolChain::Default,
            check_statements: true,
            auto_insert_semicolon: true,

            // replace output
            replace_marker: "$out".into(),
            replace_output_with_marker: false,

            input_prompt: "In: ".to_string(),
            output_prompt: "Out: ".to_string(),
            activate_scripting: false,
            executor: Executor::Sync,
            evaluator: DEFAULT_EVALUATOR
                .iter()
                .map(|part| part.to_string())
                .collect(),
            compile_time: false,
            main_result: MainResult::Unit,
            show_warnings: false,
            edition: Edition::E2021,
            debugger: Debugger::LLDB,
            shell_interpolate: true,
            local_server: false,
            local_server_adress: "127.0.0.1:9000".parse().expect("correct"),

            highlight_engine: "default".into(),
        }
    }
}

impl Options {
    pub fn save(&mut self) -> Result<()> {
        if let Some(path) = Self::config_path() {
            Self::write_config_file(path, self)?;
        }
        Ok(())
    }

    pub fn new() -> Result<Self> {
        if let Some(config_path) = Options::config_path() {
            let mut config_file = std::fs::File::open(config_path)?;
            let mut config_data = String::new();
            config_file.read_to_string(&mut config_data)?;

            toml::from_str(&config_data).map_err(|e| e.into())
        } else {
            Ok(Options::default())
        }
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn reset_evaluator(&mut self) {
        self.evaluator = DEFAULT_EVALUATOR
            .iter()
            .map(|part| part.to_string())
            .collect();
    }

    pub fn config_path() -> Option<std::path::PathBuf> {
        let config_dir = match dirs::config_dir() {
            Some(dir) => dir.join("irust"),
            None => return None,
        };

        // Ignore directory exists error
        let _ = std::fs::create_dir_all(&config_dir);
        let config_path = config_dir.join("config.toml");

        Some(config_path)
    }

    fn write_config_file(config_path: std::path::PathBuf, options: &Options) -> Result<()> {
        let config = toml::to_string(options)?;

        let mut config_file = std::fs::File::create(config_path)?;

        write!(config_file, "{}", config)?;
        Ok(())
    }
}

impl IRust {
    pub fn should_push_to_history(&self, buffer: &str) -> bool {
        let buffer: Vec<char> = buffer.chars().collect();

        if buffer.is_empty() {
            return false;
        }
        if buffer.len() == 1 {
            return buffer[0] != ':';
        }

        let irust_cmd = buffer[0] == ':' && buffer[1] != ':';
        let shell_cmd = buffer[0] == ':' && buffer[1] == ':';

        (irust_cmd && self.options.add_irust_cmd_to_history)
            || (shell_cmd && self.options.add_shell_cmd_to_history)
            || (!irust_cmd && !shell_cmd)
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum Debugger {
    LLDB,
    GDB,
}
