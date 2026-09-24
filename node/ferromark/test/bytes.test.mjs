// Markdown as UTF-8 bytes. Every entry that takes Markdown accepts a
// `Uint8Array` (a `Buffer` is one), and renders it exactly as it renders the
// string `Buffer#toString('utf8')` makes of it. The addon borrows the bytes
// where that is sound and copies them where JavaScript could change them
// during the call; see node/native/src/input.rs and
// docs/decisions/2026-09-24-node-bytes-input.md.
/* eslint-disable max-lines */
import assert from "node:assert/strict";
import { once } from "node:events";
import { readFileSync } from "node:fs";
import test from "node:test";
import { runInNewContext } from "node:vm";
import { Worker } from "node:worker_threads";
import { gunzipSync } from "node:zlib";

import {
  Renderer,
  toHtml,
  toHtmlBuffer,
  toHtmlWithHighlighter,
  transform,
  transformWithHighlighter,
} from "../index.mjs";

/** The string the contract compares bytes with. */
const decode = (bytes) =>
  Buffer.from(bytes.buffer, bytes.byteOffset, bytes.byteLength).toString("utf8");

const escape = (text) => text.replaceAll("&", "&amp;").replaceAll("<", "&lt;");
// Echoes what it receives, so the highlighter entries compare the decoded code
// and metadata too.
const highlighter = {
  codeToHtml: (code, { lang, meta }) =>
    `<pre data-lang="${escape(lang)}" data-meta="${escape(meta?.__raw ?? "")}">${escape(code)}</pre>\n`,
};
const highlightOptions = { theme: "bytes" };

function outcome(run) {
  try {
    const value = run();
    return { value: Buffer.isBuffer(value) ? { buffer: value.toString("latin1") } : value };
  } catch (error) {
    return {
      error: error instanceof Error ? `${error.constructor.name}: ${error.message}` : error,
    };
  }
}

/**
 * Every public entry that takes Markdown. The renderers are kept across
 * documents, one per input kind, so reuse is covered too.
 */
function entries(renderer) {
  return {
    "Renderer.toHtml": (markdown) => renderer.toHtml(markdown),
    "Renderer.toHtmlBuffer": (markdown) => renderer.toHtmlBuffer(markdown),
    toHtml: (markdown, options) => toHtml(markdown, options),
    toHtmlBuffer: (markdown, options) => toHtmlBuffer(markdown, options),
    toHtmlWithHighlighter: (markdown, options) =>
      toHtmlWithHighlighter(markdown, highlighter, highlightOptions, options),
    transform: (markdown, options) => transform(markdown, options),
    transformWithHighlighter: (markdown, options) =>
      transformWithHighlighter(markdown, highlighter, highlightOptions, options),
  };
}

// The forms a `Uint8Array` comes in. Each copies `bytes` into fresh memory.
const forms = {
  Buffer: (bytes) => Buffer.from(bytes),
  SharedArrayBuffer(bytes) {
    const view = new Uint8Array(new SharedArrayBuffer(bytes.length));
    view.set(bytes);
    return view;
  },
  Uint8Array: (bytes) => new Uint8Array(bytes),
  subarray(bytes) {
    const backing = new Uint8Array(bytes.length + 11);
    backing.set(bytes, 7);
    return backing.subarray(7, 7 + bytes.length);
  },
};

/**
 * Asserts that every entry renders each form of `bytes` as it renders the
 * decoded string, with each of `optionSets` (with options, the facade packs
 * them; without, it calls the object-taking export).
 */
function assertEquivalent(bytes, label, { optionSets = [undefined], renderers }) {
  const text = decode(bytes);
  for (const [name, render] of Object.entries(entries(renderers.string))) {
    for (const options of optionSets) {
      const expected = outcome(() => render(text, options));
      for (const [form, make] of Object.entries(forms)) {
        const actual = outcome(() => entries(renderers[form])[name](make(bytes), options));
        const context = `${label}: ${name} with ${form}${options ? " and options" : ""}`;
        assert.deepEqual(actual, expected, context);
      }
    }
  }
}

function freshRenderers() {
  return Object.fromEntries(["string", ...Object.keys(forms)].map((key) => [key, new Renderer()]));
}

