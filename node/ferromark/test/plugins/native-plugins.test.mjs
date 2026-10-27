import assert from "node:assert/strict";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";

import { availableCodePlugins, CodeCompiler } from "../../index.mjs";

const language = {
  name: "plugin-language",
  scopeName: "source.plugin-language",
  patterns: [{ match: "\\bconst\\b", name: "keyword" }],
};
const theme = {
  name: "plugin-theme",
  settings: [
    { settings: { foreground: "#101010", background: "#ffffff" } },
    { scope: "keyword", settings: { foreground: "#bb1100" } },
  ],
};
const ferriki = { theme, languages: [language], assets: { remote: false } };

test("the custom addon includes only the requested Ferriki factory", () => {
  assert.deepEqual(availableCodePlugins(), ["ferriki"]);
});

test("Ferriki is selected through the native factory contract for HTML and JSX", () => {
  const compiler = new CodeCompiler({
    plugins: [{ name: "ferriki", options: ferriki }],
    lineNumbers: true,
  });
  const source = '```plugin-language title="Example" {1}\nconst x = "<safe>";\n```';
  assert.match(compiler.compile(source).body, /#bb1100/i);
  assert.match(
    compiler.renderCodeBlock({ code: "const x", language: "plugin-language" }).jsx,
    /#bb1100/i,
  );
  assert.match(compiler.toHtml(source), /#bb1100/i);
  assert.doesNotMatch(compiler.toHtml(source), /<safe>/);
  assert.match(compiler.toHtml(source), /data-line-number="1"/);
  assert.match(compiler.compile(source, {}, () => "<Override />").body, /<Override/);
  assert.match(
    compiler.compile(source, { codeComponents: { "plugin-language": "CodeView" } }).body,
    /<CodeView/,
  );
  assert.match(compiler.prepare(source).render().body, /#bb1100/i);
});

test("native plugin errors propagate and the compiler recovers for later documents", () => {
  const cacheDir = mkdtempSync(path.join(tmpdir(), "ferromark-native-plugin-errors-"));
  try {
    const compiler = new CodeCompiler({
      plugins: [{ name: "ferriki", options: { ...ferriki, assets: { remote: false, cacheDir } } }],
    });
    const broken = "```rust\nfn main() {}\n```";
    assert.throws(() => compiler.toHtml(broken), /Ferriki/);
    assert.throws(() => compiler.compile(broken), /Ferriki/);
    assert.match(compiler.toHtml("```plugin-language\nconst x\n```"), /#bb1100/i);
    assert.match(compiler.compile("```plugin-language\nconst x\n```").body, /#bb1100/i);
    const recursiveRenderer = () => compiler.toHtml("nested");
    assert.throws(
      () => compiler.compile("```unknown\nx\n```", {}, recursiveRenderer),
      /recursively/,
    );
    assert.match(compiler.compile("after").body, /after/);
  } finally {
    rmSync(cacheDir, { recursive: true, force: true });
  }
});
