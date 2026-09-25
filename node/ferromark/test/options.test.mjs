// The facade reads an options object itself and passes it to private native
// entries in packed form. These tests hold that path to the object-taking
// native entries it replaces: every entry must return the same result or
// throw the same error, and make the same property gets in the same order.
/* eslint-disable max-lines */
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import test from "node:test";

import {
  Renderer,
  toHtml,
  toHtmlBuffer,
  toHtmlWithHighlighter,
  transform,
  transformWithHighlighter,
} from "../index.mjs";
import { linuxLibc, nativeTarget } from "../native-target.mjs";

/** The addon the facade loads, resolved the same way, so both share one instance. */
function loadAddon() {
  const require = createRequire(import.meta.url);
  const libc =
    process.platform === "linux" ? linuxLibc(process.report?.getReport?.(), () => "") : undefined;
  const target = nativeTarget(process.platform, process.arch, libc);
  // The target comes from the fixed native table, as in the facade's loader.
  try {
    // eslint-disable-next-line security/detect-non-literal-require
    return require(`../ferromark.${target}.node`);
  } catch (error) {
    if (error?.code !== "MODULE_NOT_FOUND") throw error;
    // eslint-disable-next-line security/detect-non-literal-require
    return require(`ferromark-${target}`);
  }
}

const addon = loadAddon();

// `Options` in napi-rs's read order: the declaration order in
// node/native/src/lib.rs.
const fields = [
  ["renderPolicy", "string"],
  ["allowHtml", "boolean"],
  ["tables", "boolean"],
  ["mergedTableCells", "boolean"],
  ["tableColgroup", "boolean"],
  ["tableColumnNames", "boolean"],
  ["tableAttributes", "boolean"],
  ["strikethrough", "boolean"],
  ["superscript", "boolean"],
  ["subscript", "boolean"],
  ["taskLists", "boolean"],
  ["autolinkLiterals", "boolean"],
  ["disallowedRawHtml", "boolean"],
  ["footnotes", "boolean"],
  ["highlight", "boolean"],
  ["inlineFootnotes", "boolean"],
  ["allowLinkRefs", "boolean"],
  ["frontMatter", "boolean"],
  ["headingIds", "boolean"],
  ["headingOffset", "number"],
  ["headingIdPrefix", "string"],
  ["headingAttributes", "boolean"],
  ["math", "boolean"],
  ["callouts", "boolean"],
  ["definitionLists", "boolean"],
  ["lineComments", "boolean"],
  ["wikiLinks", "boolean"],
  ["cjkEmphasis", "boolean"],
  ["mdx", "boolean"],
  ["linkBasePath", "string"],
];
const keys = fields.map(([key]) => key);

// Every field changes this document's HTML, so a field packed into the wrong
// bit shows up as a different output (checked below).
const probe = [
  "---",
  "title: Probe",
  "---",
  "",
  "# Heading {#custom}",
  "",
  "## Second heading",
  "",
  "<i>raw</i> and <textarea>x</textarea>",
  "",
  "| Name | Value |",
  "| --- | --- |",
  "| a | b |",
  "| merged ||",
  ": Caption {#table-id .wide}",
  "",
  "~~strike~~ x^2^ H~2~O ==mark== ^[inline note] www.example.com $x$",
  "",
  "- [x] done",
  "",
  "Footnote[^1] and [ref][r] and [[Wiki Page]] and [guide](/guide.md).",
  "",
  "[^1]: The note.",
  "",
  "[r]: /url",
  "",
  "> [!NOTE]",
  "> Callout.",
  "",
  "Term",
  ": Definition",
  "",
  "// line comment",
  "",
  "A**強調。**B",
  "",
  "<Component />",
  "",
  "```js title=probe",
  "let code;",
  "```",
  "",
].join("\n");

const highlighter = {
  codeToHtml: (code, { lang, theme }) =>
    `<pre data-lang="${lang}" data-theme="${theme}">${code}</pre>`,
};
const highlightOptions = { theme: "probe" };
// What the facade's callback returns for this highlighter, for the direct calls.
const renderCode = (code, language) =>
  highlighter.codeToHtml(code, { lang: language ?? "text", theme: highlightOptions.theme });

