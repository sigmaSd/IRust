use super::cursor::Move;
use super::racer::Cycle;
use crate::irust::printer::{Printer, PrinterItem, PrinterItemType};
use crate::irust::{IRust, IRustError};
use crate::utils::StringTools;
use crossterm::ClearType;

impl IRust {
    pub fn handle_character(&mut self, c: char) -> Result<(), IRustError> {
        // clear suggestion
        self.clear_suggestion()?;

        // check for scroll
        if self.internal_cursor.screen_pos == self.size {
            self.scroll_up(1);
        }

        // Insert input char in buffer
        StringTools::insert_at_char_idx(&mut self.buffer, self.internal_cursor.buffer_pos, c);

        // update histroy current
        self.history.update_current(&self.buffer);

        // advance buffer pos
        self.internal_cursor.move_buffer_cursor_right();

        // unbound upper limit
        self.internal_cursor.current_bounds_mut().1 = self.size.0;

        // Write input char
        self.write_insert(Some(&c.to_string()))?;

        Ok(())
    }

    pub fn handle_enter(&mut self) -> Result<(), IRustError> {
        // clear suggestion
        self.clear_suggestion()?;

        // handle incomplete input
        if self.incomplete_input() {
            self.handle_incomplete_input()?;
            return Ok(());
        }

        // create a new line
        self.write_newline()?;

        // add commands to history
        if self.should_push_to_history(&self.buffer) {
            self.history.push(self.buffer.clone());
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
        // update history current
        self.history.update_current(&self.buffer);

        // write out
        if !self.printer.is_empty() {
            self.printer.add_new_line(1);
            self.write_out()?;
        }

        // new input
        self.write_in()?;

        Ok(())
    }

    fn incomplete_input(&self) -> bool {
        self.at_line_end() && StringTools::unmatched_brackets(&self.buffer)
            || self
                .buffer
                .trim_end()
                .ends_with(|c| c == ':' || c == '.' || c == '=')
    }

    fn handle_incomplete_input(&mut self) -> Result<(), IRustError> {
        self.internal_cursor.current_bounds_mut().1 = self.internal_cursor.screen_pos.0 - 1;
        self.internal_cursor.screen_pos.0 = 4;
        self.internal_cursor.screen_pos.1 += 1;
        if self.internal_cursor.screen_pos.1 == self.size.1 {
            self.scroll_up(1);
        }
        self.internal_cursor.add_bounds();

        self.goto_cursor()?;
        Ok(())
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
            self.internal_cursor.reset_screen_cursor();
            self.move_cursor_to(
                self.internal_cursor.lock_pos.0,
                self.internal_cursor.lock_pos.1,
            )?;
            self.terminal.clear(ClearType::FromCursorDown)?;
            self.buffer = up.clone();
            self.internal_cursor.buffer_pos = self.buffer.len();

            let overflow = self.screen_height_overflow_by_str(&up);

            if overflow != 0 {
                self.scroll_up(overflow);
            }

            self.write(&up)?;
        }

        Ok(())
    }

    pub fn handle_down(&mut self) -> Result<(), IRustError> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        if let Some(down) = self.history.down() {
            self.internal_cursor.reset_screen_cursor();
            self.move_cursor_to(
                self.internal_cursor.lock_pos.0,
                self.internal_cursor.lock_pos.1,
            )?;
            self.terminal.clear(ClearType::FromCursorDown)?;
            self.buffer = down.clone();
            self.internal_cursor.buffer_pos = self.buffer.len();

            let overflow = self.screen_height_overflow_by_str(&down);

            if overflow != 0 {
                self.scroll_up(overflow);
            }

            self.write(&down)?;
        }

