use super::IRustError;
use std::fs;
use std::path;

/// Mark to keep backward-compatibility with the old way of saving history
const NEW_HISTORY_MARK: &str = "##NewHistoryMark##\n//\n";

#[derive(Default)]
pub struct History {
    history: Vec<String>,
    buffer_copy: String,
    cursor: usize,
    path: path::PathBuf,
}

impl History {
    pub fn new(path: path::PathBuf) -> Result<Self, IRustError> {
        let _ = fs::create_dir_all(&path);

        let path = path.join("history");
        if !path.exists() {
            let _ = fs::File::create(&path);
        }

        let history: String = fs::read_to_string(&path)?;

        let history: Vec<String> = if history.starts_with(NEW_HISTORY_MARK) {
            history
                .split("\n//\n")
                .skip(1)
                .map(ToOwned::to_owned)
                .collect()
        } else {
            history.lines().map(ToOwned::to_owned).collect()
        };

        let cursor = history.len();
        let buffer_copy = String::new();

        Ok(Self {
            history,
            buffer_copy,
            cursor,
            path,
        })
    }
    pub fn down(&mut self) -> Option<String> {
        let filtered = self.filter();
        self.cursor += 1;
        if self.cursor >= filtered.len() {
            self.cursor = filtered.len();
            Some(self.buffer_copy.clone())
        } else {
            Some(filtered[self.cursor].clone())
        }
    }

    pub fn up(&mut self) -> Option<String> {
        let filtered = self.filter();
        self.cursor = std::cmp::min(self.cursor, filtered.len());
        if self.cursor == 0 || filtered.is_empty() {
            None
        } else {
            self.cursor = self.cursor.saturating_sub(1);
            Some(filtered[self.cursor].clone())
        }
    }

    pub fn push(&mut self, buffer: String) {
        if !buffer.is_empty() && Some(&buffer) != self.history.last() {
            self.buffer_copy.clear();
            self.history.push(buffer);
            self.go_to_last();
        }
    }

    pub fn update_buffer_copy(&mut self, buffer: &str) {
        self.buffer_copy = buffer.to_string();
        self.cursor = self.history.len();
    }

    pub fn reset_buffer_copy(&mut self) {
        self.buffer_copy.clear();
        self.cursor = self.history.len();
    }

    pub fn save(&self) {
        let is_comment = |s: &str| -> bool { s.trim_start().starts_with("//") };
        let mut history = self.history.clone();

        if history[0] != NEW_HISTORY_MARK {
            history.insert(0, NEW_HISTORY_MARK.to_string());
        }

        let history: Vec<String> = history
            .into_iter()
            .map(|e| {
                let e: Vec<String> = e
                    .lines()
                    .filter(|l| !is_comment(l))
                    .map(ToOwned::to_owned)
                    .collect();
                e.join("\n")
            })
            .collect();
        let history = history.join("\n//\n");

        let _ = fs::write(&self.path, history);
    }

    fn filter(&self) -> Vec<String> {
        self.history
            .iter()
            .filter(|h| h.contains(&self.buffer_copy))
            .map(ToOwned::to_owned)
            .collect()
    }

    fn go_to_last(&mut self) {
        if !self.history.is_empty() {
            self.cursor = self.history.len();
        }
    }

    pub fn find(&self, needle: &str) -> Option<&String> {
        self.history.iter().find(|h| h.contains(needle))
    }
}
