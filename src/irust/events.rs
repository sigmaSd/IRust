use crate::irust::printer::{Printer, PrinterItem, PrinterItemType};
use crate::irust::IRust;
use crate::utils::StringTools;
use crossterm::ClearType;

impl IRust {
    pub fn handle_character(&mut self, c: char) -> std::io::Result<()> {
        StringTools::insert_at_char_idx(
            &mut self.buffer,
            self.internal_cursor.get_corrected_x(),
            c,
        );
        self.write_insert(Some(&c.to_string()))?;
        Ok(())
    }

    pub fn handle_enter(&mut self) -> std::io::Result<()> {
        // clear suggestion
        self.clear_suggestion()?;

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
                self.printer.add_new_line(1);
            }
        }

        // ensure buffer is cleaned
        self.buffer.clear();

        // write out
        if !self.printer.is_empty() {
            self.write_out()?;
            self.write_newline()?;
        }

        // reset wrapped lines counter
        self.internal_cursor.reset_wrapped_lines();

        // new input
        self.write_in()?;
        Ok(())
    }

    pub fn handle_tab(&mut self) -> std::io::Result<()> {
        self.update_suggestions()?;
        self.lock_racer_update();
        self.cycle_suggestions()?;
        Ok(())
    }

    pub fn handle_up(&mut self) -> std::io::Result<()> {
        self.internal_cursor.x = 4;
        self.move_cursor_to(None, self.internal_cursor.y)?;
        self.internal_cursor.reset_wrapped_lines();
        self.terminal.clear(ClearType::FromCursorDown)?;
        let up = self.history.up();
        self.buffer = up.clone();

        let overflow = self.screen_height_overflow_by_str(&up);

        if overflow != 0 {
            self.terminal.scroll_up(overflow as i16)?;
            self.internal_cursor.y -= overflow;
            self.internal_cursor.total_wrapped_lines += overflow;
            self.cursor.move_up(overflow as u16);
        }

        self.write(&up)?;

        Ok(())
    }

    pub fn handle_down(&mut self) -> std::io::Result<()> {
        self.internal_cursor.x = 4;
        self.move_cursor_to(None, self.internal_cursor.y)?;
        self.internal_cursor.reset_wrapped_lines();
        self.terminal.clear(ClearType::FromCursorDown)?;
        let down = self.history.down();
        self.buffer = down.clone();

        let overflow = self.screen_height_overflow_by_str(&down);

        if overflow != 0 {
            self.terminal.scroll_up(overflow as i16)?;
            self.internal_cursor.y -= overflow;
            self.internal_cursor.total_wrapped_lines += overflow;
            self.cursor.move_up(overflow as u16);
        }

        self.write(&down)?;

        Ok(())
    }

    pub fn handle_left(&mut self) -> std::io::Result<()> {
        // clear suggestion
        self.clear_suggestion()?;

        if self.internal_cursor.get_corrected_x() > 0 {
            self.cursor.move_left(1);
            self.move_internal_cursor_left()?;
        }
        Ok(())
    }

    pub fn handle_right(&mut self) -> std::io::Result<()> {
        if !self.at_line_end() {
            self.cursor.move_right(1);
            self.move_internal_cursor_right()?;
        } else {
            self.use_suggestion()?;
        }
        Ok(())
    }

    pub fn handle_backspace(&mut self) -> std::io::Result<()> {
        if self.internal_cursor.get_corrected_x() > 0 {
            self.cursor.move_left(1);
            self.move_internal_cursor_left()?;
            if !self.buffer.is_empty() {
                StringTools::remove_at_char_idx(
                    &mut self.buffer,
                    self.internal_cursor.get_corrected_x(),
                );
            }
            self.write_insert(None)?;
        }
        Ok(())
    }

    pub fn handle_ctrl_c(&mut self) -> std::io::Result<()> {
        // reset wrapped lines counter
        self.internal_cursor.reset_wrapped_lines();

        if self.buffer.is_empty() {
            self.exit()?;
        } else {
            // clear suggestion and invalidate racer cache
            self.clear_suggestion()?;

            self.buffer.clear();
            self.write_newline()?;
            self.write_in()?;
        }

        Ok(())
    }

    pub fn handle_ctrl_d(&mut self) -> std::io::Result<()> {
        if self.buffer.is_empty() {
            self.exit()?;
        }

        Ok(())
    }

    fn exit(&mut self) -> std::io::Result<()> {
        self.terminal.clear(ClearType::All)?;
        self.terminal.exit();

        Ok(())
    }

    pub fn handle_ctrl_z(&mut self) -> std::io::Result<()> {
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

    pub fn handle_ctrl_l(&mut self) -> std::io::Result<()> {
        self.clear()?;
        self.write_in()?;
        Ok(())
    }

    pub fn go_to_start(&mut self) -> std::io::Result<()> {
        self.clear_suggestion()?;
        self.internal_cursor.x = 4;
        self.move_cursor_to(4, self.internal_cursor.y)?;
        self.internal_cursor.current_wrapped_lines = 0;
        Ok(())
    }

    pub fn go_to_end(&mut self) -> std::io::Result<()> {
        // Already at the end of the line
        if self.at_line_end() {
            self.use_suggestion()?;
        } else {
            self.internal_cursor.x =
                StringTools::chars_count(&self.buffer) + self.internal_cursor.x_offset;
            self.internal_cursor.current_wrapped_lines = self.internal_cursor.total_wrapped_lines;
            self.move_cursor_to(
                { self.internal_cursor.x % self.size.0 },
                self.internal_cursor.y + self.internal_cursor.total_wrapped_lines,
            )?;
        }

        Ok(())
    }

    pub fn _handle_ctrl_left(&mut self) -> Option<()> {
        // clear suggestion
        let _ = self.clear_suggestion();

        if self.internal_cursor.get_corrected_x() < 1 {
            return Some(());
        }

        let buffer = self.buffer.chars().collect::<Vec<char>>();

        self.cursor.move_left(1);
        let _ = self.move_internal_cursor_left();
        if let Some(current_char) =
            buffer.get(self.internal_cursor.get_corrected_x().checked_sub(1)?)
        {
            match *current_char {
                ' ' => {
                    while buffer[self.internal_cursor.get_corrected_x()] == ' ' {
                        self.cursor.move_left(1);
                        let _ = self.move_internal_cursor_left();
                    }
                }
                c if c.is_alphanumeric() => {
                    while buffer[self.internal_cursor.get_corrected_x().checked_sub(1)?]
                        .is_alphanumeric()
                    {
                        self.cursor.move_left(1);
                        let _ = self.move_internal_cursor_left();
                    }
                }

                _ => {
                    while !buffer[self.internal_cursor.get_corrected_x().checked_sub(1)?]
                        .is_alphanumeric()
                        && buffer[self.internal_cursor.get_corrected_x().checked_sub(1)?] != ' '
                    {
                        self.cursor.move_left(1);
                        let _ = self.move_internal_cursor_left();
                    }
                }
            }
        }
        Some(())
    }

    pub fn _handle_ctrl_right(&mut self) {
        let buffer = self.buffer.chars().collect::<Vec<char>>();
        if !self.at_line_end() {
            self.cursor.move_right(1);
            let _ = self.move_internal_cursor_right();
        } else {
            let _ = self.use_suggestion();
        }
        if let Some(current_char) = buffer.get(self.internal_cursor.get_corrected_x()) {
            match *current_char {
                ' ' => {
                    while buffer.get(self.internal_cursor.get_corrected_x() + 1) == Some(&' ') {
                        self.cursor.move_right(1);
                        let _ = self.move_internal_cursor_right();
                    }
                    self.cursor.move_right(1);
                    let _ = self.move_internal_cursor_right();
                }
                c if c.is_alphanumeric() => {
                    while let Some(character) = buffer.get(self.internal_cursor.get_corrected_x()) {
                        if !character.is_alphanumeric() {
                            break;
                        }
                        self.cursor.move_right(1);
                        let _ = self.move_internal_cursor_right();
                    }
                }

                _ => {
                    while let Some(character) = buffer.get(self.internal_cursor.get_corrected_x()) {
                        if character.is_alphanumeric() || *character == ' ' {
                            break;
                        }
                        self.cursor.move_right(1);
                        let _ = self.move_internal_cursor_right();
                    }
                }
            }
        }
    }
}
