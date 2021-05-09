use crossterm::{
    //    cursor::{CursorShape, SetCursorShape},
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
    style::Color,
    terminal::ClearType,
};
use irust_api::Command;
use printer::printer::{PrintQueue, PrinterItem};

use crate::irust::{racer::Cycle, Result};
use crate::irust::{racer::Racer, IRust};
use crate::{irust::Buffer, utils::StringTools};

#[derive(Default)]
pub struct Engine {}

impl IRust {
    pub fn execute(&mut self, command: Command) -> Result<()> {
        match command {
            Command::AcceptSuggestion => {
                if let Some(suggestion) = self
                    .racer
                    .as_mut()
                    .map(|r| r.active_suggestion.take())
                    .flatten()
                {
                    for c in suggestion.chars() {
                        self.execute(Command::HandleCharacter(c))?;
                    }
                }
                Ok(())
            }
            Command::Continue => Ok(()),
            //NOTE: Maybe this command should be taken out of the engine, if the script gets access to self.buffer.current_char()
            Command::DeleteNextWord => {
                let current_char = self.buffer.current_char();
                if current_char.is_none() {
                    return Ok(());
                }
                let current_char = current_char.unwrap();

                let delete_predicate_function: &dyn Fn(&char) -> bool =
                    if current_char.is_whitespace() {
                        &|c| c.is_whitespace()
                    } else {
                        &|c| !c.is_whitespace()
                    };

                // safe unwrap because the first char is checked and the next ones will be caught inside the loop
                while delete_predicate_function(self.buffer.current_char().unwrap()) {
                    self.execute(Command::HandleDelete)?;
                    if self.buffer.current_char().is_none() {
                        break;
                    }
                }
                Ok(())
            }
            Command::Multiple(commands) => {
                for command in commands {
                    self.execute(command)?;
                }
                Ok(())
            }
            Command::HandleCharacter(c) => {
                self.buffer.insert(c);
                self.print_input()?;
                self.printer.cursor.move_right_unbounded();
                self.history.unlock();
                // Ignore RacerDisabled error
                let _ = self.racer.as_mut().map(Racer::unlock_racer_update);

                Ok(())
            }
            Command::HandleEnter(force_eval) => {
                self.history.unlock();

                let buffer = self.buffer.to_string();

                if !force_eval && !input_is_cmd_or_shell(&buffer) && incomplete_input(&buffer) {
                    self.execute(Command::HandleAltEnter)?;
                    return Ok(());
                }

                self.printer.cursor.hide();

                // create a new line
                self.printer.write_newline(&self.buffer);

                // add commands to history
                if self.should_push_to_history(&buffer) {
                    self.history.push(buffer);
                }

                // parse and handle errors
                let output = match self.parse() {
                    Ok(out) => out,
                    Err(e) => {
                        let mut printer = PrintQueue::default();
                        printer.push(PrinterItem::String(e.to_string(), self.options.err_color));
                        printer.add_new_line(1);
                        printer
                    }
                };

                // ensure buffer is cleaned
                self.buffer.clear();

                // print output
                if !output.is_empty() {
                    // clear racer suggestions is present
                    self.printer.writer.raw.clear(ClearType::FromCursorDown)?;
                    self.printer.print_output(output)?;
                    self.global_variables.operation_number += 1;
                    self.update_input_prompt();
                }

                // print a new input prompt
                self.printer.print_prompt_if_set()?;

                self.printer.cursor.show();
                Ok(())
            }
            Command::HandleAltEnter => {
                self.execute(Command::RemoveRacerSugesstion)?;
                self.buffer.insert('\n');
                self.print_input()?;
                self.printer.cursor.move_right();
                Ok(())
            }
            Command::HandleTab => {
                if self.buffer.is_at_string_line_start() {
                    const TAB: &str = "   \t";

                    self.buffer.insert_str(TAB);
                    self.print_input()?;
                    for _ in 0..4 {
                        self.printer.cursor.move_right_unbounded();
                    }
                    return Ok(());
                }

                if let Some(racer) = self.racer.as_mut() {
                    racer.update_suggestions(&self.buffer, &mut self.repl)?;
                    racer.lock_racer_update()?;
                    racer.cycle_suggestions(
                        &mut self.printer,
                        &self.buffer,
                        &self.theme,
                        Cycle::Down,
                        &self.options,
                    )?;
                }
                Ok(())
            }
            Command::HandleBackTab => {
                if let Some(racer) = self.racer.as_mut() {
                    racer.update_suggestions(&self.buffer, &mut self.repl)?;
                    racer.lock_racer_update()?;
                    racer.cycle_suggestions(
                        &mut self.printer,
                        &self.buffer,
                        &self.theme,
                        Cycle::Up,
                        &self.options,
                    )?;
                }
                Ok(())
            }
            Command::HandleUp => {
                if self.printer.cursor.is_at_first_input_line() {
                    let buffer = self.buffer.take();
                    self.handle_history(Dir::Up, buffer)?;
                    self.history.lock();
                } else {
                    self.execute(Command::RemoveRacerSugesstion)?;
                    self.print_input()?;
                    self.printer.cursor.move_up_bounded(1);
                    // set buffer cursor
                    let buffer_pos = self.printer.cursor.cursor_pos_to_buffer_pos();
                    self.buffer.set_buffer_pos(buffer_pos);
                }
                Ok(())
            }
            Command::HandleDown => {
                if self.buffer.is_empty() {
                    return Ok(());
                }
                if self.printer.cursor.is_at_last_input_line(&self.buffer) {
                    let buffer = self.buffer.take();
                    self.handle_history(Dir::Down, buffer)?;
                    self.history.lock();
                } else {
                    self.execute(Command::RemoveRacerSugesstion)?;
                    self.print_input()?;
                    self.printer.cursor.move_down_bounded(1, &self.buffer);
                    // set buffer cursor
                    let buffer_pos = self.printer.cursor.cursor_pos_to_buffer_pos();
                    self.buffer.set_buffer_pos(buffer_pos);
                }
                Ok(())
            }
            Command::HandleRight => {
                if let Some(suggestion) = self
                    .racer
                    .as_mut()
                    .map(|r| r.active_suggestion.take())
                    .flatten()
                {
                    for c in suggestion.chars() {
                        self.execute(Command::HandleCharacter(c))?;
                    }
                } else if !self.buffer.is_at_end() {
                    self.printer.cursor.move_right();
                    self.buffer.move_forward();
                }
                Ok(())
            }
            Command::HandleLeft => {
                self.execute(Command::RemoveRacerSugesstion)?;
                self.print_input()?;

                if !self.buffer.is_at_start() && !self.buffer.is_empty() {
                    self.printer.cursor.move_left();
                    self.buffer.move_backward();
                }
                Ok(())
            }
            Command::HandleBackSpace => {
                if !self.buffer.is_at_start() {
                    self.buffer.move_backward();
                    self.printer.cursor.move_left();
                    self.buffer.remove_current_char();
                    self.print_input()?;
                    self.history.unlock();
                    // Ignore RacerDisabled error
                    let _ = self.racer.as_mut().map(Racer::unlock_racer_update);
                }
                Ok(())
            }
            Command::HandleDelete => {
                if !self.buffer.is_empty() {
                    self.buffer.remove_current_char();
                    self.print_input()?;
                    self.history.unlock();
                    // Ignore RacerDisabled error
                    let _ = self.racer.as_mut().map(Racer::unlock_racer_update);
                }
                Ok(())
            }
            Command::HandleCtrlC => {
                self.buffer.clear();
                self.history.unlock();
                let _ = self.racer.as_mut().map(Racer::unlock_racer_update);
                self.printer.cursor.goto_start();
                self.printer.print_prompt_if_set()?;
                self.printer.writer.raw.clear(ClearType::FromCursorDown)?;
                self.print_input()?;
                Ok(())
            }
            Command::HandleCtrlD => {
                if !self.buffer.is_empty() {
                    return Ok(());
                }

                macro_rules! set_exit_flag_and_return {
                    () => {{
                        self.exit_flag = true;
                        break Ok(());
                    }};
                }

                self.printer.write_newline(&self.buffer);
                self.printer.write(
                    "Do you really want to exit ([y]/n)? ",
                    crossterm::style::Color::Grey,
                )?;

                loop {
                    std::io::Write::flush(&mut self.printer.writer.raw)?;

                    if let Ok(key_event) = crossterm::event::read() {
                        match key_event {
                            Event::Key(KeyEvent {
                                code: KeyCode::Char(c),
                                modifiers: KeyModifiers::NONE,
                            }) => match &c {
                                'y' | 'Y' => {
                                    set_exit_flag_and_return!()
                                }
                                _ => {
                                    self.printer.write_newline(&self.buffer);
                                    self.printer.write_newline(&self.buffer);
                                    self.printer.print_prompt_if_set()?;
                                    break Ok(());
                                }
                            },
                            Event::Key(KeyEvent {
                                code: KeyCode::Char('d'),
                                modifiers: KeyModifiers::CONTROL,
                            })
                            | Event::Key(KeyEvent {
                                code: KeyCode::Enter,
                                ..
                            }) => {
                                set_exit_flag_and_return!()
                            }
                            _ => continue,
                        }
                    }
                }
            }
            Command::HandleCtrlE => self.execute(Command::HandleEnter(true)),
            Command::HandleCtrlL => {
                self.buffer.clear();
                self.buffer.goto_start();
                self.printer.clear()?;
                self.printer.print_prompt_if_set()?;
                self.print_input()?;
                Ok(())
            }
            Command::HandleCtrlR => {
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
                            self.buffer = hit.into();
                            found_needle = true;
                        } else {
                            self.buffer = Buffer::new();
                        }
                        self.print_input()?;
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

                    if let Ok(key_event) = crossterm::event::read() {
                        match key_event {
                            Event::Key(KeyEvent {
                                code: KeyCode::Char(c),
                                modifiers: KeyModifiers::NONE,
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
                                modifiers: KeyModifiers::CONTROL,
                            }) => {
                                // forward search
                                index = index.saturating_sub(1);
                                let _ = find_and_print!();
                            }
                            Event::Key(KeyEvent {
                                code: KeyCode::Char('r'),
                                modifiers: KeyModifiers::CONTROL,
                            }) => {
                                // backward search
                                index += 1;
                                let found_needle = find_and_print!();
                                if !found_needle {
                                    index -= 1;
                                    let _ = find_and_print!();
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
                                modifiers: KeyModifiers::CONTROL,
                            }) => {
                                self.buffer.clear();
                                self.print_input()?;
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
                                modifiers: KeyModifiers::CONTROL,
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
                self.execute(Command::RemoveRacerSugesstion)?;
                self.print_input()?;
                let buffer_pos = self.printer.cursor.cursor_pos_to_buffer_pos();
                self.buffer.set_buffer_pos(buffer_pos);
                Ok(())
            }
            Command::HandleCtrlZ => {
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
                    self.execute(Command::HandleCtrlL)?;
                }
                Ok(())
            }
            Command::HandleCtrlRight => {
                self.execute(Command::HandleRight)?;

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
            Command::HandleCtrlLeft => {
                self.execute(Command::HandleLeft)?;

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
                Ok(())
            }
            Command::HandleHome => {
                while !self.printer.cursor.is_at_line_start() {
                    self.execute(Command::HandleLeft)?;
                }
                Ok(())
            }
            Command::HandleEnd => {
                while !self.buffer.is_empty() && !self.printer.cursor.is_at_line_end() {
                    self.buffer.move_forward();
                    self.printer.cursor.move_right();
                }
                // check for racer suggestion at the end
                self.execute(Command::AcceptSuggestion)?;
                Ok(())
            }
            Command::RemoveRacerSugesstion => {
                // remove any active suggestion
                let _ = self.racer.as_mut().map(|r| r.active_suggestion.take());
                Ok(())
            }
            Command::Exit => {
                self.history.save()?;
                self.options.save()?;
                self.theme.save()?;
                self.printer.write_newline(&self.buffer);
                self.printer.cursor.show();
                Ok(())
            }
            Command::SetThinCursor => Ok(()), /*Ok(crossterm::queue!(
            std::io::stdout(),
            SetCursorShape(CursorShape::Line)
            )?),*/
            Command::SetWideCursor => Ok(()), /*Ok(crossterm::queue!(
                                                  std::io::stdout(),
                                                  SetCursorShape(CursorShape::Block)
                                              )?),*/
        }
    }

    //  history helper
    fn handle_history(&mut self, direction: Dir, buffer: Vec<char>) -> Result<()> {
        let history = match direction {
            Dir::Up => self.history.up(&buffer),
            Dir::Down => self.history.down(&buffer),
        };

        if let Some(history) = history {
            self.buffer = history.into();
        } else {
            self.buffer.buffer = buffer;
        }
        self.print_input()?;

        let last_input_pos = self.printer.cursor.input_last_pos(&self.buffer);
        self.buffer.goto_end();
        self.printer.cursor.goto(last_input_pos.0, last_input_pos.1);
        Ok(())
    }
}

// History Direction
enum Dir {
    Up,
    Down,
}

// helper functions

fn incomplete_input(buffer: &str) -> bool {
    StringTools::unmatched_brackets(&buffer)
        || buffer
            .trim_end()
            .ends_with(|c| c == ':' || c == '.' || c == '=')
}

fn input_is_cmd_or_shell(buffer: &str) -> bool {
    buffer.starts_with(':') || buffer.starts_with("::")
}
