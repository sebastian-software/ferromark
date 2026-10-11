import assert from "node:assert/strict";
import { test } from "node:test";

import { prepareDocument } from "../index.mjs";

function assertSection(section, source, encoded) {
  assert.equal(
    encoded.subarray(section.textRange.start, section.textRange.end).toString(),
    section.text,
  );
  assert.ok(
    Buffer.from(source)
      .subarray(section.headingSpan.start, section.headingSpan.end)
      .toString()
      .startsWith("#"),
  );
  assert.ok(Object.isFrozen(section));
  assert.ok(Object.isFrozen(section.headingSpan));
  assert.ok(Object.isFrozen(section.sourceSpan));
  assert.ok(Object.isFrozen(section.textRange));
}

test("extractText projects Unicode text, source spans, sections, lists, and tables in bulk", () => {
  const source =
    "Intro 🎉\n\n# Café &amp; 東京\n\n[link](url) ![alt](img) `code`\n\n### Child\n\n- one\n- two\n\n| A | B |\n| - | - |\n| C | D |\n";
  const prepared = prepareDocument(source);
  const extracted = prepared.extractText();
  assert.equal(
    extracted.text,
    "Intro 🎉\n\nCafé & 東京\n\nlink alt code\n\nChild\n\none\ntwo\n\nA\tB\nC\tD",
  );
  assert.equal(extracted.sections[1].parentIndex, 0);
  assert.equal(extracted.sections[0].id, "café-東京");
  const encoded = Buffer.from(extracted.text);
  for (const section of extracted.sections) {
    assertSection(section, source, encoded);
  }
  assert.ok(Object.isFrozen(extracted));
  assert.ok(Object.isFrozen(extracted.sections));
  assert.throws(() => extracted.sections.push(extracted.sections[0]));
  assert.deepEqual(extracted, prepared.extractText());
});

test("extracted anchors follow final JSX body and module planning after native passes", () => {
  const source =
    "# Guide\n\n# :rocket: Guide\n\n## :rocket: Guide\n\n[^x]\n\n# fn-x\n\n# fnref-x\n\n[^x]: Footnote\n";
  const prepared = prepareDocument(source, {
    footnotes: true,
    passes: [{ kind: "emojiShortcodes" }],
  });
  const renderOptions = { omitTitleHeading: "Guide", headingOffset: 1, headingIdPrefix: "docs-" };
  const before = prepared.render(renderOptions);
  const result = prepared.extractText(renderOptions);
  assert.deepEqual(
    result.sections.map(({ heading, level, id, headingSpan }) => ({
      text: heading,
      level,
      id,
      start: headingSpan.start,
      end: headingSpan.end,
    })),
    before.headings,
  );
  assert.deepEqual(prepared.renderModule(renderOptions).headings, before.headings);
  assert.deepEqual(result.omittedTitleHeadingSpan, before.omittedTitleHeadingSpan);
  assert.ok(result.text.includes("🚀 Guide"));
  assert.ok(result.text.includes("Footnote"));
  assert.ok(!result.text.includes("↩"));
  assert.deepEqual(prepared.render(renderOptions), before);
  assert.equal(prepared.extractText({ headingIds: false }).sections[0].id, undefined);
});

test("block code exclusion retains inline code and static MDX children", () => {
  const prepared = prepareDocument(
    "export const runtime = 'secret';\n\n# Guide <Badge>static</Badge>\n\n<Note>\n\nText `inline` left{runtime}right <Runtime />tail.\n\n```text\nblock code\n```\n\n</Note>\n\n<script>hidden</script>\n",
    { format: "mdx" },
  );
  const included = prepared.extractText();
  assert.ok(included.text.includes("block code"));
  const excluded = prepared.extractText({ includeBlockCode: false });
  assert.equal(excluded.text, "Guide static\n\nText inline left right tail.");
  assert.deepEqual(
    excluded.sections.map(({ id }) => id),
    prepared.render().headings.map(({ id }) => id),
  );
  assert.equal(excluded.sections[0].heading, "Guide static");
});

test("extraction options reject callbacks, preparation choices, and invalid values", () => {
  const prepared = prepareDocument("# Guide\n");
  for (const options of [
    { passes: [] },
    { format: "mdx" },
    { includeBlockCode: "false" },
    { headingOffset: 0.5 },
    { headingOffset: Number.NaN },
    { headingOffset: 2_147_483_648 },
    { headingIds: null },
    { omitTitleHeading: 1 },
    { [Symbol("unknown")]: true },
  ]) {
    assert.throws(() => prepared.extractText(options), TypeError);
  }
  assert.throws(() => prepared.extractText([]), /must be an object/);
  assert.throws(() => prepared.extractText({ headingIdPrefix: "bad prefix" }), /prefix/);
  assert.deepEqual(prepared.extractText({ includeBlockCode: undefined }), prepared.extractText());
});

test("asset-free preparation extracts unknown-language code without a highlighter", () => {
  const prepared = prepareDocument(
    "# Offline\n\n```unknown-text-extraction-language\nsource code\n```\n",
  );
  assert.equal(prepared.extractText().text, "Offline\n\nsource code");
  assert.equal(prepared.extractText({ includeBlockCode: false }).text, "Offline");
});
