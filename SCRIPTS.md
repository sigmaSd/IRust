# Scripts
**Release `1.30.0`:**

Script v4 is now the only scripting interface available. (uses [rscript](https://github.com/sigmaSd/Rscript))\
The API is here https://github.com/sigmaSd/IRust/blob/master/crates/irust_api/src/lib.rs \
Scripts should depend on `irust_api` and `rscript` crates

Script examples are located here https://github.com/sigmaSd/IRust/tree/master/script_examples

## Usage:
- Set `activate_scripting` to `true` in config file.
- Compile a script (it can be oneshot/daemon/dylib(unsafe)), see examples
- Copy it to ~/.config/irust/script/

That's it you can verify that scripts are detected with `:scripts`\
You can activate/deactivate scripts with `:script $myscript activate` (or deactivate)
