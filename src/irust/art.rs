use crate::irust::{IRust, IRustError};
use crossterm::{ClearType, Color};
use std::io::Read;

impl IRust {
    pub fn wait_add(
        &mut self,
        mut add_cmd: std::process::Child,
        msg: &str,
    ) -> Result<(), IRustError> {
        self.cursor.hide();
        self.raw_terminal.set_fg(Color::Cyan)?;

        match self.wait_add_inner(&mut add_cmd, msg) {
            Ok(()) => {
                self.clean_art()?;

                if let Some(stderr) = add_cmd.stderr.as_mut() {
                    let mut error = String::new();
                    stderr.read_to_string(&mut error)?;
                    if !error.is_empty() {
                        return Err(IRustError::Custom(error));
                    }
                }
                Ok(())
            }
            Err(e) => {
                self.clean_art()?;
                Err(e)
            }
        }
    }

    fn wait_add_inner(
        &mut self,
        add_cmd: &mut std::process::Child,
        msg: &str,
    ) -> Result<(), IRustError> {
        self.write_at(
            &format!(" {}ing dep [\\]", msg),
            0,
            self.cursor.pos.current_pos.1,
        )?;
        loop {
            match add_cmd.try_wait() {
                Ok(None) => {
                    self.write_at("\\", msg.len() + 10, self.cursor.pos.current_pos.1)?;
                    self.write_at("|", msg.len() + 10, self.cursor.pos.current_pos.1)?;
                    self.write_at("/", msg.len() + 10, self.cursor.pos.current_pos.1)?;
                    self.write_at("-", msg.len() + 10, self.cursor.pos.current_pos.1)?;
                    self.write_at("\\", msg.len() + 10, self.cursor.pos.current_pos.1)?;
                    self.write_at("|", msg.len() + 10, self.cursor.pos.current_pos.1)?;
                    self.write_at("/", msg.len() + 10, self.cursor.pos.current_pos.1)?;
                    self.write_at("-", msg.len() + 10, self.cursor.pos.current_pos.1)?;
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
                Ok(Some(_)) => return Ok(()),
            }
        }
    }

    fn clean_art(&mut self) -> Result<(), IRustError> {
        self.cursor.restore_position()?;
        self.write_newline()?;
        self.cursor.show();
        self.raw_terminal.reset_color()?;
        Ok(())
    }

    pub fn welcome(&mut self) -> Result<(), IRustError> {
        self.raw_terminal.clear(ClearType::All)?;

        let default_msg = "Welcome to IRust".to_string();
        let msg = if !self.options.welcome_msg.is_empty() {
            self.fit_msg(&self.options.welcome_msg.clone())
        } else {
            self.fit_msg(&default_msg)
        };

        self.raw_terminal.set_fg(self.options.welcome_color)?;
        self.raw_terminal.write(&msg)?;
        self.raw_terminal.reset_color()?;

        self.write_newline()?;
        self.write_newline()?;

        Ok(())
    }

    pub fn ferris(&mut self) -> String {
        let ferris = r#"
     _~^~^~_
 \) /  o o  \ (/
   '_   ¬   _'
   / '-----' \
                     "#
        .lines()
        .skip(1)
        .map(|l| l.to_string() + "\n")
        .collect();

        ferris
    }

    fn fit_msg(&mut self, msg: &str) -> String {
        let slash_num = self.cursor.bound.width - msg.len();
        let slash = std::iter::repeat('-')
            .take(slash_num / 2)
            .collect::<String>();

        format!("{0}{1}{0}", slash, msg)
    }
}