// Each public entry that takes options, next to the native call the facade
// made for it before options were packed.
const entries = [
  {
    direct: (markdown, options) => addon.toHtml(markdown, options),
    facade: (markdown, options) => toHtml(markdown, options),
    name: "toHtml",
    natives: ["toHtml", "toHtmlPacked"],
  },
  {
    direct: (markdown, options) => addon.toHtmlBuffer(markdown, options),
    facade: (markdown, options) => toHtmlBuffer(markdown, options),
    name: "toHtmlBuffer",
    natives: ["toHtmlBuffer", "toHtmlBufferPacked"],
  },
  {
    direct: (markdown, options) => reuse(new addon.Renderer(options), markdown),
    facade: (markdown, options) => reuse(new Renderer(options), markdown),
    name: "Renderer",
  },
  {
    direct: (markdown, options) => addon.transform(markdown, options),
    facade: (markdown, options) => transform(markdown, options),
    name: "transform",
    natives: ["transform", "transformPacked"],
  },
  {
    direct: (markdown, options) => addon.toHtmlWithRenderer(markdown, options, renderCode),
    facade: (markdown, options) =>
      toHtmlWithHighlighter(markdown, highlighter, highlightOptions, options),
    name: "toHtmlWithHighlighter",
    natives: ["toHtmlWithRenderer", "toHtmlWithRendererPacked"],
  },
  {
    direct: (markdown, options) => addon.transformWithRenderer(markdown, options, renderCode),
    facade: (markdown, options) =>
      transformWithHighlighter(markdown, highlighter, highlightOptions, options),
    name: "transformWithHighlighter",
    natives: ["transformWithRenderer", "transformWithRendererPacked"],
  },
];

/** A renderer's first two documents, both as a string and as a Buffer. */
function reuse(renderer, markdown) {
  return [renderer.toHtml(markdown), renderer.toHtmlBuffer(markdown), renderer.toHtml(markdown)];
}

function outcome(run) {
  try {
    return { value: run() };
  } catch (error) {
    return { error };
  }
}

function describeError(error) {
  if (!(error instanceof Error)) return { thrown: error };
  return {
    code: error.code,
    constructor: error.constructor,
    keys: Object.keys(error),
    message: error.message,
  };
}

function assertSameOutcome(actual, expected, label) {
  if ("error" in expected) {
    assert.ok("error" in actual, `${label}: expected ${String(expected.error)}`);
    assert.deepEqual(describeError(actual.error), describeError(expected.error), label);
  } else {
    assert.ok(!("error" in actual), `${label}: threw ${String(actual.error)}`);
    assert.deepEqual(actual.value, expected.value, label);
  }
}

/** Logs every property get on the options object, as napi-rs or the facade makes it. */
function logged(target, log) {
  return new Proxy(target, {
    get(object, key, receiver) {
      log.push(key);
      return Reflect.get(object, key, receiver);
    },
  });
}

/**
 * Runs every entry through the facade and directly, each with its own options
 * object from `make(log)`, and compares results, errors and property gets.
 */
function assertEquivalent(markdown, make, label) {
  for (const entry of entries) {
    const directGets = [];
    const facadeGets = [];
    const expected = outcome(() => entry.direct(markdown, make(directGets)));
    const actual = outcome(() => entry.facade(markdown, make(facadeGets)));
    assertSameOutcome(actual, expected, `${entry.name} with ${label}`);
    assert.deepEqual(facadeGets, directGets, `${entry.name} with ${label}: property gets`);
  }
}

/** Compares a plain options object and a logging proxy over a copy of it. */
function assertEquivalentOptions(markdown, options, label) {
  assertEquivalent(markdown, () => ({ ...options }), label);
  assertEquivalent(markdown, (log) => logged({ ...options }, log), `${label} (proxy)`);
}

