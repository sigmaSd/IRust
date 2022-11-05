use std::{cell::RefCell, rc::Rc};

use crossterm::{cursor::*, queue};

use crate::Result;

#[derive(Debug, Clone)]
pub struct Raw<W: std::io::Write> {
    pub raw: Rc<RefCell<W>>,
}
impl<W: std::io::Write> std::io::Write for Raw<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.raw.borrow_mut().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.raw.borrow_mut().flush()
    }
}

impl<W: std::io::Write> Raw<W> {
    pub fn restore_position(&mut self) -> Result<()> {
        queue!(self, RestorePosition)?;
        Ok(())
    }

    pub fn save_position(&mut self) -> Result<()> {
        queue!(self, SavePosition)?;
        Ok(())
    }

    pub fn move_down(&mut self, n: u16) -> Result<()> {
        queue!(self, MoveDown(n))?;
        Ok(())
    }

    pub fn move_up(&mut self, n: u16) -> Result<()> {
        queue!(self, MoveUp(n))?;
        Ok(())
    }

    pub fn show(&mut self) -> Result<()> {
        queue!(self, Show)?;
        Ok(())
    }

    pub fn hide(&mut self) -> Result<()> {
        queue!(self, Hide)?;
        Ok(())
    }

    pub fn goto(&mut self, x: u16, y: u16) -> Result<()> {
        queue!(self, MoveTo(x, y))?;
        Ok(())
    }

    pub fn size(&self) -> Result<(usize, usize)> {
        Ok(crossterm::terminal::size().map(|(w, h)| (w as usize, h as usize))?)
    }

    pub fn get_current_pos(&mut self) -> Result<(usize, usize)> {
        // position only uses stdout()
        Ok(crossterm::cursor::position().map(|(w, h)| (w as usize, h as usize))?)
    }
}
