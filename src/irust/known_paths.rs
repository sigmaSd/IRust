use std::path::PathBuf;

pub struct KnownPaths {
    current_working_dir: PathBuf,
    previous_working_dir: PathBuf,
}

impl KnownPaths {
    pub fn new() -> Self {
        let cwd = std::env::current_dir().expect("Error getting current working directory");

        Self {
            current_working_dir: cwd.clone(),
            previous_working_dir: cwd,
        }
    }

    pub fn update_cwd(&mut self, cwd: PathBuf) {
        self.previous_working_dir = self.current_working_dir.clone();
        self.current_working_dir = cwd;
    }

    pub fn get_cwd(&self) -> PathBuf {
        self.current_working_dir.clone()
    }

    pub fn get_pwd(&self) -> PathBuf {
        self.previous_working_dir.clone()
    }
}
