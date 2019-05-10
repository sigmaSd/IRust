# IRust
Cross Platform Rust Repl

## Keywords / Tips & Tricks

**:reset** => reset repl

**:show** => show repl current code

**:add** *<dep_list>* => add dependencies (requires [cargo-edit](https://github.com/killercup/cargo-edit))

**:load** => load a rust script into the repl

**::** => run a shell command, example `::ls`

You can use arrow keys to cycle through commands history

## Keybindings

**ctrl-l** clear screen

**ctrl-c** exit

**HOME/END** go to line start / line end

<img src="./irust.png" width="80%" height="60%">

## Changeslog

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
