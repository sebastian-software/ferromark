import assert from "node:assert/strict";
import test from "node:test";

import {
  Renderer,
  toHtml,
  toHtmlBuffer,
  toHtmlWithHighlighter,
  transform,
  transformWithHighlighter,
} from "../index.mjs";

const options = {
  autolinkLiterals: true,
  autoAbbreviations: true,
  abbreviations: {
    API: "Custom & <API>",
    GraphQL: "",
    XYZ: null,
  },
};

const source =
  "# API\n\nAPI HTTP HTML5 UTF8 XYZ GraphQL and **TCP**. " +
  "HTTP/2 UTF-8 APIs API-like xAPI APIx.\n\n\u0060API\u0060 [API](/API) ![API](/image) " +
  "https://example.com/API\n";

test("automatic abbreviation recognition uses titles, bare wrappers and overrides", () => {
  assert.equal(
    toHtml(source, options),
    [
      '<h1 id="api"><abbr title="Custom &amp; &lt;API&gt;">API</abbr></h1>',
      '<p><abbr title="Custom &amp; &lt;API&gt;">API</abbr> <abbr title="Hypertext Transfer Protocol">HTTP</abbr> <abbr title="Hypertext Markup Language, version 5">HTML5</abbr> <abbr title="Unicode Transformation Format, 8-bit">UTF8</abbr> XYZ <abbr>GraphQL</abbr> and <strong><abbr title="Transmission Control Protocol">TCP</abbr></strong>. <abbr title="Hypertext Transfer Protocol, version 2">HTTP/2</abbr> <abbr title="Unicode Transformation Format, 8-bit">UTF-8</abbr> <abbr title="Custom &amp; &lt;API&gt;">API</abbr>s <abbr title="Custom &amp; &lt;API&gt;">API</abbr>-like xAPI APIx.</p>',
      '<p><code>API</code> <a href="/API"><abbr title="Custom &amp; &lt;API&gt;">API</abbr></a> <img src="/image" alt="API"> <a href="https://example.com/API">https://example.com/API</a></p>',
      "",
    ].join("\n"),
  );
});

test("an override map alone does not enable automatic markup", () => {
  assert.equal(toHtml("API GraphQL", { abbreviations: { API: "Custom" } }), "<p>API GraphQL</p>\n");
});

test("trusted raw HTML remains untouched and its text is not wrapped", () => {
  assert.equal(
    toHtml('<abbr title="API">API <strong>HTTP</strong></abbr> <span>XYZ</span>', {
      autoAbbreviations: true,
      renderPolicy: "trusted",
    }),
    '<p><abbr title="API">API <strong>HTTP</strong></abbr> <span>XYZ</span></p>\n',
  );
});

test("untrusted raw HTML text stays literal while later prose is annotated", () => {
  assert.equal(
    toHtml('<abbr title="authored">API</abbr> API', { autoAbbreviations: true }),
    '<p>&lt;abbr title=&quot;authored&quot;&gt;API&lt;/abbr&gt; <abbr title="Application Programming Interface">API</abbr></p>\n',
  );
});

test("all Node rendering entry points share the same opt-in behavior", () => {
  const expected = toHtml("# API\n\nXYZ API", options);
  assert.equal(toHtmlBuffer("# API\n\nXYZ API", options).toString("utf8"), expected);

  const renderer = new Renderer(options);
  assert.equal(renderer.toHtml("# API\n\nXYZ API"), expected);
  assert.equal(renderer.toHtmlBuffer("# API\n\nXYZ API").toString("utf8"), expected);
  assert.equal(renderer.toHtml("XYZ"), "<p>XYZ</p>\n");

  const transformed = transform("# API\n\nXYZ API", options);
  assert.equal(transformed.html, expected);
  assert.deepEqual(transformed.headings, [{ level: 1, id: "api", text: "API" }]);
});

test("highlighter helpers apply abbreviations to prose but not code", () => {
  const highlighter = {
    codeToHtml: (code) => `<pre>${code}</pre>`,
  };
  const markdown = "API\n\n\u0060\u0060\u0060rust\nHTTP\n\u0060\u0060\u0060\n";
  const expected = '<p><abbr title="Custom &amp; &lt;API&gt;">API</abbr></p>\n<pre>HTTP\n</pre>';
  assert.equal(toHtmlWithHighlighter(markdown, highlighter, { theme: "test" }, options), expected);
  assert.equal(
    transformWithHighlighter(markdown, highlighter, { theme: "test" }, options).html,
    expected,
  );
});

test("MDX attributes and abbreviation elements stay outside matching", () => {
  const html = toHtml('<abbr title="API">API</abbr> <Badge label="API">HTTP</Badge>', {
    autoAbbreviations: true,
    mdx: true,
    renderPolicy: "trusted",
  });
  assert.ok(html.startsWith('<p><abbr title="API">API</abbr> '));
  assert.ok(html.includes('"props":{"label":"API"}'));
  assert.ok(html.includes('<abbr title="Hypertext Transfer Protocol">HTTP</abbr>'));
  assert.ok(!html.includes('<abbr title="Application Programming Interface">API</abbr>'));
});
