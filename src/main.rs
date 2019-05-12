mod cargo_cmds;
mod format;
mod history;
mod irust;
mod repl;
mod utils;

use irust::IRust;

fn main() {
    let mut irust = IRust::new();
    irust.run().expect("IRust Out");
}
