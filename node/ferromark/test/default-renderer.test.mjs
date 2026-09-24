// One-shot calls without options render with a renderer each thread keeps
// (`node/native/src/default_renderer.rs`). These tests hold every such call to
// the output of a renderer built for that document alone, whatever the thread
// rendered before, with or without options, and after failed calls.

import assert from "node:assert/strict";
import { once } from "node:events";
import { readFileSync } from "node:fs";
import test from "node:test";
import { Worker } from "node:worker_threads";
import { gunzipSync } from "node:zlib";

import {
  Renderer,
  toHtml,
  toHtmlBuffer,
  toHtmlWithHighlighter,
  transformWithHighlighter,
} from "../index.mjs";

const repoRoot = new URL("../../../", import.meta.url);

/** The 57 documents of the frozen broad benchmark corpus. */
function broadCorpus() {
  const corpus = new URL(
    "docs/reports/2026-09-14-optimization-rounds/broad-corpus.json.gz",
    repoRoot,
  );
  const { cases } = JSON.parse(gunzipSync(readFileSync(corpus)).toString("utf8"));
  return cases.map((entry) => entry.input);
}

/**
 * The Markdown of every example in a vendored specification: the lines
 * between an example fence and its `.` line, with `→` standing for a tab.
 * @param name File name under `tests/spec_fixtures/`.
 */
function specExamples(name) {
  const text = readFileSync(new URL(`tests/spec_fixtures/${name}`, repoRoot), "utf8");
  const fence = "`".repeat(32);
  return text
    .split(`${fence} example`)
    .slice(1)
    .map((block) => {
      const start = block.indexOf("\n") + 1;
      const markdown = block.slice(start, block.indexOf("\n.\n", start - 1) + 1);
      return markdown.replaceAll("→", "\t");
    });
}

// Documents that leave state behind in a renderer, or cross its limits: more
// Markdown than the kept renderer takes (64 KiB), and reference links that
// expand a few kilobytes into more HTML than it keeps (1 MiB).
const stateful = [
  "# Same\n\n# Same\n\n## Same-1\n\n# Same {#same}\n",
  "[a] and [b][]\n\n[a]: /first\n[b]: /second 'title'\n",
  "[a] and [b][] without definitions\n",
  "> [!NOTE]\n> A callout with a [link](https://example.org).\n",
  "| a | b |\n| - | - |\n| 1 | 2 |\n\n- [x] done\n- [ ] open\n\n~~gone~~\n",
  "<script>alert(1)</script>\n\n[x](javascript:alert(1))\n",
  "```js\nconst x = 1;\n```\n\n    indented\n",
  "",
  `# Long\n\n${"A paragraph of *prose* with a [link](/u).\n\n".repeat(2000)}`,
  `[a]: /${"x".repeat(2000)}\n\n${"[a] ".repeat(600)}\n`,
];

const documents = [
  ...broadCorpus(),
  ...specExamples("commonmark-0.31.2-spec.txt"),
  ...specExamples("gfm-extensions-spec.txt"),
  ...stateful,
];

/** HTML from a renderer built for this one document. */
const fresh = (markdown, options) => new Renderer(options).toHtml(markdown);
const expected = documents.map((markdown) => fresh(markdown));

// Every public way to reach the kept renderer.
const defaultEntries = [
  ["toHtml", (markdown) => toHtml(markdown)],
  ["toHtml with null", (markdown) => toHtml(markdown, null)],
  ["toHtml with {}", (markdown) => toHtml(markdown, {})],
  ["toHtmlBuffer", (markdown) => toHtmlBuffer(markdown).toString("utf8")],
  ["toHtmlBuffer with {}", (markdown) => toHtmlBuffer(markdown, {}).toString("utf8")],
];

let step = 0;

/**
 * Renders document `index` through the next default entry and compares it
 * with a fresh renderer's HTML.
 * @param index Index into `documents`.
 */
function assertDefault(index) {
  const [entry, render] = defaultEntries[step++ % defaultEntries.length];
  assert.ok(render(documents[index]) === expected[index], `${entry}: document ${index}`);
}

test("the spec fixtures yield every example", () => {
  assert.equal(specExamples("commonmark-0.31.2-spec.txt").length, 652);
  assert.equal(specExamples("gfm-extensions-spec.txt").length, 24);
  assert.equal(documents.length, 57 + 652 + 24 + stateful.length);
});

/** Renders every document once, in order, through the default entries. */
function assertEveryDefault() {
  for (const index of documents.keys()) assertDefault(index);
}

test("renders every document like a fresh renderer after any other document", () => {
  const count = documents.length;
  for (const index of documents.keys()) {
    // A, then B, then A again, for a neighbor and for a distant document.
    for (const other of [(index + 1) % count, Math.floor(index * 7 + count / 2) % count]) {
      assertDefault(index);
      assertDefault(other);
      assertDefault(index);
    }
  }
  for (let index = count - 1; index >= 0; index--) assertDefault(index);
});

