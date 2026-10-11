import assert from "node:assert/strict";
import { performance } from "node:perf_hooks";
import { pathToFileURL } from "node:url";
const [beforePath, afterPath, filter = ""] = process.argv.slice(2);
const before = await import(pathToFileURL(beforePath));
const after = await import(pathToFileURL(afterPath));
const theme = {
  name: "performance-theme",
  settings: [{ settings: { foreground: "#111111", background: "#ffffff" } }],
};
const settings = { theme, assets: { remote: false } };
const beforeCompiler = new before.JsxCompiler(settings);
const afterCompiler = new after.JsxCompiler(settings);
const source =
  "# Performance guide\n\n" +
  '"Hello" :smile: #521 [keep](/keep) ![alt](/image) café 😄.\n\n'.repeat(256);
const small = "# Guide\n\nA paragraph with [link](/guide) and **strong** text.\n";
const passes = [
  { kind: "emojiShortcodes" },
  { kind: "githubReferences", repository: "sebastian-software/ferromark" },
];
const options = { passes };
const beforePrepared = beforeCompiler.prepare(source);
const afterPrepared = afterCompiler.prepare(source);
const beforeTransformed = beforeCompiler.prepare(source, options);
const afterTransformed = afterCompiler.prepare(source, options);
const beforeSmall = beforeCompiler.prepare(small);
const afterSmall = afterCompiler.prepare(small);
assert.equal(before.toHtml(source), after.toHtml(source));
assert.deepEqual(before.compileJsx(source), after.compileJsx(source));
assert.deepEqual(beforeSmall.render(), afterSmall.render());
for (const opts of [undefined, options]) {
  assert.deepEqual(
    beforeCompiler.prepare(source, opts).render(),
    afterCompiler.prepare(source, opts).render(),
  );
}
let sink = 0;
const cases = [
  ["compile_html", () => before.toHtml(source).length, () => after.toHtml(source).length],
  [
    "compile_jsx",
    () => before.compileJsx(source).body.length,
    () => after.compileJsx(source).body.length,
  ],
  [
    "prepare_default",
    () => beforeCompiler.prepare(source).metadata.outline.length,
    () => afterCompiler.prepare(source).metadata.outline.length,
  ],
  [
    "prepare_with_passes",
    () => beforeCompiler.prepare(source, options).metadata.outline.length,
    () => afterCompiler.prepare(source, options).metadata.outline.length,
  ],
  [
    "prepare_small",
    () => beforeCompiler.prepare(small).metadata.outline.length,
    () => afterCompiler.prepare(small).metadata.outline.length,
  ],
  [
    "render_default",
    () => beforePrepared.render().body.length,
    () => afterPrepared.render().body.length,
  ],
  [
    "render_with_passes",
    () => beforeTransformed.render().body.length,
    () => afterTransformed.render().body.length,
  ],
  ["render_small", () => beforeSmall.render().body.length, () => afterSmall.render().body.length],
];
function sample(run, milliseconds) {
  globalThis.gc?.();
  const start = performance.now();
  let iterations = 0;
  do {
    sink += run();
    iterations += 1;
  } while (performance.now() - start < milliseconds);
  return { iterations, nanoseconds: ((performance.now() - start) * 1e6) / iterations };
}
const results = [];
for (const [name, left, right] of cases) {
  if (filter && !name.includes(filter)) continue;
  sample(left, 200);
  sample(right, 200);
  const pairs = [];
  for (let index = 0; index < 16; index += 1) {
    const order =
      index % 2 === 0
        ? [
            ["before", left],
            ["after", right],
          ]
        : [
            ["after", right],
            ["before", left],
          ];
    const pair = {};
    for (const [key, run] of order) pair[key] = sample(run, 100);
    pairs.push(pair);
  }
  results.push({ name, pairs });
}
console.log(
  JSON.stringify({
    sourceBytes: Buffer.byteLength(source),
    smallBytes: Buffer.byteLength(small),
    node: process.version,
    platform: process.platform,
    arch: process.arch,
    sink,
    results,
  }),
);
