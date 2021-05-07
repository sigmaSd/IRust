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