test("alternates with calls that pass options", () => {
  const variants = [
    { renderPolicy: "trusted" },
    { headingIdPrefix: "doc-" },
    { headingOffset: 1 },
    { footnotes: true, highlight: true, superscript: true },
    { headingIds: false },
    { linkBasePath: "/docs/" },
    { renderPolicy: "untrusted" },
    { callouts: false, tables: false },
  ];
  for (const [index, markdown] of documents.entries()) {
    const options = variants[index % variants.length];
    const html = fresh(markdown, options);
    assertDefault(index);
    assert.ok(toHtml(markdown, options) === html, `options: document ${index}`);
    assert.ok(
      toHtmlBuffer(markdown, options).toString("utf8") === html,
      `buffer options: document ${index}`,
    );
    assertDefault(index);
  }
});

const nested = [
  "# Outer\n\n```md\n# Inner\n\n# Inner\n\n[a]\n\n[a]: /inner\n```\n\n# Outer\n\n[a]\n",
  `\`\`\`md\n${stateful.at(-2)}\`\`\`\n\n\`\`\`md\n${stateful.at(-1)}\`\`\`\n`,
];

test("a highlighter may render Markdown with toHtml from its callback", () => {
  const rendered = [];
  const highlighter = {
    codeToHtml(code) {
      const html = toHtml(code);
      rendered.push([code, html]);
      return `<div>${html}</div>`;
    },
  };
  const oracle = { codeToHtml: (code) => `<div>${fresh(code)}</div>` };
  const theme = { theme: "dark" };
  for (const markdown of nested) {
    assert.equal(
      toHtmlWithHighlighter(markdown, highlighter, theme),
      toHtmlWithHighlighter(markdown, oracle, theme),
    );
    assert.deepEqual(
      transformWithHighlighter(markdown, highlighter, theme),
      transformWithHighlighter(markdown, oracle, theme),
    );
  }
  // One code block, then two, each rendered by both entries.
  assert.equal(rendered.length, 6);
  for (const [code, html] of rendered) assert.ok(html === fresh(code), code.slice(0, 40));
  assertEveryDefault();
});

test("a failed call leaves later calls unaffected", () => {
  const failure = new Error("highlighter failed after rendering");
  const highlighter = {
    codeToHtml(code) {
      toHtml(code);
      throw failure;
    },
  };
  const rethrow = {
    theme: "dark",
    onHighlightError(error) {
      throw error;
    },
  };
  const deep = `${"> ".repeat(150)}deep`;
  const calls = [
    () => toHtmlWithHighlighter(nested[0], highlighter, rethrow),
    () => transformWithHighlighter(nested[0], highlighter, rethrow),
    () => toHtml(deep),
    () => toHtmlBuffer(deep),
    () => toHtml(`${"*".repeat(20_000)}a${"*".repeat(20_000)}`),
    () => toHtml(123),
  ];
  for (const [index, call] of calls.entries()) {
    assert.throws(call);
    assertDefault(documents.length - 1 - index);
    assertDefault(index);
  }
  assertEveryDefault();
});

test("worker threads render with renderers of their own", async () => {
  const facade = new URL("../index.mjs", import.meta.url).href;
  const sample = documents.filter((_, index) => index % 5 === 0);
  const source = `
    const { parentPort, workerData } = require("node:worker_threads");
    import(workerData.facade).then(({ Renderer, toHtml }) => {
      let mismatches = 0;
      for (let round = 0; round < 3; round++) {
        for (const markdown of workerData.sample) {
          if (toHtml(markdown) !== new Renderer().toHtml(markdown)) mismatches++;
        }
      }
      parentPort.postMessage(mismatches);
    });
  `;
  const run = async () => {
    const worker = new Worker(source, { eval: true, workerData: { facade, sample } });
    const [mismatches] = await once(worker, "message");
    return mismatches;
  };
  const results = Promise.all([run(), run(), run(), run()]);
  // The main thread keeps rendering while the workers do.
  assertEveryDefault();
  assert.deepEqual(await results, [0, 0, 0, 0]);

  // A worker terminated in the middle of rendering exits cleanly.
  const busy = new Worker(
    `
      const { parentPort, workerData } = require("node:worker_threads");
      import(workerData.facade).then(({ toHtml }) => {
        parentPort.postMessage("started");
        for (;;) for (const markdown of workerData.sample) toHtml(markdown);
      });
    `,
    { eval: true, workerData: { facade, sample } },
  );
  await once(busy, "message");
  assert.equal(await busy.terminate(), 1);
  assertEveryDefault();
});
