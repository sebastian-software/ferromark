import assert from "node:assert/strict";
import { test } from "node:test";

import { compileJsx, prepareDocument } from "../index.mjs";

const source = [
  "---",
  "title: Café",
  "---",
  "",
  "import {",
  "  Panel",
  '} from "./does-not-exist.js";',
  "",
  "# Café :smile:",
  "",
  "# Café :smile:",
  "",
  '<Panel href="/guide" src="/asset.svg" value={(() => { throw new Error("evaluated"); })()}>',
  "",
  "## Child",
  "",
  "Static **Unicode 🧪** [target][ref] ![alt](./image.svg).",
  "",
  'left{(() => { throw new Error("evaluated"); })()}right',
  "",
  "````unknown-language",
  'import { Example } from "example";',
  '<Example href="/ignored">{expression}</Example>',
  "````",
  "",
  "</Panel>",
  "",
  "[ref]: /resolved",
  "",
].join("\n");
const preparation = { format: "mdx", frontMatter: true, passes: [{ kind: "emojiShortcodes" }] };
const rendering = { omitTitleHeading: "Café 😄", headingIdPrefix: "docs-", headingOffset: 1 };
const projectionOptions = { frontMatter: "omit", omitTitleHeading: "Café 😄" };

function checkReferences(prepared) {
  const references = prepared.collectReferences();
  assert.deepEqual(
    references.map(({ destination }) => destination),
    ["/guide", "/asset.svg", "/resolved", "./image.svg"],
  );
  assert.ok(!references.some(({ destination }) => destination === "/ignored"));
  return references;
}

function checkText(prepared, headings) {
  const text = prepared.extractText({ ...rendering, includeBlockCode: false });
  assert.deepEqual(
    text.sections.map(({ id }) => id),
    headings.map(({ id }) => id),
  );
  assert.ok(text.text.includes("Unicode 🧪"));
  assert.ok(!text.text.includes("unknown-language"));
  assert.ok(!text.text.includes("/ignored"));
  return text;
}

function checkProjection(prepared) {
  const markdown = prepared.projectMarkdown(projectionOptions);
  assert.ok(markdown.markdown.includes("[target][ref]"));
  assert.ok(markdown.markdown.includes("[ref]: /resolved"));
  assert.ok(markdown.markdown.includes('<Example href="/ignored">{expression}</Example>'));
  assert.ok(markdown.markdown.includes("Café :smile:"));
  assert.ok(!markdown.markdown.includes("does-not-exist.js"));
  for (const { start, end } of markdown.diagnostics) {
    assert.ok(Number.isInteger(start) && Number.isInteger(end) && start <= end);
    assert.ok(end <= Buffer.byteLength(source));
  }
  return markdown;
}

function checkRepeatability(prepared, before, { references, text, markdown }) {
  assert.deepEqual(prepared.render(rendering), before);
  assert.deepEqual(prepared.collectReferences(), references);
  assert.deepEqual(prepared.extractText({ ...rendering, includeBlockCode: false }), text);
  assert.deepEqual(prepared.projectMarkdown(projectionOptions), markdown);
}

test("prepared analysis methods share owned Unicode source and preserve later rendering", () => {
  const input = Buffer.from(source);
  const prepared = prepareDocument(input, { ...preparation, trackReferences: true });
  input.fill(0);
  const before = prepared.render(rendering);
  assert.deepEqual(before, compileJsx(source, { ...preparation, ...rendering }));
  const references = checkReferences(prepared);
  const text = checkText(prepared, before.headings);
  const markdown = checkProjection(prepared);
  checkRepeatability(prepared, before, { references, text, markdown });
});

test("text and Markdown analysis need no reference snapshot after native passes", () => {
  const untracked = prepareDocument(source, preparation);
  const tracked = prepareDocument(source, { ...preparation, trackReferences: true });
  assert.throws(() => untracked.collectReferences(), /trackReferences: true/);
  assert.deepEqual(untracked.extractText(rendering), tracked.extractText(rendering));
  assert.deepEqual(
    untracked.projectMarkdown(projectionOptions),
    tracked.projectMarkdown(projectionOptions),
  );
  assert.equal(untracked.extractText, tracked.extractText);
  assert.equal(untracked.projectMarkdown, tracked.projectMarkdown);
});