const corpus = JSON.parse(
  gunzipSync(
    readFileSync(
      new URL(
        "../../../docs/reports/2026-09-14-optimization-rounds/broad-corpus.json.gz",
        import.meta.url,
      ),
    ),
  ).toString("utf8"),
).cases;

// Documents from the other Node tests: every V8 string representation, lone
// surrogates (which a string converts to U+FFFD and `Buffer.from` encodes as
// U+FFFD), and the syntax the options switch on.
const fixtures = [
  "",
  "# Hello",
  "plain *ASCII* text",
  "Grüße aus Köln: ½ × ¾, ÿ",
  "世界 *😀* € 𝄞",
  "\uDC00lead \uD800x trail\uDC00 swapped \uDE00\uD83D pair \uD83D\uDE00 end\uDBFF",
  "---\ntitle: X\n---\n# Top\n\n## Sub `code`\n",
  '+++\ntitle = "TOML"\n+++\n# Top',
  "==Text==^[a *note*] [ref]\n\n[ref]: /url",
  "# A *title*!\n\n# A title!\n\n## Explicit {#custom}",
  "| Name | Value |\n| --- | --- |\n| a | b |\n| merged ||",
  '```ts {1-3} title="Example"\ncode\n```\n\n```ts\ncode\n```',
  "```unknown\n<tag>\n```\n\n    indented\n    code\n",
  "[x](javascript:alert%281%29) ![x](data:text/html,bad) <script>x</script>",
  "Footnote[^1] and [[Wiki Page]] and [guide](/guide.md).\n\n[^1]: The note.",
  "> [!NOTE]\n> Callout.\n\nTerm\n: Definition\n\n// line comment\n\nA**強調。**B\n\n<Component />",
  "line one\r\nline two\r\n\r\n- [x] done\r\n",
  "nul \0 byte and \uFEFF inside",
  `${"[".repeat(20_000)}a${"]".repeat(20_000)}`,
];

const everyOption = {
  callouts: true,
  cjkEmphasis: true,
  definitionLists: true,
  footnotes: true,
  frontMatter: true,
  headingAttributes: true,
  headingIdPrefix: "p-",
  headingOffset: 1,
  highlight: true,
  inlineFootnotes: true,
  lineComments: true,
  linkBasePath: "/docs",
  math: true,
  mergedTableCells: true,
  renderPolicy: "trusted",
  subscript: true,
  superscript: true,
  tableColgroup: true,
  wikiLinks: true,
};
// Without options the facade calls the object-taking export; with them, the
// packed one.
const optionSets = [undefined, everyOption];

test("renders every broad-corpus document from bytes as from its string", () => {
  assert.equal(corpus.length, 57);
  const renderers = freshRenderers();
  for (const { input, name } of corpus) {
    assertEquivalent(Buffer.from(input, "utf8"), name, { optionSets, renderers });
  }
});

test("renders every fixture from bytes as from its string", () => {
  const renderers = freshRenderers();
  for (const fixture of fixtures) {
    const label = JSON.stringify(fixture.slice(0, 30));
    assertEquivalent(Buffer.from(fixture, "utf8"), label, { optionSets, renderers });
    // The decoded string of well-formed text is the text itself.
    assert.equal(decode(Buffer.from(fixture, "utf8")), fixture.toWellFormed(), label);
  }
});

test("accepts every kind of Uint8Array", () => {
  const text = "# Kinds *of* bytes\n";
  const html = toHtml(text);
  class Bytes extends Uint8Array {}
  const resizable = new ArrayBuffer(text.length, { maxByteLength: 64 });
  const growable = new SharedArrayBuffer(text.length, { maxByteLength: 64 });
  new Uint8Array(resizable).set(Buffer.from(text));
  new Uint8Array(growable).set(Buffer.from(text));
  const kinds = {
    "a Buffer from the pool": Buffer.from(text),
    "a Uint8Array subclass": Bytes.from(Buffer.from(text)),
    "a Uint8Array from another realm": runInNewContext(
      `new Uint8Array(${JSON.stringify([...Buffer.from(text)])})`,
    ),
    "a length-tracking view of a resizable ArrayBuffer": new Uint8Array(resizable),
    "a length-tracking view of a growable SharedArrayBuffer": new Uint8Array(growable),
    "a TextEncoder result": new TextEncoder().encode(text),
  };
  for (const [kind, bytes] of Object.entries(kinds)) {
    assert.equal(toHtml(bytes), html, kind);
    assert.equal(toHtml(bytes, { headingIds: false }), toHtml(text, { headingIds: false }), kind);
    assert.equal(new Renderer().toHtml(bytes), html, kind);
    assert.equal(toHtmlWithHighlighter(bytes, highlighter, highlightOptions), html, kind);
  }
});