// A boxed primitive on purpose: napi-rs rejects it where it takes the primitive.
// oxlint-disable-next-line unicorn/new-for-builtins
const boxed = (value) => Object(value);
const cyclic = { self: undefined };
cyclic.self = cyclic;
// One value of every kind napi-rs tells apart, plus the values the fields
// accept, reject or check after conversion.
const values = [
  undefined,
  null,
  true,
  false,
  0,
  -0,
  1,
  -1,
  2,
  1.5,
  2 ** 31 - 1,
  2 ** 31,
  -(2 ** 31),
  -(2 ** 31) - 1,
  Number.NaN,
  Number.POSITIVE_INFINITY,
  Number.NEGATIVE_INFINITY,
  "",
  "trusted",
  "untrusted",
  "Trusted",
  "trusted\0",
  "/docs",
  "/docs/",
  "docs-",
  "bad prefix",
  "\uD800",
  "Grüße",
  1n,
  Symbol("option"),
  {},
  [],
  [true],
  { toJSON: () => "json" },
  cyclic,
  function named() {},
  () => {},
  boxed(true),
  boxed("trusted"),
  boxed(1),
  new Date(0),
];

/**
 * `Options` as napi-rs generates it into native.d.ts, as [name, type] pairs.
 * napi-rs emits the fields in their declaration order, which is also the
 * order it reads them in.
 */
function declaredFields() {
  const declarations = readFileSync(new URL("../native.d.ts", import.meta.url), "utf8");
  const body = /^export interface Options \{\n([\s\S]*?)^\}/m.exec(declarations)?.[1];
  assert.ok(body, "native.d.ts declares no Options interface");
  return body
    .split("\n")
    .filter((line) => line.trim() !== "" && !/^\s*(?:\/\*\*|\*)/.test(line))
    .map((line) => {
      const member = /^ {2}(\w+)\?: (boolean|number|string)$/.exec(line);
      assert.ok(member, `unexpected Options member in native.d.ts: ${line}`);
      return [member[1], member[2]];
    });
}

/** The keys in the facade's `optionKeys`, the set `validateOptions` accepts. */
function facadeOptionKeys() {
  const source = readFileSync(new URL("../index.mjs", import.meta.url), "utf8");
  const list = /^const optionKeys = new Set\(\[([^\]]*)\]\);$/m.exec(source)?.[1];
  assert.ok(list, "index.mjs declares no optionKeys set");
  return [...list.matchAll(/"(\w+)"/g)].map(([, key]) => key);
}

test("the field lists follow the generated Options declaration", () => {
  const declared = declaredFields();
  const names = declared.map(([key]) => key);
  // The list these tests use: names, order and types.
  assert.deepEqual(fields, declared);

  // The keys `validateOptions` accepts: the same set, as written and as run.
  const accepted = facadeOptionKeys();
  assert.equal(new Set(accepted).size, accepted.length, "optionKeys repeats a key");
  assert.deepEqual(accepted.toSorted(), names.toSorted());
  for (const key of names) {
    assert.doesNotThrow(() => toHtml("", { [key]: undefined }), `optionKeys lacks ${key}`);
  }
});

test("the packed path gets every declared field once, in declaration order", () => {
  const declared = declaredFields();
  const names = declared.map(([key]) => key);
  // A valid value for every field, so napi-rs and the facade read them all.
  const valid = () =>
    Object.fromEntries(
      declared.map(([key, type]) => [
        key,
        key === "renderPolicy" ? "trusted" : { boolean: true, number: 1, string: "" }[type],
      ]),
    );
  for (const entry of entries) {
    const facadeGets = [];
    const directGets = [];
    entry.facade(probe, logged(valid(), facadeGets));
    entry.direct(probe, logged(valid(), directGets));
    assert.deepEqual(directGets, names, `${entry.name}: napi-rs's gets`);
    assert.deepEqual(facadeGets, names, `${entry.name}: the facade's gets`);
  }
});

test("the probe document shows every option", () => {
  // `allowHtml` and `disallowedRawHtml` only matter for trusted rendering,
  // and `tableColumnNames` only with a colgroup.
  const bases = [{}, { renderPolicy: "trusted" }, { tableColgroup: true }];
  const choices = {
    headingIdPrefix: ["", "p-"],
    headingOffset: [0, 1],
    linkBasePath: ["", "/docs"],
    renderPolicy: ["trusted", "untrusted"],
  };
  for (const key of keys) {
    const outputs = new Set();
    for (const base of bases) {
      for (const value of choices[key] ?? [true, false]) {
        outputs.add(addon.toHtml(probe, { ...base, [key]: value }));
      }
    }
    assert.ok(outputs.size > 1, `${key} does not change the probe document`);
  }
});

