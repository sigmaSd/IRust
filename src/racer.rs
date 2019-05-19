use std::env::temp_dir;
use std::io::{self, Read, Write};
use std::process::{Child, Command, Stdio};

#[derive(Debug)]
pub struct Racer {
    process: Child,
    main_file: String,
    pub cursor: (usize, usize),
    suggestions: Vec<String>,
    suggestion_idx: usize,
}

impl Racer {
    pub fn start() -> io::Result<Racer> {
        let process = Command::new("racer")
            .arg("daemon")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
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
        })
    }

    pub fn complete(&mut self) -> io::Result<()> {
        // let stdin = self.process.stdin.as_mut().unwrap();
        // let stdout = self.process.stderr.as_mut().unwrap();
        // write!(stdin, "complete 2 2 /home/mrcool/Projects/lab/src/main.rs").unwrap();
        // let mut s = [0; 1];
        // stdout.read_exact(&mut s);
        // dbg!(&String::from_utf8(s.to_vec()));

        let raw_output = Command::new("racer")
            .args(&[
                "complete",
                &self.cursor.0.to_string(),
                &self.cursor.1.to_string(),
                &self.main_file,
            ])
            .output()?
            .stdout;

        let mut raw_output = String::from_utf8(raw_output).unwrap_or_else(|_| "".to_string());
        let mut completions = vec![];

        while let Some(match_idx) = raw_output.find("MATCH") {
            // if MATCH exists than , exists we can unwrap safly
            let comman_idx = raw_output[match_idx..].find(',').unwrap() + match_idx;
            completions.push(raw_output[match_idx + 6..comman_idx].to_owned());
            raw_output = raw_output[comman_idx..].to_string();
        }
        self.suggestions = completions;

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
}