test("reads empty, detached and out-of-bounds views as empty input", () => {
  const detached = new Uint8Array(Buffer.from("# gone"));
  detached.buffer.transfer();
  const shrunk = new ArrayBuffer(8, { maxByteLength: 8 });
  const outOfBounds = new Uint8Array(shrunk, 2, 4);
  shrunk.resize(3);
  const empties = {
    "Buffer.alloc(0)": Buffer.alloc(0),
    "a detached view": detached,
    "a view out of bounds": outOfBounds,
    "an empty subarray": Buffer.from("# text").subarray(3, 3),
    "new Uint8Array(0)": new Uint8Array(0),
  };
  for (const [kind, bytes] of Object.entries(empties)) {
    assert.equal(bytes.byteLength, 0, kind);
    for (const [name, render] of Object.entries(entries(new Renderer()))) {
      assert.deepEqual(
        outcome(() => render(bytes)),
        outcome(() => render("")),
        `${name}: ${kind}`,
      );
    }
  }
});

// Byte classes of UTF-8: ASCII, each continuation range, each lead byte range
// and the bytes that never occur.
const classes = [
  0x00, 0x0a, 0x41, 0x7f, 0x80, 0x8f, 0x90, 0x9f, 0xa0, 0xbf, 0xc0, 0xc1, 0xc2, 0xdf, 0xe0, 0xe1,
  0xec, 0xed, 0xee, 0xef, 0xf0, 0xf1, 0xf3, 0xf4, 0xf5, 0xf7, 0xf8, 0xfb, 0xfc, 0xfe, 0xff,
];

/** Invalid and edge-case UTF-8, as [label, bytes]. */
function* invalidInputs() {
  yield* Object.entries(namedInputs());
  yield* truncatedInputs();
  yield* classInputs();
  yield* randomInputs(mulberry32(0xb7_e5));
}

/** Each kind of invalid UTF-8, and byte order marks. */
function namedInputs() {
  return {
    "every byte": Buffer.from(Array.from({ length: 256 }, (_, byte) => byte)),
    "lone continuation bytes": Buffer.from([0x61, 0x80, 0x62, 0xbf, 0x80, 0x63]),
    "overlong encodings": Buffer.from([
      0xc0, 0x80, 0x20, 0xc1, 0xbf, 0x20, 0xe0, 0x80, 0x80, 0x20, 0xe0, 0x9f, 0xbf, 0x20, 0xf0,
      0x80, 0x80, 0x80, 0x20, 0xf0, 0x8f, 0xbf, 0xbf,
    ]),
    "surrogate encodings": Buffer.from([
      0xed, 0xa0, 0x80, 0x20, 0xed, 0xbf, 0xbf, 0x20, 0xed, 0xa0, 0xbd, 0xed, 0xb8, 0x80, 0x20,
      0xed, 0x9f, 0xbf,
    ]),
    "code points above U+10FFFF": Buffer.from([
      0xf4, 0x90, 0x80, 0x80, 0x20, 0xf4, 0x8f, 0xbf, 0xbf,
    ]),
    "bytes F5 to FF": Buffer.from([
      0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff,
    ]),
    "a byte order mark": Buffer.from([0xef, 0xbb, 0xbf, 0x23, 0x20, 0x48]),
    "two byte order marks": Buffer.from([0xef, 0xbb, 0xbf, 0xef, 0xbb, 0xbf, 0x23, 0x20, 0x48]),
    "a broken byte order mark": Buffer.from([0xef, 0xbb, 0x23, 0x20, 0x48]),
  };
}

/** Every valid sequence cut short, at the end of the input and before ASCII. */
function* truncatedInputs() {
  for (const sequence of ["é", "€", "😀", "\uFFFF", "\u{10FFFF}"]) {
    const encoded = Buffer.from(sequence);
    for (let cut = 1; cut < encoded.length; cut++) {
      const prefix = encoded.subarray(0, cut);
      yield [`${sequence} cut to ${cut} at the end`, Buffer.concat([Buffer.from("x "), prefix])];
      yield [`${sequence} cut to ${cut} before text`, Buffer.concat([prefix, Buffer.from("x")])];
    }
  }
}

