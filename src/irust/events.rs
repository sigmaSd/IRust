use super::racer::Cycle;
use super::{CTRL_KEYMODIFIER, NO_MODIFIER};
use crate::irust::printer::{PrintQueue, PrinterItem};
use crate::irust::{IRust, IRustError};
use crate::utils::StringTools;
use crossterm::{event::*, style::Color, terminal::ClearType};

mod history_events;

impl IRust {
    pub fn handle_character(&mut self, c: char) -> Result<(), IRustError> {
        self.buffer.insert(c);
        self.printer.print_input(&self.buffer, &self.theme)?;
        self.printer.cursor.move_right_unbounded();
        self.history.unlock();
        // Ignore RacerDisabled error
        let _ = self.racer.as_mut()?.unlock_racer_update();

        Ok(())
    }

    pub fn handle_enter(&mut self, force_eval: bool) -> Result<(), IRustError> {
        self.history.unlock();

        let buffer = self.buffer.to_string();

        if !force_eval && !self.input_is_cmd_or_shell(&buffer) && self.incomplete_input(&buffer) {
            self.buffer.insert('\n');
            self.printer.print_input(&self.buffer, &self.theme)?;
            self.printer.cursor.move_right();
            return Ok(());
        }

        self.printer.cursor.hide();

        // create a new line
        self.printer.write_newline(&self.buffer)?;

        // add commands to history
        if self.should_push_to_history(&buffer) {
            self.history.push(buffer);
        }

        // parse and handle errors
        let mut output = match self.parse() {
            Ok(out) => out,
            Err(e) => {
                let mut printer = PrintQueue::default();
                printer.push(PrinterItem::String(e.to_string(), self.options.err_color));
                printer
            }
        };

        // ensure buffer is cleaned
        self.buffer.clear();

        // write out
        output.add_new_line(1);
        if !output.is_empty() {
            self.printer.print_output(output)?;
        }

        self.printer
            .write_from_terminal_start(super::IN, Color::Yellow)?;

        self.printer.cursor.show();
        Ok(())
    }

    pub fn handle_alt_enter(&mut self) -> Result<(), IRustError> {
        self.buffer.insert('\n');
        self.printer.print_input(&self.buffer, &self.theme)?;
        self.printer.cursor.move_right();
        Ok(())
    }

