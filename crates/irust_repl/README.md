# IRust Repl

Repl engine used by IRust to interpret rust code

The core is `println!("{:?}", expression)` with tricks to conserve variables and context

Example:

```rust
use irust_repl::{Repl, ToolChain};

let mut repl = Repl::new(ToolChain::Stable).unwrap();
repl.insert("let a = 5");
assert_eq!(repl.eval("a+a").unwrap().output, "10");
```
Checkout the examples and tests folders for more info.


## Jupyter Kernel
A basic jupyer kernel is provided for demo https://github.com/sigmaSd/IRust/tree/master/crates/irust_repl/irustkernel, to use it:

- Compile `re` example with `cargo build --examples --release`
- Cp `re` to a folder in your `$PATH` so it can be used by the kernel, `cp target/release/examples/re $folder_in_path`
- Install the kernel with `jupyter kernelspec install --user irustkernel`, it should be listed now in `jupyter kernelspec list`
- Cd to irustkernel, and run jupyter, `jupyter lab .`, note: cding into irustkernel is important so python can find the module `irust` (irust.py), the path is hardcoded in `kernel.json`

That's it! `irust.ipynb` is provided as an example