/** Every pair of byte classes, and a pair followed by a third byte. */
function* classInputs() {
  for (const first of classes) {
    const pairs = [];
    for (const second of classes) {
      pairs.push(first, second, 0x20);
      for (const third of [0x80, 0xbf, 0x41]) pairs.push(first, second, third, 0x20);
    }
    yield [`class 0x${first.toString(16)}`, Buffer.from(pairs)];
  }
}

/** Seeded random input, mostly bytes at or above 0x80. */
function* randomInputs(random) {
  for (let round = 0; round < 60; round++) {
    const bytes = Buffer.alloc(1 + Math.floor(random() * 700));
    for (let index = 0; index < bytes.length; index++) {
      const roll = random();
      bytes[index] =
        roll < 0.45
          ? 0x80 + Math.floor(random() * 0x40)
          : roll < 0.8
            ? 0xc0 + Math.floor(random() * 0x40)
            : Math.floor(random() * 0x1_00);
    }
    yield [`random ${round}`, bytes];
  }
}

test("replaces invalid UTF-8 as Buffer#toString does", () => {
  const renderers = freshRenderers();
  let replaced = 0;
  for (const [name, bytes] of invalidInputs()) {
    if (decode(bytes).includes("\uFFFD")) replaced++;
    assertEquivalent(bytes, name, { renderers });
    // Inside a fence, the highlighter receives the decoded code itself.
    const fenced = Buffer.concat([Buffer.from("```x\n"), bytes, Buffer.from("\n```\n")]);
    assertEquivalent(fenced, `${name} in a fence`, { renderers });
  }
  assert.ok(replaced > 100, "the inputs exercise the replacement");
});

test("keeps a byte order mark where Buffer#toString keeps it", () => {
  const bom = Buffer.from([0xef, 0xbb, 0xbf]);
  const cases = [
    ["# Heading", '<h1 id="heading">Heading</h1>\n'],
    ["\uFEFF# Heading", "<p>\uFEFF# Heading</p>\n"],
  ];
  for (const [text, html] of cases) {
    const bytes = Buffer.concat([bom, Buffer.from(text)]);
    // The parser treats one leading U+FEFF as metadata, from a string too.
    assert.equal(toHtml(`\uFEFF${text}`), html);
    assert.equal(toHtml(bytes), html);
    // TextDecoder strips one more mark, so its string can render differently.
    const decoded = new TextDecoder().decode(bytes);
    assert.equal(toHtml(decoded) === html, text === "# Heading");
  }
  const frontMatter = Buffer.concat([bom, Buffer.from("---\ntitle: X\n---\n# Top\n")]);
  assert.deepEqual(
    transform(frontMatter, { frontMatter: true }),
    transform(`\uFEFF${frontMatter.subarray(3)}`, { frontMatter: true }),
  );
});

const highlighted =
  "# Title\n\n```js\nfirst\n```\n\nBetween *text*.\n\n```\nsecond\n```\n\n## End\n";

/** `text` in a resizable buffer of its own. */
function resizableCopy(text) {
  const buffer = new ArrayBuffer(text.length, { maxByteLength: text.length });
  const bytes = new Uint8Array(buffer);
  bytes.set(Buffer.from(text));
  return bytes;
}

// Buffers of `highlighted` that a highlighter may detach, overwrite or shrink.
const mutableForms = {
  "a SharedArrayBuffer": () => forms.SharedArrayBuffer(Buffer.from(highlighted)),
  "a resizable buffer": () => resizableCopy(highlighted),
  "an owned buffer": () => new Uint8Array(Buffer.from(highlighted)),
};

// What a highlighter does to the bytes, and the forms it can do it to.
const mutations = [
  [
    "detach",
    (bytes) => bytes.byteLength > 0 && bytes.buffer.transfer(),
    ["a resizable buffer", "an owned buffer"],
  ],
  ["overwrite with invalid UTF-8", (bytes) => bytes.fill(0xff), Object.keys(mutableForms)],
  ["overwrite with other text", (bytes) => bytes.fill(0x2a), Object.keys(mutableForms)],
  ["shrink", (bytes) => bytes.buffer.resize(0), ["a resizable buffer"]],
];

