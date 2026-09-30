import assert from "node:assert/strict";
import test from "node:test";
import { createNodeRender } from "./node-adapters.mjs";

for (const engine of ["marked", "markdown-it", "remark", "showdown", "commonmark"]) {
  test(`${engine}: retained configuration parses each call without reference state leaking`, async () => {
    const render = await createNodeRender(engine, false);
    const first = render("[x][ref]\n\n[ref]: /first\n");
    assert.match(first, /href="\/first"/);
    assert.match(render("[x][ref]\n"), /\[x\]\[ref\]/);
    assert.equal(render("[x][ref]\n\n[ref]: /first\n"), first);
    assert.doesNotMatch(render("# Title\n"), / id=/);
    assert.match(render("<script>raw()</script>\n"), /<script>raw\(\)<\/script>/);
    assert.doesNotMatch(render("https://example.com\n"), /<a /);
    assert.match(render("```txt\nx  y\n\n z\n```\n"), /x  y\n\n z\n/);
  });
}

test("commonmark.js cannot silently enable unsupported GFM", async () => {
  await assert.rejects(createNodeRender("commonmark", true), /CommonMark-only/);
});