test("packs every field and value as the object path reads it", () => {
  for (const key of keys) {
    for (const value of values) {
      assertEquivalentOptions(probe, { [key]: value }, `${key}: ${describe(value)}`);
    }
  }
});

test("packs every boolean field in combination with trusted rendering", () => {
  for (const [key, type] of fields) {
    if (type !== "boolean") continue;
    for (const value of [true, false]) {
      for (const renderPolicy of ["trusted", "untrusted"]) {
        assertEquivalentOptions(probe, { renderPolicy, [key]: value }, `${key}: ${value}`);
      }
    }
  }
});

test("packs every field at once", () => {
  const all = {};
  for (const [key, type] of fields) {
    all[key] = { boolean: true, number: 1, string: "" }[type];
  }
  all.renderPolicy = "trusted";
  all.headingIdPrefix = "docs-";
  all.linkBasePath = "/docs";
  assertEquivalentOptions(probe, all, "every field");
  for (const [key, type] of fields) {
    if (type === "boolean") {
      assertEquivalentOptions(probe, { ...all, [key]: false }, `every field, ${key}: false`);
    }
  }
});

const validValues = {
  boolean: [true, false],
  number: [-2, -1, 0, 1, 2],
  renderPolicy: ["trusted", "untrusted"],
  string: ["", "docs-", "/docs", "/docs/"],
};

/** Half the fields set, mostly to valid values and sometimes to any value. */
function randomOptions(random) {
  const pick = (list) => list[Math.floor(random() * list.length)];
  const options = {};
  for (const [key, type] of fields) {
    const roll = random();
    if (roll >= 0.53) {
      options[key] = pick(validValues[key === "renderPolicy" ? key : type]);
    } else if (roll >= 0.5) {
      options[key] = pick(values);
    }
  }
  return options;
}

test("matches random option combinations, valid and invalid", () => {
  const random = mulberry32(0x5e_ed);
  for (let round = 0; round < 400; round++) {
    const options = randomOptions(random);
    assertEquivalentOptions(probe, options, `round ${round}: ${describe(options)}`);
  }
});

test("reports the first problem in napi-rs's order", () => {
  const cases = [
    // Conversion errors in field order, before any later check.
    [{ footnotes: "x", tables: 1 }, /on Options\.tables/],
    [{ mdx: 1, renderPolicy: "bad" }, /on Options\.mdx/],
    [{ headingOffset: 1.5, linkBasePath: 1 }, /on Options\.linkBasePath/],
    // Then renderPolicy, headingOffset and headingIdPrefix, in that order.
    [{ headingOffset: 1.5, renderPolicy: "bad" }, /renderPolicy must be either/],
    [{ headingIdPrefix: "bad prefix", headingOffset: 1.5 }, /headingOffset must be an integer/],
    [{ headingIdPrefix: "bad prefix", tables: false }, /heading ID prefixes/i],
  ];
  for (const [options, message] of cases) {
    assertEquivalentOptions(probe, options, describe(options));
    for (const entry of entries) {
      assert.throws(() => entry.facade(probe, { ...options }), message, entry.name);
    }
  }
});

test("rejects Markdown that is not a string before reading any option", () => {
  for (const markdown of [undefined, null, 1, {}, boxed("# text")]) {
    for (const options of [{}, { tables: 1 }, { renderPolicy: "bad" }, { superscript: true }]) {
      assertEquivalent(
        markdown,
        (log) => logged({ ...options }, log),
        `${describe(markdown)} Markdown`,
      );
      for (const entry of entries.filter(({ name }) => name !== "Renderer")) {
        const gets = [];
        assert.throws(
          () => entry.facade(markdown, logged({ ...options }, gets)),
          /string/i,
          entry.name,
        );
        assert.deepEqual(gets, [], `${entry.name} read options for invalid Markdown`);
      }
    }
  }
});

