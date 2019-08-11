use super::IRustError;
use crate::irust::IRust;
use crate::utils::{read_until_bytes, StringTools};
use crossterm::ClearType;
use std::env::temp_dir;
use std::io::{self, Write};
use std::process::{Child, Command, Stdio};

pub enum Cycle {
    Up,
    Down,
}

pub struct Racer {
    process: Child,
    main_file: String,
    cursor: (usize, usize),
    // suggestions: (Name, definition)
    suggestions: Vec<(String, String)>,
    suggestion_idx: usize,
    cmds: [String; 8],
    update_lock: bool,
}

impl Racer {
    pub fn start() -> Result<Racer, IRustError> {
        let process = Command::new("racer")
            .arg("daemon")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            // Disable Racer if unable to start it
            .map_err(|_| IRustError::RacerDisabled)?;
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
            "type".to_string(),
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

    fn complete_code(&mut self) -> io::Result<()> {
        // check for lock
        if self.update_lock {
            return Ok(());
        }

        // reset suggestions
        self.suggestions.clear();
        self.goto_first_suggestion();

        let stdin = self.process.stdin.as_mut().unwrap();
        let stdout = self.process.stdout.as_mut().unwrap();

        writeln!(
            stdin,
            "complete {} {} {}",
            self.cursor.0, self.cursor.1, self.main_file
        )?;

        // read till END
        let mut raw_output = vec![];
        read_until_bytes(
            &mut std::io::BufReader::new(stdout),
            b"END",
            &mut raw_output,
        )?;
        let raw_output = String::from_utf8(raw_output.to_vec()).unwrap();

        for suggestion in raw_output.lines().skip(1) {
            if suggestion == "END" {
                break;
            }
            let mut try_parse = || -> Option<()> {
                let start_idx = suggestion.find("MATCH ")? + 6;
                let mut indices = suggestion.match_indices(',');
                let name = suggestion[start_idx..indices.nth(0)?.0].to_owned();
                let definition = suggestion[indices.nth(3)?.0..].to_owned();
                self.suggestions.push((name, definition[1..].to_owned()));
                Some(())
            };

            try_parse();
        }

        // remove duplicates
        self.suggestions.sort();
        self.suggestions.dedup();

        Ok(())
    }

    fn goto_next_suggestion(&mut self) {
        if self.suggestion_idx >= self.suggestions.len() {
            self.suggestion_idx = 0
        }
        self.suggestion_idx += 1;
    }

    fn goto_previous_suggestion(&mut self) {
        self.suggestion_idx = self
            .suggestion_idx
            .checked_sub(1)
            .unwrap_or_else(|| self.suggestions.len());
        if self.suggestion_idx == 0 {
            self.suggestion_idx = self.suggestions.len();
        }
    }

    fn current_suggestion(&self) -> Option<(String, String)> {
        if self.suggestion_idx > 1 {
            self.suggestions
                .get(self.suggestion_idx - 1)
                .map(ToOwned::to_owned)
        } else {
            self.suggestions.get(0).map(ToOwned::to_owned)
        }
    }

    fn goto_first_suggestion(&mut self) {
        self.suggestion_idx = 0;
    }

    fn full_suggestion(s: &(String, String)) -> String {
        if !s.1.is_empty() {
            s.0.to_owned() + ": " + &s.1
        } else {
            s.0.to_owned()
        }
    }
}

impl IRust {
    pub fn update_suggestions(&mut self) -> Result<(), IRustError> {
        // return if we're not at the end of the line
        if !self.cursor.is_at_line_end(&self) {
            return Ok(());
        }

        // don't autocomplete shell commands
        if self.buffer.starts_with("::") {
            return Ok(());
        }

        self.show_suggestions_inner()?;

        Ok(())
    }

    fn show_suggestions_inner(&mut self) -> Result<(), IRustError> {
        if self.buffer.starts_with(':') {
            // Auto complete IRust commands
            self.racer.as_mut()?.suggestions = self
                .racer
                .as_ref()?
                .cmds
                .iter()
                .filter(|c| c.starts_with(&self.buffer[1..]))
                // place holder for IRust command definitions
                .map(|c| (c.to_owned(), String::new()))
                .collect();
        } else {
            // Auto complete rust code
            let mut racer = self.racer.as_mut()?;
            racer.cursor.0 = self.repl.body.len();
            // add +1 for the \t
            racer.cursor.1 = StringTools::chars_count(&self.buffer) + 1;

            self.repl
                .exec_in_tmp_repl(self.buffer.clone(), move || -> Result<(), IRustError> {
                    racer.complete_code().map_err(From::from)
                })?;

            // reset debouncer
            self.debouncer.reset_timer();
        }

        Ok(())
    }

    fn write_next_suggestion(&mut self) -> Result<(), IRustError> {
        self.racer.as_mut()?.goto_next_suggestion();
        self.write_current_suggestion()?;

        Ok(())
    }

    fn write_previous_suggestion(&mut self) -> Result<(), IRustError> {
        self.racer.as_mut()?.goto_previous_suggestion();
        self.write_current_suggestion()?;

        Ok(())
    }

