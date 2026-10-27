import assert from "node:assert/strict";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";

import {
  availableCodePlugins,
  CodeCompiler,
  compileJsx,
  JsxCompiler,
  toHtml,
  transform,
} from "../index.mjs";

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

test("the native compiler initializes no plugins by default and preserves plain output", () => {
  assert.deepEqual(availableCodePlugins(), ["ferriki"]);
  const compiler = new CodeCompiler();
  const source =
    '# Demo\n\n<script>unsafe</script>\n\n```unknown title="Test"\n<literal> {code}\n```';
  assert.equal(compiler.toHtml(source), toHtml(source));
  assert.deepEqual(compiler.transform(Buffer.from(source)), transform(source));
  assert.deepEqual(compiler.compile(source), compileJsx(source));
  const prepared = compiler.prepare(source);
  assert.deepEqual(prepared.render(), compiler.compile(source));
  assert.match(prepared.render({ codeComponents: { unknown: "CodeView" } }).body, /<CodeView/);
  assert.match(compiler.compile(source, { output: "module" }).code, /MDXContent/);
});

test("Ferriki is selected through the native factory contract for HTML and JSX", () => {
  const compiler = new CodeCompiler({
    plugins: [{ name: "ferriki", options: ferriki }],
    lineNumbers: true,
  });
  const legacy = new JsxCompiler({ ...ferriki, lineNumbers: true });
  const source = '```plugin-language title="Example" {1}\nconst x = "<safe>";\n```';
  assert.deepEqual(compiler.compile(source), legacy.compile(source));
  assert.deepEqual(
    compiler.renderCodeBlock({ code: "const x", language: "plugin-language" }),
    legacy.renderCodeBlock({ code: "const x", language: "plugin-language" }),
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

test("HTML-only plugin rendering preserves transform output without collecting its metadata", () => {
  const compiler = new CodeCompiler();
  const source =
    '---\ntitle: Example\n---\n\n# Same\n\nA "quote" and a footnote[^n].\n\n# Same\n\n[^n]: Footnote text.\n';
  const options = {
    frontMatter: true,
    footnotes: true,
    headingIdPrefix: "page-",
    headingOffset: 1,
    typography: { language: "en" },
  };
  const expected = transform(source, options);
  assert.equal(compiler.toHtml(source, options), expected.html);
  assert.deepEqual(compiler.transform(source, options), expected);
  assert.equal(expected.headings.length, 2);
  assert.match(expected.frontMatter, /title: Example/);
  assert.match(expected.html, /page-same-1/);
});

test("native configuration rejects unavailable plugins and invalid settings", () => {
  assert.throws(
    () => new CodeCompiler({ plugins: [{ name: "not-linked" }] }),
    /not available in this addon build/,
  );
  assert.throws(() => new CodeCompiler({ plugins: {} }), /array/);
  assert.throws(() => new CodeCompiler({ plugins: [{}] }), /plugin.name/);
  assert.throws(
    () => new CodeCompiler({ plugins: [{ name: "ferriki", extra: true }] }),
    /Unknown plugin option/,
  );
  assert.throws(
    () => new CodeCompiler({ plugins: [{ name: "ferriki", options: null }] }),
    /object/,
  );
  assert.throws(() => new CodeCompiler({ lineNumbers: "yes" }), /boolean/);
  assert.throws(
    () => new CodeCompiler({ lineNumbers: "yes", plugins: [{ name: "not-linked" }] }),
    /boolean/,
  );
  assert.throws(() => new CodeCompiler({ surprise: true }), /Unknown code compiler option/);
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
