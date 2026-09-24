// Verifies how the addon converts a Markdown argument (`Utf8Input` and
// `OwnedUtf8Input` in node/native/src/input.rs), through the `boundary-bench`
// diagnostic exports:
//
// - Strings: the single-pass conversion returns the bytes of napi-rs's
//   `String` conversion, which the exports used before, and of `Buffer.from`,
//   on strings chosen to hit every representation and edge case.
// - UTF-8 bytes: both input types return the text `Buffer#toString('utf8')`
//   makes of the bytes, on every sequence of up to three bytes, on four-byte
//   sequences across all byte classes, on truncated sequences and on seeded
//   random input, in every form of `Uint8Array`. Valid bytes of a plain
//   `ArrayBuffer` are borrowed; shared and invalid bytes are not.
// - Anything else: the `TypeError` names the value's kind and runs none of
//   its code.
//
// This builds a throwaway addon with the feature, as verify-panic-unwind.mjs
// does with `panic-test`. With `--addon <file>` it only runs the checks against
// that addon. The build step runs it that way in a child process, so the addon
// is never loaded into the process that deletes it (Windows cannot delete a
// loaded DLL).

import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtemp, readdir, rm } from "node:fs/promises";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { parseArgs } from "node:util";
import { serialize } from "node:v8";

async function buildAndCompare() {
  const buildScript = join(import.meta.dirname, "build-native.mjs");
  const outputDir = await mkdtemp(join(tmpdir(), "ferromark-input-conversion-"));
  try {
    const build = spawnSync(process.execPath, [buildScript], {
      cwd: import.meta.dirname,
      encoding: "utf8",
      env: {
        ...process.env,
        FERROMARK_NAPI_FEATURES: "boundary-bench",
        FERROMARK_NATIVE_OUTPUT_DIR: outputDir,
      },
    });
    assert.ifError(build.error);
    assert.equal(
      build.status,
      0,
      `Could not build the input-conversion verification addon:\n${build.stderr}`,
    );

    const outputFiles = await readdir(outputDir);
    const addon = outputFiles.find((file) => file.endsWith(".node"));
    assert.ok(addon, "the input-conversion verification build did not produce a .node addon");

    const check = spawnSync(
      process.execPath,
      [import.meta.filename, "--addon", join(outputDir, addon)],
      { encoding: "utf8" },
    );
    assert.ifError(check.error);
    assert.equal(check.status, 0, `the Markdown input conversion failed a check:\n${check.stderr}`);
    process.stdout.write(check.stdout);
  } finally {
    await rm(outputDir, { force: true, recursive: true });
  }
}

/** V8's storage for a string: its serializer tags one-byte strings '"'. */
function storage(text) {
  return serialize(text)[2] === 0x22 ? "one-byte" : "two-byte";
}

/** The same characters stored as two-byte (from 13 characters on). */
function widen(text) {
  return `${text}Ā`.slice(0, -1);
}

/** Strings covering both storages, every UTF-8 width and lone surrogates. */
function strings() {
  const cases = [
    "",
    "a",
    "a\0b",
    "\uFFFD\uFFFE\uFFFF",
    "\uD800\uDC00\uDBFF\uDFFF",
    widen("plain ASCII stored as two-byte"),
    widen("Grüße aus Köln, stored as two-byte"),
    // Node.js decodes large buffers into external strings.
    Buffer.alloc(1 << 20, 0xe9).toString("latin1"),
    Buffer.alloc(2 << 20, 0x4e).toString("utf16le"),
    Buffer.alloc(1 << 20, 0xd8).toString("utf16le"),
  ];
  // Every UTF-16 code unit alone, between ASCII and next to each surrogate
  // kind: this covers each UTF-8 width and every lone surrogate.
  for (let unit = 0; unit <= 0xff_ff; unit++) {
    const char = String.fromCharCode(unit);
    cases.push(char, `a${char}b`, `\uD800${char}`, `${char}\uDC00`);
  }
  // Runs of each width, around small buffer sizes and at a few MiB, where the
  // lone surrogate and three-byte runs fill the reservation's bound exactly.
  const widths = ["x", "é", "€", "😀", "\uD800", "\uDC00", "\uDC00\uD800", "x€\uD800é😀"];
  const lengths = [1, 2, 3, 4, 5, 7, 8, 13, 15, 16, 17, 31, 32, 33, 64, 127, 128, 1024, 4099];
  for (const width of widths) {
    for (const length of [...lengths, 1 << 20]) {
      cases.push(width.repeat(length), `${width.repeat(length)}\uD83D`);
    }
  }
  // Unflattened concatenations and slices of both storages.
  cases.push(`${"a".repeat(100)}${"世".repeat(100)}`, "x€\uD800".repeat(5000).slice(1, -1));
  return cases;
}

