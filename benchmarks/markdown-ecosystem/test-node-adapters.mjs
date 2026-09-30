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

for (const engine of [
  "remarkable",
  "markdown-exit",
  "markdown-it-ts",
  "satteri",
  "md4x-napi",
  "md4x-wasm",
  "ox-content-napi",
]) {
  test(`${engine}: alternating documents and Unicode retain no reference state`, async () => {
    const render = await createNodeRender(engine, false);
    const source = "[x][ref]\n\n[ref]: /first\n\n🪐 é 漢字\n";
    const first = render(source);
    assert.match(first, /href="\/first"/);
    assert.match(render("[x][ref]\n"), /\[x\]\[ref\]/);
    assert.equal(render(source), first);
    assert.match(first, /🪐 é 漢字/);
    assert.match(render("```txt\nx  y\n\n z\n```\n"), /x  y\n\n z\n/);
    assert.match(render("<script>raw()</script>\n"), /<script>raw\(\)<\/script>/);
  });
}

for (const engine of ["markdown-exit", "markdown-it-ts", "satteri", "ox-content-napi"]) {
  test(`${engine}: public switches disable tables, tasks, strike, and footnotes`, async () => {
    const source = "| A | B |\n| --- | --- |\n| x | y |\n\n- [x] Done\n\n~~old~~\n";
    const plain = await createNodeRender(engine, false);
    const gfm = await createNodeRender(engine, true);
    assert.doesNotMatch(plain(source), /<table>|type="checkbox"|<(s|del)>/);
    assert.match(gfm(source), /<table>/);
    assert.match(gfm(source), /type="checkbox"/);
    assert.match(gfm(source), /<(s|del)>old<\/(s|del)>/);
    assert.doesNotMatch(gfm("Text[^n]\n\n[^n]: Note\n"), /class="footnotes"/);
  });
}

test("MD4X's explicit NAPI and WASM APIs agree on Unicode, large strings, and fixed extensions", async () => {
  const napi = await createNodeRender("md4x-napi", false);
  const wasm = await createNodeRender("md4x-wasm", false);
  for (const source of [
    "",
    "🪐 é 漢字\n".repeat(10_000),
    "# Title\n",
    "- [x] Done\n\n~~old~~\n\nhttps://example.com\n",
    "Text[^n]\n\n[^n]: Note\n",
    "---\ntitle: metadata\n---\n",
  ]) {
    assert.equal(napi(source), wasm(source));
  }
  assert.doesNotMatch(napi("# Title\n"), / id=/);
  assert.match(napi("- [x] Done\n"), /type="checkbox"/);
  assert.match(wasm("Text[^n]\n\n[^n]: Note\n"), /class="footnotes"/);
  assert.equal(napi("---\ntitle: metadata\n---\n"), "");
});

test("unavoidable public defaults remain visible", async () => {
  const satteri = await createNodeRender("satteri", true);
  assert.match(satteri("https://example.com\n"), /<a /);
  const ox = await createNodeRender("ox-content-napi", false);
  assert.match(ox("# Title\n"), /id="title"/);
  assert.match(ox("https://example.com\n"), /target="_blank"/);
  assert.match(ox("> [!NOTE]\n> Note\n"), /ox-callout/);
  await assert.rejects(createNodeRender("remarkable", true), /CommonMark-only/);
});

test("TanStack Markdown measures complete public rendering with its fixed subset semantics", async () => {
  const render = await createNodeRender("tanstack-markdown", false);
  const gfmRender = await createNodeRender("tanstack-markdown", true);
  const input = "| A | B |\n| --- | --- |\n| x | y |\n\n- [x] Done\n\n~~old~~\n";
  assert.equal(render(input), gfmRender(input));
  assert.match(render(input), /<table>/);
  assert.match(render(input), /type="checkbox"/);
  assert.match(render(input), /<del>old<\/del>/);
  assert.doesNotMatch(render("# Title\n"), / id=/);
  assert.match(render("---\ntitle: Visible\n---\n"), /title: Visible/);
  assert.match(render("<script>raw()<\/script>\n"), /<script>raw\(\)<\/script>/);
  assert.match(render("[x](javascript:alert)\n"), /href="javascript:alert"/);
  assert.match(render("Text[^n]\n\n[^n]: note\n"), /data-footnotes/);
  assert.match(render("[x][ref]\n\n[ref]: /first\n"), /href="\/first"/);
  assert.match(render("[x][ref]\n"), /\[x\]\[ref\]/);
  const unicode = "🪐 é 漢字 ".repeat(2048);
  const first = render(unicode);
  assert.match(first, /🪐 é 漢字/);
  render("different input");
  assert.equal(render(unicode), first);
});
