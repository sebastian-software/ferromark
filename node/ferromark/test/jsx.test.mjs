import assert from "node:assert/strict";
import { test } from "node:test";

import { compileJsx, toHtml } from "../index.mjs";

function failingCodeRenderer() {
  throw new Error("hook failed");
}

function asynchronousCodeRenderer() {
  return Promise.resolve("invalid");
}

test("JSX compilation returns a framework-neutral body and matching heading metadata", () => {
  const source = "---\ntitle: Native\n---\n\n# What's included?\n\nText {literal}.";
  const result = compileJsx(source, {
    frontMatter: true,
    componentPrefix: "_components",
  });
  assert.match(result.body, /<_components.h1 id=\{"what-s-included"\}/);
  assert.deepEqual(
    result.headings.map(({ level, id, text }) => ({ level, id, text })),
    [{ level: 1, id: "what-s-included", text: "What's included?" }],
  );
  assert.equal(result.frontMatter, "title: Native\n");
  assert.equal(result.frontMatterKind, "yaml");
  assert.equal(result.headings[0].start, source.indexOf("#"));
  assert.equal(
    Buffer.from(source)
      .subarray(result.frontMatterSpan.start, result.frontMatterSpan.end)
      .toString(),
    "---\ntitle: Native\n---\n",
  );
  assert.ok(result.elements.includes("p"));
  assert.deepEqual(result.components, []);
  assert.ok(!result.body.includes("react"));
  assert.ok(result.mappings.length > 0);
});

test("MDX preserves ESM, JSX bindings, JavaScript grammar and UTF-8 byte spans", () => {
  const source =
    // eslint-disable-next-line no-template-curly-in-string -- authored MDX fixture contains a JavaScript template
    'export const label = "日本語";\n\n# <Badge>Visible</Badge>\n\n<Panel {...props}>{/}/.test(label) ? `a${label}` : null}</Panel>';
  const result = compileJsx(new TextEncoder().encode(source), { format: "mdx" });
  assert.equal(result.esm[0].value.trim(), 'export const label = "日本語";');
  assert.equal(
    Buffer.from(source).subarray(result.esm[0].start, result.esm[0].end).toString(),
    result.esm[0].value,
  );
  assert.deepEqual(result.components, ["Badge", "Panel"]);
  assert.equal(result.headings[0].text, "Visible");
  assert.equal(result.headings[0].id, "visible");
  assert.match(result.body, /<Panel \{\.\.\.props\}>/);
  assert.match(result.body, /\/\}\/.test\(label\)/);
});

test("native passes affect both the JSX body and its outline", () => {
  const result = compileJsx("# Hello :smile:\n\nSee #7", {
    passes: [{ kind: "emojiShortcodes" }, { kind: "githubReferences", repository: "acme/docs" }],
  });
  assert.match(result.headings[0].text, /😄/);
  assert.match(result.body, /https:\/\/github.com\/acme\/docs\/issues\/7/);
});

test("code hooks receive unmodified source, propagate failures and can select default rendering", () => {
  const source = '```ts title="Example"\nconst value = "<safe>";\n```';
  const seen = [];
  const result = compileJsx(source, undefined, (code, language, meta) => {
    seen.push([code, language, meta]);
    return '<CodeView code={"replacement"} />';
  });
  assert.equal(seen.length, 1);
  assert.deepEqual(seen[0], ['const value = "<safe>";\n', "ts", 'title="Example"']);
  assert.equal(result.codeBlocks[0].code, seen[0][0]);
  assert.match(result.body, /CodeView/);
  assert.match(compileJsx(source, undefined, () => null).body, /<pre(?:\s|>)/);
  assert.throws(() => compileJsx(source, undefined, failingCodeRenderer), /hook failed/);
  assert.throws(() => compileJsx(source, undefined, asynchronousCodeRenderer), /String|string/);
});

test("invalid JSX options and MDX fail without changing HTML rendering", () => {
  assert.throws(() => compileJsx("text", { renderPolicy: "trusted" }), /unknown JSX option/);
  assert.throws(() => compileJsx("text", { format: "tsx" }), /format/);
  assert.throws(() => compileJsx("text", { componentPrefix: "bad;code" }), /identifiers/);
  assert.throws(() => compileJsx("{const =}", { format: "mdx" }), /invalid MDX/);
  assert.throws(
    () => compileJsx("> export const value = 1;\n", { format: "mdx" }),
    /document top level/,
  );
  assert.equal(toHtml("# Preserved"), '<h1 id="preserved">Preserved</h1>\n');
});
