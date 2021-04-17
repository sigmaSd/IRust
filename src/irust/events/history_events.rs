use crate::irust::buffer::Buffer;
use crate::irust::{Result, CTRL_KEYMODIFIER, NO_MODIFIER};
use crate::utils::StringTools;
use crossterm::{event::*, style::Color};

enum Dir {
    Up,
    Down,
}

impl super::IRust {
    pub fn handle_up(&mut self) -> Result<()> {
        if self.printer.cursor.is_at_first_input_line() {
            let buffer = self.buffer.take();
            self.handle_history(Dir::Up, buffer)?;
            self.history.lock();
        } else {
            self.remove_racer_sugesstion_and_reprint()?;
            self.printer.cursor.move_up_bounded(1);
            // set buffer cursor
            let buffer_pos = self.printer.cursor.cursor_pos_to_buffer_pos();
            self.buffer.set_buffer_pos(buffer_pos);
        }
        Ok(())
    }

    pub fn handle_down(&mut self) -> Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        if self.printer.cursor.is_at_last_input_line(&self.buffer) {
            let buffer = self.buffer.take();
            self.handle_history(Dir::Down, buffer)?;
            self.history.lock();
        } else {
            self.remove_racer_sugesstion_and_reprint()?;
            self.printer.cursor.move_down_bounded(1, &self.buffer);
            // set buffer cursor
            let buffer_pos = self.printer.cursor.cursor_pos_to_buffer_pos();
            self.buffer.set_buffer_pos(buffer_pos);
        }
        Ok(())
    }

    fn handle_history(&mut self, direction: Dir, buffer: Vec<char>) -> Result<()> {
        let history = match direction {
            Dir::Up => self.history.up(&buffer),
            Dir::Down => self.history.down(&buffer),
        };

        if let Some(history) = history {
            self.buffer = Buffer::from_string(&history);
        } else {
            self.buffer.buffer = buffer;
        }
        self.printer.print_input(&self.buffer, &self.theme)?;

        let last_input_pos = self.printer.cursor.input_last_pos(&self.buffer);
        self.buffer.goto_end();
        self.printer.cursor.goto(last_input_pos.0, last_input_pos.1);
        Ok(())
    }

    pub fn handle_ctrl_r(&mut self) -> Result<()> {
        // make space for the search bar
        if self.printer.cursor.is_at_last_terminal_row() {
            self.printer.scroll_up(1);
        }
        self.printer.cursor.goto_input_start_col();

        const SEARCH_TITLE: &str = "search history: ";
        const TITLE_WIDTH: usize = 16; // SEARCH_TITLE.chars().count()
        self.printer.write_at_no_cursor(
            &SEARCH_TITLE,
            Color::Red,
            0,
            self.printer.cursor.height() - 1,
        )?;

        let mut needle = String::new();
        let mut index = 0;

        macro_rules! find_and_print {
            () => {{
                let mut found_needle = false;
                // search history
                if let Some(hit) = self.history.reverse_find_nth(&needle, index) {
                    self.buffer = Buffer::from_string(&hit);
                    found_needle = true;
                } else {
                    self.buffer = Buffer::new();
                }
                self.printer.print_input(&self.buffer, &self.theme)?;
                self.printer.clear_last_line()?;
                self.printer.write_at_no_cursor(
                    &SEARCH_TITLE,
                    Color::Red,
                    0,
                    self.printer.cursor.height() - 1,
                )?;
                self.printer.write_at_no_cursor(
                    &needle,
                    Color::White,
                    TITLE_WIDTH,
                    self.printer.cursor.height() - 1,
                )?;
                found_needle
            }};
        }

        use std::io::Write;
        loop {
            self.printer.writer.raw.flush()?;

            if let Ok(key_event) = read() {
                match key_event {
                    Event::Key(KeyEvent {
                        code: KeyCode::Char(c),
                        modifiers: NO_MODIFIER,
                    }) => {
                        // reset index
                        index = 0;
                        // max search len
                        if StringTools::chars_count(&needle) + TITLE_WIDTH
                            == self.printer.cursor.width() - 1
                        {
                            continue;
                        }
                        needle.push(c);
                        let _ = find_and_print!();
                    }
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('s'),
                        modifiers: CTRL_KEYMODIFIER,
                    }) => {
                        // forward search
                        index = index.saturating_sub(1);
                        let _ = find_and_print!();
                    }
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('r'),
                        modifiers: CTRL_KEYMODIFIER,
                    }) => {
                        // backward search
                        index += 1;
                        let found_needle = find_and_print!();
                        if !found_needle {
                            index -= 1;
                        }
                    }
                    Event::Key(KeyEvent {
                        code: KeyCode::Backspace,
                        ..
                    }) => {
                        // reset index
                        index = 0;
                        needle.pop();
                        let _ = find_and_print!();
                    }
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: CTRL_KEYMODIFIER,
                    }) => {
                        self.buffer.clear();
                        self.printer.print_input(&self.buffer, &self.theme)?;
                        needle.clear();
                        self.printer.clear_last_line()?;
                        self.printer.write_at_no_cursor(
                            &SEARCH_TITLE,
                            Color::Red,
                            0,
                            self.printer.cursor.height() - 1,
                        )?;
                    }
                    Event::Key(KeyEvent {
                        code: KeyCode::Enter,
                        ..
                    })
                    | Event::Key(KeyEvent {
                        code: KeyCode::Esc, ..
                    }) => break,
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('d'),
                        modifiers: CTRL_KEYMODIFIER,
                    }) => {
                        if needle.is_empty() {
                            break;
                        }
                    }
                    _ => (),
                }
            }
        }
        self.printer.clear_last_line()?;
        self.remove_racer_sugesstion_and_reprint()?;
        let buffer_pos = self.printer.cursor.cursor_pos_to_buffer_pos();
        self.buffer.set_buffer_pos(buffer_pos);
        Ok(())
    }
}
