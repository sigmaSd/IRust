use super::buffer::Buffer;
use super::racer::Cycle;
use crate::irust::printer::{Printer, PrinterItem, PrinterItemType};
use crate::irust::{IRust, IRustError};
use crate::utils::StringTools;
use crossterm::{ClearType, Color};

impl IRust {
    pub fn handle_character(&mut self, c: char) -> Result<(), IRustError> {
        self.buffer.insert(c);
        self.history.update_buffer_copy(&self.buffer.to_string());
        self.write_input()?;
        self.cursor.move_right_unbounded();

        // Ignore RacerDisabled error
        let _ = self.unlock_racer_update();
        Ok(())
    }

    pub fn handle_enter(&mut self) -> Result<(), IRustError> {
        let buffer = self.buffer.to_string();

        if self.incomplete_input(&buffer) {
            self.buffer.insert('\n');
            self.write_input()?;
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
        let mut output = match self.parse() {
            Ok(out) => out,
            Err(e) => Printer::new(PrinterItem::new(e.to_string(), PrinterItemType::Err)),
        };

        // ensure buffer is cleaned
        self.buffer.clear();

        // reset history current
        self.history.reset_buffer_copy();

        // write out
        if !output.is_empty() {
            output.add_new_line(1);
            self.write_output(output)?;
        }

        self.write_input()?;
        self.write_from_terminal_start(super::IN, Color::Yellow)?;

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
        if self.cursor.is_at_first_input_line() {
            self.handle_history("up")?;
        } else {
            self.cursor.move_up_bounded(1);
            // set buffer cursor
            let buffer_pos = self.cursor.cursor_pos_to_buffer_pos();
            self.buffer.set_buffer_pos(buffer_pos);
        }
        Ok(())
    }

    pub fn handle_down(&mut self) -> Result<(), IRustError> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        if self.cursor.is_at_last_input_line(&self.buffer) {
            self.handle_history("down")?;
        } else {
            self.cursor.move_down_bounded(1);
            // set buffer cursor
            let buffer_pos = self.cursor.cursor_pos_to_buffer_pos();
            self.buffer.set_buffer_pos(buffer_pos);
        }
        Ok(())
    }

    fn handle_history(&mut self, direction: &str) -> Result<(), IRustError> {
        let history = match direction {
            "up" => self.history.up(),
            "down" => self.history.down(),
            _ => panic!("incorrect usage of handle_history function"),
        };

        if let Some(history) = history {
            self.buffer =
                Buffer::from_str(&history, self.cursor.bound.width - super::INPUT_START_COL);

            self.write_input()?;

            let last_input_pos = self.cursor.input_last_pos(&self.buffer);
            self.buffer.goto_end();
            self.cursor.goto(last_input_pos.0, last_input_pos.1);
        }
        Ok(())
    }

    pub fn handle_right(&mut self) -> Result<(), IRustError> {
        if !self.buffer.is_at_end() {
            self.cursor.move_right();
            self.buffer.move_forward();
        } else {
            let _ = self.use_suggestion();
        }
        Ok(())
    }

    pub fn handle_left(&mut self) -> Result<(), IRustError> {
        if !self.buffer.is_at_start() && !self.buffer.is_empty() {
            self.cursor.move_left();
            self.buffer.move_backward();
        }
        Ok(())
    }

    pub fn handle_backspace(&mut self) -> Result<(), IRustError> {
        if !self.buffer.is_at_start() {
            self.buffer.move_backward();
            self.cursor.move_left();
            self.buffer.remove_current_char();

            // update histroy current
            self.history.update_buffer_copy(&self.buffer.to_string());

            self.write_input()?;

            // Ignore RacerDisabled error
            let _ = self.unlock_racer_update();
        }
        Ok(())
    }

    pub fn handle_del(&mut self) -> Result<(), IRustError> {
        if !self.buffer.is_empty() {
            self.buffer.remove_current_char();
            self.history.update_buffer_copy(&self.buffer.to_string());
            self.write_input()?;
        }
        Ok(())
    }

    pub fn handle_ctrl_c(&mut self) -> Result<(), IRustError> {
        if self.buffer.is_empty() {
            self.exit()?;
        } else {
            self.write_newline()?;
            self.raw_terminal.clear(ClearType::FromCursorDown)?;
            self.write_from_terminal_start(super::IN, Color::Yellow)?;
            self.buffer.clear();
        }
        Ok(())
    }

    pub fn handle_ctrl_d(&mut self) -> Result<(), IRustError> {
        if self.buffer.is_empty() {
            self.exit()?;
        }
        Ok(())
    }

    fn exit(&mut self) -> Result<(), IRustError> {
        self.history.save();
        self.raw_terminal.clear(ClearType::All)?;
        self.raw_terminal.exit();
        Ok(())
    }

    pub fn handle_ctrl_z(&mut self) -> Result<(), IRustError> {
        #[cfg(unix)]
        {
            use nix::{
                sys::signal::{kill, Signal},
                unistd::Pid,
            };
            self.raw_terminal.clear(ClearType::All)?;
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
        self.buffer.goto_start();
        self.cursor.goto(4, self.cursor.pos.starting_pos.1);
        Ok(())
    }

    pub fn handle_end_key(&mut self) -> Result<(), IRustError> {
        let last_input_pos = self.cursor.input_last_pos(&self.buffer);
        self.buffer.goto_end();
        self.cursor.goto(last_input_pos.0, last_input_pos.1);
        Ok(())
    }

    pub fn handle_ctrl_left(&mut self) {
        if self.buffer.is_empty() || self.buffer.is_at_start() {
            return;
        }

        self.cursor.move_left();
        self.buffer.move_backward();

        if let Some(current_char) = self.buffer.current_char() {
            match *current_char {
                ' ' => {
                    while self.buffer.previous_char() == Some(&' ') {
                        self.cursor.move_left();
                        self.buffer.move_backward()
                    }
                }
                c if c.is_alphanumeric() => {
                    while let Some(previous_char) = self.buffer.previous_char() {
                        if previous_char.is_alphanumeric() {
                            self.cursor.move_left();
                            self.buffer.move_backward()
                        } else {
                            break;
                        }
                    }
                }

                _ => {
                    while let Some(previous_char) = self.buffer.previous_char() {
                        if !previous_char.is_alphanumeric() && *previous_char != ' ' {
                            self.cursor.move_left();
                            self.buffer.move_backward()
                        } else {
                            break;
                        }
                    }
                }
            }
        }
    }

    pub fn handle_ctrl_right(&mut self) {
        if !self.buffer.is_at_end() {
            self.cursor.move_right();
            self.buffer.move_forward();
        } else {
            let _ = self.use_suggestion();
        }

        if let Some(current_char) = self.buffer.current_char() {
            match *current_char {
                ' ' => {
                    while self.buffer.next_char() == Some(&' ') {
                        self.cursor.move_right();
                        self.buffer.move_forward();
                    }
                    self.cursor.move_right();
                    self.buffer.move_forward();
                }
                c if c.is_alphanumeric() => {
                    while let Some(character) = self.buffer.current_char() {
                        if !character.is_alphanumeric() {
                            break;
                        }
                        self.cursor.move_right();
                        self.buffer.move_forward();
                    }
                }

                _ => {
                    while let Some(character) = self.buffer.current_char() {
                        if character.is_alphanumeric() || *character == ' ' {
                            break;
                        }
                        self.cursor.move_right();
                        self.buffer.move_forward();
                    }
                }
            }
        }
    }
}