test("rejects Markdown that is neither a string nor a Uint8Array before reading any option", () => {
  const markdowns = [
    new Uint8ClampedArray(Buffer.from("# text")),
    new Int8Array(Buffer.from("# text")),
    new DataView(Buffer.from("# text").buffer),
    Buffer.from("# text").buffer,
    [...Buffer.from("# text")],
    new Proxy(Buffer.from("# text"), {}),
  ];
  for (const markdown of markdowns) {
    for (const options of [{}, { tables: 1 }, { renderPolicy: "bad" }, { superscript: true }]) {
      assertEquivalent(
        markdown,
        (log) => logged({ ...options }, log),
        `${Object.prototype.toString.call(markdown)} Markdown`,
      );
      for (const entry of entries.filter(({ name }) => name !== "Renderer")) {
        const gets = [];
        assert.throws(
          () => entry.facade(markdown, logged({ ...options }, gets)),
          (error) => error instanceof TypeError && error.code === "ERR_INVALID_ARG_TYPE",
          entry.name,
        );
        assert.deepEqual(gets, [], `${entry.name} read options for invalid Markdown`);
      }
    }
  }
});

test("packs options for Uint8Array Markdown as the object path reads them", () => {
  // The same Markdown as bytes, in memory the options cannot reach.
  const bytes = () => Buffer.from(probe);
  for (const key of keys) {
    for (const value of [undefined, true, false, 1, 1.5, "trusted", "bad prefix", "/docs", {}]) {
      assertEquivalent(bytes(), () => ({ [key]: value }), `bytes, ${key}: ${describe(value)}`);
    }
  }
  const random = mulberry32(0xb7_7e);
  for (let round = 0; round < 100; round++) {
    const options = randomOptions(random);
    assertEquivalentOptions(bytes(), options, `bytes, round ${round}: ${describe(options)}`);
  }
});

/** `text` in a length-tracking view of a resizable buffer. */
function resizable(text) {
  const bytes = new Uint8Array(new ArrayBuffer(text.length, { maxByteLength: 64 }));
  bytes.set(Buffer.from(text));
  return bytes;
}

test("reads Uint8Array Markdown only after the option getters ran", () => {
  // An object-taking export converts Markdown first and options second, and
  // the options' getters are JavaScript that can write, detach or resize the
  // bytes. The addon reads them when it starts to render, so it renders what
  // the getter left behind, or empty input for a detached or emptied buffer.
  const cases = [
    ["overwrite", () => Buffer.from("# Before"), (bytes) => bytes.write("# After!"), "# After!"],
    [
      "overwrite with invalid UTF-8",
      () => Buffer.from("# Before"),
      (b) => b.fill(0xff, 2),
      "# \uFFFD".padEnd(8, "\uFFFD"),
    ],
    ["detach", () => new Uint8Array(Buffer.from("# Before")), (b) => b.buffer.transfer(), ""],
    ["shrink", () => resizable("# Before"), (b) => b.buffer.resize(3), "# B"],
    [
      "grow",
      () => resizable("# Before"),
      (b) => {
        b.buffer.resize(12);
        b.set(Buffer.from(" now"), 8);
      },
      "# Before now",
    ],
  ];
  for (const entry of entries.filter(({ natives }) => natives)) {
    for (const [label, create, change, text] of cases) {
      const bytes = create();
      const options = {
        get superscript() {
          change(bytes);
          return true;
        },
      };
      assert.deepEqual(
        outcome(() => entry.direct(bytes, options)),
        outcome(() => entry.direct(text, { superscript: true })),
        `${entry.name}: ${label}`,
      );
    }
  }
});

test("rejects unknown keys before Markdown and before any get", () => {
  for (const entry of entries) {
    const gets = [];
    assert.throws(
      () => entry.facade(123, logged({ superscript: true, taskList: true }, gets)),
      (error) => error instanceof TypeError && error.message === 'unknown option "taskList"',
      entry.name,
    );
    assert.deepEqual(gets, [], entry.name);
  }
});

test("treats absent options as no options", () => {
  for (const options of [undefined, null]) {
    assertEquivalent(probe, () => options, String(options));
  }
});

test("reads inherited properties like the object path", () => {
  const prototype = { renderPolicy: "trusted", superscript: true, headingIdPrefix: "base-" };
  assertEquivalent(probe, () => Object.create(prototype), "inherited fields");
  assertEquivalent(
    probe,
    () => Object.assign(Object.create(prototype), { superscript: false }),
    "a shadowed field",
  );
  assertEquivalent(probe, () => Object.create(null), "a null prototype");
  assertEquivalent(
    probe,
    () => Object.create(Object.create(null, { tables: { value: 1 } })),
    "an inherited invalid field",
  );

  class Settings {
    get mdx() {
      return true;
    }
  }
  assertEquivalent(probe, () => new Settings(), "a class getter");
});

