import assert from "node:assert/strict";
import test from "node:test";
import { createConformanceRender } from "./conformance-adapters.mjs";

test("markdown-it spec profile recognizes www links while CommonMark stays literal", async () => {
  const commonmark = await createConformanceRender("markdown-it", false);
  const gfm = await createConformanceRender("markdown-it", true);
  assert.equal(commonmark("www.commonmark.org\n"), "<p>www.commonmark.org</p>\n");
  assert.equal(
    gfm("www.commonmark.org\n"),
    '<p><a href="http://www.commonmark.org">www.commonmark.org</a></p>\n',
  );
});

test("micromark enables tagfilter only in the GFM profile and preserves code whitespace", async () => {
  const commonmark = await createConformanceRender("micromark", false);
  const gfm = await createConformanceRender("micromark", true);
  const raw = "<title>raw</title>\n";
  assert.match(commonmark(raw), /<title>raw<\/title>/);
  assert.doesNotMatch(gfm(raw), /<title>/);
  assert.match(gfm(raw), /&lt;title>/);
  assert.match(gfm("```txt\na  b\n z\n```\n"), /a  b\n z\n/);
});

test("public remark task-list classes remain visible in measured output", async () => {
  const gfm = await createConformanceRender("remark", true);
  const html = gfm("- [x] Done\n");
  assert.match(html, /class="task-list-item"/);
  assert.match(html, /type="checkbox" checked disabled/);
});

test("CommonMark-only adapters cannot pretend to run the GFM suite", async () => {
  for (const engine of ["commonmark", "remarkable"]) {
    await assert.rejects(createConformanceRender(engine, true), /CommonMark-only/);
  }
});
