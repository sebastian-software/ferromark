import assert from "node:assert/strict";
import { createRequire } from "node:module";

const addon = createRequire(import.meta.url)(process.argv[2]);
const combined = process.argv[3] === "combined";
assert.deepEqual(addon.availableCodePlugins(), combined ? ["ferriki", "ferrigura"] : ["ferrigura"]);
if (!combined) assert.equal(addon.JsxCompiler, undefined);
const language = {
  name: "native-fixture",
  scopeName: "source.native-fixture",
  patterns: [{ match: "\\bconst\\b", name: "keyword" }],
};
const theme = {
  name: "native-theme",
  settings: [
    { settings: { foreground: "#101010", background: "#ffffff" } },
    { scope: "keyword", settings: { foreground: "#bb1100" } },
  ],
};
const plugins = [{ name: "ferrigura", options: { idPrefix: "article", colorScheme: "auto" } }];
if (combined)
  plugins.unshift({
    name: "ferriki",
    options: { theme, languages: [language], assets: { remote: false } },
  });
let compiler = new addon.CodeCompiler(JSON.stringify({ plugins }));
const source =
  '# Demo\n\n> ```merm&#97;id title="Flow"\n> flowchart LR\n> A["Hello {literal} & world"] --> B[World]\n> ```\n\n```native-fixture\nconst x = 1;\n```\n\n<script>unsafe</script>';
const result = compiler.transform(source);
assert.equal(result.headings[0].text, "Demo");
assert.match(result.html, /<blockquote>\n<svg /);
assert.match(result.html, /id="article-1"/);
assert.match(result.html, /prefers-color-scheme/);
if (combined) assert.match(result.html, /#bb1100/i);
assert.doesNotMatch(result.html, /<script>/);
const jsx = compiler.compile(source);
assert.match(jsx.body, /id=\{"article-2"\}/);
assert.match(jsx.body, /className=/);
assert.match(jsx.body, /style=\{\{/);
assert.match(jsx.body, /\{literal\}/);
assert.doesNotMatch(jsx.body, /dangerouslySetInnerHTML/);
assert.equal(jsx.codeBlocks.length, 2);
assert.match(
  compiler.compile('```mermaid\npie\n"A" : 1\n```', undefined, undefined, () => "<Override />")
    .body,
  /<Override/,
);
const prepared = compiler.prepare(source);
compiler = null;
assert.equal(compiler, null);
globalThis.gc();
assert.match(prepared.render().body, /article-3/);
assert.match(prepared.renderModule().code, /article-4/);

const diagram = '```mermaid\npie\n"A" : 1\n```';
const first = new addon.CodeCompiler('{"plugins":[{"name":"ferrigura"}]}');
const second = new addon.CodeCompiler('{"plugins":[{"name":"ferrigura"}]}');
const firstId = first.toHtml(diagram).match(/<svg[^>]* id="([^"]+)"/)[1];
const secondId = second.toHtml(diagram).match(/<svg[^>]* id="([^"]+)"/)[1];
assert.notEqual(firstId, secondId);
const broken = "```mermaid\nnot a valid diagram\n```";
assert.throws(() => first.toHtml(broken), /Ferrigura rendering failed/);
assert.throws(() => first.compile(broken), /Ferrigura rendering failed/);
assert.match(first.toHtml(diagram), /<svg /);
const fallback = new addon.CodeCompiler(
  '{"plugins":[{"name":"ferrigura","options":{"errorPolicy":"code"}}]}',
);
assert.equal(fallback.toHtml(broken), addon.toHtml(broken));
assert.equal(fallback.compile(broken).body, addon.compileJsx(broken).body);
for (const options of [
  { idPrefix: "0bad" },
  { colorScheme: "wrong" },
  { errorPolicy: "wrong" },
  { config: null },
  { unknown: true },
]) {
  assert.throws(
    () => new addon.CodeCompiler(JSON.stringify({ plugins: [{ name: "ferrigura", options }] })),
  );
}
