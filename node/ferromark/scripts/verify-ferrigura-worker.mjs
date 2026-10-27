import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";

const addon = createRequire(import.meta.url)(process.argv[2]);
const combined = process.argv[3] === "combined";
assert.deepEqual(addon.availableCodePlugins(), combined ? ["ferriki", "ferrigura"] : ["ferrigura"]);
assert.equal(addon.JsxCompiler, undefined);
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
assert.match(result.html, /id="article-10"/);
assert.match(result.html, /prefers-color-scheme/);
if (combined) assert.match(result.html, /#bb1100/i);
assert.doesNotMatch(result.html, /<script>/);
const jsx = compiler.compile(source);
assert.match(jsx.body, /id=\{"article-10"\}/);
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
assert.match(prepared.render().body, /article-10/);
assert.match(prepared.renderModule().code, /article-10/);

const diagram = '```mermaid\npie\n"A" : 1\n```';
const first = new addon.CodeCompiler('{"plugins":[{"name":"ferrigura"}]}');
const second = new addon.CodeCompiler('{"plugins":[{"name":"ferrigura"}]}');
const firstId = first.toHtml(diagram).match(/<svg[^>]* id="([^"]+)"/)[1];
const secondId = second.toHtml(diagram).match(/<svg[^>]* id="([^"]+)"/)[1];
assert.equal(firstId, secondId);
assert.equal(first.toHtml(diagram), first.toHtml(diagram));
const broken = "```mermaid\nnot a valid diagram\n```";
assert.equal(first.toHtml(broken), addon.toHtml(broken));
assert.equal(first.compile(broken).body, addon.compileJsx(broken).body);
assert.equal(first.takeDiagnostics().length, 2);
assert.deepEqual(first.takeDiagnostics(), []);
const strict = new addon.CodeCompiler(
  '{"plugins":[{"name":"ferrigura","options":{"errorPolicy":"throw"}}]}',
);
assert.throws(() => strict.toHtml(broken), /Ferrigura rendering failed/);
assert.throws(() => strict.compile(broken), /Ferrigura rendering failed/);
assert.equal(strict.takeDiagnostics().length, 2);
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

const fixtures = JSON.parse(
  readFileSync(
    new URL("../../../plugins/ferrigura/tests/fixtures/diagrams.json", import.meta.url),
    "utf8",
  ),
);
for (const fixture of fixtures) {
  const session = new addon.CodeCompiler(
    '{"plugins":[{"name":"ferrigura","options":{"idPrefix":"fixture"}}]}',
  );
  const markdown = `\`\`\`mermaid\n${fixture.source}\n\`\`\``;
  const html = session.toHtml(markdown);
  assert.equal(createHash("sha256").update(html).digest("hex"), fixture.sha256, fixture.name);
  assert.equal(session.toHtml(markdown), html);
  assert.equal(session.prepare(markdown).render().body, session.compile(markdown).body);
  assert.deepEqual(session.takeDiagnostics(), []);
}
const invalidFence = "```mermaid\nflowchart LR\nA[unterminated\n```";
const unsupportedFence = "```mermaid\nfutureDiagram\nA --> B\n```";
const failed = `é\n\n${invalidFence}\n\n${unsupportedFence}`;
const failedPrepared = first.prepare(Buffer.from(failed));
assert.deepEqual(first.takeDiagnostics(), []);
assert.equal(failedPrepared.render().body, addon.compileJsx(failed).body);
const diagnostics = failedPrepared.takeDiagnostics();
assert.equal(diagnostics.length, 2);
for (const [index, diagnostic] of diagnostics.entries()) {
  assert.equal(diagnostic.plugin, "ferrigura");
  assert.equal(
    Buffer.from(failed).subarray(diagnostic.start, diagnostic.end).toString().trimEnd(),
    [invalidFence, unsupportedFence][index],
  );
}
assert.deepEqual(first.takeDiagnostics(), []);

failedPrepared.renderModule();
assert.equal(failedPrepared.takeDiagnostics().length, 2);
