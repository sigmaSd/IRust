**1.72.0**
- Update codebase to edition 2024
- Add edition 2024 and default to it for the repl
- Do not override irust config if it has external changes

**1.71.30**
- Mention `:help` in default welcome message

**1.71.29**
- Hot fix for mac build

**1.71.28**
- Workaround macos permissions issue, by using the cache dir as default location for repls

**1.71.27**
- Reset color at exit

**1.71.26**
- Allow loading partial config

**1.71.25**
- revert for now "don't save options automaticly, add new command `save_options` to save options explicitly"

**1.71.24**
- don't save options automaticly, add new command `save_options` to save options explicitly
- give a little ui feedback for slow commands
- remove once cell

**1.71.23**
- update libc inorder to support the loongarch64 architecture

**1.71.22**
- Make rust analyzer comple private members of structs
- Allow configuring irust directories via env variables

**1.71.21**
- General cleanup and deps update

**1.71.20**
- Fix formating of output on newer version of rust

**1.71.18**
- Make :add command more robust

**1.71.15**
- fix edit command (don't drop repl)

**1.71.14**
- Remove all mention of racer (rust-analyzer is whats being used)

**1.71.10**
- package irust as a flatpak

**1.71.4**
- Optimize binary artifacts
- Switch from gnu to musl for linux binary artifact

**1.71.3**
- Add support for cargo binstall

**1.71.2**
- Disable optional deps lookup when using `--default-config`

**1.71.1**
- Update deps
- Disable rust anlayzer by default, to enable set `enable_racer` to true in the config
- Add new cli argument `--default-config`, that runs irust with default config in that run, but will not overwrite the saved config

**1.71.0**
- Support `NO_COLOR` env variable to disable all colors
- Add new option `new_lines_after_output` to specify the number
of new lines after output (defaults to 1)

**1.70.0**
- Use rust-analyzer instead of racer for completions

**1.69.0**
- feat: make each repl unique :
    - Give each repl struct in irust_repl a unique cargo project
    - This allows repls to coexist, each in its own project
    - Also update irust to make use of this, so now multiple irust instances can't corrupt each other

**1.68.0**
- Add CompileMode (Debug/Release) to EvalConfig in irust_repl
- Expose CompileMode in irust, with compile_mode command

**1.67.4**
- allow bracketed paste
- clippy + fmt

**1.67.3**
- fix #109 typing twice in windows with crossterm-0.26+

**1.67.2**
- Update crossterm to fix a panic

**1.67.0**
- Update rust+deps
- Use crossterm use-dev-tty feature

**1.66.0**
- Allow multiple themes, see themes section in README

**1.65.1**
- Use cargo-show-asm instead of cargo-asm
- Allow cargo-expand a specifc function `:expand myfn`
- Add another key to force new lines (ctrl-s)
- Add a new rust script that allows supporting mixed rust code with irust commands
- Update deps, fix clippy, internal fixes

**1.65.0**
- Change script4 to script, theme to theme.toml and config to config.toml
- Update deps, fix clippy

**1.64.0**
- Update crossterm

**1.63.4**
- Use `cargo` instead of `cargo-add` now that it's builtin

**1.63.3**
- Fix `:expand` command

**1.63.2**
- Fix clippy warnings

**1.63.1**
- Update crossterm
- Use https://github.com/pksunkara/cargo-workspaces to improve dev experience

**1.63.0**
- Ignore server's client errors
- Hide some warnings when evaluating

**1.62.1**
- Allow multiple irust instances again

**1.62.0**
- Add `:hard_load_crate` command, its a more powerful version of `hard_load` that copies the whole crate src dir to irust directory, which allows using internal and external imports, see https://github.com/sigmaSd/irust-vim-plugin for an example

of usage
**1.61.1**
- Update racer install method

**1.61.0**
- Add `hard_load` command, it manually sets the repl internal buffer and cursor (useful for vim plugin)

**1.60.0**
- Add the ability to spawn a local server, this allows irust to be controlled programmatically, checkout https://github.com/sigmaSd/irust-vim-plugin that uses this

**1.50.0**
- Rework Startup/Shutdown hooks

**1.49.0**
- Fix a bug with tokio runtime making it unusable

**1.48.0**
- `:dbg` command: use rust-gdb rust-lldb
- `:scripts` command: Add more style

**1.47.0**
- Activate Before/After compiling hooks and add prompt animation example

**1.46.0**
- Make shell interpolation more resilient, and fix its usage on windows

**1.45.0**
- Add cargo expand support via `:expand`

**1.44.0**
- Evaluate input from Stdin if provided, example: `echo "hello".chars() | irust`

**1.43.0**
- Fix windows bug (Path needs to be escaped in prelude Cargo.toml)

**1.42.0**
- Remove unneeded empty expression restriction on debugger

**1.41.0**
- Actually fix the aforementioned regression (Forgot to bump the repl, aka rust + workspace)

**1.40.0**
- Fix cold boot timing regression

**1.39.0**
- IRust now automatically creates `irust_prelude` crate at `xdg_data_dir/irust/irust_prelude`, this crate is imported at startup, any changes to it (that are marked with `pub`) will be immediately reflected on the repl after saving.

**1.38.0**
- Shell commands can now be interpolated with rust code via '$$', example: `let a = $$ls -l$$`;

**1.37.5**
- Some fixes to improve gitpod support in the repo
- `:help` now shows a compact version by default, for the old behavior you can use `:help full`
- Clippy + Update rustc lexer

**1.37.4**
- Fix color leak (in input prompt)

**1.37.3**
- Remove artificial delay when running an expression (99ms less)

**1.37.2**
- Don't error if the process has ended (when writing to its stdin)

**1.37.1**
- Bump repl, So the last update actually  works (aka: cargo workspace can be tricky)

**1.37.0**
- Pipe process stdin and write events to it by default, this allows code like `std::io::stdin().read*` to work

**1.36.0**
- Add `:dbg {expression}` command => spawns lldb/gdb with the specified expression, example: `:dbg fact(4)`

**1.35.0**
- Use `rustc_lexer` instead of the current custom lexer to highlight the code

**1.34.0**
- Add Parse, PrintOutput commands
- Modify OutputEvent to output Command instead of String
- Extract `:fun` command to its own script instead
- Refactor + switch back to dirs crate

**1.33.0**
- Add `:fun` command to create custom macros (checkout the README for more info)

**1.32.0**
- Add irust macros (macros can be saved with `ctrl-o, (key) -> user_macro_input -> ctrl-o` and played with `ctrl-p, key`
- Add irust undo/redo support (undo: `ctrl-u`, redo: `ctrl-y`)
- Add `ctrl-x` to delete current line

**1.31.0**
- Allow specifying the edition (default to 2021)
- Update codebase to the 2021 edition

**1.30.6**
- Add `SetTitle` `SetWelcomeMsg` hooks

**1.30.5**
- Organize IRust API

**1.30.4**
- Activate OutputEvent hook

**1.30.3**
- Update rscript

**1.30.2**
- Update rscript

**1.30.1**
- Update dependencies

**1.30.0**
- Refactor script interface, keep only v4 as supported

**1.29.0**
- Fix a bug where an output > SIZE can hang indefinitely (see https://users.rust-lang.org/t/weird-interaction-between-stdio-piped-and-child-try-wait/65898)

**1.28.0**
- :edit command now looks for a EDITOR variable for an editor it its invoked with no arguments
- BugFix: Fix output position after Enter

**1.27.0**
- Add more completions to racer
- Add :exit :quit commands
- Fix Typo in :type

**1.26.0**
- Downgrade crossterm due to upstream bug https://github.com/crossterm-rs/crossterm/issues/601

**1.25.0**
- Handle rust2021 new error format
- Add `show_warngings` option
- Add `macro` to keywords
- Add `macro` to auto statement
- Add `const fn` and `unsafe fn` to statements that don't require `;` to be inserted, also score some style points with pattern matching
- Add new expect tests
- Update deps

**1.24.0**
- Fix and simplify `read_until_bytes` (used by racer)
- Fix crash on unicode char boundary
-Improve the emoji situation 2: Accept that dealing with emoji is too troublesome and just replace any char with width != 1 with '�'. The char is not actually replaced, only the printing is affected, this means the output wont change

**1.23.0**
- Improvement to `:load` command
    - irust `{path_to_rust_file}` will now start IRust with the file loaded in the repl
    - Simplified `:load` command by removing `find and remove main` logic
- Update dependencies

**1.22.2**
- Fix regressions (:asm)

**1.22.1**
- Fix regressions (:type :evaluator)

**1.22.0**
- repl:
    - remove `new` and change `new_with_executor` to `new`
    - implement Default for Repl
    - Add a new type MainResult, available options are Unit or Result
- irust:
    - Add new command `main_result` to switch between main result types, using MainResult::Result allows using `?` with no boilerplate

**1.21.0**
- Process all scripts shutdown commands instead of the just the first one

**1.20.0**
- Update to rscript 0.8.0, the advantage is scripts can specify different comparators for specifying the irust version requirement (used to be only exact versioning was supported)

**1.19.0**
- Update to rscript 0.7.0, the advantage is scripts are now checked for incompatibility with versioning, previously an incompatibile script could be loaded by IRust which lead to subtle undefined behaviors

**1.18.0**
- Add a new `:compile_time` command, set it to on will make IRust print compiling time on each input
- Update to rscript 0.6.0 (improves script writing ergonomics, scripts also updated)


**1.17.0**
- Add SetInputPrompt/SetOutputPrompt script v4 hooks, ResetPrompt command, and a irust_prompt script example
- Update v4 scripts to rscript 0.5.0

**1.16.0**
- Add persistence for scripts states: (active/inactive will be saved and restored automatically)
- Add a new hook `Startup`
- `:scripts $script activate/deactivate`, will now send Hook::Startup and hook::Shutdown respectively
- Improvement to irust-vim mode script

**1.15.0**
- Improve `:scripts` command, scripts can now be individually activated/deactivated
- Add Script v4 support (see Scripts.md for more info)

**1.14.0**
- Add a shutdown hook (gives a change for scritps to cleanup)
- Vim-mode script: use different cursor shapes for different modes
- Update to crossterm 0.20.0

**1.13.0**
- IRust: Add `:evaluator` command, it allows changing the evaluator used by IRust, checkout the README for more info on its usage
- irust-repl: Make the evaluator swap-able
- Update deps
- Improve vim-mode script
- Color `unsafe` keyword

**1.12.0**
- Add more commands/ update vim script

**1.11.0**
- Fix regression: Restore history caching to disk
- Add default toolchain, and use it by default
- Add macro_rules to automatic `;` insert list
- Fix regression: Reflect repl changes on main_extern so it can be seen by gui editors (when using :edit command)

**1.10.0**
- Add Script V3 support, check out the README for more info
- Add `:scripts` command that list currently loaded scripts (Script V3 only)
- Improve the performance of printing the input (by not moving the cursor when not needed)
- Add new commands to the API
- Improve tab behavior
- Bugfixes

**1.9.0**
- Refactor internals to use Engine/Commands style and add a new script `input_event`, this script can intercept crossterm events and return irust_api::Command to control IRust behavior, a vim mode example based on this script is also provided

- Fix race condition with cargo rm
- Expose prompt length in the irust api
**1.8.1**
- Fix typo in script manager, that caused `output_prompt` script to be ignored - @Aloxaf

**1.8.0**
- Improve cold startup time by making the first build async
- Merge `:set_executor/executor` commands in one command `:executor`, if no arguments are provided it will act as a getter, else as setter, `:toolchain` command is updated to have the same behavior
- Use --rust option with cargo-asm to interleave rust code
- Organize code with workspace
- irust_repl: add Repl::new_with_executor to the api, Repl::new defaults to sync

**1.7.2**
- Add `:set_executor` command to set the executor used by IRust, available options are: `sync` `tokio` `async_std`, by  using an async executor, `await` becomes usable with no other modifications
- Hide warnings from evaluation output
- Added a book with tips and tricks https://sigmasd.github.io/irust_book/

**1.7.1**
-Internal refactor: split the repl engine in its own crate

**1.7.0**
- Introduce new scripting method codename: scriptv2 , see README for more info

**1.6.2**
- Improve the help command by parsing the README.md markdown
- Organize the template script better
- BugFix: print prompt correctly after clearing the screen

**1.6.1**
- Fix script dynamic library path on windows
- Don't overshoot when using backward search

**1.6.0**
- Improve scripting: Instead of a script file, now if scripting option is set, IRust will create a script project in $config/irust/script with a default template. This make it easy to add external dependencies among other advantages.
- Make script return type ffi safe

**1.5.0**
- Add scripting to IRust: Add a script manager to irust, scripting feature can be activated by a new option in the configuration file. If this option is set IRust will look for a script file in `$config/irust/script.rs` and load the functions specified there.
 Currently only two functions are supported: `input_prompt` `output_prompt`

- Add input_prompt/output_prompt to the configuration file

**1.4.1**
- Clear racer suggestions before printing output

**1.4.0**
- Input prompt is now customizable
- Split printer system in its own crate `printer`

**1.3.2**
- Make home/end key go to start/end of line respectively instead of start/end of input
- Improve history search, `ctrl-r` continues to search backward, `ctrl-s` searches forward

**1.3.1**
- Fix regression: check buffer before jumping left

**1.3.0**
- Add an option to use last output as a variable. The marker for the last output can be configured
- Remove the library part and move the tests to src
- Improve the handling of wide characters
- Rename `irust` directory to `irust_repl` to avoid conflict with irust(binary) if a global target directory is set (conflict happens in windows)
- Trim shell output (via `::` command)

**1.2.8**
- Fix some of Racer printing suggestions edge-cases

**1.2.7**
- Racer suggestions/completions can now happen even inside a line (it used to be only allowed at the end of a line)
- Bug fixes

**1.2.6**
- Fix panic: update bounds when updating dimensions

**1.2.5**
- Simplify printing output logic a lot by relying on crossterm::cursor::position + bug fix
- Printing art no longer busy loops
- Remove legacy error handling code
- Improve input hack error message
- Bug fixes

**1.2.4**
- Output can now be cancelled with Ctrl-c (in case of an infinite loop for example)
- Auto inserting semicolons is now configurable
- Update dependencies

**1.2.3**
- Update crossterm to 0.19
- Handle quoted arguments in :add command(useful specifying multiple features, example: `:add --features "a b"`)

**1.2.2**
- Fix regression: Handle multiline string

**1.2.1**
- Use `CARGO_TARGET_DIR` path if set instead of overwriting it

**1.2.0**
- Remove notify
- :edit command work as usual with terminal editors, for gui editors like vs code, the `:sync` command needs to be used after the writing
- Bug fixes
- Add benchmark

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
