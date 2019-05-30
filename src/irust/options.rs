use crate::irust::IRust;
use crate::utils::VecTools;
use crossterm::Color;
use std::io::Write;
mod parser;

#[derive(Clone)]
pub struct Options {
    pub add_irust_cmd_to_history: bool,
    pub add_shell_cmd_to_history: bool,
    pub ok_color: Color,
    pub show_color: Color,
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
    pub enable_racer: bool,
    pub racer_inline_suggestion_color: Color,
    pub racer_suggestions_table_color: Color,
    pub racer_selected_suggestion_color: Color,
    pub racer_max_suggestions: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            // [Histroy]
            add_irust_cmd_to_history: false,
            add_shell_cmd_to_history: false,

            // [Colors]
            ok_color: Color::Blue,
            show_color: Color::DarkCyan,
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
            #[cfg(unix)]
            enable_racer: true,
            #[cfg(windows)]
            enable_racer: false,

            racer_inline_suggestion_color: Color::Cyan,
            racer_suggestions_table_color: Color::Green,
            racer_selected_suggestion_color: Color::DarkRed,
            racer_max_suggestions: 5,
        }
    }
}

impl Options {
    pub fn new() -> std::io::Result<Self> {
        if let Some(config_path) = Options::config_path() {
            match std::fs::File::open(&config_path) {
                Ok(config_file) => Options::parse(config_file),
                Err(_) => Options::create_config(config_path),
            }
        } else {
            Ok(Options::default())
        }
    }

    pub fn reset_config(config_path: std::path::PathBuf) {
        let _ = Options::create_config(config_path);
    }

    pub fn config_path() -> Option<std::path::PathBuf> {
        let config_dir = match dirs::config_dir() {
            Some(dir) => dir.join("irust"),
            None => return None,
        };

        let _ = std::fs::create_dir(&config_dir);
        let config_path = config_dir.join("config");

        Some(config_path)
    }

    fn create_config(config_path: std::path::PathBuf) -> std::io::Result<Options> {
        let config = Options::default_config();

        let mut config_file = std::fs::File::create(&config_path)?;

        write!(config_file, "{}", config)?;

        Ok(Options::default())
    }
}

impl Options {
    fn str_to_bool(value: &str) -> bool {
        match value {
            "true" => true,
            "false" => false,
            value => {
                eprintln!("Unknown option value: {}", value);
                false
            }
        }
    }

    fn str_to_color(value: &str) -> Result<Color, &str> {
        match value.to_lowercase().as_ref() {
            "black" => Ok(Color::Black),
            "red" => Ok(Color::Red),
            "darkred" => Ok(Color::DarkRed),
            "green" => Ok(Color::Green),
            "darkgreen" => Ok(Color::DarkGreen),
            "yellow" => Ok(Color::Yellow),
            "darkyellow" => Ok(Color::DarkYellow),
            "blue" => Ok(Color::Blue),
            "darkblue" => Ok(Color::DarkBlue),
            "magenta" => Ok(Color::Magenta),
            "darkmagenta" => Ok(Color::DarkMagenta),
            "cyan" => Ok(Color::Cyan),
            "darkcyan" => Ok(Color::DarkCyan),
            "grey" => Ok(Color::Grey),
            "white" => Ok(Color::White),
            value => {
                eprintln!("Unknown option value: {}", value);
                Err("Unknown option value")
            }
        }
    }

    fn get_section(lines: &[String], section_name: String) -> Vec<(String, String)> {
        let sec_start = match VecTools::index(lines, &section_name).get(0) {
            Some(idx) => *idx,
            None => {
                eprintln!("Section {} not found", section_name);
                return Vec::new();
            }
        };

        let sec_end = VecTools::index(lines, "[")
            .into_iter()
            .find(|elem| *elem > sec_start)
            .unwrap_or_else(|| lines.len());

        lines[sec_start + 1..sec_end]
            .iter()
            .filter_map(|line| {
                let lines_part = line.split('=').map(str::trim).collect::<Vec<&str>>();
                if lines_part.len() == 2 {
                    Some((lines_part[0].to_string(), lines_part[1].to_string()))
                } else {
                    eprintln!("Unknown line: {}", line);
                    None
                }
            })
            .collect()
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
