import assert from "node:assert/strict";
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

const matchingCases = [
  {
    code: "",
    language: undefined,
    meta: undefined,
    expected: { language: undefined, title: undefined, label: undefined, lineNumbers: true },
    newline: "\n",
  },
  {
    code: '日本語 <safe> {literal} & "quoted"\n\n',
    language: "unknown-fixture-language",
    meta: '[example] title="A title with spaces" {2} :no-line-numbers',
    expected: {
      language: "unknown-fixture-language",
      title: "A title with spaces",
      label: "example",
      lineNumbers: false,
    },
    newline: "\n",
  },
  {
    code: "const value = 1;\nbeta\ngamma\n",
    language: "native-alias",
    meta: '[demo] title="Override.ts" {2} :line-numbers=8 :no-line-numbers :line-numbers=3',
    expected: {
      language: "native-alias",
      title: "Override.ts",
      label: "demo",
      lineNumbers: true,
    },
    newline: "\n",
  },
  {
    code: "const crlf = true;\r\nsecond line\r\n",
    language: "native-fixture:line-numbers=6",
    meta: undefined,
    expected: {
      language: "native-fixture",
      title: undefined,
      label: undefined,
      lineNumbers: true,
    },
    newline: "\r\n",
  },
  {
    code: "const noTrailingNewline = true;",
    language: "native-fixture",
    meta: "title='No trailing newline'",
    expected: {
      language: "native-fixture",
      title: "No trailing newline",
      label: undefined,
      lineNumbers: true,
    },
    newline: "\n",
  },
];

function compiler(options = {}) {
  return new JsxCompiler({
    theme: { light, dark },
    languages: [language],
    assets: { remote: false },
    ...options,
  });
}

function renderFenceAndBlock(session, { code, language, meta, newline = "\n" }) {
  const info = `${language ?? ""}${meta === undefined ? "" : ` ${meta}`}`;
  const source = `\`\`\`${info}${newline}${code}`;
  const block = session.renderCodeBlock({ code, language, meta });
  assert.equal(session.compile(source).body, `<>\n${block.jsx}</>`);
  return block;
}

function parsedMetadata(block) {
  return {
    language: block.language,
    title: block.title,
    label: block.label,
    lineNumbers: block.lineNumbers,
  };
}

function assertDualThemeTokens(jsx) {
  assert.match(jsx, /--shiki-light.*#bb1100/i);
  assert.match(jsx, /--shiki-dark.*#ffaa22/i);
  assert.match(jsx, /--shiki-dark-font-weight.*bold/);
  assert.match(jsx, /--shiki-dark-text-decoration.*underline/);
  assert.ok(!jsx.includes("dangerouslySetInnerHTML"));
}

test("standalone code blocks equal fence markup and return parsed metadata", () => {
  const session = compiler({ lineNumbers: true });
  for (const input of matchingCases) {
    const block = renderFenceAndBlock(session, input);
    assert.deepEqual(parsedMetadata(block), input.expected);
  }
});

test("standalone highlighting keeps dual themes, line selections, and line starts", () => {
  const input = {
    code: 'const one = 1;\nconst two = "selected";\nconst three = 3;\nconst four = 4;\n',
    language: "native-fixture",
    meta: "{2,4} :line-numbers=9",
  };
  const block = renderFenceAndBlock(compiler({ lineNumbers: true }), input);
  assert.equal(block.lineNumbers, true);
  assert.match(block.jsx, /data-line-number-start=\{"9"\}/);
  assert.match(block.jsx, /data-ln=\{"9"\}/);
  assert.match(block.jsx, /data-ln=\{"10"\}/);
  assert.match(block.jsx, /data-ln=\{"12"\}/);
  assert.match(block.jsx, /className=\{"line ox-code-line ox-code-line--highlight highlighted"\}/);
  assertDualThemeTokens(block.jsx);
});

test("standalone output uses the configured single theme and plain unknown fallback", () => {
  const session = compiler({ theme: light });
  const plain = renderFenceAndBlock(session, {
    code: "日本語 <safe> & {literal}\n",
    language: "unknown-fixture-language",
    meta: undefined,
  });
  assert.ok(!plain.jsx.includes('className={"shiki'));
  assert.ok(plain.jsx.includes('"日本語 <safe> & {literal}\\n"'));

  const input = { code: "const singleTheme = true;\n", language: "native-fixture" };
  const highlighted = renderFenceAndBlock(session, input);
  assert.match(highlighted.jsx, /#bb1100/i);
  assert.ok(!highlighted.jsx.includes("--shiki-dark"));
  assert.equal(highlighted.lineNumbers, false);
});

test("standalone code-block input validation rejects invalid shapes", () => {
  const session = compiler();
  assert.throws(() => session.renderCodeBlock(null), /input must be an object/);
  assert.throws(() => session.renderCodeBlock({}), /input.code must be a string/);
  assert.throws(
    () => session.renderCodeBlock({ code: "x", language: 1 }),
    /language must be a string or null/,
  );
  assert.throws(
    () => session.renderCodeBlock({ code: "x", extra: true }),
    /unknown code block input/,
  );
});
