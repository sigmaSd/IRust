use crate::irust::IRust;
use crate::utils::StringTools;
use crossterm::ClearType;
use std::env::temp_dir;
use std::io::{self, Read, Write};
use std::process::{Child, Command, Stdio};

#[derive(Debug)]
pub struct Racer {
    process: Child,
    main_file: String,
    pub cursor: (usize, usize),
    pub suggestions: Vec<String>,
    suggestion_idx: usize,
    cmds: [String; 7],
    update_lock: bool,
}

impl Racer {
    pub fn start() -> io::Result<Racer> {
        let process = Command::new("racer")
            .arg("daemon")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let main_file = temp_dir()
            .join("irust/src/main.rs")
            .to_str()
            .unwrap()
            .to_owned();
        let cursor = (2, 0);
        let cmds = [
            "show".to_string(),
            "help".to_string(),
            "pop".to_string(),
            "del".to_string(),
            "add".to_string(),
            "reset".to_string(),
            "load".to_string(),
        ];

        let mut racer = Racer {
            process,
            main_file,
            cursor,
            suggestions: vec![],
            suggestion_idx: 0,
            cmds,
            update_lock: false,
        };
        racer.complete_code()?;

        Ok(racer)
    }

    pub fn complete_code(&mut self) -> io::Result<()> {
        if self.update_lock {
            return Ok(());
        }

        let stdin = self.process.stdin.as_mut().unwrap();
        let stdout = self.process.stdout.as_mut().unwrap();

        writeln!(
            stdin,
            "complete {} {} {}",
            self.cursor.0, self.cursor.1, self.main_file
        )?;

        // read till END
        let mut raw_output = [0; 100_000];
        'outer: loop {
            let _ = stdout.read(&mut raw_output)?;
            let mut count = 0;
            for e in raw_output.iter().rev() {
                if *e == b"D"[0] {
                    count += 1;
                    continue;
                }
                if count == 1 && *e == b"N"[0] {
                    count += 1;
                    continue;
                }
                if count == 2 && *e == b"E"[0] {
                    break 'outer;
                }
                count = 0;
            }
        }

        let mut raw_output = String::from_utf8(raw_output.to_vec()).unwrap();
        let mut completions = vec![];

        while let Some(match_idx) = raw_output.find("H ") {
            // if MATCH exists than , exists we can unwrap safely
            let command_idx = raw_output[match_idx..].find(',').unwrap() + match_idx;
            completions.push(raw_output[match_idx + 2..command_idx].to_owned());
            raw_output = raw_output[command_idx..].to_string();
        }
        self.suggestions = completions;

        Ok(())
    }

    pub fn next_suggestion(&mut self) -> Option<&String> {
        if self.suggestion_idx >= self.suggestions.len() {
            self.suggestion_idx = 0
        }

        if self.suggestions.is_empty() {
            return None;
        }

        let suggestion = &self.suggestions[self.suggestion_idx];

        self.suggestion_idx += 1;

        Some(suggestion)
    }

    pub fn current_suggestion(&self) -> Option<String> {
        if self.suggestion_idx > 1 {
            self.suggestions
                .get(self.suggestion_idx - 1)
                .map(ToOwned::to_owned)
        } else {
            self.suggestions.get(0).map(ToOwned::to_owned)
        }
    }
}

impl IRust {
    pub fn start_racer(&mut self) {
        self.racer = if self.options.enable_racer {
            match Racer::start() {
                Ok(r) => Some(r),
                Err(e) => {
                    eprintln!("Error while starting racer: {}", e);
                    None
                }
            }
        } else {
            None
        };
    }

    pub fn update_suggestions(&mut self) -> std::io::Result<()> {
        // return if we're not at the end of the line
        if !self.at_line_end() {
            return Ok(());
        }

        // don't autocomplete shell commands
        if self.buffer.starts_with("::") {
            return Ok(());
        }

        let racer = self.racer.take();
        if let Some(mut racer) = racer {
            if self.show_suggestions_inner(&mut racer).is_err() {
                eprintln!("Something happened while fetching suggestions");
            }
            self.racer = Some(racer);
        }

        Ok(())
    }

    fn show_suggestions_inner(&mut self, mut racer: &mut Racer) -> std::io::Result<()> {
        let mut tmp_repl = self.repl.clone();
        let y_pos = tmp_repl.body.len();
        tmp_repl.insert(self.buffer.clone());
        tmp_repl.write()?;

        racer.cursor.0 = y_pos;
        racer.cursor.1 = StringTools::chars_count(&self.buffer) + 1;
        self.update_racer(&mut racer)?;

        Ok(())
    }

    fn update_racer(&mut self, racer: &mut Racer) -> std::io::Result<()> {
        if self.buffer.starts_with(':') {
            // Auto complete IRust commands
            racer.suggestions = racer
                .cmds
                .iter()
                .filter(|c| c.starts_with(&self.buffer[1..]))
                .map(ToOwned::to_owned)
                .collect();
        } else {
            // Auto complete rust code
            racer.complete_code()?;
        }

        Ok(())
    }

    pub fn write_next_suggestion(&mut self) -> std::io::Result<()> {
        if self.at_line_end() {
            if let Some(mut racer) = self.racer.take() {
                if let Some(suggestion) = racer.next_suggestion() {
                    let mut suggestion = suggestion.to_string();
                    self.color.set_fg(self.options.racer_color)?;
                    self.cursor.save_position()?;
                    self.internal_cursor.save_position();
                    self.terminal.clear(ClearType::FromCursorDown)?;

                    StringTools::strings_unique(&self.buffer, &mut suggestion);
                    if self.will_overflow_screen_height(&suggestion) {
                        self.clear()?;
                    } else {
                        self.write(&suggestion)?;
                    }
                    self.cursor.reset_position()?;
                    self.internal_cursor.reset_position();
                    self.color.reset()?;
                }
                self.racer = Some(racer);
            }
        }

        Ok(())
    }

    pub fn use_suggestion(&mut self) -> std::io::Result<()> {
        if let Some(racer) = self.racer.take() {
            if let Some(mut suggestion) = racer.current_suggestion() {
                StringTools::strings_unique(&self.buffer, &mut suggestion);
                // update total wrapped lines count each time we touch the buffer
                self.buffer.push_str(&suggestion);
                self.update_total_wrapped_lines();

                if self.will_overflow_screen_height(&suggestion) {
                    self.clear()?;
                } else {
                    self.write(&suggestion)?;
                }
                self.racer = Some(racer);
            }
        }

        Ok(())
    }

    pub fn lock_racer_update(&mut self) {
        if let Some(mut racer) = self.racer.take() {
            racer.update_lock = true;
            self.racer = Some(racer);
        }
    }

    pub fn unlock_racer_update(&mut self) {
        if let Some(mut racer) = self.racer.take() {
            racer.update_lock = false;
            self.racer = Some(racer);
        }
    }
}
