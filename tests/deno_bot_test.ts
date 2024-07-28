#!/usr/bin/env -S deno run --unstable-ffi --allow-all
import { Pty } from "jsr:@sigma/pty-ffi@0.26.2";
import { stripAnsiCode } from "jsr:@std/fmt@0.220.1/colors";
import { assertEquals, assertMatch } from "jsr:@std/assert@0.220.1";

const ENCODER = new TextEncoder();

if (import.meta.main) {
  const pty = new Pty({
    cmd: "cargo",
    args: ["run", "--", "--default-config"],
    env: [["NO_COLOR", "1"]],
  });

  while (true) {
    let { data: input, done } = await pty.read();
    if (done) break;
    input = stripAnsiCode(input);

    if (input.includes("In:")) break;
    await sleep(100);
  }

  const write = async (input: string) => await pty.write(`${input}\n\r`);
  const evalRs = async (input: string) => {
    await write(input);
    // detect output
    // the plan is:
    // TODO
    let lastResult = "";
    let idx = 0;
    let start = 0;
    while (true) {
      let { data: output, done } = await pty.read();
      if (done) break;
      output = stripAnsiCode(output).trim();
      if (output && output !== "In:") lastResult = output;

      if (output && start === 0) {
        start = 1;
      }
      if (!output && start === 1) {
        start = 2;
      }
      if (output && start === 2) {
        start = 3;
      }

      if (start === 3 && !output && lastResult) {
        idx++;
      } else {
        idx = 0;
      }

      if (idx === 5) {
        const result = lastResult.replace(/^Out:/, "").trim();
        return result;
      }
      await sleep(100);
    }
    // not really needed
    return "";
  };

  const test = async (
    input: string,
    expected: string | RegExp,
  ) => {
    Deno.stdout.write(ENCODER.encode(`eval: ${input}`));
    const output = await evalRs(input);
    // try catch just to add a new line before the error
    try {
      if (typeof expected === "string") {
        assertEquals(
          output,
          expected,
        );
      } // exepected is a regex
      else {
        assertMatch(output, expected);
      }
    } catch (e) {
      console.log();
      throw e;
    }
    console.log(" [OK]");
  };

  await write('let a = "hello";');
  await test(":type a", "`&str`");

  await write(`fn fact(n: usize) -> usize {
      match n {
        1 => 1,
        n => n * fact(n-1)
      }
  }`);
  await test("fact(4)", "24");

  await test("5+4", "9");
  await test("z", /cannot find value `z`/);
  await test("let a = 2; a + a", "4");
  // NOTE: this requires network, is it a good idea to enable it ?
  // await evalRs(":add regex");
  // await test('regex::Regex::new("a.*a")', 'Ok(Regex("a.*a"))');
}

async function sleep(ms: number) {
  await new Promise((r) => setTimeout(r, ms));
}
