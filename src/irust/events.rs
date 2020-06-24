use super::racer::Cycle;
use super::{CTRL_KEYMODIFIER, NO_MODIFIER};
use crate::irust::printer::{Printer, PrinterItem, PrinterItemType};
use crate::irust::{IRust, IRustError};
use crate::utils::StringTools;
use crossterm::{event::*, style::Color, terminal::ClearType};

mod history_events;

impl IRust {
    pub fn handle_character(&mut self, c: char) -> Result<(), IRustError> {
        self.buffer.insert(c);
        self.history.update_buffer_copy(&self.buffer.to_string());
        self.print_input()?;
        self.cursor.move_right_unbounded();
        // Ignore RacerDisabled error
        let _ = self.unlock_racer_update();

        Ok(())
    }

    pub fn handle_enter(&mut self) -> Result<(), IRustError> {
        let buffer = self.buffer.to_string();

        if !self.input_is_cmd_or_shell(&buffer) && self.incomplete_input(&buffer) {
            self.write_from_next_line()?;
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
            self.print_output(output)?;
        }

        self.print_input()?;
        self.write_from_terminal_start(super::IN, Color::Yellow)?;

        self.cursor.show();
        Ok(())
    }

    pub fn handle_alt_enter(&mut self) -> Result<(), IRustError> {
        self.write_from_next_line()
    }

    pub fn handle_tab(&mut self) -> Result<(), IRustError> {
        if self.buffer.is_at_string_line_start() {
            const TAB: &str = "   \t";

            self.buffer.insert_str(TAB);
            self.print_input()?;
            for _ in 0..4 {
                self.cursor.move_right_unbounded();
            }
            return Ok(());
        }

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
            self.print_input()?;
            // Ignore RacerDisabled error
            let _ = self.unlock_racer_update();
        }
        Ok(())
    }

    pub fn handle_del(&mut self) -> Result<(), IRustError> {
        if !self.buffer.is_empty() {
            self.buffer.remove_current_char();
            self.history.update_buffer_copy(&self.buffer.to_string());
            self.print_input()?;
            // Ignore RacerDisabled error
            let _ = self.unlock_racer_update();
        }
        Ok(())
    }

    pub fn handle_ctrl_c(&mut self) -> Result<(), IRustError> {
        if !self.buffer.is_empty() {
            self.write_newline()?;
            self.raw_terminal.clear(ClearType::FromCursorDown)?;
            self.write_from_terminal_start(super::IN, Color::Yellow)?;
            self.buffer.clear();
        }
        Ok(())
    }

    pub fn handle_ctrl_d(&mut self) -> Result<(), IRustError> {
        if self.buffer.is_empty() {
            self.write_newline()?;
            self.write("Do you really want to exit ([y]/n)? ", Color::Grey)?;

            loop {
                self.raw_terminal.flush()?;

                if let Ok(key_event) = read() {
                    match key_event {
                        Event::Key(KeyEvent {
                            code: KeyCode::Char(c),
                            modifiers: NO_MODIFIER,
                        }) => match &c {
                            'y' | 'Y' => self.exit()?,
                            _ => {
                                self.write_newline()?;
                                self.write_newline()?;
                                self.write_from_terminal_start(super::IN, Color::Yellow)?;
                                return Ok(());
                            }
                        },
                        Event::Key(KeyEvent {
                            code: KeyCode::Char('d'),
                            modifiers: CTRL_KEYMODIFIER,
                        })
                        | Event::Key(KeyEvent {
                            code: KeyCode::Enter,
                            ..
                        }) => {
                            self.exit()?;
                        }
                        _ => continue,
                    }
                }
            }
        }
        Ok(())
    }

    fn exit(&mut self) -> Result<(), IRustError> {
        self.history.save();
        self.write_newline()?;
        super::RawTerminal::exit(0);
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
        todo!()
        //self.buffer.goto_start();
        //self.cursor.goto(4, self.cursor.pos.starting_pos.1);
        //Ok(())
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
