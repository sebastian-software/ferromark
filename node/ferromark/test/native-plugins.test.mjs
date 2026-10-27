import assert from "node:assert/strict";
import test from "node:test";

import { availableCodePlugins, CodeCompiler, compileJsx, toHtml, transform } from "../index.mjs";

test("the native compiler initializes no plugins by default and preserves plain output", () => {
  assert.deepEqual(availableCodePlugins(), []);
  for (const name of ["ferriki", "ferrigura"]) {
    assert.throws(() => new CodeCompiler({ plugins: [{ name }] }), /not available/);
  }
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

test("the engine-free compiler renders standalone code and keeps prepared source alive", () => {
  const compiler = new CodeCompiler({ lineNumbers: true });
  const block = compiler.renderCodeBlock({
    code: "<safe> {literal}",
    language: "unknown",
    meta: 'title="plain.txt"',
  });
  assert.equal(block.lineNumbers, true);
  assert.equal(block.title, "plain.txt");
  assert.ok(block.jsx.includes('{"<safe> {literal}"}'));
  const source = Buffer.from("# Original\n\n```unknown\nplain\n```");
  const prepared = compiler.prepare(source);
  source.fill(0);
  assert.match(prepared.render().body, /Original/);
  assert.match(prepared.renderModule().code, /Original/);
  assert.match(prepared.render().body, /data-ln=/);
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
