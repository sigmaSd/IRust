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

Deno.test("bare repl", async () => {
  const reader = irust.stdout.getReader();
  const testRepl = async (input: string, expectedOutput: string) => {
    await writer.write(
      new TextEncoder().encode("IRUST_INPUT_START" + input + "IRUST_INPUT_END"),
    );

    let readData = "";
    while (true) {
      readData += await reader.read().then((result) =>
        new TextDecoder().decode(result.value)
      );

      if (readData.includes("IRUST_OUTPUT_END")) {
        const output =
          readData.split("IRUST_OUTPUT_END")[0].split("IRUST_OUTPUT_START")[1];
        return output.trim() === expectedOutput.trim();
      }
    }
  };

  assert(await testRepl("1 + 1", "2"));
  assert(await testRepl("let a = 4;", ""));
  assert(await testRepl("a * 2", "8"));
});
