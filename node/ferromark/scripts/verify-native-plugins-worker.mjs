import assert from "node:assert/strict";
import { createRequire } from "node:module";

const addon = createRequire(import.meta.url)(process.argv[2]);
assert.deepEqual(addon.availableCodePlugins(), ["fixture"]);
assert.equal(addon.JsxCompiler, undefined);
assert.throws(() => new addon.CodeCompiler('{"plugins":[{"name":"ferriki"}]}'), /not available/);
const plain = new addon.CodeCompiler("{}");
assert.equal(
  plain.toHtml("```native-plugin\n<safe>\n```"),
  addon.toHtml("```native-plugin\n<safe>\n```"),
);
let compiler = new addon.CodeCompiler('{"plugins":[{"name":"fixture"},{"name":"fixture"}]}');
const source =
  '> ```native-plug&#105;n title="Test"\n> <literal> {code}\n> ```\n\n```native-omit\nomitted\n```\n\n```unknown\n<safe>\n```';
const html = compiler.toHtml(source);
assert.match(html, /<blockquote>\n<figure data-native="1">&lt;literal&gt; \{code\}/);
assert.doesNotMatch(html, /omitted|data-native="2"/);
assert.match(html, /&lt;safe&gt;/);
const jsx = compiler.compile(source);
assert.match(jsx.body, /<figure data-native="2">\{"<literal> \{code\}\\n"\}/);
assert.doesNotMatch(jsx.body, /omitted/);
assert.equal(jsx.codeBlocks.length, 3);
assert.match(
  compiler.compile("```native-plugin\nx\n```", undefined, undefined, () => "<Override />").body,
  /<Override/,
);
assert.throws(
  () =>
    compiler.compile("```unknown\nx\n```", undefined, undefined, () => compiler.toHtml("nested")),
  /recursively/,
);
const prepared = compiler.prepare(source);
compiler = null;
globalThis.gc();
assert.equal(prepared.metadata.codeBlocks.length, 3);
assert.match(prepared.render().body, /data-native="3"/);
assert.match(prepared.renderModule().code, /data-native="4"/);
