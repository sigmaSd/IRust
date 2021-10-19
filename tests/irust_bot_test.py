#!/bin/python
# type: ignore

# This test works probably only on unix, and there are some race conditions so sometimes it needs to be rerun

import pexpect
import sys

sys.tracebacklimit = 0

child = pexpect.spawn("cargo run")


def sink(n):
    for _ in range(n):
        child.readline()


def send(op):
    child.send(f"{op}\r")


def assert_eq(op, val, sinkN=0):
    print(f"[testing] `{op}` => `{val}`")
    send(op)
    sink(sinkN)
    out = child.readline().strip().decode("utf-8")
    if out == op:
        print("Race condition detected, test needs to be restarted (a couple of retries might be needed)")
        exit(1)

    out = out[len(out)-len(val):]
    assert out == val, f"got: `{out}` expected: `{val}`"
    print("success!")


# prelude
sink(2)

assert_eq(":add regex", "Ok!")
assert_eq('regex::Regex::new("a.*a").unwrap()', "a.*a")

assert_eq("z", "cannot find value `z` in this scope\x1b[0m")
sink(4)

assert_eq("1+2", "3")

send('let a = "hello";')
assert_eq(':type a', "`&str`")

send("""fn fact(n: usize) -> usize {
    match n {
        1 => 1,
        n => n * fact(n-1)
    }
}""")
assert_eq("fact(4)", "24")

assert_eq(':toolchain nightly', "Ok!")
send('#![feature(decl_macro)]')
send('macro inc($n: expr) {$n + 1}')
assert_eq('inc!({1+1})', '3')


print("All tests passed!")
