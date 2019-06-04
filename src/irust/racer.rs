use crate::irust::IRust;
use crate::utils::StringTools;
use crossterm::ClearType;
use std::env::temp_dir;
use std::io::{self, Read, Write};
use std::process::{Child, Command, Stdio};
use super::IRustError;

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

        let raw_output = String::from_utf8(raw_output.to_vec()).unwrap();

        self.suggestions.clear();
        for suggestion in raw_output
            .lines()
            .skip(1)
            .filter(|l| !l.chars().all(|c| c == '\u{0}'))
        {
            if suggestion == "END" {
                break;
            }
            let mut try_parse = || -> Option<()> {
                let start_idx = suggestion.find("H ")? + 2;
                let name = suggestion[start_idx..suggestion.find(',')?].to_owned();
                let definition = suggestion[suggestion.rfind(',')?..].to_owned();
                self.suggestions.push(name + ": " + &definition[1..]);
                Some(())
            };

            try_parse();
        }

        // remove duplicates
        self.suggestions.sort();
        self.suggestions.dedup();

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
    pub fn update_suggestions(&mut self) -> std::io::Result<()> {
        // return if we're not at the end of the line
        if !self.at_line_end() {
            return Ok(());
        }

        // don't autocomplete shell commands
        if self.buffer.starts_with("::") {
            return Ok(());
        }

        let _ = self.show_suggestions_inner();

        Ok(())
    }

    fn show_suggestions_inner(&mut self) -> Result<(), IRustError> {
        let mut tmp_repl = self.repl.clone();
        let y_pos = tmp_repl.body.len();
        tmp_repl.insert(self.buffer.clone());
        tmp_repl.write()?;

        self.racer
            .as_mut()?
            .cursor
            .0 = y_pos;
        self.racer
            .as_mut()
            ?
            .cursor
            .1 = StringTools::chars_count(&self.buffer) + 1;
        self.update_racer()?;

        Ok(())
    }

    fn update_racer(&mut self) -> Result<(), IRustError> {
        if self.buffer.starts_with(':') {
            // Auto complete IRust commands
            self.racer
                .as_mut()
                ?
                .suggestions = self
                .racer
                .as_ref()
                ?
                .cmds
                .iter()
                .filter(|c| c.starts_with(&self.buffer[1..]))
                .map(ToOwned::to_owned)
                .collect();
        } else {
            // Auto complete rust code
            self.racer
                .as_mut()
                ?
                .complete_code()?;
        }

        Ok(())
    }

    pub fn write_next_suggestion(&mut self) -> Result<(), IRustError> {
        let suggestion = self
            .racer
            .as_mut()
            ?
            .next_suggestion()
            .cloned();
        if let Some(suggestion) = suggestion {
            if self.at_line_end() {
                let mut suggestion = suggestion[..suggestion.find(':').unwrap_or(0)].to_owned();

                self.color
                    .set_fg(self.options.racer_inline_suggestion_color)?;
                self.save_cursor_position()?;
                self.terminal.clear(ClearType::FromCursorDown)?;

                StringTools::strings_unique(&self.buffer, &mut suggestion);

                let overflow = self.screen_height_overflow_by_str(&suggestion);
                if overflow != 0 {
                    self.internal_cursor.total_wrapped_lines += overflow;
                }

                self.write(&suggestion)?;

                self.reset_cursor_position()?;

                if overflow != 0 {
                    self.cursor.move_up(overflow as u16);
                    self.internal_cursor.y -= overflow;
                }

                self.color.reset()?;
            }
        }

        Ok(())
    }

    pub fn write_first_suggestion(&mut self) -> Result<(), IRustError> {
        let suggestion = self
            .racer
            .as_ref()
            ?
            .suggestions
            .get(0);

        if let Some(suggestion) = suggestion {
            if self.at_line_end() {
                let mut suggestion = suggestion[..suggestion.find(':').unwrap_or(0)].to_owned();

                self.color
                    .set_fg(self.options.racer_inline_suggestion_color)?;
                self.save_cursor_position()?;
                self.terminal.clear(ClearType::FromCursorDown)?;

                StringTools::strings_unique(&self.buffer, &mut suggestion);

                let overflow = self.screen_height_overflow_by_str(&suggestion);
                if overflow != 0 {
                    self.internal_cursor.total_wrapped_lines += overflow;
                }

                self.write(&suggestion)?;

                self.reset_cursor_position()?;

                if overflow != 0 {
                    self.cursor.move_up(overflow as u16);
                    self.internal_cursor.y -= overflow;
                }

                self.color.reset()?;
            }
        }

        Ok(())
    }

    pub fn cycle_suggestions(&mut self) -> Result<(), IRustError> {
        if self.at_line_end() {
            // Clear screen from cursor down
            self.terminal.clear(ClearType::FromCursorDown)?;

            // No suggestions to show
            if self
                .racer
                .as_ref()
                ?
                .suggestions
                .is_empty()
            {
                return Ok(());
            }

            // Write inline suggestion
            self.write_next_suggestion()?;

            // Max suggestions number to show
            let suggestions_num = std::cmp::min(
                self.racer
                    .as_ref()
                    ?
                    .suggestions
                    .len(),
                self.options.racer_max_suggestions,
            );

            // Handle screen height overflow
            let height_overflow = self.screen_height_overflow_by_new_lines(suggestions_num);
            if height_overflow != 0 {
                self.terminal.scroll_up((height_overflow) as i16)?;
                self.cursor.move_up((height_overflow) as u16);
                self.internal_cursor.y -= height_overflow;
            }

            // Save cursors postions from this point (Input position)
            self.save_cursor_position()?;

            // Write from screen start if a suggestion will be truncated
            let mut max_width = self.size.0 - self.internal_cursor.x % self.size.0;
            if self
                .racer
                .as_ref()
                ?
                .suggestions
                .iter()
                .any(|s| s.len() > max_width)
            {
                self.internal_cursor.x = 0;
                self.go_to_cursor()?;

                self.cursor
                    .move_down(self.internal_cursor.current_wrapped_lines as u16);

                max_width = self.size.0 - self.internal_cursor.x % self.size.0;
            }

            // Write the suggestions
            self.color
                .set_fg(self.options.racer_suggestions_table_color)?;
            let current_suggestion = self
                .racer
                .as_ref()
                ?
                .current_suggestion()
                .map(|s| s.to_string());

            for (idx, suggestion) in self
                .racer
                .as_ref()
                ?
                .suggestions
                .iter()
                .skip(
                    ((self
                        .racer
                        .as_ref()
                        ?
                        .suggestion_idx
                        - 1)
                        / suggestions_num)
                        * suggestions_num,
                )
                .take(suggestions_num)
                .enumerate()
            {
                // color selected suggestion
                if Some(suggestion) == current_suggestion.as_ref() {
                    self.color
                        .set_bg(self.options.racer_selected_suggestion_color)?;
                }
                // trancuate long suggestions
                let mut suggestion = suggestion.to_owned();
                if suggestion.len() > max_width {
                    suggestion.truncate(max_width - 3);
                    suggestion.push_str("...");
                }
                // move one + idx row down
                self.cursor.move_down(idx as u16 + 1);

                // write suggestion
                self.terminal.write(&suggestion)?;

                // move back to initial position
                self.cursor.move_up(idx as u16 + 1);
                self.cursor.move_left(suggestion.len() as u16);

                // reset color in case of current suggestion
                self.color.set_bg(crossterm::Color::Reset)?;
            }

            // reset to input position and color
            self.color.reset()?;
            self.reset_cursor_position()?;
        }

        Ok(())
    }

    pub fn use_suggestion(&mut self) -> Result<(), IRustError> {
        if let Some(suggestion) = self
            .racer
            .as_ref()
            ?
            .current_suggestion()
        {
            let mut suggestion = suggestion[..suggestion.find(':').unwrap_or(0)].to_owned();
            StringTools::strings_unique(&self.buffer, &mut suggestion);

            // update total wrapped lines count each time we touch the buffer
            self.buffer.push_str(&suggestion);
            self.update_total_wrapped_lines();

            self.write(&suggestion)?;
        }

        Ok(())
    }

    pub fn lock_racer_update(&mut self) -> Result<(), IRustError> {
        self.racer
            .as_mut()
            ?
            .update_lock = true;
        Ok(())
    }

    pub fn unlock_racer_update(&mut self) -> Result<(), IRustError> {
        self.racer
            .as_mut()
            ?
            .update_lock = false;
        Ok(())
    }

    pub fn racer_update_locked(&mut self) -> Result<bool, IRustError> {
        Ok(self
            .racer
            .as_ref()
            ?
            .update_lock)
    }

    pub fn check_racer_callback(&mut self) -> Result<(), IRustError> {
        if let Some(character) = self.buffer.chars().last() {
            if character.is_alphanumeric()
                && !self.racer_update_locked()?
                && self.debouncer.recv.try_recv().is_ok()
            {
                self.update_suggestions()?;
                self.lock_racer_update()?;
                self.write_first_suggestion()?;
                self.debouncer.reset_timer();
            }
        }

        Ok(())
    }
}
