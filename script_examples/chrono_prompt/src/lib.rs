use chrono::prelude::*;
/// This script prints an input/output prompt with the number of the
/// evaluation prefixed to it
use std::{ffi::CString, os::raw::c_char, path::PathBuf};

// the signature must be this
pub struct GlobalVariables {
    // Current directory that IRust is in
    _current_working_dir: PathBuf,
    // Previous directory that IRust was in, this current directory can change if the user uses the `:cd` command
    _previous_working_dir: PathBuf,
    // Last path to a rust file loaded with `:load` command
    _last_loaded_code_path: Option<PathBuf>,
    /// Last successful printed output
    _last_output: Option<String>,
    /// A variable that increases with each input/output cycle
    _operation_number: usize,
}

#[no_mangle]
// the signature must be this
pub extern "C" fn input_prompt(_global_varibales: &GlobalVariables) -> *mut c_char {
    let utc: DateTime<Utc> = Utc::now();
    // Default script
    CString::new(format!(
        "In [{}]: ",
        utc.format("%Y-%m-%d %H:%M").to_string()
    ))
    .unwrap()
    .into_raw()
}

#[no_mangle]
// the signature must be this
pub extern "C" fn output_prompt(_global_varibales: &GlobalVariables) -> *mut c_char {
    let utc: DateTime<Utc> = Utc::now();
    // Default script
    CString::new(format!(
        "Out [{}]: ",
        utc.format("%Y-%m-%d %H:%M").to_string()
    ))
    .unwrap()
    .into_raw()
}
