import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { test } from "node:test";

import { compileJsx, JsxCompiler, prepareDocument } from "../index.mjs";

function offlineCompiler() {
  return new JsxCompiler({
    theme: { name: "references-test", settings: [{ settings: { foreground: "#111111" } }] },
    assets: { remote: false },
  });
}

function sourceSlice(source, span) {
  return Buffer.from(source).subarray(span.start, span.end).toString();
}

function prepareWithGetter(prepare) {
  const source = Buffer.from("[a](/before)");
  return prepare(source, {
    get frontMatter() {
      source.write("[b](/after_)");
      return false;
    },
  });
}

function assertPreparedImmutability(prepared, other) {
  assert.deepEqual(Reflect.ownKeys(prepared), ["metadata", "render", "renderModule"]);
  assert.ok(Object.isFrozen(prepared));
  assert.ok(Object.isFrozen(Object.getPrototypeOf(prepared)));
  assert.ok(Object.isFrozen(prepared.metadata));
  for (const key of ["metadata", "render", "renderModule"]) {
    assert.throws(() => {
      prepared[key] = other[key];
    }, TypeError);
  }
}

test("asset-free preparation works offline, owns bytes, and keeps ordinary code rendering", () => {
  const source = Buffer.from(
    "# Guide\n\n[Guide](/guide)\n\n```unknown-language\n[example](/ignored)\n```\n",
  );
  const original = source.toString();
  const prepared = prepareDocument(source);
  source.fill(0);
  assert.equal(prepared.collectReferences()[0].destination, "/guide");
  assert.deepEqual(prepared.render(), compileJsx(original));
  assert.deepEqual(prepared.renderModule(), compileJsx(original, { output: "module" }));
  const offline = spawnSync(
    process.execPath,
    [
      "--input-type=module",
      "-e",
      `
    import { prepareDocument } from './ferromark/index.mjs';
    const document = prepareDocument('[ok](/local)');
    if (document.collectReferences()[0].destination !== '/local') process.exit(1);
  `,
    ],
    {
      cwd: new URL("../..", import.meta.url),
      env: {
        ...process.env,
        FERRIKI_REMOTE: "0",
        FERRIKI_CACHE_DIR: "/nonexistent-ferromark-analysis-cache",
      },
      encoding: "utf8",
    },
  );
  assert.equal(offline.status, 0, offline.stderr);
});

test("references cover all Markdown forms with immutable ordered occurrence ranges", () => {
  const source =
    "é [inline](./a(b)c) [full][id] [id][] [id] ![alt *text*](./image.svg) ![id][] ![id] ![full][id] <https://example.test> <a@example.test> www.example.test [missing]\n\n[id]: /resolved\n";
  const prepared = prepareDocument(source, { autolinkLiterals: true });
  const refs = prepared.collectReferences();
  assert.deepEqual(
    refs.map(({ kind }) => kind),
    [
      "link",
      "link",
      "link",
      "link",
      "image",
      "image",
      "image",
      "image",
      "autolink",
      "autolink",
      "autolink",
    ],
  );
  assert.equal(refs[0].span.start, 3);
  assert.equal(refs[0].destination, "./a(b)c");
  assert.equal(refs[4].text, "alt text");
  assert.equal(sourceSlice(source, refs[1].span), "[full][id]");
  assert.ok(
    refs.every(
      (ref) => ref.provenance === "source" && Object.isFrozen(ref) && Object.isFrozen(ref.span),
    ),
  );
  assert.ok(Object.isFrozen(refs));
  assert.throws(() => refs.push(refs[0]), TypeError);
  assert.throws(() => {
    refs[0].destination = "/change";
  }, TypeError);
  assert.deepEqual(prepared.collectReferences(), refs);
});

test("code, raw HTML, unresolved references, and disabled GFM syntax do not enter inventories", () => {
  const source =
    '`[code](/ignored)`\n\n```md\n[code](/ignored)\n```\n\n    [code](/ignored)\n\n<a href="/raw">Raw</a>\n\n[missing] www.example.test <https://example.test>\n';
  const refs = prepareDocument(source, { autolinkLiterals: false }).collectReferences();
  assert.deepEqual(
    refs.map(({ destination }) => destination),
    ["https://example.test"],
  );
});

test("MDX literal attributes preserve custom component origin and ignore expressions/spreads", () => {
  const source =
    '[before][id]\n\n<Card href="/card?a=1&amp;b=2" src=\'/asset.png\' {...props}>\n\n![inside][id]\n\n</Card>\n\nText <a href={url} src={"dynamic"} href="/static" />\n\n[id]: /resolved\n';
  const prepared = prepareDocument(source, { format: "mdx" });
  const refs = prepared.collectReferences();
  assert.deepEqual(
    refs.map(({ destination }) => destination),
    ["/resolved", "/card?a=1&b=2", "/asset.png", "/resolved", "/static"],
  );
  assert.equal(refs[1].kind, "mdxAttribute");
  assert.equal(refs[1].element, "Card");
  assert.equal(refs[1].attribute, "href");
  assert.equal(refs[1].text, undefined);
  assert.equal(sourceSlice(source, refs[1].span), 'href="/card?a=1&amp;b=2"');
  // The literal is an authored occurrence; {...props} can override it at runtime.
  assert.deepEqual(refs, offlineCompiler().prepare(source, { format: "mdx" }).collectReferences());
});

