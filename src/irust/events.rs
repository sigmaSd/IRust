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

        if self.cursor.pos.screen_pos.0 == self.size.0 - 1 {
            self.cursor.pos.screen_pos.0 = 4;
            self.cursor.pos.screen_pos.1 += 1;
        } else {
            self.cursor.pos.screen_pos.0 += 1;
        }
        self.cursor.goto_internal_pos()?;

        let _ = self.unlock_racer_update();

        Ok(())
    }

    pub fn handle_enter(&mut self) -> Result<(), IRustError> {
        let buffer = self.buf.to_string();

        if self.incomplete_input(&buffer) {
            self.buf.insert('\n');
            self.print()?;
            self.cursor.goto(4, self.cursor.pos.screen_pos.1 + 1);
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
        self.buffer.clear();
        self.buf.buffer_pos = 0;
        self.buf.clear();

        // update history current
        self.history.update_current(&self.buf.to_string());

        // write out
        if !self.printer.is_empty() {
            self.printer.add_new_line(1);
            self.write_out()?;
        }

        self.print()?;

        self.cursor.pos.screen_pos.0 = 4;
        self.cursor.goto_internal_pos()?;
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
        if let Some(up) = self.history.up() {
            self.buf = Buffer::from_str(&up, self.size.0 - 1);

            let y = self.last_line_row();
            let height_overflow = y.saturating_sub(self.size.1 - 1);
            if height_overflow > 0 {
                self.scroll_up(height_overflow);
            }

            self.buf.goto_start();
            self.cursor.goto(4, self.cursor.pos.starting_pos.1);
            self.print()?;
        }

        Ok(())
    }

    pub fn handle_down(&mut self) -> Result<(), IRustError> {
        if self.buf.is_empty() {
            return Ok(());
        }

        if let Some(down) = self.history.down() {
            self.buf = Buffer::from_str(&down, self.size.0 - 1);

            let y = self.last_line_row();
            let height_overflow = y.saturating_sub(self.size.1 - 1);
            if height_overflow > 0 {
                self.scroll_up(height_overflow);
            }

            self.buf.goto_start();
            self.cursor.goto(4, self.cursor.pos.starting_pos.1);
            self.print()?;
        }

        Ok(())
    }

    pub fn handle_left(&mut self) -> Result<(), IRustError> {
        if !self.buf.is_at_start() && !self.buf.is_empty() {
            self.cursor.move_screen_cursor_left();
            self.buf.move_backward();
        }
        Ok(())
    }

    pub fn handle_right(&mut self) -> Result<(), IRustError> {
        if !self.buf.is_at_end() {
            self.buf.move_forward();
            self.cursor.move_screen_cursor_right();
        } else {
            let _ = self.use_suggestion();
        }
        Ok(())
    }

    pub fn handle_backspace(&mut self) -> Result<(), IRustError> {
        self.color.reset()?;
        if !self.buf.is_at_start() {
            self.buf.move_backward();
            self.cursor.move_screen_cursor_left();
            let _ = self.buf.remove_current_char();

            // update histroy current
            self.history.update_current(&self.buf.to_string());

            self.print()?;
            let _ = self.unlock_racer_update();
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
            self.buf.clear();

            self.terminal.clear(ClearType::FromCursorDown)?;
            self.color.set_fg(crossterm::Color::Yellow)?;
            self.terminal.write("In: ")?;
            self.cursor.pos.screen_pos.0 = 4;
            self.color.reset()?;
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

    pub fn go_to_start(&mut self) -> Result<(), IRustError> {
        self.buf.goto_start();
        self.cursor.goto(4, self.cursor.pos.starting_pos.1);
        Ok(())
    }

    pub fn go_to_end(&mut self) -> Result<(), IRustError> {
        // TODO
        Ok(())
    }

    pub fn handle_ctrl_left(&mut self) -> Option<()> {
        if self.buf.is_empty() || self.buf.is_at_start() {
            return Some(());
        }

        self.cursor.move_screen_cursor_left();
        self.buf.move_backward();

        if let Some(current_char) = self.buf.get(self.buf.buffer_pos.checked_sub(1)?) {
            match *current_char {
                ' ' => {
                    while self.buf.get(self.buf.buffer_pos) == Some(&' ') {
                        self.cursor.move_screen_cursor_left();
                        self.buf.move_backward()
                    }
                }
                c if c.is_alphanumeric() => {
                    while self
                        .buf
                        .get(self.buf.buffer_pos.checked_sub(1)?)
                        .unwrap()
                        .is_alphanumeric()
                    {
                        self.cursor.move_screen_cursor_left();
                        self.buf.move_backward()
                    }
                }

                _ => {
                    while !self
                        .buf
                        .get(self.buf.buffer_pos.checked_sub(1)?)
                        .unwrap()
                        .is_alphanumeric()
                        && self.buf.get(self.buf.buffer_pos.checked_sub(1)?) != Some(&' ')
                    {
                        self.cursor.move_screen_cursor_left();
                        self.buf.move_backward()
                    }
                }
            }
        }
        Some(())
    }

    pub fn handle_ctrl_right(&mut self) {
        if !self.buf.is_at_end() {
            self.cursor.move_screen_cursor_right();
            self.buf.move_forward();
        } else {
            let _ = self.use_suggestion();
        }
        if let Some(current_char) = self.buf.get(self.buf.buffer_pos) {
            match *current_char {
                ' ' => {
                    while self.buf.get(self.buf.buffer_pos + 1) == Some(&' ') {
                        self.cursor.move_screen_cursor_right();
                        self.buf.move_forward();
                    }
                    self.cursor.move_screen_cursor_right();
                    self.buf.move_forward();
                }
                c if c.is_alphanumeric() => {
                    while let Some(character) = self.buf.get(self.buf.buffer_pos) {
                        if !character.is_alphanumeric() {
                            break;
                        }
                        self.cursor.move_screen_cursor_right();
                        self.buf.move_forward();
                    }
                }

                _ => {
                    while let Some(character) = self.buf.get(self.buf.buffer_pos) {
                        if character.is_alphanumeric() || *character == ' ' {
                            break;
                        }
                        self.cursor.move_screen_cursor_right();
                        self.buf.move_forward();
                    }
                }
            }
        }
    }
}
