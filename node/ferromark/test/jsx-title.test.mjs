import assert from "node:assert/strict";
import { test } from "node:test";

import { compileJsx } from "../index.mjs";

test("title omission reports the skipped source range and reserves no heading ID", () => {
  const source = "# Native\n\n## Native\n\nBody";
  const output = compileJsx(source, { omitTitleHeading: "Native" });
  assert.equal(output.headings.length, 1);
  assert.equal(output.headings[0].id, "native");
  assert.equal(output.headings[0].level, 2);
  const range = output.omittedTitleHeadingSpan;
  assert.ok(range);
  assert.match(Buffer.from(source).subarray(range.start, range.end).toString(), /^# Native/);
});
