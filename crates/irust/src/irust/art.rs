use crate::irust::{IRust, Result};
use crossterm::style::Color;
use std::io::Read;

impl IRust {
    pub fn wait_add(&mut self, mut add_cmd: std::process::Child, msg: &str) -> Result<()> {
        self.printer.cursor.save_position();
        self.printer.cursor.hide();
        self.printer.writer.raw.set_fg(Color::Cyan)?;

        match self.wait_add_inner(&mut add_cmd, msg) {
            Ok(()) => {
                self.clean_art()?;

                if let Some(stderr) = add_cmd.stderr.as_mut() {
                    let mut error = String::new();
                    stderr.read_to_string(&mut error)?;
                    if !error.is_empty() {
                        return Err(error.into());
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

    fn wait_add_inner(&mut self, add_cmd: &mut std::process::Child, msg: &str) -> Result<()> {
        self.printer.write_at(
            &format!(" {}ing dep [\\]", msg),
            0,
            self.printer.cursor.current_pos().1,
        )?;
        loop {
            match add_cmd.try_wait() {
                Ok(None) => {
                    self.printer.write_at(
                        "\\",
                        msg.len() + 10,
                        self.printer.cursor.current_pos().1,
                    )?;
                    self.printer.write_at(
                        "|",
                        msg.len() + 10,
                        self.printer.cursor.current_pos().1,
                    )?;
                    self.printer.write_at(
                        "/",
                        msg.len() + 10,
                        self.printer.cursor.current_pos().1,
                    )?;
                    self.printer.write_at(
                        "-",
                        msg.len() + 10,
                        self.printer.cursor.current_pos().1,
                    )?;
                    self.printer.write_at(
                        "\\",
                        msg.len() + 10,
                        self.printer.cursor.current_pos().1,
                    )?;
                    self.printer.write_at(
                        "|",
                        msg.len() + 10,
                        self.printer.cursor.current_pos().1,
                    )?;
                    self.printer.write_at(
                        "/",
                        msg.len() + 10,
                        self.printer.cursor.current_pos().1,
                    )?;
                    self.printer.write_at(
                        "-",
                        msg.len() + 10,
                        self.printer.cursor.current_pos().1,
                    )?;
                }
                Err(e) => {
                    return Err(e.into());
                }
                Ok(Some(_)) => return Ok(()),
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    fn clean_art(&mut self) -> Result<()> {
        self.printer.cursor.restore_position();
        self.printer.write_newline(&self.buffer);
        self.printer.cursor.show();
        self.printer.writer.raw.reset_color()?;
        Ok(())
    }

    pub fn welcome(&mut self) -> Result<()> {
        let default_msg = "Welcome to IRust".to_string();
        let msg = (|| {
            if let Some(msg) = self.trigger_set_msg_hook() {
                return self.fit_msg(&msg);
            }

            if !self.options.welcome_msg.is_empty() {
                self.fit_msg(&self.options.welcome_msg.clone())
            } else {
                self.fit_msg(&default_msg)
            }
        })();

        self.printer.writer.raw.set_fg(self.options.welcome_color)?;
        self.printer.writer.raw.write(&msg)?;
        self.printer.writer.raw.reset_color()?;

        self.printer.write_newline(&self.buffer);
        self.printer.write_newline(&self.buffer);

        Ok(())
    }

    pub fn warn_highlight_engine(&mut self) -> Result<()> {
        #[cfg(feature = "rustc_lexer")]
        #[cfg(not(feature = "change_highlight"))]
        let supported = ["default", "rustc_lexer"];
        #[cfg(feature = "syntect")]
        #[cfg(not(feature = "change_highlight"))]
        let supported = ["default", "syntect"];
        #[cfg(feature = "change_highlight")]
        let supported = ["default", "syntect", "rustc_lexer"];

        if !supported.contains(&self.options.highlight_engine.as_str()) {
            let msg = format!(
                "highlight engine {} is not supported, use {} instead",
                &self.options.highlight_engine, supported[0]
            );
            self.printer.writer.raw.set_fg(self.options.err_color)?;
            self.printer.writer.raw.write(&msg)?;
            self.printer.writer.raw.reset_color()?;

            self.printer.write_newline(&self.buffer);
            self.printer.write_newline(&self.buffer);
        }

        Ok(())
    }

    pub fn ferris(&mut self) -> String {
        r#"
     _~^~^~_
 \) /  o o  \ (/
   '_   ¬   _'
   / '-----' \
                     "#
        .lines()
        .skip(1)
        .map(|l| l.to_string() + "\n")
        .collect()
    }

    fn fit_msg(&mut self, msg: &str) -> String {
        let slash_num = self.printer.cursor.width() - msg.len();
        let slash = "-".repeat(slash_num / 2);

        format!("{0}{1}{0}", slash, msg)
    }
}