/** A highlighter that runs `mutate` on the bytes `current()` returns, then highlights. */
function mutatingHighlighter(current, mutate) {
  const calls = [];
  return {
    calls,
    codeToHtml(code, context) {
      calls.push(code);
      mutate(current());
      return highlighter.codeToHtml(code, context);
    },
  };
}

test("copies bytes before a highlighter could change them", () => {
  const expectedHtml = toHtmlWithHighlighter(highlighted, highlighter, highlightOptions);
  const expectedTransform = transformWithHighlighter(highlighted, highlighter, highlightOptions);
  assert.match(expectedHtml, /second/);
  for (const [mutation, mutate, kinds] of mutations) {
    for (const kind of kinds) {
      const label = `${mutation} of ${kind}`;
      let bytes = mutableForms[kind]();
      const mutating = mutatingHighlighter(() => bytes, mutate);
      assert.equal(toHtmlWithHighlighter(bytes, mutating, highlightOptions), expectedHtml, label);
      assert.deepEqual(mutating.calls, ["first\n", "second\n"], label);
      bytes = mutableForms[kind]();
      const result = transformWithHighlighter(bytes, mutating, highlightOptions);
      assert.deepEqual(result, expectedTransform, label);
    }
  }
});

test("lets a highlighter render the same bytes again", () => {
  const bytes = Buffer.from("```\ninner *text*\n```\n");
  const nested = {
    codeToHtml: () => toHtml(bytes),
  };
  assert.equal(
    toHtmlWithHighlighter(bytes, nested, highlightOptions),
    toHtml("```\ninner *text*\n```\n"),
  );
});

/**
 * Starts a worker that flips every byte of `shared` between "a" and 0xFF until
 * `stop()`, so each read of a byte sees one or the other.
 * @returns A promise for the running writer, with `stop()` to end it.
 */
async function startWriter(shared) {
  const control = new Int32Array(new SharedArrayBuffer(4));
  const worker = new Worker(
    `
      const { parentPort, workerData } = require("node:worker_threads");
      const bytes = new Uint8Array(workerData.shared);
      const control = new Int32Array(workerData.control);
      parentPort.postMessage("ready");
      let seed = 1;
      while (Atomics.load(control, 0) === 0) {
        for (let index = 0; index < bytes.length; index++) {
          seed = (seed * 1103515245 + 12345) >>> 0;
          bytes[index] = seed & 0x1_00 ? 0xff : 0x61;
        }
      }
    `,
    { eval: true, workerData: { control: control.buffer, shared } },
  );
  const exited = once(worker, "exit");
  await once(worker, "message");
  return {
    async stop() {
      Atomics.store(control, 0, 1);
      await exited;
    },
  };
}

test("renders a SharedArrayBuffer soundly while a worker writes it", async () => {
  const length = 4096;
  const shared = new SharedArrayBuffer(length);
  new Uint8Array(shared).fill(0x61);
  const writer = await startWriter(shared);
  try {
    const bytes = new Uint8Array(shared);
    const paragraph = new RegExp(`^<p>[a\\uFFFD]{${length}}</p>\\n$`, "u");
    const renderer = new Renderer();
    for (let round = 0; round < 100; round++) {
      assert.match(toHtml(bytes), paragraph);
      assert.match(renderer.toHtml(bytes), paragraph);
      assert.match(toHtmlBuffer(bytes).toString("utf8"), paragraph);
      assert.match(transform(bytes).html, paragraph);
      assert.match(toHtmlWithHighlighter(bytes, highlighter, highlightOptions), paragraph);
    }
  } finally {
    await writer.stop();
  }
});

function namedFunction() {}

