# IRust
Cross Platform Rust Repl

## Keywords / Tips & Tricks

**:help** => print help

**:reset** => reset repl

**:show** => show repl current code

**:add** *<dep_list>* => add dependencies (requires [cargo-edit](https://github.com/killercup/cargo-edit))

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

**Tab** cycle through auto-completion suggestions (requires [racer](https://github.com/racer-rust/racer))

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
    racer_color = DarkCyan

    [Colors]
    insert_color = White
    input_color = Yellow
    out_color = Red
    ok_color = Blue
    show_color = DarkCyan
    eval_color = White
    irust_color = DarkBlue
    irust_warn_color = Cyan
    shell_color = DarkYellow
    err_color = DarkRed

    [Welcome]
    welcome_msg = Welcome to IRust
    welcome_color = DarkBlue

## Changeslog
**0.4.5**
- Handle `CtrlLeft` `CtrlRight` (Only on master branch)
- Handle lines overflow

**0.4.4**
- Some improvement to autocompletion

**0.4.3**
- Autocomplete IRust commands
- Debounce from Racer calls

**0.4.2**
- Add `:pop` `:del` commands

**0.4.1**
- Racer is now optional

**0.4.0**
- Use Tab instead of BackTab

**0.3.10**
- Hotfix to workaround a tab bug for now

**0.3.9**
- Add Autocompletion support! (via racer)

**0.3.8**
- Add cli commands `--help` `--reset-config`

**0.3.7**
- Add the abilty to configure welcome message and color

**0.3.6**
- Add colors to config, now you can modify all of IRust colors!

**0.3.5**
- Use cargo colors

**0.3.4**
- Add `:help` command

**0.3.3**
- Reworked Output, now colors are everywhere and easier to add!
- IRust now talks to you (outputs some warning for now)

**0.3.2**
- Format rustc errors to be way more better looking

**0.3.1**
- Fix regression: Readd expressions to history

**0.3.0**
- Handle characters like `é`, `ù`

**0.2.1x**
- Don't upload artifacts to crates.io

**0.2.0**

Credits to this release goes to the awesome suggestions and contributions of @pzmarzly

- add `Ctrl-Z` `Ctrl-C` `Ctrl-D` keybindings
- add configuration file

**0.1.7**
- IRust
- Nicer output (handle multiline and singleline diffrently)

**0.1.6**
- Add keybindings `HOME` `END`
- Better add_cmd animation
- Refactor code

**0.1.5**
- Add keybindings `ctrl-c` `ctr-l`
- Fix history regression

**0.1.4**
- Handle parsing errors and output useful info
- Fix add dep regression

**0.1.3**
- Rely on a custom cursor struct to avoid a lot of headaches

**0.1.2**
- Load scripts that contains main fn

**0.1.1**
- Add **::** to execute shell cmds
- Bugfixes

