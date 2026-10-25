import assert from "node:assert/strict";
import test from "node:test";

import {
  compileJsx,
  composeCodeRenderers,
  createHighlighterCodeRenderer,
  toHtml,
  toHtmlWithCodeRenderer,
  toHtmlWithHighlighter,
  transformWithCodeRenderer,
} from "../index.mjs";

async function promiseResult() {
  await Promise.resolve();
  return "<svg/>";
}

function invalidResult() {
  return 42;
}
function decline() {
  return null;
}
function fail() {
  throw new Error("diagram failure");
}

function diagram(code, language, meta) {
  if (language !== "mermaid") return null;
  assert.equal(code, "A --> B\n");
  assert.equal(meta, 'title="Flow"');
  return '<svg data-diagram="yes"></svg>\n';
}

test("HTML code renderers compose with highlighters and preserve document metadata", () => {
  const calls = [];
  const highlighter = {
    codeToHtml(code, options) {
      calls.push([code, options]);
      return "<pre><code>highlighted</code></pre>\n";
    },
  };
  const render = composeCodeRenderers(
    diagram,
    createHighlighterCodeRenderer(highlighter, { theme: "test" }),
  );
  const source =
    '# Demo\n\n> ```merm&#97;id title="Flow"\n> A --> B\n> ```\n\n```rust\nlet x = 1;\n```\n';
  const result = transformWithCodeRenderer(source, render);
  assert.deepEqual(result.headings, [{ level: 1, id: "demo", text: "Demo" }]);
  assert.match(result.html, /<blockquote>\n<svg data-diagram="yes"><\/svg>\n<\/blockquote>/);
  assert.match(result.html, /<pre><code>highlighted<\/code><\/pre>/);
  assert.deepEqual(calls, [["let x = 1;\n", { lang: "rust", theme: "test" }]]);
  assert.equal(toHtmlWithCodeRenderer(source, render), result.html);
  assert.equal(toHtmlWithCodeRenderer(Buffer.from(source), render), result.html);
  const code = "```rust\nlet x = 1;\n```\n";
  assert.equal(
    toHtmlWithCodeRenderer(code, createHighlighterCodeRenderer(highlighter, { theme: "test" })),
    toHtmlWithHighlighter(code, highlighter, { theme: "test" }),
  );
});

test("declining a block preserves escaped output and the document trust policy", () => {
  const source = "<script>alert(1)</script>\n\n```unknown\n<unsafe>\n```\n\n    indented\n";
  const seen = [];
  const output = toHtmlWithCodeRenderer(source, (code, language) => {
    seen.push([code, language]);
  });
  assert.equal(output, toHtml(source));
  assert.match(output, /&lt;script&gt;/);
  assert.match(output, /&lt;unsafe&gt;/);
  assert.equal(seen.length, 2);
  assert.equal(seen[1][1], null);
});

test("chains stop on empty strings and share the JSX callback contract", () => {
  const render = composeCodeRenderers(
    () => {
      /* An implicit undefined also declines the block. */
    },
    () => "",
    () => {
      throw new Error("must not run");
    },
  );
  const source = "```mermaid\nA --> B\n```\n";
  assert.equal(toHtmlWithCodeRenderer(source, render), "");
  assert.ok(!compileJsx(source, {}, render).body.includes("A --"));
  assert.equal(toHtmlWithCodeRenderer(source, composeCodeRenderers()), toHtml(source));
});

test("callbacks reject invalid results and propagate failures", () => {
  const source = "```mermaid\nA --> B\n```\n";
  assert.throws(() => composeCodeRenderers(null), /synchronous functions/);
  assert.throws(() => toHtmlWithCodeRenderer(source, promiseResult), /synchronously/);
  assert.throws(() => toHtmlWithCodeRenderer(source, invalidResult), /synchronously/);
  assert.throws(() => toHtmlWithCodeRenderer(source, fail), /diagram failure/);
  assert.throws(() => toHtmlWithCodeRenderer(source, decline, { unknown: true }), /unknown option/);
});

test("highlighter adapters retain fallback and optional error observation", () => {
  const failure = new Error("no grammar");
  let seen;
  const render = createHighlighterCodeRenderer(
    {
      codeToHtml() {
        throw failure;
      },
    },
    {
      theme: "test",
      onHighlightError(error, context) {
        seen = [error, context];
      },
    },
  );
  const source = "```rust\n<safe>\n```\n";
  assert.equal(toHtmlWithCodeRenderer(source, render), toHtml(source));
  assert.deepEqual(seen, [failure, { lang: "rust" }]);
});
