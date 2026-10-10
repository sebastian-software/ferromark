import assert from "node:assert/strict";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";

import { compileJsx, JsxCompiler } from "../index.mjs";

const language = {
  name: "prepared-fixture",
  aliases: [],
  scopeName: "source.prepared-fixture",
  patterns: [{ match: "\\bconst\\b", name: "keyword.prepared-fixture" }],
};
const theme = {
  name: "prepared-theme",
  settings: [
    { settings: { foreground: "#111111", background: "#ffffff" } },
    { scope: "keyword", settings: { foreground: "#bb1100" } },
  ],
};

function compiler(options = {}) {
  return new JsxCompiler({
    theme,
    languages: [language],
    assets: { remote: false },
    ...options,
  });
}

function assertPreparedMetadata(metadata) {
  assert.equal(metadata.frontMatter, "title: Guide\n");
  assert.deepEqual(
    metadata.esm.map(({ value }) => value.trim()),
    ['export const label = "guide";'],
  );
  assert.equal(metadata.codeBlocks[0].code, "notes\n");
  assert.deepEqual(
    metadata.outline.map(({ id, text }) => ({ id, text })),
    [
      { id: "guide", text: "Guide" },
      { id: "guide-1", text: "Guide" },
    ],
  );
  assert.ok(Object.isFrozen(metadata));
  assert.ok(Object.isFrozen(metadata.outline));
  assert.throws(() => metadata.outline.push({ level: 1, text: "mutated" }));
}

function failingPreparedCode() {
  throw new Error("prepared callback failure");
}

test("prepared body and module output match the one-shot JSX APIs", () => {
  const source =
    '---\ntitle: Guide\n---\n\nimport { Badge } from "./badge.js"\n\n# Guide\n\n## Usage\n\n<Badge />\n';
  const preparationOptions = { format: "mdx", frontMatter: true };
  const bodyOptions = {
    ...preparationOptions,
    componentPrefix: "ui",
    omitTitleHeading: "Guide",
    headingOffset: 1,
    headingIdPrefix: "docs-",
    callouts: false,
  };
  const prepared = compiler().prepare(source, preparationOptions);

  assert.deepEqual(
    prepared.render({
      componentPrefix: bodyOptions.componentPrefix,
      omitTitleHeading: bodyOptions.omitTitleHeading,
      headingOffset: bodyOptions.headingOffset,
      headingIdPrefix: bodyOptions.headingIdPrefix,
      callouts: bodyOptions.callouts,
    }),
    compileJsx(source, bodyOptions),
  );

  const moduleOptions = {
    ...preparationOptions,
    output: "module",
    providerImportSource: "docs/provider",
    filename: "guide.mdx",
    omitTitleHeading: "Guide",
    headingOffset: 1,
    headingIdPrefix: "docs-",
  };
  const modulePrepared = compiler().prepare(source, preparationOptions);
  assert.deepEqual(
    modulePrepared.renderModule({
      providerImportSource: moduleOptions.providerImportSource,
      filename: moduleOptions.filename,
      omitTitleHeading: moduleOptions.omitTitleHeading,
      headingOffset: moduleOptions.headingOffset,
      headingIdPrefix: moduleOptions.headingIdPrefix,
    }),
    compileJsx(source, moduleOptions),
  );
});

test("prepared metadata is a frozen snapshot and final headings follow render choices", () => {
  const source =
    '---\ntitle: Guide\n---\n\nexport const label = "guide";\n\n# Guide\n\n# Guide\n\n```text title="notes.txt"\nnotes\n```\n';
  const prepared = compiler().prepare(source, { format: "mdx", frontMatter: true });
  assertPreparedMetadata(prepared.metadata);
  assert.ok(Object.isFrozen(prepared));

  const omitted = prepared.render({ omitTitleHeading: "Guide" });
  assert.deepEqual(
    omitted.headings.map(({ id }) => id),
    ["guide"],
  );
  assert.ok(omitted.omittedTitleHeadingSpan);
  assert.equal(prepared.render().body, prepared.render().body);
  assert.equal(omitted.body, prepared.render({ omitTitleHeading: "Guide" }).body);
});

