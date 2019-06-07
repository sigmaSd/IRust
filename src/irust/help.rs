use crate::irust::printer::{ColoredPrinterItem, Printer};
use crate::irust::{IRust, IRustError};
use crossterm::Color;

impl IRust {
    pub fn help(&mut self) -> Result<Printer, IRustError> {
        let mut outputs = Printer::default();

        outputs.push("### Keywords / Tips & Tricks ###".to_output(Color::DarkYellow));
        outputs.push(
            "
:help => print help

:reset => reset repl

:show => show repl current code

:add <dep_list> => add dependencies (requires cargo-edit)

:load => load a rust script into the repl

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

HOME/END go to line start / line end"
                .to_output(Color::DarkCyan),
        );

        Ok(outputs)
    }
}
