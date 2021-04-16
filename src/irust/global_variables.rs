use std::path::PathBuf;

pub struct GlobalVariables {
    current_working_dir: PathBuf,
    previous_working_dir: PathBuf,
    last_loaded_code_path: Option<PathBuf>,
    /// last successful output
    last_output: Option<String>,
}

impl GlobalVariables {
    pub fn new() -> Self {
        let cwd = std::env::current_dir().expect("Error getting current working directory");

        Self {
            current_working_dir: cwd.clone(),
            previous_working_dir: cwd,
            last_loaded_code_path: None,
            last_output: None,
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

    pub fn set_last_loaded_coded_path(&mut self, path: PathBuf) {
        self.last_loaded_code_path = Some(path);
    }

    pub fn get_last_loaded_coded_path(&self) -> Option<PathBuf> {
        self.last_loaded_code_path.clone()
    }

    pub fn get_last_output(&self) -> Option<&String> {
        self.last_output.as_ref()
    }

    pub fn set_last_output(&mut self, out: String) {
        self.last_output = Some(out);
    }
}
