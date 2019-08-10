use crate::irust::printer::{ColoredPrinterItem, Printer};
use crate::irust::{IRust, IRustError};
use crossterm::Color;

impl IRust {
    pub fn help(&mut self) -> Result<Printer, IRustError> {
        let mut outputs = Printer::default();

        outputs.push(
            "### Keywords / Tips & Tricks ###".to_output(Color::DarkYellow),
        );
        outputs.push(
            "
:help => print help

:reset => reset repl

:show => show repl current code (optionally depends on rustfmt to format output)

:add <dep_list> => add dependencies (requires cargo-edit)

:type <expression> => shows the expression type, example :type vec!(5)

:load => load a rust script into the repl

:pop => remove last repl code line

:del <line_num> => remove a specific line from repl code (line count starts at 1 from the first expression statement)


:: => run a shell command, example ::ls

You can use arrow keys to cycle through commands history"
                .to_output(Color::DarkCyan),
        );
        outputs.push(
            "
### Keybindings ###"
                .to_output(Color::DarkYellow),
        );
        outputs.push(
            "

ctrl-l clear screen

ctrl-c clear line, double click to exit

ctrl-d exit if buffer is empty

ctrl-z [unix only] send IRust to the background

ctrl-left/right jump through words

HOME/END go to line start / line end

Tab/ShiftTab cycle through auto-completion suggestions (requires racer)"
                .to_output(Color::DarkCyan),
        );

        Ok(outputs)
    }
}
