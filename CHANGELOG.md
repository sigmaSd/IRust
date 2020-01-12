**0.8.0**
- Try to canonicalize paths used with `:add` command, so now this for example works `:add regex --path ./regex` (Adding local dependency regex)
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
