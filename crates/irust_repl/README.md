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
A basic jupyer kernel is provided for demo https://github.com/sigmaSd/IRust/tree/master/crates/irust_repl/irust_kernel, to use it:

Installation
------------

This requires IPython 3.

    pip install irust_kernel
    python -m irust_kernel.install

To use it, run one of:

    jupyter notebook
    # In the notebook interface, select IRust from the 'New' menu
    jupyter qtconsole --kernel irust
    jupyter console --kernel irust


Developement
------------

This requires https://github.com/pypa/flit

To start developping locally use `flint install --symlink` optionally followed by `python -m irust_kernel.install --local-build` if there are changes to `Re` executable

Examples
----------

irust.ipynb (simple showcase) and evcxr.ipynb (showcase of evcxr protocol) are provided as an example
