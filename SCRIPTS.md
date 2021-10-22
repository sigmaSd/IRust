# Scripts
**Release `1.30.0`:**

Script v4 is now the only scripting interface available. (uses [rscript](https://github.com/sigmaSd/Rscript))\
The API is here https://github.com/sigmaSd/IRust/blob/master/crates/irust_api/src/lib.rs\
Scripts should depend on `irust_api` and `rscript` crates

Script examples are located here https://github.com/sigmaSd/IRust/tree/master/script_examples

## Usage:
- Set `activate_scripting` to `true` in config file.
- Compile a script (it can be oneshot/daemon/dylib(unsafe)), see examples
- Copy it to ~/.config/irust/script4/

That's it you can verify that scripts are detected with `:scripts`\
You can activate/deactivate scripts with `:script $myscript activate` (or deactivate)

## Old informations

**Since release `1.15.0` `IRust` has script v4 support**

The scripting engine is now extracted in its own crate [rscript](https://github.com/sigmaSd/Rscript)\
The advantages over v3 are mainly ergonomics and more compile time guarantees\
Also the script manager now supports Activating/deactivating individual scripts\
Scripts should be located under `$config/irust/script4`\
Script examples are located here https://github.com/sigmaSd/IRust/tree/master/scripts_examples/script4

**Since release `1.10.0` `IRust` has script v3 support**

the advantages are:
- No need to hardcode binaries name
- One script can listen onto multiple hooks
- Scripts can run on daemon or oneshot mode

IRust will look for any exectuable under `$Config/irust/script3` and run it, it needs to exchange a greeting message at startup and specify which hooks its interested in, later it will be called by IRust when a specified hook is triggered, check out the [examples](https://github.com/sigmaSd/IRust/tree/master/scripts_examples/script3) for more info

**Since release `1.7.0` `IRust` has a new script mechanism codename script2, the old method is still available but deprecated for now see below if you still want to use it.**

The main advantages are:

- No unsafe, scripts should not be able to break IRust (not 100%)
- Hot reloading! recompiling a script will immediatly take effect on IRust without restarting


To activate this feature, set `activate_scripting2` to `true` in the configuration file. (it will take precedence over script1 if its set to true)

Now IRust will look in `$config/irust/script2` for executables.

It will launch them when required and comminucate via stdin/stdout (with bincode as a relay).

The executables need to have the following properties:

| Name             | Input                       | Output  | What it should do
| ---------------- | --------------------------- | ------- | -------------------------------------------------
| input_prompt     | irust_api::GlobalVariables  | String  | return the input prompt value as a string
| output_prompt    | irust_api::GlobalVariables  | String  | return the output prompt value as a string
| while_compiling  | irust_api::GlobalVariables  | ()      | do arbitrary things while IRust is compiling an expression (print some waiting animation for example)
| input_event      | irust_api::GlobalVariables, crossterm::event::Event  |  irust_api::Command      | all crossterm events will be passed to this script, it can choose to act upon it and return a `Some(irust_api::Command)` or let the normal IRust flow continue by returning `None` (See examlpes for vi-mode built upon this)

All scripts should add bincode and irust_api as dependecy

For more concrete example see [scripts_examples](https://github.com/sigmaSd/IRust/tree/master/scripts_examples/script2) directory


**Old method**

Since release `1.5.0` `IRust` introduced scripting feature.

To activate it, set `activate_scripting` to `true` in the configuration file.

Now IRust will create a cargo project named `script` located at `$config/irust/script`

This project has a default template, that showcases the available features.

Currently Supported functions (see example):
```rust
pub extern "C" fn input_prompt(global_varibales: &GlobalVariables) -> *mut c_char
```
```rust
pub extern "C" fn output_prompt(global_varibales: &GlobalVariables) -> *mut c_char
```

Important points:
- Scripting is currently unsafe, using it incorrectly will cause IRust to crash or segfault
- Scripts have a higher precedence then options (for example prompt functions will override the prompt set in the configuration)

Template/Example:
```rust
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
    operation_number: usize,
}

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
```
