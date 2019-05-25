use crate::irust::printer::{Printer, PrinterItem, PrinterItemType};
use crate::irust::IRust;
use crate::utils::StringTools;
use crossterm::ClearType;

impl IRust {
    pub fn handle_character(&mut self, c: char) -> std::io::Result<()> {
        StringTools::insert_at_char_idx(&mut self.buffer, self.internal_cursor.get_x(), c);
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
        self.write_in()?;

        Ok(())
    }

    pub fn handle_tab(&mut self) -> std::io::Result<()> {
        self.write_next_suggestion()?;
        Ok(())
    }

    pub fn handle_up(&mut self) -> std::io::Result<()> {
        self.internal_cursor.x = 0;
        self.move_cursor_to(4, None)?;
        self.terminal.clear(ClearType::UntilNewLine)?;
        let up = self.history.up();
        self.buffer = up.clone();
        self.write(&up)?;
        Ok(())
    }

    pub fn handle_down(&mut self) -> std::io::Result<()> {
        self.internal_cursor.x = 0;
        self.move_cursor_to(4, None)?;
        self.terminal.clear(ClearType::UntilNewLine)?;
        let down = self.history.down();
        self.buffer = down.clone();
        self.write(&down)?;
        Ok(())
    }

    pub fn handle_left(&mut self) -> std::io::Result<()> {
        // clear suggestion
        self.clear_suggestion()?;

        if self.internal_cursor.get_x() > 0 {
            self.cursor.move_left(1);
            self.move_internal_cursor_left()?;
        }
        Ok(())
    }

    pub fn handle_right(&mut self) -> std::io::Result<()> {
        if self.buffer.len() != self.internal_cursor.get_x() {
            self.cursor.move_right(1);
            self.move_internal_cursor_right()?;
        } else {
            self.use_suggestion()?;
        }
        Ok(())
    }

    pub fn handle_backspace(&mut self) -> std::io::Result<()> {
        if self.internal_cursor.get_x() > 0 {
            self.cursor.move_left(1);
            self.move_internal_cursor_left()?;
            if !self.buffer.is_empty() {
                StringTools::remove_at_char_idx(&mut self.buffer, self.internal_cursor.get_x());
            }
            self.write_insert(None)?;
        }
        Ok(())
    }

    pub fn handle_ctrl_c(&mut self) -> std::io::Result<()> {
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

    pub fn clear(&mut self) -> std::io::Result<()> {
        self.terminal.clear(ClearType::All)?;
        self.internal_cursor.reset();
        self.move_cursor_to(0, 1)?;
        self.buffer.clear();
        self.write_in()?;
        Ok(())
    }

    pub fn go_to_start(&mut self) -> std::io::Result<()> {
        self.clear_suggestion()?;
        self.internal_cursor.x = 4;
        self.move_cursor_to(4, self.internal_cursor.y)?;
        Ok(())
    }

    pub fn go_to_end(&mut self) -> std::io::Result<()> {
        let end_idx = StringTools::chars_count(&self.buffer);
        // Already at the end of the line
        if self.internal_cursor.get_x() == end_idx {
            self.use_suggestion()?;
        } else {
            self.internal_cursor.x = end_idx + 4;
            self.move_cursor_to(
                self.internal_cursor.x % self.size.0,
                self.internal_cursor.get_y(),
            )?;
        }

        Ok(())
    }

    pub fn handle_ctrl_left(&mut self) -> Option<()> {
        if self.internal_cursor.get_x() < 1 {
            return Some(());
        }

        let buffer = self.buffer.chars().collect::<Vec<char>>();

        self.cursor.move_left(1);
        let _ = self.move_internal_cursor_left();
        if let Some(current_char) = buffer.get(self.internal_cursor.get_x().checked_sub(1)?) {
            match *current_char {
                ' ' => {
                    while buffer[self.internal_cursor.get_x()] == ' ' {
                        self.cursor.move_left(1);
                        let _ = self.move_internal_cursor_left();
                    }
                }
                c if c.is_alphanumeric() => {
                    while buffer[self.internal_cursor.get_x().checked_sub(1)?].is_alphanumeric() {
                        self.cursor.move_left(1);
                        let _ = self.move_internal_cursor_left();
                    }
                }

                _ => {
                    while !buffer[self.internal_cursor.get_x().checked_sub(1)?].is_alphanumeric()
                        && buffer[self.internal_cursor.get_x().checked_sub(1)?] != ' '
                    {
                        self.cursor.move_left(1);
                        let _ = self.move_internal_cursor_left();
                    }
                }
            }
        }
        Some(())
    }

    pub fn handle_ctrl_right(&mut self) {
        let buffer = self.buffer.chars().collect::<Vec<char>>();
        if buffer.len() != self.internal_cursor.get_x() {
            self.cursor.move_right(1);
            let _ = self.move_internal_cursor_right();
        } else {
            let _ = self.use_suggestion();
        }
        if let Some(current_char) = buffer.get(self.internal_cursor.get_x()) {
            match *current_char {
                ' ' => {
                    while buffer.get(self.internal_cursor.get_x() + 1) == Some(&' ') {
                        self.cursor.move_right(1);
                        let _ = self.move_internal_cursor_right();
                    }
                    self.cursor.move_right(1);
                    let _ = self.move_internal_cursor_right();
                }
                c if c.is_alphanumeric() => {
                    while let Some(character) = buffer.get(self.internal_cursor.get_x()) {
                        if !character.is_alphanumeric() {
                            break;
                        }
                        self.cursor.move_right(1);
                        let _ = self.move_internal_cursor_right();
                    }
                }

                _ => {
                    while let Some(character) = buffer.get(self.internal_cursor.get_x()) {
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