test("reads options inherited from Object.prototype like the object path", () => {
  const cases = [
    ["superscript", true],
    ["tables", 1],
    ["renderPolicy", "bad"],
  ];
  for (const [key, value] of cases) {
    // Restored below; napi-rs reads inherited fields, so the facade must too.
    // eslint-disable-next-line no-extend-native
    Object.defineProperty(Object.prototype, key, { configurable: true, value, writable: true });
    try {
      assertEquivalent(probe, () => ({}), `Object.prototype.${key}`);
      assertEquivalent(probe, () => ({ headingOffset: 1.5 }), `Object.prototype.${key} and more`);
      // Inherits nothing, so only its own rejected value may decide the error.
      assertEquivalent(
        probe,
        () => Object.assign(Object.create(null), { mdx: 1 }),
        `Object.prototype.${key} and a null prototype`,
      );
    } finally {
      delete Object.prototype[key];
    }
  }
});

test("runs getters once each, in napi-rs's order, and stops where napi-rs stops", () => {
  const withGetters = (log, overrides = {}) => {
    const options = {};
    for (const key of keys) {
      Object.defineProperty(options, key, {
        enumerable: true,
        get() {
          log.push(key);
          return overrides[key];
        },
      });
    }
    return options;
  };
  assertEquivalent(probe, (log) => withGetters(log), "getters");
  assertEquivalent(probe, (log) => withGetters(log, { superscript: true }), "getter values");
  assertEquivalent(probe, (log) => withGetters(log, { math: 1 }), "a rejected getter value");
  assertEquivalent(
    probe,
    (log) => withGetters(log, { renderPolicy: "bad" }),
    "an unknown renderPolicy from a getter",
  );

  // A getter that returns a different value on every call.
  assertEquivalent(
    probe,
    (log) => {
      let calls = 0;
      return Object.defineProperty({}, "superscript", {
        get() {
          log.push("superscript");
          calls += 1;
          return calls === 1;
        },
      });
    },
    "a changing getter",
  );
});

test("propagates a throwing getter or proxy trap unchanged", () => {
  const sentinel = new Error("getter failed");
  const thrown = Symbol("thrown");
  for (const key of ["renderPolicy", "tables", "headingOffset", "linkBasePath"]) {
    assertEquivalent(
      probe,
      (log) =>
        logged(
          Object.defineProperty({ superscript: true }, key, {
            get() {
              throw sentinel;
            },
          }),
          log,
        ),
      `a throwing ${key} getter`,
    );
    assertEquivalent(
      probe,
      (log) =>
        new Proxy(
          {},
          {
            get(target, property) {
              log.push(property);
              if (property === key) throw thrown;
              return Reflect.get(target, property);
            },
          },
        ),
      `a proxy throwing on ${key}`,
    );
    for (const entry of entries) {
      assert.throws(
        () =>
          entry.facade(
            probe,
            Object.defineProperty({}, key, {
              get() {
                throw sentinel;
              },
            }),
          ),
        (error) => error === sentinel,
        entry.name,
      );
    }
  }
});

test("formats a rejected value only as the object path does", () => {
  for (const key of ["renderPolicy", "headingIdPrefix", "tables", "headingOffset"]) {
    const calls = { direct: 0, facade: 0 };
    for (const entry of entries) {
      for (const path of ["direct", "facade"]) {
        const value = {
          toJSON() {
            calls[path] += 1;
            return "value";
          },
        };
        assert.throws(() => entry[path](probe, { [key]: value }), entry.name);
      }
    }
    assert.deepEqual(calls.facade, calls.direct, key);
  }
});

/** A function that passes as options: without its own `length` and `name`. */
function functionOptions() {
  // A fresh function object each call, without a nested definition.
  const options = String.bind(String);
  delete options.length;
  delete options.name;
  options.superscript = true;
  return options;
}

test("accepts functions as options objects like the object path", () => {
  assertEquivalent(probe, functionOptions, "a function");
});

