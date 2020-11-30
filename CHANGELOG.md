**1.1.2**
- Hotfix: Actually Add missing exit after (irust -x)

**1.1.1**
- Hotfix: Add missing exit after (irust -x)

**1.1.0**
- Add `:asm` command => shows the assembly of the specified function(requires[cargo-asm](https://github.com/gnzlbg/cargo-asm))
- More refactoring
- Bug fixes

**1.0.0**
- Under-the-hood: 
  - Refactor most of the code base
  - Extract Printer as a an independent unit 
  - Add some tests now that its finally possible to do so
- Multiple Bug fixes
- Maybe new bugs?

**0.9.8**
- Fix `:add .` on windows
- More resilient build errors detection
- Fix `:load` regression

**0.9.8**
- More performance optimizations
- Fix scrolling regression

**0.9.7**
- Flamegraph based performance optimizations

**0.9.6**
- Highlight matching parenthesis with the same color
- Clean up and document highlight parsing code

**0.9.5**
- Changed `:bench` command to `:time`
- Added `:time_release` => same as time but with release mode
- Added `:bench` command => runs cargo bench and returns the output

**0.9.4**
- Add `:bench` command, it measures the time an expression took to execute, example: `:bench my_func(arg1, arg2)`

**0.9.3**
- Use dirs::cache directory for faster cold startup time (falls back to $temp in case its not specified)
- Un-silence errors (Remove old let = _)
- update deps

**0.9.2**
- Fix a subtle bug (make sure the fn main is always written at least in two lines)
- More errors handling work

**0.9.1**
- Improve handling of threads panics

**0.9.0**
- Improve error handling
- Update dependencies

**0.8.60**
- Statements are now checked with cargo_check before being inserted into the repl (this behavior is configurable with `check_statements` command)

**0.8.50**
- Improvement to `:edit` command:
  - Add a new file $temp/irust/src/main_extern.rs, Any modification to this file will be immediately reflected on the repl (after saving)
  - On windows use "cmd /C"
- Make sure to set `CARGO_TARGET_DIR` to the correct path (needed for user who use a custom cargo target dir)
- Handle AltGr on Windows
- Update to crossterm 0.18.0, which contains among other cool stuff, fixes for winapi, also disable cargo coloring when using winapi

**0.8.17**
- Handle crate_attributes correctly (Insert outside of main)
- Add `:toolchain` command (supported value: stable, nightly, beta)
- `extern` keyword doesn't require `;`
- Detect build error after `:add` command
- Update dependencies

**0.8.16**
- fix duplicate building after using `:add` command

**0.8.15**
- Add `Ctrl-e` to force evaluation (useful for casese where incomplete_input fn can't handle)
- Add `dyn` to keywrods
- Upgrade deps

**0.8.14**
- Save current working directory when using cargo run so now for example: `std::process:Command::new("pwd")` will give the expected output instead of `/tmp/irust`
- `:add` command now tries to parse paths more agressivly, this is usefull for relative paths like `:add .`
- Add `while` keyword to the highlight parser and to the statments that doesn't require `;` at the end
- Improve error message when reacer is not properly configured

**0.8.13**
- Fix crash when history file is first created
- Update dependencies

**0.8.12**
- Fix `rustfmt` install command
- Add a reminder to reload shell after installing a dependency, fix clippy warning

**0.8.11**
- Use `toml/serde` crates to parse irust config file instead of manual parsing, theme and config files are now changed to toml.
- Add the ability to invoke irust with a path to a file, that will be loaded into the repl automatically, exp: `irust src/lib.rs`
- Add a check for required dependencies
- Add a one time warning for optional dependencies, and the ability to install them automatically
- Add `:color` command -> change highlight color at runtime, exp: `:color function red`, `:color keyword #ffab12`




**0.8.10**
- Match IPython ctrlc and new lines behavior
- Bunch of changes to `load` command in order to improve the interactive usage of the repl:
  - `load` now compiles the codes before loading it and output errors if present
  - `load` resets the repl before importing the code
  - Add `reload` which reloads the last specified path

**0.8.9**
- Add the ability to read a theme file to be used for the repl color highlighting (detail of usage on the README)

**0.8.8**
- remove syntect, use a custom made parser instead -> big runtime/compile time improvement + big decrease in dependencies
- use `dirs_next` crate instead of deprecated `dirs`

**0.8.7**
- Improve start time with cold cache
- Write a new line at exit (needed for some shells like bash, powershell..)

**0.8.6**
- Fix regression: racer suggestions cycling
- Bug-fix: add buffer bound check for remove_current_char

**0.8.5**
- Improve performance by queuing output and flushing only when needed
- Handle terminal size change (a bit hacky but works)
- Remove racer inline callback, this was always a source of problems, and didn't give much value since you can trigger auto-completion with Tab
- Improve logging
- Don't clear screen when starting IRust
- Bug fix: bound adjustment
- Code improvements

**0.8.4**
- add `:cd` to racer suggestions
- expression starting with `pub` dont require ';'
- don't clear screen after exiting

**0.8.3**
- Add `:cd` command
- Don't scan for incomplete input when its a builtin cmd or a shell cmd
- Set terminal title according to current working directory

**0.8.2**
- Update all dependencies
- `syntect`: switched to the new [fancy-regex](https://github.com/trishume/syntect#pure-rust-fancy-regex-mode-without-onig) engine

**0.8.1**
- Update `:type` to work with latest stable compiler version (Might break with future update, maybe use `Any` trait to determine type?)

**0.8.0**
- Try to canonicalize paths used with `:add` command, so now this for example works `:add regex --path ./regex` or for a short version `:add ./` (Adding local dependency regex)
- Update dependencies

**0.7.51**
- bug-fix: Keep the cursor in bound when hitting down key

**0.7.50**
  - update crossterm to 0.14

**0.7.40**
- funcitons, enums, structs, traits, now won't require `;` at the end of ther definition
- minor bug fix

**0.7.30**
- Update IRust to crossterm 0.13*
- Update all dependencies

**0.7.20(broken)**
- Add feris
- Add search history function with `Ctrl-r`

**0.7.14**
- Add confirmation dialog to exit with `Ctrl-d`
- Remove exit function from `Ctrl-c`

**0.7.13**
- Handle error gracefully when racer is improperly configured

**0.7.12**
- Bug fixes

**0.7.11**
- Add `:edit` command -> edit internal buffer using an external editor, example: `:edit gedit`

**0.7.10**
- Refactor printer.rs (with some bug fixes)

**0.7.9**
- Update deps
- Update codebase to use crossterm 0.11

**0.7.8**
`Tab` will now add 4 spaces if the current line is is empty

**0.7.7**
- Add `Alt-Enter` keyevent -> add line break

**0.7.6**
- Make Racer optional again

**0.7.5**
- Update dependencies
- Refactor + Clean up
- Bug-fix: reset bounds after clear

**0.7.4**
- Up/Down can move cursor in multi-line input
- Disable some optional `syntect` features

**0.7.3**
- More work on code-base refactoring
- Bug-fixes
- Add version to Cli flags (-v)

**0.7.2**
- Restore History filtering based on current buffer

**0.7.1**
- Clean up
- Bug fixes
- Restore disabled functionalities (disabled in 0.7.0)

**0.7.0**
- Start of major code-base refactor
- Input is now highlighted
- Credits to @smolck for his awesome ideas and contributions!

**0.6.14**
- Fix scrolling bug
- Improve highlight fn (using lazy evaluation)

**0.6.13**
- Use `chars_count` instead of `len` to handle special chars

**0.6.12**
- Fix line end detection

**0.6.11**
- Warn about empty outputs
- Disable cargo warnings
- Handle main panic
- Bug fixes

**0.6.10**
- Fix scrolling bug

**0.6.9**
- Refactor code
- Use `once_cell` crate for globals
- Small improvements to `unmatched_brackets` fn and `load_script` cmd

**0.6.8**
- Fix raw mode bug
- Improve `type` cmd

**0.6.7**
- Add `type` cmd (prints an expression type)

**0.6.6**
- Limit persistant histroy

**0.6.5**
- Make history persistant
- Fix `add` animation bug

**0.6.4**
- Scrolling bug fix

**0.6.3**
- Improve History

**0.6.2**
- Refactor code
- Rework Internal Cursor
- Handle multilines input correctly

**0.6.1**
- Update to crossterm '0.9.6'
- Activate `ctrl + arrow` on published build
- Improvements to incomplete input detection
- Simulate multilines when pasting multilines input (temporary needs rework)
- Try to keep current input when cycling history

**0.6.0**
- Handle `shift-tab` key (cycle sugestions backward)
- Handle `delete` key
- More Racer fixes
- More incomplete input detection

**0.5.12**
- Hotfix for multilines paste handling

**0.5.11**
- Handle multilines paste

**0.5.10**
- Enable the highlight feature by default for published build

**0.5.9**
- Improve `show` command:
    - highlight rust_code via [syntect](https://github.com/trishume/syntect)
    - format output if [rustfmt](https://github.com/rust-lang/rustfmt) is preset on the system
- Fix `load` command bug

**0.5.8**
- Improve `add` command

**0.5.7**
- Racer bug fix

**0.5.6**
- Make optional dependencies optional again

**0.5.5**
- Racer rework
- Better Errors handling
- Internal code refactor

**0.5.4**
- Use a real debouncer method
- Fix diffrent bugs in racer

**0.5.3**
- Revert auto-complete (issues with pasting)

**0.5.2**
- More bug fixes

**0.5.1**
- Bug fixes

**0.5.0**
- auto-complete `(` `{` `[`

**0.4.9**
- Reworked Racer, now it shows suggestions table + the inline suggestion

**0.4.8**
- Use scrolling instead of clearing at screen end

**0.4.7**
- Bug fixes (lines overflow)

**0.4.6**
- More lines overflow handling

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
