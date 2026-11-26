import assert from "node:assert/strict";
import test from "node:test";

import {
  CodeCompiler,
  compileJsx,
  composeCodeHooks,
  createDiagramCodeRenderer,
  createLineHighlighter,
  prepareDocument,
  toHtml,
  toHtmlWithCodeRenderer,
  transformWithCodeRenderer,
} from "../index.mjs";

const fence = '```ts title="a.ts" {1}\nconst a = 1\nlet b\n```\n';

/** @param code Code block source. */
function redLines(code) {
  return code.split("\n").map((line) => (line ? `<span style="color:#f00">${line}</span>` : ""));
}

test("the highlighting stage keeps the code block wrapper, metadata and component", () => {
  const seen = [];
  const hooks = {
    // Code hooks share the public (code, language, meta, block) shape.
    // eslint-disable-next-line max-params
    highlightCodeBlock(code, language, meta, block) {
      seen.push([language, meta, block]);
      return {
        lines: redLines(code),
        foreground: "#111",
        background: "#fff",
        darkForeground: "#eee",
        darkBackground: "#000",
      };
    },
  };

  const html = toHtmlWithCodeRenderer(fence, hooks);
  assert.match(html, /^<pre style="background-color:#fff;color:#111"><code class="language-ts">/);
  assert.match(html, /<span style="color:#f00">let b<\/span>/);

  const { body } = compileJsx(fence, { codeBlockComponent: "CodeBlock" }, hooks);
  assert.match(
    body,
    /<CodeBlock code=\{"const a = 1\\nlet b\\n"\} language=\{"ts"\} title=\{"a.ts"\}/,
  );
  assert.match(
    body,
    /"--shiki-dark": "#eee", "--shiki-light-bg": "#fff", "--shiki-dark-bg": "#000"/,
  );
  assert.match(
    body,
    /ox-code-line--highlight highlighted"\} data-line=\{"1"\}><span style=\{\{color: "#f00"\}\}>/,
  );

  // Both entry points pass the normalized language, raw metadata and UTF-8 block range.
  const location = { start: 0, end: Buffer.byteLength(fence) };
  assert.deepEqual(seen, [
    ["ts", 'title="a.ts" {1}', location],
    ["ts", 'title="a.ts" {1}', location],
  ]);
});

test("a highlighter result with the wrong line count falls back to plain code", () => {
  const hooks = { highlightCodeBlock: () => ({ lines: ["only one"] }) };
  assert.equal(toHtmlWithCodeRenderer(fence, hooks), toHtml(fence));
});

test("whole-block replacements precede highlighting and the first result wins", () => {
  const order = [];
  const hooks = composeCodeHooks(
    (_code, language) => {
      order.push(`render:${language}`);
      return language === "mermaid" ? { html: "<svg></svg>" } : null;
    },
    {
      highlightCodeBlock(code) {
        order.push("highlight:first");
        return { lines: redLines(code) };
      },
    },
    {
      highlightCodeBlock() {
        order.push("highlight:second");
        return null;
      },
    },
  );
  const html = toHtmlWithCodeRenderer("```mermaid\nA\n```\n\n```js\nx\n```\n", hooks);
  assert.match(html, /^<svg><\/svg>/);
  assert.match(html, /<span style="color:#f00">x<\/span>/);
  assert.deepEqual(order, ["render:mermaid", "render:js", "highlight:first"]);
});

