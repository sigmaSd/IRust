# IRust
Cross Platform Rust Repl

## Keywords / Tips & Tricks

**:help** => print help

**:reset** => reset repl

**:show** => show repl current code (optionally depends on [rustfmt](https://github.com/rust-lang/rustfmt) to format output)

**:add** *<dep_list>* => add dependencies (requires [cargo-edit](https://github.com/killercup/cargo-edit))

**:type** *<expression>* => shows the expression type, example `:type vec!(5)`

**:load** => load a rust script into the repl

**:pop** => remove last repl code line

**:del** *<line_num>* => remove a specific line from repl code (line count starts at 1 from the first expression statement)

**::** => run a shell command, example `::ls`

You can use arrow keys to cycle through commands history

## Keybindings

**ctrl-l** clear screen

**ctrl-c** clear line, double click to exit

**ctrl-d** exit if buffer is empty

**ctrl-z** [unix only]  send IRust to the background

**ctrl-left/right** jump through words

**HOME/END** go to line start / line end

**Tab/ShiftTab** cycle through auto-completion suggestions (requires [racer](https://github.com/racer-rust/racer))

**Alt-Enter** add line break

<img src="./irust.png" width="80%" height="60%">

## Cli commands

**--help** prints help message

**--reset-config** reset IRust configuration to default

## Configuration

IRust config file is located in:

**Linux**: */home/$USER/.config/irust/config*

**Win**: *C:\Users\\$USER\AppData\Roaming/irust/config*

**Mac**: */Users/$USER/Library/Preferences/irust/config*

*default config:*

    [History]
    add_irust_cmd_to_history = false
    add_shell_cmd_to_history = false

    [Racer]
    enable_racer = true
    racer_inline_suggestion_color = Cyan
    racer_suggestions_table_color = Green
    racer_selected_suggestion_color = DarkRed
    racer_max_suggestions = 5

    [Colors]
    insert_color = White
    input_color = Yellow
    out_color = Red
    ok_color = Blue
    eval_color = White
    irust_color = DarkBlue
    irust_warn_color = Cyan
    shell_color = DarkYellow
    err_color = DarkRed

    [Welcome]
    welcome_msg = Welcome to IRust
    welcome_color = DarkBlue

## [Changelog](./CHANGELOG.md)