test("native passes preserve honest provenance and repeated extraction preserves body/module output", () => {
  const source = "[keep](/keep) [Hello :smile:](/changed) #521\n";
  const prepared = prepareDocument(source, {
    trackReferences: true,
    passes: [
      { kind: "emojiShortcodes" },
      { kind: "githubReferences", repository: "sebastian-software/ferromark" },
    ],
  });
  const beforeBody = prepared.render();
  const beforeModule = prepared.renderModule();
  const refs = prepared.collectReferences();
  assert.deepEqual(
    refs.map(({ provenance }) => provenance),
    ["source", "transformed", "transformed"],
  );
  assert.ok(refs[0].span);
  assert.equal(refs[1].span, undefined);
  assert.equal(sourceSlice(source, refs[1].originalSpan), "[Hello :smile:](/changed)");
  assert.equal(refs[1].text, "Hello 😄");
  assert.deepEqual(
    {
      kind: refs[2].kind,
      destination: refs[2].destination,
      span: refs[2].span,
      originalSpan: refs[2].originalSpan,
    },
    {
      kind: "link",
      destination: "https://github.com/sebastian-software/ferromark/issues/521",
      span: undefined,
      originalSpan: undefined,
    },
  );
  assert.deepEqual(refs, prepared.collectReferences());
  assert.deepEqual(beforeBody, prepared.render());
  assert.deepEqual(beforeModule, prepared.renderModule());
});

test("prepareDocument shares preparation option validation and UTF-8 input errors", () => {
  for (const options of [
    { headingOffset: 1 },
    { output: "module" },
    { assets: { remote: false } },
    { format: "tsx" },
    { trackReferences: null },
    { trackReferences: 1 },
    { trackReferences: "true" },
    { unexpected: true },
  ]) {
    assert.throws(() => prepareDocument("text", options), TypeError);
    assert.throws(() => offlineCompiler().prepare("text", options), TypeError);
  }
  assert.throws(() => prepareDocument("{const =}", { format: "mdx" }), /invalid MDX/);
  assert.throws(() => prepareDocument(42), { code: "ERR_INVALID_ARG_TYPE" });
  assert.throws(() => prepareDocument(new ArrayBuffer(8)), { code: "ERR_INVALID_ARG_TYPE" });
  assert.deepEqual(prepareDocument(new Uint8Array([0xff])).render(), compileJsx("�"));
});

test("preparation copies bytes after option getters, matching compiler preparation", () => {
  const analysis = prepareWithGetter(prepareDocument);
  const session = offlineCompiler();
  const highlighted = prepareWithGetter(session.prepare.bind(session));
  assert.equal(analysis.collectReferences()[0].destination, "/after_");
  assert.equal(analysis.collectReferences()[0].text, "b");
  assert.deepEqual(analysis.collectReferences(), highlighted.collectReferences());
});

test("native passes require explicit reference tracking while ordinary prepared rendering stays available", () => {
  const session = offlineCompiler();
  for (const prepare of [prepareDocument, session.prepare.bind(session)]) {
    const source = "[Hello :smile:](/target)";
    const passes = [{ kind: "emojiShortcodes" }];
    for (const options of [{ passes }, { passes, trackReferences: false }]) {
      const prepared = prepare(source, options);
      const before = prepared.render();
      assert.ok(before.body.includes("Hello 😄"));
      assert.throws(() => prepared.collectReferences(), /trackReferences: true during preparation/);
      assert.deepEqual(prepared.render(), before);
      assert.ok(prepared.renderModule().code.includes("Hello 😄"));
    }
    assert.equal(
      prepare(source, { passes, trackReferences: true }).collectReferences()[0].provenance,
      "transformed",
    );
    assert.equal(prepare(source, { passes: [] }).collectReferences()[0].provenance, "source");
    assert.equal(
      prepare(source, { trackReferences: true }).collectReferences()[0].provenance,
      "source",
    );
  }
});

test("tracking getters are validated once, including inherited getters, for both preparations", () => {
  const session = offlineCompiler();
  for (const prepare of [prepareDocument, session.prepare.bind(session)]) {
    let reads = 0;
    const options = Object.create({
      get trackReferences() {
        reads += 1;
        return reads === 1 ? true : "invalid second read";
      },
    });
    options.passes = [{ kind: "emojiShortcodes" }];
    const prepared = prepare("[Hello :smile:](/target)", options);
    assert.equal(reads, 1);
    assert.equal(prepared.collectReferences()[0].provenance, "transformed");
    assert.throws(
      () => prepare("text", Object.create({ trackReferences: "true" })),
      /trackReferences must be a boolean/,
    );
  }
  assert.throws(() => compileJsx("text", { trackReferences: true }), /unknown JSX option/);
  assert.throws(() => session.compile("text", { trackReferences: true }), /unknown JSX option/);
  assert.throws(
    () => prepareDocument("text").render({ trackReferences: true }),
    /unknown JSX render option/,
  );
});

test("prepared handles share analysis methods and preserve immutable, destructured rendering", () => {
  const session = offlineCompiler();
  for (const prepare of [prepareDocument, session.prepare.bind(session)]) {
    const prepared = prepare("# Title\n\n[Guide](/guide)");
    const other = prepare("[Other](/other)");
    assert.equal(prepared.collectReferences, other.collectReferences);
    assert.equal(Object.hasOwn(prepared, "collectReferences"), false);
    assertPreparedImmutability(prepared, other);
    const { render, renderModule } = prepared;
    assert.deepEqual(render({ headingOffset: 1 }), prepared.render({ headingOffset: 1 }));
    assert.deepEqual(
      renderModule({ filename: "guide.mdx" }),
      prepared.renderModule({ filename: "guide.mdx" }),
    );
    assert.equal(prepared.collectReferences()[0].destination, "/guide");
  }
});