test("{ html } replacements become JSX, with case-preserving SVG attributes", () => {
  const svg =
    '<svg viewBox="0 0 1 1" class="d"><marker markerWidth="3" refX="1"/>' +
    '<path stroke-width="2" xlink:href="#a" data-x="1" style="fill:red;stroke-dasharray:2"/>' +
    "<style>.a>b{x:1}</style><text>a &amp; b</text></svg>";
  const { body } = compileJsx("```mermaid\nx\n```\n", {}, () => ({ html: svg }));
  assert.equal(
    body,
    '<>\n<svg viewBox="0 0 1 1" className="d"><marker markerWidth="3" refX="1" />' +
      '<path strokeWidth="2" xlinkHref="#a" data-x="1" style={{fill: "red", strokeDasharray: "2"}} />' +
      '<style>{".a>b{x:1}"}</style><text>{"a & b"}</text></svg>\n</>',
  );
  assert.match(
    toHtmlWithCodeRenderer("```mermaid\nx\n```\n", () => ({ html: svg })),
    /^<svg /,
  );
});

test("diagram renderers work in HTML, JSX and prepared documents", () => {
  const renderer = {
    render(_text, options) {
      return `<svg id="${options.id}"><g stroke-width="1"></g></svg>`;
    },
  };
  const diagrams = createDiagramCodeRenderer(renderer);
  const source = "Intro\n\n```mermaid\nA --> B\n```\n";
  assert.match(toHtmlWithCodeRenderer(source, diagrams), /<svg id="fm-mermaid-7">/);
  assert.match(compileJsx(source, {}, diagrams).body, /<svg id="fm-mermaid-7"><g strokeWidth="1">/);
  const prepared = prepareDocument(source);
  assert.equal(prepared.render({}, diagrams).body, compileJsx(source, {}, diagrams).body);
  assert.match(transformWithCodeRenderer(source, diagrams).html, /<svg id="fm-mermaid-7">/);
});

const stringHighlight = () => "<b>x</b>";
const linesRender = () => ({ lines: [] });

test("invalid hook results and hook objects are rejected", () => {
  assert.throws(
    () => toHtmlWithCodeRenderer(fence, { highlightCodeBlock: stringHighlight }),
    /code highlighters must return \{ lines \}/,
  );
  assert.throws(
    () => toHtmlWithCodeRenderer(fence, linesRender),
    /code renderers must return a string, \{ html \}/,
  );
  assert.throws(() => toHtmlWithCodeRenderer(fence, { renderCode() {} }), /unknown code hook/);
  assert.throws(() => compileJsx(fence, {}, { highlightCodeBlock: 1 }), /must be a synchronous/);
  assert.throws(() => toHtmlWithCodeRenderer(fence, undefined), /code renderers must be/);
  const failure = new Error("hook failed");
  assert.throws(
    () =>
      compileJsx(
        fence,
        {},
        {
          highlightCodeBlock() {
            throw failure;
          },
        },
      ),
    (error) => error === failure || error.message.includes("hook failed"),
  );
});

/**
 * @param line One code line.
 * @param dual Whether light and dark themes were requested.
 */
function lineTokens(line, dual) {
  if (line === "") return [];
  const first = dual
    ? {
        content: line,
        offset: 0,
        variants: { light: { color: "#CF222E", fontStyle: 1 }, dark: { color: "#FF7B72" } },
      }
    : { content: line, offset: 0, color: "#CF222E", fontStyle: 2 };
  return [first, { content: " <&>", offset: line.length }];
}

/** A Shiki-compatible stand-in that reports tokens and `<pre>` colors to transformers. */
function tokenHighlighter() {
  const calls = [];
  return {
    calls,
    codeToHtml(code, options) {
      calls.push(options);
      if (options.lang === "missing") throw new Error("Language `missing` not found");
      const dual = options.themes != null;
      const tokens = code.split("\n").map((line) => lineTokens(line, dual));
      const style = dual
        ? "--shiki-light:#1f2328;--shiki-dark:#e6edf3;--shiki-light-bg:#ffffff;--shiki-dark-bg:#0d1117"
        : "background-color:#ffffff;color:#1f2328";
      for (const transformer of options.transformers) {
        transformer.tokens?.(tokens);
        transformer.pre?.({ properties: { style } });
      }
      return "<pre>unused</pre>";
    },
  };
}

