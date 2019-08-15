use super::buffer::Buffer;
use super::racer::Cycle;
use crate::irust::printer::{Printer, PrinterItem, PrinterItemType};
use crate::irust::{IRust, IRustError};
use crate::utils::StringTools;
use crossterm::ClearType;

impl IRust {
    pub fn handle_character(&mut self, c: char) -> Result<(), IRustError> {
        self.buf.insert(c);
        self.print()?;
        self.cursor.move_right_unbounded();
        self.unlock_racer_update()?;
        Ok(())
    }

    pub fn handle_enter(&mut self) -> Result<(), IRustError> {
        let buffer = self.buf.to_string();

        if self.incomplete_input(&buffer) {
            self.buf.insert('\n');
            self.print()?;
            self.cursor.goto(4, self.cursor.pos.current_pos.1 + 1);
            return Ok(());
        }

        self.cursor.hide();

        // create a new line
        self.write_newline()?;

        // add commands to history
        if self.should_push_to_history(&buffer) {
            self.history.push(buffer);
        }

        // parse and handle errors
        match self.parse() {
            Ok(out) => {
                self.printer = out;
            }
            Err(e) => {
                self.printer = Printer::new(PrinterItem::new(e.to_string(), PrinterItemType::Err));
            }
        }

        // ensure buffer is cleaned
        self.buf.clear();

        // update history current
        self.history.update_current("");

        // write out
        if !self.printer.is_empty() {
            self.printer.add_new_line(1);
            self.write_out()?;
        }

        self.print()?;
        self.write_in()?;

        self.cursor.show();
        Ok(())
    }

    fn incomplete_input(&self, buffer: &str) -> bool {
        StringTools::unmatched_brackets(&buffer)
            || buffer
                .trim_end()
                .ends_with(|c| c == ':' || c == '.' || c == '=')
    }

