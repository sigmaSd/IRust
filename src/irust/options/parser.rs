use super::Options;
use std::io::Read;

impl Options {
    pub fn parse(mut config_path: std::fs::File) -> std::io::Result<Options> {
        let mut options = Options::default();

        let config = {
            let mut config = String::new();
            config_path.read_to_string(&mut config)?;
            config
        };

        let lines: Vec<String> = config
            .lines()
            .filter(|l| !l.starts_with('#') && !l.is_empty())
            .map(ToOwned::to_owned)
            .collect();

        for (option, value) in Options::get_section(&lines, "[History]".to_string()).into_iter() {
            match (option.to_lowercase().as_str(), value.clone()) {
                ("add_irust_cmd_to_history", value) => {
                    options.add_irust_cmd_to_history = Options::str_to_bool(&value);
                }
                ("add_shell_cmd_to_history", value) => {
                    options.add_shell_cmd_to_history = Options::str_to_bool(&value);;
                }
                _ => eprintln!("Unknown config option: {} {}", option, value),
            }
        }

        for (option, value) in Options::get_section(&lines, "[Colors]".to_string()).into_iter() {
            match (option.to_lowercase().as_ref(), value.clone()) {
                ("ok_color", value) => {
                    if let Ok(value) = Options::str_to_color(&value) {
                        options.ok_color = value;
                    }
                }
                ("show_color", value) => {
                    if let Ok(value) = Options::str_to_color(&value) {
                        options.show_color = value;
                    }
                }
                ("eval_color", value) => {
                    if let Ok(value) = Options::str_to_color(&value) {
                        options.eval_color = value;
                    }
                }
                ("irust_color", value) => {
                    if let Ok(value) = Options::str_to_color(&value) {
                        options.irust_color = value;
                    }
                }
                ("irust_warn_color", value) => {
                    if let Ok(value) = Options::str_to_color(&value) {
                        options.irust_warn_color = value;
                    }
                }
                ("shell_color", value) => {
                    if let Ok(value) = Options::str_to_color(&value) {
                        options.shell_color = value;
                    }
                }
                ("err_color", value) => {
                    if let Ok(value) = Options::str_to_color(&value) {
                        options.err_color = value;
                    }
                }
                ("out_color", value) => {
                    if let Ok(value) = Options::str_to_color(&value) {
                        options.out_color = value;
                    }
                }
                ("input_color", value) => {
                    if let Ok(value) = Options::str_to_color(&value) {
                        options.input_color = value;
                    }
                }
                ("insert_color", value) => {
                    if let Ok(value) = Options::str_to_color(&value) {
                        options.insert_color = value;
                    }
                }
                _ => eprintln!("Unknown config option: {} {}", option, value),
            }
        }

        for (option, value) in Options::get_section(&lines, "[Welcome]".to_string()).into_iter() {
            match (option.to_lowercase().as_str(), value.clone()) {
                ("welcome_msg", value) => {
                    if !value.is_empty() {
                        options.welcome_msg = value;
                    }
                }
                ("welcome_color", value) => {
                    if let Ok(value) = Options::str_to_color(&value) {
                        options.welcome_color = value;
                    }
                }
                _ => eprintln!("Unknown config option: {} {}", option, value),
            }
        }

        for (option, value) in Options::get_section(&lines, "[Racer]".to_string()).into_iter() {
            match (option.to_lowercase().as_str(), value.clone()) {
                ("enable_racer", value) => {
                    options.enable_racer = Options::str_to_bool(&value);
                }
                ("racer_inline_suggestion_color", value) => {
                    if let Ok(value) = Options::str_to_color(&value) {
                        options.racer_inline_suggestion_color = value;
                    }
                }
                ("racer_suggestions_table_color", value) => {
                    if let Ok(value) = Options::str_to_color(&value) {
                        options.racer_suggestions_table_color = value;
                    }
                }
                ("racer_selected_suggestion_color", value) => {
                    if let Ok(value) = Options::str_to_color(&value) {
                        options.racer_selected_suggestion_color = value;
                    }
                }
                ("racer_max_suggestions", value) => {
                    if let Ok(value) = value.parse() {
                        options.racer_max_suggestions = value;
                    }
                }
                _ => eprintln!("Unknown config option: {} {}", option, value),
            }
        }

        Ok(options)
    }

    pub fn default_config() -> String {
        let history = "\
[History]
add_irust_cmd_to_history = false
add_shell_cmd_to_history = false";

        #[cfg(unix)]
        let racer = "enable_racer = true";
        #[cfg(windows)]
        let racer = "enable_racer = false";

        let racer = format!(
            "\
[Racer]
{}
racer_inline_suggestion_color = Cyan
racer_suggestions_table_color = Green
racer_selected_suggestion_color = DarkRed
racer_max_suggestions = 5",
            racer
        );

        let colors = "\
[Colors]
insert_color = White
input_color = Yellow
out_color = Red
ok_color = Blue
show_color = DarkCyan
eval_color = White
irust_color = DarkBlue
irust_warn_color = Cyan
shell_color = DarkYellow
err_color = DarkRed";

        let welcome = "\
[Welcome]
welcome_msg = Welcome to IRust
welcome_color = DarkBlue";

        format!("{}\n\n{}\n\n{}\n\n{}", history, racer, colors, welcome)
    }
}