    fn write_first_suggestion(&mut self) -> Result<(), IRustError> {
        self.racer.as_mut()?.goto_first_suggestion();
        self.write_current_suggestion()?;
        Ok(())
    }

    fn write_current_suggestion(&mut self) -> Result<(), IRustError> {
        if let Some(suggestion) = self.racer.as_ref()?.current_suggestion() {
            if self.cursor.is_at_line_end(&self) {
                let mut suggestion = suggestion.0;

                self.color
                    .set_fg(self.options.racer_inline_suggestion_color)?;
                self.cursor.hide();
                self.cursor.save_position()?;
                self.terminal.clear(ClearType::FromCursorDown)?;

                StringTools::strings_unique(&self.buffer, &mut suggestion);

                let overflow = self.cursor.screen_height_overflow_by_str(&self, &suggestion);

                self.write(&suggestion)?;

                self.cursor.reset_position()?;
                self.cursor.show();

                if overflow != 0 {
                    self.cursor.move_up(overflow as u16);
                    self.cursor.pos.screen_pos.1 -= overflow;
                }

                self.color.reset()?;
            }
        }

        Ok(())
    }

    pub fn cycle_suggestions(&mut self, cycle: Cycle) -> Result<(), IRustError> {
        if self.cursor.is_at_line_end(&self) {
            // Clear screen from cursor down
            self.terminal.clear(ClearType::FromCursorDown)?;

            // No suggestions to show
            if self.racer.as_ref()?.suggestions.is_empty() {
                return Ok(());
            }

            // Write inline suggestion
            match cycle {
                Cycle::Down => self.write_next_suggestion()?,
                Cycle::Up => self.write_previous_suggestion()?,
            }

            // Max suggestions number to show
            let suggestions_num = std::cmp::min(
                self.racer.as_ref()?.suggestions.len(),
                self.options.racer_max_suggestions,
            );

            // Handle screen height overflow
            let height_overflow = self.cursor.screen_height_overflow_by_new_lines(&self, suggestions_num + 1);
            if height_overflow != 0 {
                self.scroll_up(height_overflow);
            }

            // Save cursors postions from this point (Input position)
            self.cursor.save_position()?;

            // Write from screen start if a suggestion will be truncated
            let mut max_width = self.size.0 - self.cursor.pos.screen_pos.0;
            if self
                .racer
                .as_ref()?
                .suggestions
                .iter()
                .any(|s| Racer::full_suggestion(s).len() > max_width)
            {
                self.cursor.pos.screen_pos.0 = 0;
                self.cursor.goto_internal_pos()?;

                max_width = self.size.0 - self.cursor.pos.screen_pos.0;
            }

            // Write the suggestions
            self.color
                .set_fg(self.options.racer_suggestions_table_color)?;
            let current_suggestion = self.racer.as_ref()?.current_suggestion();

            for (idx, suggestion) in self
                .racer
                .as_ref()?
                .suggestions
                .iter()
                .skip(
                    ((self.racer.as_ref()?.suggestion_idx - 1) / suggestions_num) * suggestions_num,
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
                let mut suggestion = Racer::full_suggestion(suggestion);
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
                self.cursor.move_left(suggestion.len() as u16)?;

                // reset color in case of current suggestion
                self.color.set_bg(crossterm::Color::Reset)?;
            }

            // reset to input position and color
            self.color.reset()?;
            self.cursor.reset_position()?;
        }

        Ok(())
    }

    pub fn use_suggestion(&mut self) -> Result<(), IRustError> {
        if let Some(suggestion) = self.racer.as_ref()?.current_suggestion() {
            // suggestion => `name: definition`
            // suggestion example => `assert!: macro_rules! assert {`

            // get the name
            let mut suggestion = suggestion.0;

            // get the unique part of the name
            StringTools::strings_unique(&self.buffer, &mut suggestion);

            self.buffer.push_str(&suggestion);
            self.cursor.pos.buffer_pos = StringTools::chars_count(&self.buffer);

            // clear screen from cursor down
            self.terminal.clear(ClearType::FromCursorDown)?;

            // write the suggestion
            self.write(&suggestion)?;

            // Unlock racer suggestions update
            let _ = self.unlock_racer_update();
        }

        Ok(())
    }

    pub fn lock_racer_update(&mut self) -> Result<(), IRustError> {
        self.racer.as_mut()?.update_lock = true;
        Ok(())
    }

    pub fn unlock_racer_update(&mut self) -> Result<(), IRustError> {
        self.racer.as_mut()?.update_lock = false;
        Ok(())
    }

    fn racer_update_locked(&mut self) -> Result<bool, IRustError> {
        Ok(self.racer.as_ref()?.update_lock)
    }

    pub fn check_racer_callback(&mut self) -> Result<(), IRustError> {
        let mut inner = || -> Result<(), IRustError> {
            if let Some(character) = self.buffer.chars().last() {
                if character.is_alphanumeric()
                    && !self.racer_update_locked()?
                    && self.debouncer.recv.try_recv().is_ok()
                {
                    self.update_suggestions()?;
                    self.write_first_suggestion()?;
                }
            }
            Ok(())
        };

        match inner() {
            Ok(_) | Err(IRustError::RacerDisabled) => Ok(()),
            Err(e) => Err(e),
        }
    }
}
