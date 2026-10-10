import { createInterface } from "node:readline";
import { createConformanceRender } from "../markdown-ecosystem/conformance-adapters.mjs";
const [engine, profile] = process.argv.slice(2);
if (!engine || !["commonmark", "gfm"].includes(profile)) throw new Error("worker ENGINE PROFILE");
let render;
try {
  render = await createConformanceRender(engine, profile === "gfm");
} catch (error) {
  process.stderr.write(String(error.stack ?? error));
  process.exit(2);
}
for await (const line of createInterface({ input: process.stdin, crlfDelay: Infinity })) {
  const { markdown } = JSON.parse(line);
  let result;
  try {
    const html = await render(markdown);
    if (typeof html !== "string") throw new Error("Renderer did not return an HTML string");
    result = { html, error: null };
  } catch (error) {
    result = { html: null, error: String(error.stack ?? error) };
  }
  process.stdout.write(`${JSON.stringify(result)}\n`);
}