/** Spies on native entries for one test; every spy calls through. */
function spyOn(t, names) {
  const spies = Object.fromEntries(names.map((name) => [name, t.mock.method(addon, name)]));
  return {
    args: (name) => spies[name].mock.calls[0]?.arguments ?? [],
    count: (name) => spies[name].mock.callCount(),
    reset() {
      for (const spy of Object.values(spies)) spy.mock.resetCalls();
    },
  };
}

/** Options go to the packed entry, and absent options to the object entry. */
function assertPacked(entry, spies) {
  const [object, packed] = entry.natives;
  spies.reset();
  entry.facade(probe, { superscript: true });
  assert.deepEqual([spies.count(packed), spies.count(object)], [1, 0], `${entry.name} packs`);
  spies.reset();
  entry.facade(probe);
  assert.deepEqual([spies.count(packed), spies.count(object)], [0, 1], `${entry.name} bare`);
}

/** A rejected value and invalid Markdown go to the object entry. */
function assertObjectPath(entry, spies) {
  const [object, packed] = entry.natives;
  const options = { superscript: true, tables: 1 };
  spies.reset();
  assert.throws(() => entry.facade(probe, options));
  assert.equal(spies.count(packed), 0, `${entry.name} with a rejected value`);
  const [, rejected] = spies.args(object);
  assert.equal(Object.getPrototypeOf(rejected), null, `${entry.name} rejected prototype`);
  assert.deepEqual(Object.entries(rejected), [["tables", 1]], `${entry.name} rejected value`);
  spies.reset();
  assert.throws(() => entry.facade(1, options));
  assert.equal(spies.args(object)[1], options, `${entry.name} with invalid Markdown`);
}

test("routes options through the packed entries", (t) => {
  const routed = entries.filter(({ natives }) => natives);
  const spies = spyOn(
    t,
    routed.flatMap(({ natives }) => natives),
  );
  for (const entry of routed) {
    assertPacked(entry, spies);
    assertObjectPath(entry, spies);
  }
});

test("routes Uint8Array Markdown through the packed entries", (t) => {
  const routed = entries.filter(({ natives }) => natives);
  const spies = spyOn(
    t,
    routed.flatMap(({ natives }) => natives),
  );
  for (const bytes of [Buffer.from(probe), new Uint8Array(Buffer.from(probe))]) {
    for (const entry of routed) {
      const [object, packed] = entry.natives;
      spies.reset();
      entry.facade(bytes, { superscript: true });
      assert.deepEqual([spies.count(packed), spies.count(object)], [1, 0], `${entry.name} packs`);
      assert.equal(spies.args(packed)[0], bytes, `${entry.name} passes the bytes on`);
      spies.reset();
      entry.facade(bytes);
      assert.deepEqual([spies.count(packed), spies.count(object)], [0, 1], `${entry.name} bare`);
    }
  }
});

test("reads options in JavaScript for every entry, including Renderer", () => {
  for (const entry of entries) {
    let stack = "";
    entry.facade(
      probe,
      Object.defineProperty({}, "superscript", {
        get() {
          stack = new Error("trace").stack ?? "";
          return true;
        },
      }),
    );
    assert.match(stack, /PackedOptions\.read/, entry.name);
  }
});

function describe(value) {
  if (typeof value === "string") return JSON.stringify(value);
  if (typeof value === "bigint") return `${value}n`;
  if (typeof value === "symbol" || typeof value === "function") return String(value);
  if (value === cyclic) return "cyclic";
  if (value && typeof value === "object") {
    return `{ ${Object.entries(value)
      .map(([key, entry]) => `${key}: ${describe(entry)}`)
      .join(", ")} }`;
  }
  return Object.is(value, -0) ? "-0" : String(value);
}

/** A small seeded generator, so a failing combination reproduces. */
function mulberry32(seed) {
  let state = seed;
  return () => {
    state = (state + 0x6d_2b_79_f5) | 0;
    let value = Math.imul(state ^ (state >>> 15), 1 | state);
    value = (value + Math.imul(value ^ (value >>> 7), 61 | value)) ^ value;
    return ((value ^ (value >>> 14)) >>> 0) / 4_294_967_296;
  };
}