/** Both conversions return the same bytes as `Buffer.from` for every string. */
function compareStrings(addon) {
  const stored = { "one-byte": 0, "two-byte": 0 };
  for (const text of strings()) {
    stored[storage(text)]++;
    const label = `${text.length} units starting ${JSON.stringify(text.slice(0, 8))}`;
    const napiString = addon.boundaryNapiStringBytes(text);
    const singlePass = addon.boundaryInputBytes(text);
    assert.ok(singlePass.equals(napiString), `bytes differ from napi-rs's String: ${label}`);
    assert.ok(singlePass.equals(Buffer.from(text, "utf8")), `bytes differ from Buffer: ${label}`);
    assert.ok(addon.boundaryOwnedInputBytes(text).equals(singlePass), `owned: ${label}`);
    assert.equal(addon.boundaryLen(text), napiString.length, label);
    assert.ok(addon.boundaryEcho(text) === text.toWellFormed(), `round trip: ${label}`);
  }
  assert.ok(stored["one-byte"] > 0 && stored["two-byte"] > 0, "both storages are covered");
  return stored;
}

// Byte classes of UTF-8: ASCII, each continuation range a lead byte accepts,
// each lead byte range and the bytes that never occur.
const classes = [
  0x00, 0x41, 0x7f, 0x80, 0x8f, 0x90, 0x9f, 0xa0, 0xbf, 0xc0, 0xc1, 0xc2, 0xdf, 0xe0, 0xed, 0xef,
  0xf0, 0xf4, 0xf5, 0xff,
];

const hex = (byte) => `0x${byte.toString(16).padStart(2, "0")}`;

/** A buffer of exactly `length` bytes, which `write` fills through `put(...bytes)`. */
function records(length, write) {
  const buffer = Buffer.alloc(length);
  let offset = 0;
  write((...bytes) => {
    for (const byte of bytes) buffer[offset++] = byte;
  });
  assert.equal(offset, length);
  return buffer;
}

/**
 * Byte strings that cover every way UTF-8 can be invalid, as [label, bytes].
 * Separated records start from the decoder's initial state; records without a
 * separator carry an incomplete sequence into the next one.
 */
function* byteStrings() {
  yield [
    "every one- and two-byte sequence",
    records(256 * 2 + 256 * 256 * 3, (put) => {
      for (let first = 0; first < 256; first++) put(first, 0x20);
      for (let first = 0; first < 256; first++) {
        for (let second = 0; second < 256; second++) put(first, second, 0x20);
      }
    }),
  ];
  // A sequence led by an ASCII byte decodes as that byte and a two-byte
  // sequence, which the records above cover.
  for (let lead = 0x80; lead < 0x1_00; lead++) yield threeByteSequences(lead);
  for (let lead = 0xc0; lead < 0x1_00; lead++) {
    yield fourByteSequences(lead);
    yield backToBack(lead);
  }
  yield* randomByteStrings(mulberry32(0x0f_ff_fd));
}

