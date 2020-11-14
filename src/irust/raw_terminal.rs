use super::IRustError;
use crossterm::{cursor::*, queue, style::*, terminal::*};
use std::fmt::Display;
use std::io::{stdout, Write};

pub struct RawTerminal {}

impl RawTerminal {
    pub fn new() -> Self {
        Self {}
    }

    pub fn scroll_up(&self, n: u16) -> Result<(), IRustError> {
        queue!(stdout(), ScrollUp(n))?;
        Ok(())
    }

    pub fn clear(&self, clear_type: ClearType) -> Result<(), IRustError> {
        queue!(stdout(), Clear(clear_type))?;
        Ok(())
    }

    pub fn _write<D: Display + Clone>(value: D) -> Result<(), IRustError> {
        queue!(stdout(), Print(value))?;
        Ok(())
    }

    pub fn write<D: Display + Clone>(&self, value: D) -> Result<(), IRustError> {
        Self::_write(value)
    }

    pub fn write_with_color<D: Display + Clone>(
        &self,
        value: D,
        color: Color,
    ) -> Result<(), IRustError> {
        self.set_fg(color)?;
        self.write(value)?;
        self.reset_color()?;
        Ok(())
    }

    pub fn size(&self) -> Result<(u16, u16), IRustError> {
        Ok(size()?)
    }

    pub fn reset_color(&self) -> Result<(), IRustError> {
        queue!(stdout(), ResetColor)?;
        Ok(())
    }

    pub fn set_fg(&self, color: Color) -> Result<(), IRustError> {
        queue!(stdout(), SetForegroundColor(color))?;
        Ok(())
    }

    pub fn set_bg(&self, color: Color) -> Result<(), IRustError> {
        queue!(stdout(), SetBackgroundColor(color))?;
        Ok(())
    }

    pub fn set_title(&self, title: &str) -> Result<(), IRustError> {
        queue!(stdout(), SetTitle(title))?;
        Ok(())
    }

    pub fn disable_raw_mode() -> Result<(), IRustError> {
        Ok(crossterm::terminal::disable_raw_mode()?)
    }

    pub fn enable_raw_mode() -> Result<(), IRustError> {
        Ok(crossterm::terminal::enable_raw_mode()?)
    }

    pub fn flush(&self) -> Result<(), IRustError> {
        Ok(stdout().flush()?)
    }
}

pub struct RawCursor {}

impl RawCursor {
    pub fn new() -> RawCursor {
        Self {}
    }
    pub fn restore_position(&self) -> Result<(), IRustError> {
        queue!(stdout(), RestorePosition)?;
        Ok(())
    }

    pub fn save_position(&self) -> Result<(), IRustError> {
        queue!(stdout(), SavePosition)?;
        Ok(())
    }

    pub fn move_down(&self, n: u16) -> Result<(), IRustError> {
        queue!(stdout(), MoveDown(n))?;
        Ok(())
    }

    pub fn move_up(&self, n: u16) -> Result<(), IRustError> {
        queue!(stdout(), MoveUp(n))?;
        Ok(())
    }

    pub fn show(&self) -> Result<(), IRustError> {
        queue!(stdout(), Show)?;
        Ok(())
    }

    pub fn hide(&self) -> Result<(), IRustError> {
        queue!(stdout(), Hide)?;
        Ok(())
    }

    pub fn goto(&self, x: u16, y: u16) -> Result<(), IRustError> {
        queue!(stdout(), MoveTo(x, y))?;
        Ok(())
    }

    pub fn get_current_pos() -> Result<(usize, usize), IRustError> {
        let pos = crossterm::cursor::position();
        let pos = pos.map(|(x, y)| (x as usize, y as usize));
        Ok(pos?)
    }
}