/** Values that are neither a string nor a `Uint8Array`, and how the error names them. */
function rejectedValues() {
  const cyclic = {};
  cyclic.self = cyclic;
  return [
    [123, "a number"],
    [Number.NaN, "a number"],
    [true, "a boolean"],
    [null, "null"],
    [undefined, "undefined"],
    [Symbol("markdown"), "a symbol"],
    [10n, "a bigint"],
    [{}, "an object"],
    [cyclic, "an object"],
    [[35, 32, 104], "an array"],
    [namedFunction, "a function"],
    // oxlint-disable-next-line unicorn/new-for-builtins -- a String object is not a string
    [Object("# text"), "an object"],
    [new Date(0), "an object"],
    [new ArrayBuffer(4), "an ArrayBuffer"],
    [new SharedArrayBuffer(4), "an object"],
    [new DataView(new ArrayBuffer(4)), "a DataView"],
    [new Int8Array(4), "an Int8Array"],
    [new Uint8ClampedArray(4), "a Uint8ClampedArray"],
    [new Int16Array(4), "an Int16Array"],
    [new Uint16Array(4), "a Uint16Array"],
    [new Int32Array(4), "an Int32Array"],
    [new Uint32Array(4), "a Uint32Array"],
    [new Float32Array(4), "a Float32Array"],
    [new Float64Array(4), "a Float64Array"],
    [new BigInt64Array(4), "a BigInt64Array"],
    [new BigUint64Array(4), "a BigUint64Array"],
    [new Proxy(new Uint8Array(4), {}), "an object"],
  ];
}

test("rejects every other value with a TypeError", () => {
  for (const [value, received] of rejectedValues()) {
    for (const [name, render] of Object.entries(entries(new Renderer()))) {
      assert.throws(
        () => render(value),
        (error) => {
          assert.ok(error instanceof TypeError, `${name}: ${received}`);
          assert.equal(error.code, "ERR_INVALID_ARG_TYPE", `${name}: ${received}`);
          assert.equal(
            error.message,
            `markdown must be a string or a Uint8Array, received ${received}`,
            name,
          );
          return true;
        },
      );
    }
  }
});

test("describes a rejected value without running its code", () => {
  const ran = [];
  const traps = new Proxy(new Uint8Array(4), {
    get(target, key) {
      ran.push(`get ${String(key)}`);
      return Reflect.get(target, key);
    },
    getPrototypeOf(target) {
      ran.push("getPrototypeOf");
      return Reflect.getPrototypeOf(target);
    },
  });
  const values = [
    traps,
    {
      toJSON() {
        ran.push("toJSON");
      },
      toString() {
        ran.push("toString");
        return "";
      },
    },
    Object.defineProperty(() => {}, "name", {
      get() {
        ran.push("name");
        return "named";
      },
    }),
  ];
  for (const value of values) {
    for (const render of Object.values(entries(new Renderer()))) {
      assert.throws(() => render(value), TypeError);
    }
  }
  assert.deepEqual(ran, []);
});

test("rejects unknown options before the Markdown, and the Markdown before option values", () => {
  for (const markdown of [new Float64Array(1), new ArrayBuffer(1), 1]) {
    assert.throws(
      () => toHtml(markdown, { taskList: true }),
      (error) => error instanceof TypeError && error.message === 'unknown option "taskList"',
    );
    for (const options of [{ tables: 1 }, { renderPolicy: "bad" }, { headingOffset: 1.5 }]) {
      assert.throws(
        () => toHtml(markdown, options),
        (error) => error instanceof TypeError && error.code === "ERR_INVALID_ARG_TYPE",
      );
    }
  }
  // With valid bytes, option values are checked as for a string.
  assert.throws(() => toHtml(Buffer.from("x"), { renderPolicy: "bad" }), /renderPolicy must be/);
  assert.throws(() => toHtml(Buffer.from("x"), { tables: 1 }), /Options\.tables/);
});

test("rejects bytes beyond the parser's 4 GiB offset range", (t) => {
  let bytes;
  try {
    // Zero-filled pages are only mapped, and the addon rejects the length
    // before it reads a byte.
    bytes = new Uint8Array(2 ** 32);
  } catch {
    t.skip("cannot allocate a 4 GiB Uint8Array here");
    return;
  }
  for (const [name, render] of Object.entries(entries(new Renderer()))) {
    assert.throws(
      () => render(bytes),
      (error) =>
        error instanceof RangeError &&
        error.code === "ERR_OUT_OF_RANGE" &&
        error.message ===
          "markdown must be at most 4294967295 bytes long, received 4294967296 bytes",
      name,
    );
  }
});

/** A small seeded generator, so a failing input reproduces. */
function mulberry32(seed) {
  let state = seed;
  return () => {
    state = (state + 0x6d_2b_79_f5) | 0;
    let value = Math.imul(state ^ (state >>> 15), 1 | state);
    value = (value + Math.imul(value ^ (value >>> 7), 61 | value)) ^ value;
    return ((value ^ (value >>> 14)) >>> 0) / 4_294_967_296;
  };
}
