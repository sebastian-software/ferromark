import assert from "node:assert/strict";
import { test } from "node:test";

import { JsxCompiler } from "../index.mjs";

function prepare(source, options = {}) {
  return new JsxCompiler({
    theme: { name: "projection-fixture", settings: [{ settings: { foreground: "#111111" } }] },
    assets: { remote: false },
  }).prepare(source, { format: "mdx", frontMatter: true, ...options });
}

const projectionSource = [
  "---",
  "title: Guide",
  "---",
  "",
  "import {",
  "  Box",
  '} from "./components.js";',
  "",
  "export const metadata = { title: 'Guide' };",
  "",
  '<Box title="a > b" options={{ nested: true }}>',
  "",
  "# Guide",
  "",
  "<>**Static** [link][ref] {dynamic}<Badge /></>",
  "",
  "[ref]: /guide",
  "",
  "````mdx",
  "```nested",
  "import x from 'x';",
  "<Nested>{code}</Nested>",
  "````",
  "",
  "~~~~~text",
  "<Example /> {example}",
  "~~~~~",
  "",
  "</Box>",
  "",
].join("\n");

function assertProjectionContent(result) {
  assert.ok(result.markdown.startsWith("---\ntitle: Guide\n---\n"));
  assert.ok(result.markdown.includes("# Guide\n"));
  assert.ok(result.markdown.includes("**Static** [link][ref]"));
  assert.ok(result.markdown.includes("[ref]: /guide\n"));
  assert.ok(
    result.markdown.includes(
      "````mdx\n```nested\nimport x from 'x';\n<Nested>{code}</Nested>\n````",
    ),
  );
  assert.ok(result.markdown.includes("~~~~~text\n<Example /> {example}\n~~~~~"));
  assert.ok(!result.markdown.includes("metadata ="));
  assert.ok(!result.markdown.includes("{dynamic}"));
}

test("prepared Markdown projection retains original Markdown and code without rendering", () => {
  const prepared = prepare(projectionSource);
  const before = prepared.render({}, () => "<pre>example</pre>");
  const result = prepared.projectMarkdown();
  assertProjectionContent(result);
  assert.deepEqual(prepared.projectMarkdown(), result);
  assert.deepEqual(
    prepared.render({}, () => "<pre>example</pre>"),
    before,
  );
  assert.ok(Object.isFrozen(result));
  assert.ok(Object.isFrozen(result.diagnostics));
  assert.ok(result.diagnostics.every(Object.isFrozen));
});

test("projection diagnostics use original Unicode byte spans and preserve token boundaries", () => {
  const source =
    "日本<A>語</A><B>text</B>fin{value}ish\n\n*left*{value}*right*\n\n`left`{value}`right`\n";
  const result = prepare(source).projectMarkdown();
  assert.equal(result.markdown, "日本 語 text fin ish\n\n*left* *right*\n\n`left` `right`\n");
  const bytes = Buffer.from(source);
  const expression = result.diagnostics.find(({ code }) => code === "expression-omitted");
  assert.equal(bytes.subarray(expression.start, expression.end).toString(), "{value}");
  assert.equal(expression.start, Buffer.byteLength(source.slice(0, source.indexOf("{value}"))));
});

test("front matter and only the matching first top-level H1 can be omitted explicitly", () => {
  const source = "---\ntitle: Guide\n---\n\n# <Badge>Guide</Badge>\n\n# Guide\n";
  const prepared = prepare(source);
  const result = prepared.projectMarkdown({ frontMatter: "omit", omitTitleHeading: " Guide " });
  assert.ok(!result.markdown.includes("title: Guide"));
  assert.ok(!result.markdown.includes("Badge"));
  assert.equal(result.markdown.match(/# Guide/g).length, 1);
  assert.equal(
    Buffer.from(source)
      .subarray(result.omittedTitleHeadingSpan.start, result.omittedTitleHeadingSpan.end)
      .toString()
      .trim(),
    "# <Badge>Guide</Badge>",
  );
  assert.ok(Object.isFrozen(result.omittedTitleHeadingSpan));
  assert.ok(prepared.projectMarkdown().markdown.includes("title: Guide"));
  assert.equal(
    prepared.projectMarkdown({ omitTitleHeading: "Different" }).omittedTitleHeadingSpan,
    undefined,
  );
});

test("raw text elements are omitted and native passes do not rewrite projected source", () => {
  const source =
    "<script>const hidden = 1;</script>\n\n<style>{'body { color: red; }'}</style>\n\n<title>Internal</title>\n\n<textarea>Editable</textarea>\n\n<div>Static text</div>\n\nReadable -- text...\n";
  const prepared = prepare(source, { passes: [{ kind: "typography", language: "en" }] });
  const result = prepared.projectMarkdown();
  assert.ok(result.markdown.includes("Readable -- text..."));
  assert.ok(prepared.render().body.includes("Readable\u00a0— text…"));
  assert.ok(!result.markdown.includes("hidden"));
  assert.equal(
    result.diagnostics.filter(({ code }) => code === "raw-text-element-omitted").length,
    4,
  );
});

test("multiline component attributes retain Markdown container prefixes", () => {
  const source =
    '> <Box\n> title="quoted >"\n> >\n> # Quoted\n> </Box>\n\n- <Box\n    title="listed"\n  >\n  ## Listed\n  </Box>\n';
  const result = prepare(source).projectMarkdown();
  assert.ok(result.markdown.includes("> # Quoted\n"));
  assert.ok(result.markdown.includes("  ## Listed\n"));
  assert.ok(!result.markdown.includes("Box"));
  assert.ok(!result.diagnostics.some(({ code }) => code === "unsupported-jsx-omitted"));
});

test("projection rejects unknown choices and invalid option types", () => {
  const prepared = prepare("# Guide\n");
  for (const options of [
    null,
    [],
    1,
    { frontMatter: true },
    { frontMatter: "drop" },
    { omitTitleHeading: true },
    { format: "mdx" },
    { [Symbol("unknown")]: true },
  ]) {
    assert.throws(() => prepared.projectMarkdown(options), TypeError);
  }
});

test("Markdown grammar also omits inline raw text element bodies", () => {
  const source =
    "before<script>const hidden = 1;</script>after\n\nbefore<style>secret</style>after\n";
  const result = prepare(source, { format: "md" }).projectMarkdown();
  assert.equal(result.markdown, "before after\n\nbefore after\n");
  assert.equal(
    result.diagnostics.filter(({ code }) => code === "raw-text-element-omitted").length,
    2,
  );
});
