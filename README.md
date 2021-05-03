# IRust
Cross Platform Rust Repl

## Keywords / Tips & Tricks

**:help** => print help

**:reset** => reset repl

**:show** => show repl current code (optionally depends on [rustfmt](https://github.com/rust-lang/rustfmt) to format output)

**:add** *<dep_list>* => add dependencies (requires [cargo-edit](https://github.com/killercup/cargo-edit)) also it accepts most `cargo-edit` arguments

**:type** *\<expression\>* => shows the expression type, example `:type vec!(5)`
  
**:time** *\<expression\>* => return the amount of time the expression took to execute. example: `:time 5+4` `:time my_fun(arg1,arg2)`

**:time_release** *\<expression\>* => same as `time` command but with release mode

**:load** => load a rust file into the repl

**:reload** => reload the last specified file

**:pop** => remove last repl code line

**:del** *<line_num>* => remove a specific line from repl code (line count starts at 1 from the first expression statement)

**:edit** *\<editor\>* => edit internal buffer using an external editor, example: `:edit micro`, Note some gui terminal requires using `:sync` command after the edit (vscode)

**:sync** sync the changes written after using :edit with a gui editor (vscode) to the repl

**:cd** => change current working directory

**:color** *\<key\>* *\<value\>* => change token highlight color at runtime, for the token list and value representation check the Theme section, exp: `:color function red` `:color macro #ff12ab` `:color reset`

**:toolchain** *\<value\>* => switch between toolchains, supported value are: `stable`, `beta`, `nighty`
  
**:check_statements** *true*/*false* => If its set to true, irust will check each statemnt (input that ends with ;) with cargo_check before inserting it to the repl

**:bench** => run `cargo bench`

**:asm** *\<function\>* => shows assembly of the specified function, note that the function needs to be public (requires [cargo-asm](https://github.com/gnzlbg/cargo-asm))

**::** => run a shell command, example `::ls`

You can use arrow keys to cycle through commands history

## Keybindings

**ctrl-l** clear screen

**ctrl-c** clear line

**ctrl-d** exit if buffer is empty

**ctrl-z** [unix only]  send IRust to the background

**ctrl-r** search history, hitting **ctrl-r** again continues searching the history backward, hitting **ctrl-s** searches the history forward

**ctrl-left/right** jump through words

**HOME/END** go to line start / line end

**Tab/ShiftTab** cycle through auto-completion suggestions (requires [racer](https://github.com/racer-rust/racer))

**Alt-Enter** add line break

**ctrl-e** force evaluation

<img src="./irust.png" width="200%" height="60%">

## Cli commands

**--help** prints help message

**--reset-config** reset IRust configuration to default

## Configuration

IRust config file is located in:

**Linux**: */home/$USER/.config/irust/config*

**Win**: *C:\Users\\$USER\AppData\Roaming/irust/config*

**Mac**: */Users/$USER/Library/Preferences/irust/config*

*default config:*
```
  # history
  add_irust_cmd_to_history = true
  add_shell_cmd_to_history = false

  # colors
  ok_color = "Blue"
  eval_color = "White"
  irust_color = "DarkBlue"
  irust_warn_color = "Cyan"
  out_color = "Red"
  shell_color = "DarkYellow"
  err_color = "DarkRed"
  input_color = "Green"
  insert_color = "White"
  welcome_msg = ""
  welcome_color = "DarkBlue"

  # racer
  racer_inline_suggestion_color = "Cyan"
  racer_suggestions_table_color = "Green"
  racer_selected_suggestion_color = "DarkRed"
  racer_max_suggestions = 5
  enable_racer = true

  # other
  first_irust_run = false
  toolchain = "stable"
  check_statements = true
  auto_insert_semicolon = true
  
  // use last output by replacing the specified marker
  replace_marker = "$out"
  replace_output_with_marker = false
  
  # modify input prmopt
  input_prompt = "In: "
  output_prompt = "Out: "
  
  # activate scripting feature
  activate_scripting = false
```

## Theme
Since release `0.8.9` `IRust` can now parse a theme file located on `$config_dir/irust/theme` and use it for the highlighting colors.

Colors can be specified as names ("red") or as hex representation ("#ff12ab").

Default theme file:

```
  keyword = "magenta"
  keyword2 = "dark_red"
  function = "blue"
  type = "cyan"
  number = "dark_yellow"
  symbol = "red"
  macro = "dark_yellow"
  string_literal = "yellow"
  character = "green"
  lifetime = "dark_magenta"
  comment = "dark_grey"
  const = "dark_green"
  x = "white"

```
## Scripts
Since release `1.7.0` `IRust` has a new script mechanism codename script2, the old method is still available but deprecated for now see below if you still want to use it.

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

All scripts should add bincode and irust_api as dependecy

For more concrete example see scripts_examples directory


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


## Releases
   Automatic releases by github actions are uploaded here https://github.com/sigmaSd/irust/releases

## Building
    cargo b --release

## FAQ

**1- Why is autocompletion not working**

    -> you need racer installed and configured correctly
        cargo +nightly install racer
        rustup component add rust-src
        
**2- Racer fails to build**

You can try `rustup update --force` https://github.com/racer-rust/racer/issues/1141

**3- I want to hack on irust but `dbg!` overlaps with the output!!**

Personaly I do this:
- Run 2 terminals side by side
- run `tty` in the first which should output something like `/dev/pts/4`
- run `cargo r 2>/dev/pts4` in the second

Now the `dbg!` statements are printed on the second terminal and the output in the first terminal is not messed up.

## [Changelog](./CHANGELOG.md)