    pub fn handle_tab(&mut self) -> Result<(), IRustError> {
        if self.buffer.is_at_string_line_start() {
            const TAB: &str = "   \t";

            self.buffer.insert_str(TAB);
            self.printer.print_input(&self.buffer, &self.theme)?;
            for _ in 0..4 {
                self.printer.cursor.move_right_unbounded();
            }
            return Ok(());
        }

        match || -> Result<(), IRustError> {
            self.racer.as_mut()?.update_suggestions(
                &self.buffer,
                &mut self.printer,
                &mut self.repl,
            )?;
            self.racer.as_mut()?.lock_racer_update()?;
            self.racer.as_mut()?.cycle_suggestions(
                &mut self.printer,
                &self.buffer,
                &self.theme,
                Cycle::Down,
                &self.options,
            )?;
            Ok(())
        }() {
            Ok(_) | Err(IRustError::RacerDisabled) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn handle_back_tab(&mut self) -> Result<(), IRustError> {
        match || -> Result<(), IRustError> {
            self.racer.as_mut()?.update_suggestions(
                &self.buffer,
                &mut self.printer,
                &mut self.repl,
            )?;
            self.racer.as_mut()?.lock_racer_update()?;
            self.racer.as_mut()?.cycle_suggestions(
                &mut self.printer,
                &self.buffer,
                &self.theme,
                Cycle::Up,
                &self.options,
            )?;
            Ok(())
        }() {
            Ok(_) | Err(IRustError::RacerDisabled) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn handle_right(&mut self) -> Result<(), IRustError> {
        if !self.buffer.is_at_end() {
            self.printer.cursor.move_right();
            self.buffer.move_forward();
        } else {
            self.use_racer_suggestion()?;
        }
        Ok(())
    }

    pub fn handle_left(&mut self) -> Result<(), IRustError> {
        if !self.buffer.is_at_start() && !self.buffer.is_empty() {
            self.printer.cursor.move_left();
            self.buffer.move_backward();
        }
        Ok(())
    }

    pub fn handle_backspace(&mut self) -> Result<(), IRustError> {
        if !self.buffer.is_at_start() {
            self.buffer.move_backward();
            self.printer.cursor.move_left();
            self.buffer.remove_current_char();
            self.printer.print_input(&self.buffer, &self.theme)?;
            // Ignore RacerDisabled error
            self.history.unlock();
            let _ = self.racer.as_mut()?.unlock_racer_update();
        }
        Ok(())
    }

    pub fn handle_del(&mut self) -> Result<(), IRustError> {
        if !self.buffer.is_empty() {
            self.buffer.remove_current_char();
            self.printer.print_input(&self.buffer, &self.theme)?;
            // Ignore RacerDisabled error
            self.history.unlock();
            let _ = self.racer.as_mut()?.unlock_racer_update();
        }
        Ok(())
    }

    pub fn handle_ctrl_c(&mut self) -> Result<(), IRustError> {
        self.buffer.clear();
        self.history.unlock();
        let _ = self.racer.as_mut()?.unlock_racer_update();
        self.printer.cursor.goto_start();
        self.printer
            .write_from_terminal_start(super::IN, Color::Yellow)?;
        self.printer.writer.raw.clear(ClearType::FromCursorDown)?;
        self.printer.print_input(&self.buffer, &self.theme)?;
        Ok(())
    }

    pub fn handle_ctrl_d(&mut self) -> Result<bool, IRustError> {
        if self.buffer.is_empty() {
            self.printer.write_newline(&self.buffer)?;
            self.printer
                .write("Do you really want to exit ([y]/n)? ", Color::Grey)?;

            loop {
                std::io::Write::flush(&mut self.printer.writer.raw)?;

                if let Ok(key_event) = read() {
                    match key_event {
                        Event::Key(KeyEvent {
                            code: KeyCode::Char(c),
                            modifiers: NO_MODIFIER,
                        }) => match &c {
                            'y' | 'Y' => return Ok(true),
                            _ => {
                                self.printer.write_newline(&self.buffer)?;
                                self.printer.write_newline(&self.buffer)?;
                                self.printer
                                    .write_from_terminal_start(super::IN, Color::Yellow)?;
                                return Ok(false);
                            }
                        },
                        Event::Key(KeyEvent {
                            code: KeyCode::Char('d'),
                            modifiers: CTRL_KEYMODIFIER,
                        })
                        | Event::Key(KeyEvent {
                            code: KeyCode::Enter,
                            ..
                        }) => return Ok(true),
                        _ => continue,
                    }
                }
            }
        }
        Ok(false)
    }

    pub fn exit(&mut self) -> Result<(), IRustError> {
        self.history.save()?;
        self.options.save()?;
        self.theme.save()?;
        self.printer.write_newline(&self.buffer)?;
        self.printer.cursor.show();
        Ok(())
    }

    pub fn handle_ctrl_z(&mut self) -> Result<(), IRustError> {
        #[cfg(unix)]
        {
            use nix::{
                sys::signal::{kill, Signal},
                unistd::Pid,
            };
            self.printer.writer.raw.clear(ClearType::All)?;
            kill(Pid::this(), Some(Signal::SIGTSTP))
                .map_err(|e| format!("failed to sigstop irust. {}", e))?;

            // display empty prompt after SIGCONT
            self.handle_ctrl_l()?;
        }
        Ok(())
    }

    pub fn handle_ctrl_l(&mut self) -> Result<(), IRustError> {
        self.buffer.clear();
        self.buffer.goto_start();
        self.printer.clear()?;
        self.printer.print_input(&self.buffer, &self.theme)?;
        Ok(())
    }

    pub fn handle_home_key(&mut self) -> Result<(), IRustError> {
        self.buffer.goto_start();
        self.printer
            .cursor
            .goto(4, self.printer.cursor.pos.starting_pos.1);
        Ok(())
    }

    pub fn handle_end_key(&mut self) -> Result<(), IRustError> {
        let last_input_pos = self.printer.cursor.input_last_pos(&self.buffer);
        self.buffer.goto_end();
        self.printer.cursor.goto(last_input_pos.0, last_input_pos.1);
        Ok(())
    }

    pub fn handle_ctrl_left(&mut self) {
        if self.buffer.is_empty() || self.buffer.is_at_start() {
            return;
        }

        self.printer.cursor.move_left();
        self.buffer.move_backward();

        if let Some(current_char) = self.buffer.current_char() {
            match *current_char {
                ' ' => {
                    while self.buffer.previous_char() == Some(&' ') {
                        self.printer.cursor.move_left();
                        self.buffer.move_backward()
                    }
                }
                c if c.is_alphanumeric() => {
                    while let Some(previous_char) = self.buffer.previous_char() {
                        if previous_char.is_alphanumeric() {
                            self.printer.cursor.move_left();
                            self.buffer.move_backward()
                        } else {
                            break;
                        }
                    }
                }

                _ => {
                    while let Some(previous_char) = self.buffer.previous_char() {
                        if !previous_char.is_alphanumeric() && *previous_char != ' ' {
                            self.printer.cursor.move_left();
                            self.buffer.move_backward()
                        } else {
                            break;
                        }
                    }
                }
            }
        }
    }

    pub fn handle_ctrl_right(&mut self) -> Result<(), IRustError> {
        if !self.buffer.is_at_end() {
            self.printer.cursor.move_right();
            self.buffer.move_forward();
        } else {
            self.use_racer_suggestion()?;
        }

        if let Some(current_char) = self.buffer.current_char() {
            match *current_char {
                ' ' => {
                    while self.buffer.next_char() == Some(&' ') {
                        self.printer.cursor.move_right();
                        self.buffer.move_forward();
                    }
                    self.printer.cursor.move_right();
                    self.buffer.move_forward();
                }
                c if c.is_alphanumeric() => {
                    while let Some(character) = self.buffer.current_char() {
                        if !character.is_alphanumeric() {
                            break;
                        }
                        self.printer.cursor.move_right();
                        self.buffer.move_forward();
                    }
                }

                _ => {
                    while let Some(character) = self.buffer.current_char() {
                        if character.is_alphanumeric() || *character == ' ' {
                            break;
                        }
                        self.printer.cursor.move_right();
                        self.buffer.move_forward();
                    }
                }
            }
        }
        Ok(())
    }

    pub fn handle_ctrl_e(&mut self) -> Result<(), IRustError> {
        self.handle_enter(true)
    }

    pub fn use_racer_suggestion(&mut self) -> Result<(), IRustError> {
        if let Some(suggestion) = self.racer.as_ref()?.current_suggestion() {
            // suggestion => `name: definition`
            // suggestion example => `assert!: macro_rules! assert {`

            // get the name
            let mut suggestion = suggestion.0;

            // get the unique part of the name
            StringTools::strings_unique(
                &self
                    .buffer
                    .buffer
                    .iter()
                    .take(self.buffer.buffer_pos)
                    .collect::<String>(),
                &mut suggestion,
            );

            self.buffer.insert_str(&suggestion);
            let chars_count = StringTools::chars_count(&suggestion);

            for _ in 0..chars_count {
                self.printer.cursor.move_right_unbounded();
            }

            self.printer.print_input(&self.buffer, &self.theme)?;
        }
        Ok(())
    }

    // helper functions
    fn incomplete_input(&self, buffer: &str) -> bool {
        StringTools::unmatched_brackets(&buffer)
            || buffer
                .trim_end()
                .ends_with(|c| c == ':' || c == '.' || c == '=')
    }

    fn input_is_cmd_or_shell(&self, buffer: &str) -> bool {
        buffer.starts_with(':') || buffer.starts_with("::")
    }
}
