import assert from "node:assert/strict";
import test from "node:test";

import { Renderer, toHtml, transform } from "../index.mjs";

const source = [
  "![Pipeline](pipeline.svg){.diagram}",
  "",
  ": The processing pipeline",
  "",
  "> A memorable passage.",
  ": Jane Doe",
  "",
  "We ++added++ this ==today== with an API and x^2^.",
  "",
  "// A source-only note",
  "Term",
  ": Definition",
  "",
].join("\n");

test("the ffm preset enables Ferromark Flavored Markdown syntax", () => {
  const html = toHtml(source, { preset: "ffm" });
  assert.match(html, /<figure>\n<img src="pipeline\.svg" alt="Pipeline" class="diagram">/);
  assert.match(html, /<figcaption>The processing pipeline<\/figcaption>/);
  assert.match(html, /<figcaption>Jane Doe<\/figcaption>/);
  assert.match(html, /<ins>added<\/ins>/);
  assert.match(html, /<mark>today<\/mark>/);
  assert.match(html, /<sup>2<\/sup>/);
  assert.match(html, /<abbr title="Application Programming Interface">API<\/abbr>/);
  assert.match(html, /<dt>Term<\/dt>/);
  assert.doesNotMatch(html, /source-only note/);
});

test("without the preset the same document keeps the Node defaults", () => {
  const html = toHtml(source);
  assert.doesNotMatch(html, /<figure>|<ins>|<mark>|<abbr|<sup>|<dl/);
  assert.match(html, /source-only note/);
});

test("individual options override the preset", () => {
  const html = toHtml(source, { preset: "ffm", insertions: false, autoAbbreviations: false });
  assert.match(html, /\+\+added\+\+/);
  assert.doesNotMatch(html, /<abbr/);
  assert.match(html, /<mark>today<\/mark>/);
});

test("the preset keeps the Node package's safe output and autolink defaults", () => {
  const html = toHtml("<b>raw</b> www.example.com", { preset: "ffm" });
  assert.match(html, /&lt;b&gt;raw&lt;\/b&gt;/);
  assert.doesNotMatch(html, /<a /);
});

test("every entry point honors the preset", () => {
  const expected = toHtml(source, { preset: "ffm" });
  assert.equal(new Renderer({ preset: "ffm" }).toHtml(source), expected);
  assert.equal(transform(source, { preset: "ffm" }).html, expected);
});

test("an unknown preset is rejected", () => {
  assert.throws(() => toHtml(source, { preset: "gfm" }), /preset must be 'ffm'/);
  assert.throws(() => toHtml(source, { preset: 1 }), /preset/);
});
