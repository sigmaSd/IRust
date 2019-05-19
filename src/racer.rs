use std::env::temp_dir;
use std::io::{self, Read, Write};
use std::process::{Child, Command, Stdio};
use crate::irust::IRust;

#[derive(Debug)]
pub struct Racer {
    process: Child,
    main_file: String,
    pub cursor: (usize, usize),
    suggestions: Vec<String>,
    suggestion_idx: usize,
    pub needs_update: bool,
}

impl Racer {
    pub fn start() -> io::Result<Racer> {
        let process = Command::new("racer")
            .arg("daemon")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let main_file = temp_dir()
            .join("irust/src/main.rs")
            .to_str()
            .unwrap()
            .to_owned();
        let cursor = (2, 5);

        Ok(Racer {
            process,
            main_file,
            cursor,
            suggestions: vec![],
            suggestion_idx: 0,
            needs_update: true,
        })
    }

    pub fn complete(&mut self) -> io::Result<()> {
        let stdin = self.process.stdin.as_mut().unwrap();
        let stdout = self.process.stdout.as_mut().unwrap();

        writeln!(
            stdin,
            "complete {} {} {}",
            self.cursor.0, self.cursor.1, self.main_file
        )?;

        // read till END
        let mut raw_output = [0; 100_000];
        'outer: loop {
            let _ = stdout.read(&mut raw_output)?;
            let mut count = 0;
            for e in raw_output.iter().rev() {
                if *e == b"D"[0] {
                    count += 1;
                    continue;
                }
                if count == 1 && *e == b"N"[0] {
                    count += 1;
                    continue;
                }
                if count == 2 && *e == b"E"[0] {
                    break 'outer;
                }
                count = 0;
            }
        }

        let mut raw_output = String::from_utf8(raw_output.to_vec()).unwrap();
        let mut completions = vec![];

        while let Some(match_idx) = raw_output.find("H ") {
            // if MATCH exists than , exists we can unwrap safly
            let comman_idx = raw_output[match_idx..].find(',').unwrap() + match_idx;
            completions.push(raw_output[match_idx + 2..comman_idx].to_owned());
            raw_output = raw_output[comman_idx..].to_string();
        }
        self.suggestions = completions;
        self.needs_update = false;

        Ok(())
    }

    pub fn next_suggestion(&mut self) -> Option<&String> {
        if self.suggestion_idx >= self.suggestions.len() {
            self.suggestion_idx = 0
        }

        if self.suggestions.is_empty() {
            return None;
        }

        let suggestion = &self.suggestions[self.suggestion_idx];

        self.suggestion_idx += 1;

        Some(suggestion)
    }

    pub fn current_suggestion(&self) -> Option<&String> {
        if self.suggestion_idx > 1 {
            self.suggestions.get(self.suggestion_idx - 1)
        } else {
            self.suggestions.get(0)
        }
    }
}

impl IRust {
    pub fn show_suggestions(&mut self) -> std::io::Result<()> {
        // return if we're not at the end of the line
        if self.buffer.len() != self.internal_cursor.x {
            return Ok(());
        }

        let mut tmp_repl = self.repl.clone();
        let y_pos = tmp_repl.body.len();
        tmp_repl.insert(self.buffer.clone());
        tmp_repl.write()?;

        if self.racer.needs_update {
            self.racer.cursor.0 = y_pos;
            self.racer.cursor.1 = self.buffer.len() + 1;
            self.racer.complete()?;
        }

        if let Some(suggestion) = self.racer.next_suggestion() {
            self.color.set_fg(self.options.racer_color)?;
            self.cursor.save_position()?;
            self.terminal.clear(ClearType::UntilNewLine)?;

            let mut suggestion = suggestion.to_string();
            StringTools::strings_unique(&self.buffer, &mut suggestion);

            self.terminal.write(suggestion)?;
            self.cursor.reset_position()?;
            self.color.reset()?;
        }

        Ok(())
    }
}
