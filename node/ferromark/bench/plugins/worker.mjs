import assert from "node:assert/strict";
import { performance } from "node:perf_hooks";

import { fixtures } from "./fixtures.mjs";

const [fixture, format, lane] = process.argv.slice(2);
const { source, diagrams } = fixtures[fixture];
globalThis.gc();
const before = process.memoryUsage();
const loadStart = performance.now();
const api = await import("ferromark");
if (lane === "ferrigura") assert.ok(api.availableCodePlugins().includes("ferrigura"));
else api.availableCodePlugins();
const loadMs = performance.now() - loadStart;
const afterLoad = process.memoryUsage();
const initStart = performance.now();
const compiler =
  lane === "plain"
    ? null
    : new api.CodeCompiler({
        plugins:
          lane === "ferrigura" ? [{ name: "ferrigura", options: { idPrefix: "bench" } }] : [],
      });
const initMs = performance.now() - initStart;
const afterInit = process.memoryUsage();
const render =
  format === "html"
    ? () => (compiler ? compiler.toHtml(source) : api.toHtml(source))
    : () => (compiler ? compiler.compile(source) : api.compileJsx(source)).body;
const firstStart = performance.now();
const first = render();
const firstRenderMs = performance.now() - firstStart;
assert.equal((first.match(/<svg /g) ?? []).length, diagrams);
if (!diagrams)
  assert.equal(first, format === "html" ? api.toHtml(source) : api.compileJsx(source).body);
const outputBytes = Buffer.byteLength(first);
for (let i = 0; i < 5; i++) render();
globalThis.gc();
const samples = [];
let sink = 0;
for (let sample = 0; sample < 11; sample++) {
  let count = 0;
  const start = performance.now();
  do {
    sink ^= render().length;
    count++;
  } while (performance.now() - start < 20);
  samples.push({ iterations: count, millisecondsPerRender: (performance.now() - start) / count });
}
globalThis.gc();
const retained = process.memoryUsage();
process.stdout.write(
  JSON.stringify({
    fixture,
    format,
    lane,
    inputBytes: Buffer.byteLength(source),
    outputBytes,
    loadMs,
    initMs,
    firstRenderMs,
    samples,
    sink,
    memory: {
      before,
      afterLoad,
      afterInit,
      retained,
      maxRssBytes: process.resourceUsage().maxRSS * 1024,
    },
  }),
);
