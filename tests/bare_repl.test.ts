import readline from "node:readline";
import process from "node:process";
import assert from "node:assert";

const irust = new Deno.Command("cargo", {
  args: ["run", "--bin", "irust", "--", "--bare-repl"],
  stdin: "piped",
  stdout: "piped",
  stderr: "null",
}).spawn();
const writer = irust.stdin.getWriter();

if (import.meta.main) {
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
  });

  rl.on("line", async (line: string) => {
    await writer.write(
      new TextEncoder().encode("IRUST_INPUT_START" + line + "IRUST_INPUT_END"),
    );
  });
  irust.stdout.pipeTo(Deno.stdout.writable);
}

async function inputRepl(
  reader: ReadableStreamDefaultReader,
  input: string,
): Promise<string> {
  await writer.write(
    new TextEncoder().encode("IRUST_INPUT_START" + input + "IRUST_INPUT_END"),
  );

  let readData = "";
  while (true) {
    readData += await reader.read().then((result) => {
      // if (!result.done) {
      //   console.log("result:", new TextDecoder().decode(result.value));
      // }
      return new TextDecoder().decode(result.value);
    });

    if (readData.includes("IRUST_OUTPUT_END")) {
      const output =
        readData.split("IRUST_OUTPUT_END")[0].split("IRUST_OUTPUT_START")[1];
      return output.trim();
    }
  }
}

export async function testRepl(
  reader: ReadableStreamDefaultReader,
  input: string,
  expectedOutput: string,
): Promise<boolean> {
  return await inputRepl(reader, input) === expectedOutput.trim();
}

Deno.test("bare repl", async (t) => {
  const reader = irust.stdout.getReader();
  await t.step("simple", async () => {
    assert(await testRepl(reader, "1 + 1", "2"));
    assert(await testRepl(reader, "let a = 4;", ""));
    assert(await testRepl(reader, "a * 2", "8"));
  });

  await t.step("add deps", async () => {
    assert(await testRepl(reader, ":add scolor", "Ok!"));
    assert(await testRepl(reader, "scolor::ColorType::Fg", "Fg"));
    // assert(
    //   await testRepl(
    //     reader,
    //     ":add scolordosntexist",
    //     "Failed to add dependency",
    //   ),
    // );
  });
});