    pub fn handle_tab(&mut self) -> Result<(), IRustError> {
        match || -> Result<(), IRustError> {
            self.update_suggestions()?;
            self.lock_racer_update()?;
            self.cycle_suggestions(Cycle::Down)?;
            Ok(())
        }() {
            Ok(_) | Err(IRustError::RacerDisabled) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn handle_back_tab(&mut self) -> Result<(), IRustError> {
        match || -> Result<(), IRustError> {
            self.update_suggestions()?;
            self.lock_racer_update()?;
            self.cycle_suggestions(Cycle::Up)?;
            Ok(())
        }() {
            Ok(_) | Err(IRustError::RacerDisabled) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn handle_up(&mut self) -> Result<(), IRustError> {
        self.handle_history("up")
    }

    pub fn handle_down(&mut self) -> Result<(), IRustError> {
        if self.buf.is_empty() {
            return Ok(());
        }
        self.handle_history("down")
    }

    fn handle_history(&mut self, direction: &str) -> Result<(), IRustError> {
        let history = match direction {
            "up" => self.history.up(),
            "down" => self.history.down(),
            _ => panic!("incorrect usage of handle_history function"),
        };

        if let Some(history) = history {
            self.buf = Buffer::from_str(&history, self.size.0 - 1);

            // scroll if needed
            let last_input_row = self.input_last_pos().1;
            let height_overflow = last_input_row.saturating_sub(self.size.1 - 1);
            if height_overflow > 0 {
                self.scroll_up(height_overflow);
            }

            self.buf.goto_start();
            self.cursor.goto_start();
            self.print()?;
            self.write_in()?;
        }
        Ok(())
    }

    pub fn handle_right(&mut self) -> Result<(), IRustError> {
        if !self.buf.is_at_end() {
            self.cursor.move_right();
            self.buf.move_forward();
        } else {
            let _ = self.use_suggestion();
        }
        Ok(())
    }

    pub fn handle_left(&mut self) -> Result<(), IRustError> {
        if !self.buf.is_at_start() && !self.buf.is_empty() {
            self.cursor.move_left();
            self.buf.move_backward();
        }
        Ok(())
    }

    pub fn handle_backspace(&mut self) -> Result<(), IRustError> {
        if !self.buf.is_at_start() {
            self.buf.move_backward();
            self.cursor.move_left();
            self.buf.remove_current_char();

            // update histroy current
            self.history.update_current(&self.buf.to_string());

            self.print()?;
            self.unlock_racer_update()?;
        }
        Ok(())
    }

    pub fn handle_del(&mut self) -> Result<(), IRustError> {
        if !self.buf.is_empty() {
            self.buf.remove_current_char();
            self.history.update_current(&self.buf.to_string());
            self.print()?;
        }
        Ok(())
    }

    pub fn handle_ctrl_c(&mut self) -> Result<(), IRustError> {
        if self.buf.is_empty() {
            self.exit()?;
        } else {
            self.write_newline()?;
            self.terminal.clear(ClearType::FromCursorDown)?;
            self.write_in()?;
            self.buf.clear();
        }
        Ok(())
    }

    pub fn handle_ctrl_d(&mut self) -> Result<(), IRustError> {
        if self.buf.is_empty() {
            self.exit()?;
        }
        Ok(())
    }

    fn exit(&mut self) -> Result<(), IRustError> {
        crossterm::RawScreen::disable_raw_mode()?;
        self.history.save();
        self.terminal.clear(ClearType::All)?;
        self.terminal.exit();
        Ok(())
    }

    pub fn handle_ctrl_z(&mut self) -> Result<(), IRustError> {
        #[cfg(unix)]
        {
            use nix::{
                sys::signal::{kill, Signal},
                unistd::Pid,
            };
            self.terminal.clear(ClearType::All)?;
            let _ = kill(Pid::this(), Some(Signal::SIGTSTP));

            // display empty prompt after SIGCONT
            self.clear()?;
        }
        Ok(())
    }

    pub fn handle_ctrl_l(&mut self) -> Result<(), IRustError> {
        self.clear()?;
        Ok(())
    }

    pub fn handle_home_key(&mut self) -> Result<(), IRustError> {
        self.buf.goto_start();
        self.cursor.goto(4, self.cursor.pos.starting_pos.1);
        Ok(())
    }

    pub fn handle_end_key(&mut self) -> Result<(), IRustError> {
        // TODO
        Ok(())
    }

    pub fn handle_ctrl_left(&mut self) {
        if self.buf.is_empty() || self.buf.is_at_start() {
            return;
        }

        self.cursor.move_left();
        self.buf.move_backward();

        if let Some(current_char) = self.buf.current_char() {
            match *current_char {
                ' ' => {
                    while self.buf.previous_char() == Some(&' ') {
                        self.cursor.move_left();
                        self.buf.move_backward()
                    }
                }
                c if c.is_alphanumeric() => {
                    while let Some(previous_char) = self.buf.previous_char() {
                        if previous_char.is_alphanumeric() {
                            self.cursor.move_left();
                            self.buf.move_backward()
                        } else {
                            break;
                        }
                    }
                }

                _ => {
                    while let Some(previous_char) = self.buf.previous_char() {
                        if !previous_char.is_alphanumeric() && *previous_char != ' ' {
                            self.cursor.move_left();
                            self.buf.move_backward()
                        } else {
                            break;
                        }
                    }
                }
            }
        }
    }

    pub fn handle_ctrl_right(&mut self) {
        if !self.buf.is_at_end() {
            self.cursor.move_right();
            self.buf.move_forward();
        } else {
            let _ = self.use_suggestion();
        }

        if let Some(current_char) = self.buf.current_char() {
            match *current_char {
                ' ' => {
                    while self.buf.next_char() == Some(&' ') {
                        self.cursor.move_right();
                        self.buf.move_forward();
                    }
                    self.cursor.move_right();
                    self.buf.move_forward();
                }
                c if c.is_alphanumeric() => {
                    while let Some(character) = self.buf.current_char() {
                        if !character.is_alphanumeric() {
                            break;
                        }
                        self.cursor.move_right();
                        self.buf.move_forward();
                    }
                }

                _ => {
                    while let Some(character) = self.buf.current_char() {
                        if character.is_alphanumeric() || *character == ' ' {
                            break;
                        }
                        self.cursor.move_right();
                        self.buf.move_forward();
                    }
                }
            }
        }
    }
}
