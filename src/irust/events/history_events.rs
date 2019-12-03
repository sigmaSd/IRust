use crate::irust::buffer::Buffer;
use crate::irust::cursor::INPUT_START_COL;
use crate::irust::irust_error::IRustError;
use crate::utils::StringTools;
use crossterm::{input::input, input::InputEvent, input::KeyEvent, style::Color};

impl super::IRust {
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
            self.buffer = Buffer::from_str(&history, self.cursor.bound.width - INPUT_START_COL);

            self.print_input()?;

            let last_input_pos = self.cursor.input_last_pos(&self.buffer);
            self.buffer.goto_end();
            self.cursor.goto(last_input_pos.0, last_input_pos.1);
        }
        Ok(())
    }

    pub fn handle_ctrl_r(&mut self) -> Result<usize, IRustError> {
        // make space for the search bar
        if self.cursor.is_at_last_terminal_row() {
            self.scroll_up(1);
        }
        self.cursor.goto_input_start_col();

        const SEARCH_TITLE: &str = "search history: ";
        const TITLE_WIDTH: usize = 16; // SEARCH_TITLE.chars().count()
        self.write_at_no_cursor(&SEARCH_TITLE, Color::Red, 0, self.cursor.bound.height - 1)?;

        let mut needle = String::new();

        let mut stdin = input().read_sync();
        let mut events_num = 0;
        loop {
            if let Some(key_event) = stdin.next() {
                events_num += 1;
                match key_event {
                    InputEvent::Keyboard(KeyEvent::Char(c)) => {
                        // max search len
                        if StringTools::chars_count(&needle) + TITLE_WIDTH
                            == self.cursor.bound.width - 1
                        {
                            continue;
                        }
                        needle.push(c);
                        // search history
                        if let Some(hit) = self.history.find(&needle) {
                            self.buffer =
                                Buffer::from_str(&hit, self.cursor.bound.width - INPUT_START_COL);
                        } else {
                            self.buffer = Buffer::new(self.cursor.bound.width - INPUT_START_COL);
                        }
                        self.print_input()?;
                        self.clear_last_line()?;
                        self.write_at_no_cursor(
                            &SEARCH_TITLE,
                            Color::Red,
                            0,
                            self.cursor.bound.height - 1,
                        )?;
                        self.write_at_no_cursor(
                            &needle,
                            Color::White,
                            TITLE_WIDTH,
                            self.cursor.bound.height - 1,
                        )?;
                    }
                    InputEvent::Keyboard(KeyEvent::Backspace) => {
                        needle.pop();
                        // search history
                        if !needle.is_empty() {
                            if let Some(hit) = self.history.find(&needle) {
                                self.buffer = Buffer::from_str(
                                    &hit,
                                    self.cursor.bound.width - INPUT_START_COL,
                                );
                            } else {
                                self.buffer =
                                    Buffer::new(self.cursor.bound.width - INPUT_START_COL);
                            }
                            self.print_input()?;
                        } else {
                            self.buffer = Buffer::new(self.cursor.bound.width - INPUT_START_COL);
                            self.print_input()?;
                        }
                        self.clear_last_line()?;
                        self.write_at_no_cursor(
                            &SEARCH_TITLE,
                            Color::Red,
                            0,
                            self.cursor.bound.height - 1,
                        )?;
                        self.write_at_no_cursor(
                            &needle,
                            Color::White,
                            TITLE_WIDTH,
                            self.cursor.bound.height - 1,
                        )?;
                    }
                    InputEvent::Keyboard(KeyEvent::Ctrl('c')) => {
                        self.buffer.clear();
                        self.print_input()?;
                        needle.clear();
                        self.clear_last_line()?;
                        self.write_at_no_cursor(
                            &SEARCH_TITLE,
                            Color::Red,
                            0,
                            self.cursor.bound.height - 1,
                        )?;
                    }
                    InputEvent::Keyboard(KeyEvent::Enter) | InputEvent::Keyboard(KeyEvent::Esc) => {
                        break
                    }
                    InputEvent::Keyboard(KeyEvent::Ctrl('d')) => {
                        if needle.is_empty() {
                            break;
                        }
                    }
                    _ => (),
                }
            }
        }
        self.clear_last_line()?;
        self.print_input()?;
        Ok(events_num)
    }
}
