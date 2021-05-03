/// This file should not be changed
use std::path::PathBuf;

// the signature must be this
/// Global IRust variables accessible to the script
pub struct GlobalVariables {
    /// Current directory that IRust is in
    pub current_working_dir: PathBuf,
    /// Previous directory that IRust was in, this current directory can change if the user uses the `:cd` command
    pub previous_working_dir: PathBuf,
    /// Last path to a rust file loaded with `:load` command
    pub last_loaded_code_path: Option<PathBuf>,
    /// Last successful printed output
    pub last_output: Option<String>,
    /// A variable that increases with each input/output cycle
    pub operation_number: usize,
}