test("line highlighters adapt Shiki-compatible tokens for one theme", () => {
  const highlighter = tokenHighlighter();
  const highlight = createLineHighlighter(highlighter, { theme: "github-light" });
  assert.deepEqual(highlight("a\n", "ts"), {
    lines: ['<span style="color:#CF222E;font-weight:bold">a</span> &lt;&amp;&gt;', ""],
    foreground: "#1f2328",
    background: "#ffffff",
  });
  assert.equal(highlighter.calls[0].theme, "github-light");
  assert.equal(highlight("a", null), null);
});

test("line highlighters mirror the native dual-theme fragments", () => {
  const highlight = createLineHighlighter(tokenHighlighter(), {
    themes: { light: "github-light", dark: "github-dark" },
  });
  const result = highlight("a", "ts");
  assert.deepEqual(result, {
    lines: [
      '<span style="color:#CF222E;--shiki-light:#CF222E;--shiki-dark:#FF7B72;' +
        'font-style:italic;--shiki-light-font-style:italic;--shiki-dark-font-style:normal">a</span>' +
        " &lt;&amp;&gt;",
    ],
    foreground: "#1f2328",
    background: "#ffffff",
    darkForeground: "#e6edf3",
    darkBackground: "#0d1117",
  });
  const { body } = compileJsx("```ts\na\n```\n", {}, { highlightCodeBlock: highlight });
  assert.match(body, /"--shiki-dark-bg": "#0d1117"/);
});

test("line highlighters report failures with the block location and fall back", () => {
  const errors = [];
  const highlight = createLineHighlighter(tokenHighlighter(), {
    theme: "x",
    onHighlightError(error, context) {
      errors.push([error.message, context]);
    },
  });
  const source = "```missing\ncode\n```\n";
  assert.equal(toHtmlWithCodeRenderer(source, { highlightCodeBlock: highlight }), toHtml(source));
  assert.deepEqual(errors, [
    ["Language `missing` not found", { lang: "missing", start: 0, end: 20 }],
  ]);
});

test("line highlighter options are validated", () => {
  const highlighter = tokenHighlighter();
  assert.throws(() => createLineHighlighter({}, { theme: "x" }), /codeToHtml/);
  assert.throws(() => createLineHighlighter(highlighter, {}), /theme name or themes/);
  assert.throws(
    () => createLineHighlighter(highlighter, { theme: "x", themes: { light: "a", dark: "b" } }),
    /theme name or themes/,
  );
  assert.throws(
    () => createLineHighlighter(highlighter, { themes: { light: "a" } }),
    /theme name or themes/,
  );
  assert.throws(
    () => createLineHighlighter(highlighter, { theme: "x", onHighlightError: 1 }),
    /onHighlightError/,
  );
});

test("standalone code blocks use the highlighting hook only", () => {
  const seen = [];
  const hooks = {
    renderCodeBlock() {
      throw new Error("standalone blocks have no whole-block stage");
    },
    // Code hooks share the public (code, language, meta, block) shape.
    // eslint-disable-next-line max-params
    highlightCodeBlock(code, language, meta, block) {
      seen.push([language, meta, block]);
      return { lines: redLines(code), foreground: "#111", background: "#fff" };
    },
  };
  const result = new CodeCompiler().renderCodeBlock(
    { code: "let a\n", language: "ts", meta: 'title="a.ts"' },
    hooks,
  );
  assert.equal(result.title, "a.ts");
  assert.match(result.jsx, /^<pre className=\{"shiki ox-code-block"\} style=\{\{color: "#111"/);
  assert.match(result.jsx, /<span style=\{\{color: "#f00"\}\}>\{"let a"\}<\/span>/);
  assert.deepEqual(seen, [["ts", 'title="a.ts"', { start: 0, end: 0 }]]);
  assert.doesNotMatch(new CodeCompiler().renderCodeBlock({ code: "x" }).jsx, /shiki/);
});
