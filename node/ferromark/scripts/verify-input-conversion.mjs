// Compares the addon's single-pass Markdown conversion (`Utf8Input`) with
// napi-rs's `String` conversion, which the exports used before, on strings
// chosen to hit every representation and edge case. Both conversions come from
// the `boundary-bench` diagnostic exports, so this builds a throwaway addon
// with that feature, as verify-panic-unwind.mjs does with `panic-test`.
//
// With `--addon <file>` it only runs the comparison against that addon. The
// build step runs it that way in a child process, so the addon is never loaded
// into the process that deletes it (Windows cannot delete a loaded DLL).

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
    assert.equal(
      check.status,
      0,
      `the single-pass input conversion differs from napi-rs's String conversion:\n${check.stderr}`,
    );
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

const nonStrings = [
  123,
  -0,
  Number.NaN,
  true,
  null,
  undefined,
  {},
  { markdown: "x" },
  [1, "x"],
  function named() {},
  () => {},
  Symbol("s"),
  10n,
  // A String object, which is not a string.
  new Object("x"),
  new Date(0),
];

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
    assert.equal(addon.boundaryLen(text), napiString.length, label);
    assert.ok(addon.boundaryEcho(text) === text.toWellFormed(), `round trip: ${label}`);
  }
  assert.ok(stored["one-byte"] > 0 && stored["two-byte"] > 0, "both storages are covered");
  return stored;
}

/** A value that is not a string fails with napi-rs's own error. */
function compareErrors(addon) {
  const cyclic = {};
  cyclic.self = cyclic;
  const values = [...nonStrings, cyclic];
  for (const value of values) {
    const expected = thrown(() => addon.boundaryNapiStringBytes(value));
    assert.deepEqual(
      thrown(() => addon.boundaryInputBytes(value)),
      expected,
    );
  }
  // A missing argument arrives as `undefined`.
  assert.deepEqual(
    thrown(() => addon.boundaryInputBytes()),
    thrown(() => addon.boundaryNapiStringBytes()),
  );
  return values.length + 1;
}

function compare(addon) {
  const stored = compareStrings(addon);
  const nonStringCount = compareErrors(addon);
  console.log(
    `The single-pass input conversion matches napi-rs's String conversion ` +
      `(${stored["one-byte"]} one-byte and ${stored["two-byte"]} two-byte strings, ` +
      `${nonStringCount} non-strings).`,
  );
}

/** The error a call throws, as comparable fields. */
function thrown(call) {
  try {
    call();
  } catch (error) {
    assert.ok(error instanceof Error);
    return { code: "code" in error ? error.code : undefined, message: error.message };
  }
  assert.fail("expected the call to throw");
}

const { values } = parseArgs({ options: { addon: { type: "string" } } });

if (values.addon === undefined) {
  await buildAndCompare();
} else {
  compare(createRequire(import.meta.url)(values.addon));
}
