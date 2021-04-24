/// This script prints an input/output prompt with the number of the
/// evaluation prefixed to it

/// Generated module that have accessible global variables
/// See its signature for more info
mod types;
use types::GlobalVariables;

use std::{ffi::CString, os::raw::c_char};

#[no_mangle]
// the signature must be this
pub extern "C" fn input_prompt(global_varibales: &GlobalVariables) -> *mut c_char {
    // Default script
    CString::new(format!("In [{}]: ", global_varibales.operation_number))
        .unwrap()
        .into_raw()
}

#[no_mangle]
// the signature must be this
pub extern "C" fn output_prompt(global_varibales: &GlobalVariables) -> *mut c_char {
    // Default script
    CString::new(format!("Out[{}]: ", global_varibales.operation_number))
        .unwrap()
        .into_raw()
}