test("preparation does not highlight and keeps the caller's source bytes", () => {
  const cacheDir = mkdtempSync(path.join(tmpdir(), "ferromark-prepared-jsx-"));
  try {
    const source = Buffer.from("# Owned\n\n```rust\nfn main() {}\n```\n");
    const prepared = compiler({ assets: { remote: false, cacheDir } }).prepare(source);
    assert.equal(prepared.metadata.codeBlocks[0].code, "fn main() {}\n");
    source.fill(0);
    assert.match(prepared.render({}, () => "<Code />").body, /Owned/);
    assert.throws(() => prepared.render(), /Ferriki JSX highlighting/);
  } finally {
    rmSync(cacheDir, { recursive: true, force: true });
  }
});

test("repeated renders preserve passes, mappings, and UTF-16 source positions", () => {
  const source =
    'export const icon = "🧪";\n\n<Badge title="🧪" value={icon} />\n\n# Hello :smile:\n';
  const preparationOptions = {
    format: "mdx",
    passes: [{ kind: "emojiShortcodes" }],
  };
  const prepared = compiler().prepare(source, preparationOptions);
  const first = prepared.render({ componentPrefix: "_view" });
  const second = prepared.render({ componentPrefix: "_view" });
  assert.deepEqual(first, second);
  assert.equal(prepared.metadata.outline[0].text, "Hello 😄");
  const expressionColumn = source.split("\n")[2].indexOf("{icon") + 1;
  assert.ok(
    first.mappings.some(
      ({ sourceLine, sourceColumn }) => sourceLine === 2 && sourceColumn === expressionColumn,
    ),
    `expected a UTF-16 mapping at line 2, column ${expressionColumn}`,
  );
  assert.deepEqual(first, compileJsx(source, { ...preparationOptions, componentPrefix: "_view" }));
});

test("final prepared headings use render settings and footnote ID planning", () => {
  const source = "A footnote[^note].\n\n# fn-note\n\n[^note]: Footnote text.\n";
  const prepared = compiler().prepare(source, { footnotes: true });
  const renderOptions = { headingOffset: 1 };
  const result = prepared.render(renderOptions);
  const oneShot = compileJsx(source, { footnotes: true, ...renderOptions });

  assert.deepEqual(result, oneShot);
  assert.deepEqual(
    result.headings.map(({ level, id }) => ({ level, id })),
    [{ level: 2, id: "fn-note-1" }],
  );
});

test("prepared handles validate option categories and strict MDX errors", () => {
  const session = compiler();
  assert.throws(() => session.prepare("text", { componentPrefix: "Panel" }), /render option/);
  assert.throws(() => session.prepare("text", { headingOffset: 1 }), /render option/);
  assert.throws(() => session.prepare("text", { format: "tsx" }), /format/);
  assert.throws(() => session.prepare("{const =}", { format: "mdx" }), /invalid MDX/);

  const prepared = session.prepare("text");
  assert.throws(() => prepared.render({ passes: [] }), /unknown JSX render option/);
  assert.throws(() => prepared.renderModule({ output: "module" }), /unknown JSX render option/);
  assert.throws(
    () => prepared.render({ providerImportSource: "docs/provider" }),
    /unknown JSX render option/,
  );
});

test("prepared callback failures and recursive compiler use are safe", () => {
  const source = "```prepared-fixture\nconst value = 1;\n```\n";
  let prepared;
  const call = () => {
    const session = compiler();
    prepared = session.prepare(source);
    return prepared;
  };
  prepared = call();

  assert.throws(() => prepared.render({}, failingPreparedCode), /prepared callback failure/);
  const recursiveRender = () => prepared.render();
  assert.throws(() => prepared.render({}, recursiveRender), /recursively/);
  assert.match(prepared.render().body, /#bb1100/i);
});
