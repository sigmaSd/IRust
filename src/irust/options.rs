use crate::irust::IRust;
use std::io::{Read, Write};

pub struct Options {
    add_irust_cmd_to_history: bool,
    add_shell_cmd_to_history: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            add_irust_cmd_to_history: false,
            add_shell_cmd_to_history: false,
        }
    }
}

impl Options {
    pub fn new() -> std::io::Result<Self> {
        let config_dir = match dirs::config_dir() {
            Some(dir) => dir.join("irust"),
            None => return Ok(Options::default()),
        };

        let _ = std::fs::create_dir(&config_dir);

        let config_path = config_dir.join("config");

        match std::fs::File::open(&config_path) {
            Ok(config_file) => Options::parse(config_file),
            Err(_) => Options::create_config(config_path),
        }
    }

    fn parse(mut config_path: std::fs::File) -> std::io::Result<Options> {
        let mut options = Options::default();

        let config = {
            let mut config = String::new();
            config_path.read_to_string(&mut config)?;
            config
        };

        for line in config.lines() {
            // skip comments
            if let Some(c) = line.trim_start().chars().nth(0) {
                if c == '#' {
                    continue;
                }
            }

            // we're looking for option = value
            let line_parts = line.split('=').map(str::trim).collect::<Vec<&str>>();

            if line_parts.len() == 2 {
                let (option, value) = (line_parts[0], line_parts[1]);

                match (option, value) {
                    ("add_irust_cmd_to_history", "false") => {
                        options.add_irust_cmd_to_history = false;
                    }
                    ("add_irust_cmd_to_history", "true") => {
                        options.add_irust_cmd_to_history = true;
                    }
                    ("add_shell_cmd_to_history", "false") => {
                        options.add_shell_cmd_to_history = false;
                    }
                    ("add_shell_cmd_to_history", "true") => {
                        options.add_shell_cmd_to_history = true;
                    }
                    _ => eprintln!("Unknown config option: {} {}", option, value),
                }
            } else {
                eprintln!("Unknown line in config: {}", line);
            }
        }

        Ok(options)
    }

    fn create_config(config_path: std::path::PathBuf) -> std::io::Result<Options> {
        let config = "add_irust_cmd_to_history = false
add_shell_cmd_to_history = false";

        let mut config_file = std::fs::File::create(config_path)?;

        write!(config_file, "{}", config)?;

        Ok(Options::default())
    }
}

impl IRust {
    pub fn should_push_to_history(&self, buffer: &str) -> bool {
        let buffer: Vec<char> = buffer.chars().collect();

        if buffer.len() == 0 { return false; }
        if buffer.len() == 1 { return buffer[0] != ':'; }

        let irust_cmd = buffer[0] == ':' && buffer[1] != ':';
        let shell_cmd = buffer[0] == ':' && buffer[1] == ':';

        (irust_cmd && self.options.add_irust_cmd_to_history)
            || (shell_cmd && self.options.add_shell_cmd_to_history)
            || (!irust_cmd && !shell_cmd)
    }
}