function threeByteSequences(lead) {
  return [
    `every three-byte sequence led by ${hex(lead)}`,
    records(256 * 256 * 4, (put) => {
      for (let second = 0; second < 256; second++) {
        for (let third = 0; third < 256; third++) put(lead, second, third, 0x20);
      }
    }),
  ];
}

function fourByteSequences(lead) {
  return [
    `four-byte sequences led by ${hex(lead)}, any second byte, classes after`,
    records(256 * classes.length ** 2 * 5, (put) => {
      for (let second = 0; second < 256; second++) {
        for (const third of classes) {
          for (const fourth of classes) put(lead, second, third, fourth, 0x20);
        }
      }
    }),
  ];
}

function backToBack(lead) {
  return [
    `three-byte sequences led by ${hex(lead)}, back to back`,
    records(256 * 256 * 3, (put) => {
      for (let second = 0; second < 256; second++) {
        for (let third = 0; third < 256; third++) put(lead, second, third);
      }
    }),
  ];
}

/** Seeded random input, mostly bytes at or above 0x80. */
function* randomByteStrings(random) {
  for (let round = 0; round < 32; round++) {
    const bytes = Buffer.alloc(1 << 16);
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

/** Sequences cut short at the very end of the input, where no byte follows. */
function* truncations() {
  for (let lead = 0xc0; lead < 0x1_00; lead++) {
    yield Buffer.from([0x61, lead]);
    for (let second = 0; second < 256; second++) {
      yield Buffer.from([0x61, lead, second]);
      for (const third of classes) yield Buffer.from([0x61, lead, second, third]);
    }
  }
}

// The forms a `Uint8Array` comes in; each copies `bytes` into fresh memory.
const forms = {
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

/** Both input types return what `Buffer#toString('utf8')` makes of the bytes. */
function compareBytes(addon) {
  let checked = 0;
  let replaced = 0;
  const check = (bytes, label, allForms) => {
    const expected = Buffer.from(bytes.toString("utf8"), "utf8");
    const actual = addon.boundaryInputBytes(bytes);
    assert.ok(actual.equals(expected), `bytes differ from Buffer#toString: ${label}`);
    assert.ok(addon.boundaryOwnedInputBytes(bytes).equals(expected), `owned: ${label}`);
    if (allForms) {
      for (const [form, make] of Object.entries(forms)) {
        assert.ok(addon.boundaryInputBytes(make(bytes)).equals(expected), `${form}: ${label}`);
        assert.ok(addon.boundaryOwnedInputBytes(make(bytes)).equals(expected), `${form}: ${label}`);
      }
    }
    checked++;
    if (!expected.equals(bytes)) replaced++;
  };
  let chunk = 0;
  for (const [label, bytes] of byteStrings()) {
    // Every form on every eighth chunk and every random input keeps the run short.
    check(bytes, label, chunk++ % 8 === 0 || label.startsWith("random"));
  }
  for (const bytes of truncations()) check(bytes, `truncated ${bytes.toString("hex")}`, false);
  assert.ok(replaced > checked / 2, "most inputs are invalid");
  return checked;
}

/** Valid bytes of a plain `ArrayBuffer` are borrowed, and nothing else is. */
function compareOrigins(addon) {
  const text = Buffer.from("# Grüße *aus* Köln\n");
  const resizable = new Uint8Array(new ArrayBuffer(text.length, { maxByteLength: 64 }));
  resizable.set(text);
  const growable = new Uint8Array(new SharedArrayBuffer(text.length, { maxByteLength: 64 }));
  growable.set(text);
  const detached = new Uint8Array(text);
  detached.buffer.transfer();
  const cases = [
    ["a string", text.toString(), "string"],
    ["a Buffer", Buffer.from(text), "borrowed"],
    ["a Uint8Array", new Uint8Array(text), "borrowed"],
    ["a subarray", forms.subarray(text), "borrowed"],
    ["a resizable buffer", resizable, "borrowed"],
    ["invalid UTF-8", Buffer.from([0x23, 0x20, 0xff]), "owned"],
    ["a SharedArrayBuffer", forms.SharedArrayBuffer(text), "owned"],
    ["a growable SharedArrayBuffer", growable, "owned"],
    ["no bytes", new Uint8Array(0), "owned"],
    ["a detached buffer", detached, "owned"],
  ];
  for (const [label, markdown, origin] of cases) {
    assert.equal(addon.boundaryInputOrigin(markdown), origin, label);
  }
  return cases.length;
}

function namedFunction() {}

/** The error for a value that is neither a string nor a `Uint8Array`. */
const rejection = (received) => ({
  code: "ERR_INVALID_ARG_TYPE",
  constructor: TypeError,
  message: `markdown must be a string or a Uint8Array, received ${received}`,
});

/** Values that are neither a string nor a `Uint8Array`, and how the error names them. */
function rejectedValues(ran) {
  const cyclic = {};
  cyclic.self = cyclic;
  const loud = {
    toJSON() {
      ran.push("toJSON");
    },
  };
  const named = Object.defineProperty(() => {}, "name", {
    get() {
      ran.push("name");
      return "named";
    },
  });
  const trapped = new Proxy(new Uint8Array(4), {
    get(target, key) {
      ran.push(`get ${String(key)}`);
      return Reflect.get(target, key);
    },
  });
  return [
    [123, "a number"],
    [-0, "a number"],
    [Number.NaN, "a number"],
    [true, "a boolean"],
    [null, "null"],
    [undefined, "undefined"],
    [{}, "an object"],
    [{ markdown: "x" }, "an object"],
    [cyclic, "an object"],
    [loud, "an object"],
    [[1, "x"], "an array"],
    [namedFunction, "a function"],
    [named, "a function"],
    [() => {}, "a function"],
    [Symbol("s"), "a symbol"],
    [10n, "a bigint"],
    // A String object, which is not a string.
    [new Object("x"), "an object"],
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
    [trapped, "an object"],
  ];
}

/** Any other value fails with a `TypeError` that runs none of its code. */
function compareErrors(addon) {
  const ran = [];
  const values = rejectedValues(ran);
  for (const [value, received] of values) {
    for (const convert of [addon.boundaryInputBytes, addon.boundaryOwnedInputBytes]) {
      assert.deepEqual(
        thrown(() => convert(value)),
        rejection(received),
        received,
      );
    }
  }
  // A missing argument arrives as `undefined`.
  assert.deepEqual(
    thrown(() => addon.boundaryInputBytes()),
    rejection("undefined"),
  );
  assert.deepEqual(ran, [], "describing a rejected value ran its code");
  return values.length + 1;
}

function compare(addon) {
  const stored = compareStrings(addon);
  const byteStringCount = compareBytes(addon);
  const originCount = compareOrigins(addon);
  const rejectedCount = compareErrors(addon);
  console.log(
    `The Markdown input conversion matches napi-rs's String conversion ` +
      `(${stored["one-byte"]} one-byte and ${stored["two-byte"]} two-byte strings) and ` +
      `Buffer#toString('utf8') (${byteStringCount} byte strings), borrows only valid bytes ` +
      `of a plain ArrayBuffer (${originCount} kinds of input), and rejects ` +
      `${rejectedCount} other values with a TypeError.`,
  );
}

/** The error a call throws, as comparable fields. */
function thrown(call) {
  try {
    call();
  } catch (error) {
    assert.ok(error instanceof Error);
    return {
      code: "code" in error ? error.code : undefined,
      constructor: error.constructor,
      message: error.message,
    };
  }
  assert.fail("expected the call to throw");
}

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

const { values } = parseArgs({ options: { addon: { type: "string" } } });

if (values.addon === undefined) {
  await buildAndCompare();
} else {
  compare(createRequire(import.meta.url)(values.addon));
}
