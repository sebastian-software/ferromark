import assert from "node:assert/strict";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";

import { JsxCompiler } from "../index.mjs";

const language = {
  name: "native-fixture",
  aliases: ["native-alias"],
  scopeName: "source.native-fixture",
  patterns: [
    { match: "\\bconst\\b", name: "keyword.native-fixture" },
    { begin: '"', end: '"', name: "string.native-fixture" },
  ],
};
const light = {
  name: "fixture-light",
  settings: [
    { settings: { foreground: "#101010", background: "#ffffff" } },
    { scope: "keyword", settings: { foreground: "#bb1100", fontStyle: "italic" } },
    { scope: "string", settings: { foreground: "#008800" } },
  ],
};
const dark = {
  name: "fixture-dark",
  settings: [
    { settings: { foreground: "#eeeeee", background: "#111111" } },
    { scope: "keyword", settings: { foreground: "#ffaa22", fontStyle: "bold" } },
    { scope: "string", settings: { foreground: "#55dd55", fontStyle: "underline" } },
  ],
};

function compiler(options = {}) {
  return new JsxCompiler({
    theme: { light, dark },
    languages: [language],
    assets: { remote: false },
    ...options,
  });
}

function assertDualThemeTokens(body) {
  assert.match(body, /--shiki-light.*#bb1100/i);
  assert.match(body, /--shiki-dark.*#ffaa22/i);
  assert.match(body, /--shiki-dark-font-weight.*bold/);
  assert.match(body, /--shiki-dark-text-decoration.*underline/);
  assert.ok(!body.includes("dangerouslySetInnerHTML"));
  assert.ok(!body.includes('from "react"'));
}

function failingCodeRenderer() {
  throw new Error("callback failure");
}

test("native JSX compiler highlights reusable custom grammars with dual themes and metadata", () => {
  const session = compiler({ lineNumbers: true });
  const code = 'const value = "日本語 <safe> {literal}";\n';
  const result = session.compile(`\`\`\`native-alias title="Demo.ts" [demo] {1}\n${code}\`\`\``, {
    codeBlockComponent: "_components.CodeBlock",
  });
  assert.match(result.body, /<_components.CodeBlock/);
  assert.match(result.body, /title=\{"Demo.ts"\}/);
  assert.match(result.body, /data-label=\{"demo"\}/);
  assert.match(result.body, /lineNumbers=\{true\}/);
  assert.match(result.body, /className=\{"shiki/);
  assert.match(result.body, /className=\{"line[^"}]*highlighted"\}/);
  assert.match(result.body, /data-ln=\{"1"\}/);
  assertDualThemeTokens(result.body);
  assert.equal(result.codeBlocks[0].code, code);
  assert.match(session.compile("```native-fixture\nconst again = 1;\n```").body, /#bb1100/i);
});

test("native single-theme JSX keeps token styles and unknown languages stay plain", () => {
  const session = compiler({ theme: light });
  const result = session.compile("```native-fixture\nconst value = 1;\n```");
  assert.match(result.body, /#bb1100/i);
  assert.match(result.body, /italic/);
  assert.ok(!result.body.includes("--shiki-dark"));
  const unknown = session.compile("```nonexistent-language\n<safe> {literal}\n```");
  assert.ok(!unknown.body.includes("#bb1100"));
  assert.equal(unknown.codeBlocks[0].code, "<safe> {literal}\n");
});

test("component mappings and callbacks override native highlighting", () => {
  const session = compiler();
  const source = "```native-fixture\nconst value = 1;\n```";
  const mapped = session.compile(source, { codeComponents: { "native-fixture": "Diagram" } });
  assert.match(mapped.body, /<Diagram/);
  assert.ok(!mapped.body.includes("#bb1100"));
  const replaced = session.compile(
    source,
    { codeComponents: { "native-fixture": "Diagram" } },
    () => '<Replacement code={"unchanged"} />',
  );
  assert.match(replaced.body, /<Replacement/);
  assert.ok(!replaced.body.includes("<Diagram"));
  assert.match(session.compile(source, {}, () => null).body, /#bb1100/i);
  assert.throws(() => session.compile(source, {}, failingCodeRenderer), /callback failure/);
  const recursiveRenderer = () => session.compile("nested").body;
  assert.throws(() => session.compile(source, {}, recursiveRenderer), /recursively/);
  assert.match(session.compile(source).body, /#bb1100/i);
});

test("native JSX compiler rejects invalid asset and registration settings", () => {
  assert.throws(() => compiler({ surprise: true }), /unknown JSX compiler option/i);
  assert.throws(() => compiler({ lineNumbers: "yes" }), /boolean/);
  assert.throws(() => compiler({ theme: { light } }), /theme.dark/);
  assert.throws(() => compiler({ theme: { settings: [] } }), /name/);
  assert.throws(() => compiler({ languages: {} }), /array/);
  assert.throws(
    () => compiler({ assets: { assetRoot: "/missing", remote: false } }),
    /cannot be combined/,
  );
});

test("missing offline assets fail only when native highlighting needs them", () => {
  const cacheDir = mkdtempSync(path.join(tmpdir(), "ferromark-jsx-cache-"));
  try {
    const session = compiler({ assets: { remote: false, cacheDir } });
    const source = "```rust\nfn main() {}\n```";
    assert.throws(() => session.compile(source), /Ferriki JSX highlighting/);
    assert.match(
      session.compile(source, { codeComponents: { rust: "RustView" } }).body,
      /<RustView/,
    );
    assert.match(session.compile(source, {}, () => "<Override />").body, /<Override/);
    assert.match(session.compile("```native-fixture\nconst okay = 1;\n```").body, /#bb1100/i);
  } finally {
    rmSync(cacheDir, { recursive: true, force: true });
  }
});