        Ok(())
    }

    pub fn handle_left(&mut self) -> Result<(), IRustError> {
        self.clear_suggestion()?;

        if self.internal_cursor.buffer_pos > 0 {
            self.move_cursor_left(Move::Free)?;
            self.internal_cursor.move_buffer_cursor_left();
        }
        Ok(())
    }

    pub fn handle_right(&mut self) -> Result<(), IRustError> {
        if !self.at_line_end() {
            self.move_cursor_right()?;
            self.internal_cursor.move_buffer_cursor_right();
        } else {
            let _ = self.use_suggestion();
        }
        Ok(())
    }

    pub fn handle_backspace(&mut self) -> Result<(), IRustError> {
        if self.internal_cursor.buffer_pos > 0 {
            self.move_cursor_left(Move::Modify)?;
            self.internal_cursor.move_buffer_cursor_left();
            self.delete_char()?;
            // update histroy current
            self.history.update_current(&self.buffer);
        }
        Ok(())
    }

    pub fn handle_del(&mut self) -> Result<(), IRustError> {
        if self.internal_cursor.buffer_pos > 0 {
            self.delete_char()?;
        }
        Ok(())
    }

    pub fn handle_ctrl_c(&mut self) -> Result<(), IRustError> {
        if self.buffer.is_empty() {
            self.exit()?;
        } else {
            self.clear_suggestion()?;

            self.buffer.clear();
            self.write_newline()?;
            self.write_in()?;
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
        self.clear_suggestion()?;
        let distance_to_start =
            self.internal_cursor.screen_pos.0 - self.internal_cursor.current_bounds_mut().0;
        if distance_to_start != 0 {
            self.cursor.move_left(distance_to_start as u16);
            self.internal_cursor.screen_pos.0 -= distance_to_start;
            self.internal_cursor.buffer_pos -= distance_to_start;
        }

        Ok(())
    }

    pub fn go_to_end(&mut self) -> Result<(), IRustError> {
        if self.at_line_end() {
            let _ = self.use_suggestion();
        } else {
            let distance_to_end = {
                let c1 =
                    self.internal_cursor.current_bounds_mut().1 - self.internal_cursor.screen_pos.0;
                let c2 = self.buffer.len() - self.internal_cursor.buffer_pos;
                std::cmp::min(c1, c2)
            };
            if distance_to_end != 0 {
                self.cursor.move_right(distance_to_end as u16);
                self.internal_cursor.screen_pos.0 += distance_to_end;
                self.internal_cursor.buffer_pos += distance_to_end;
            }
        }

        Ok(())
    }

    pub fn handle_ctrl_left(&mut self) -> Option<()> {
        let _ = self.clear_suggestion();

        if self.internal_cursor.buffer_pos < 1 {
            return Some(());
        }

        let buffer = self.buffer.chars().collect::<Vec<char>>();

        let _ = self.move_cursor_left(Move::Free);
        self.internal_cursor.move_buffer_cursor_left();

        if let Some(current_char) = buffer.get(self.internal_cursor.buffer_pos.checked_sub(1)?) {
            match *current_char {
                ' ' => {
                    while buffer[self.internal_cursor.buffer_pos] == ' ' {
                        let _ = self.move_cursor_left(Move::Free);
                        self.internal_cursor.move_buffer_cursor_left();
                    }
                }
                c if c.is_alphanumeric() => {
                    while buffer[self.internal_cursor.buffer_pos.checked_sub(1)?].is_alphanumeric()
                    {
                        let _ = self.move_cursor_left(Move::Free);
                        self.internal_cursor.move_buffer_cursor_left();
                    }
                }

                _ => {
                    while !buffer[self.internal_cursor.buffer_pos.checked_sub(1)?].is_alphanumeric()
                        && buffer[self.internal_cursor.buffer_pos.checked_sub(1)?] != ' '
                    {
                        let _ = self.move_cursor_left(Move::Free);
                        self.internal_cursor.move_buffer_cursor_left();
                    }
                }
            }
        }
        Some(())
    }

    pub fn handle_ctrl_right(&mut self) {
        let buffer = self.buffer.chars().collect::<Vec<char>>();
        if !self.at_line_end() {
            let _ = self.move_cursor_right();
            self.internal_cursor.move_buffer_cursor_right();
        } else {
            let _ = self.use_suggestion();
        }
        if let Some(current_char) = buffer.get(self.internal_cursor.buffer_pos) {
            match *current_char {
                ' ' => {
                    while buffer.get(self.internal_cursor.buffer_pos + 1) == Some(&' ') {
                        let _ = self.move_cursor_right();
                        self.internal_cursor.move_buffer_cursor_right();
                    }
                    let _ = self.move_cursor_right();
                    self.internal_cursor.move_buffer_cursor_right();
                }
                c if c.is_alphanumeric() => {
                    while let Some(character) = buffer.get(self.internal_cursor.buffer_pos) {
                        if !character.is_alphanumeric() {
                            break;
                        }
                        let _ = self.move_cursor_right();
                        self.internal_cursor.move_buffer_cursor_right();
                    }
                }

                _ => {
                    while let Some(character) = buffer.get(self.internal_cursor.buffer_pos) {
                        if character.is_alphanumeric() || *character == ' ' {
                            break;
                        }
                        let _ = self.move_cursor_right();
                        self.internal_cursor.move_buffer_cursor_right();
                    }
                }
            }
        }
    }
}
