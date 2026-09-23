import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { copyFile, mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";
import { pathToFileURL } from "node:url";

// This integration suite covers the complete public Node API.
/* eslint-disable max-lines */
import {
  Renderer,
  toHtml,
  toHtmlBuffer,
  toHtmlWithHighlighter,
  transform,
  transformWithHighlighter,
} from "../index.mjs";
import { linuxLibc, nativeTarget } from "../native-target.mjs";

test("renders Markdown through the native binding", () => {
  assert.equal(toHtml("# Hello"), '<h1 id="hello">Hello</h1>\n');
});

test("renders UTF-8 HTML directly into Node.js Buffers", () => {
  const expected = '<h1 id="grüße">Grüße</h1>\n';
  const output = toHtmlBuffer("# Grüße");

  assert.ok(Buffer.isBuffer(output));
  assert.equal(output.toString("utf8"), expected);

  const renderer = new Renderer({ headingIds: false });
  const reusedOutput = renderer.toHtmlBuffer("# Grüße");
  assert.ok(Buffer.isBuffer(reusedOutput));
  assert.equal(reusedOutput.toString("utf8"), "<h1>Grüße</h1>\n");
});

test("rejects non-string Markdown across every public render entry point", () => {
  const highlighter = { codeToHtml: () => "<pre><code></code></pre>\n" };
  const calls = [
    () => toHtml(123),
    () => toHtmlBuffer(123),
    () => transform(null),
    () => new Renderer().toHtml({}),
    () => new Renderer().toHtmlBuffer({}),
    () => toHtmlWithHighlighter(123, highlighter, { theme: "dark" }),
    () => transformWithHighlighter(123, highlighter, { theme: "dark" }),
  ];

  for (const call of calls) {
    assert.throws(call, (error) => error instanceof Error && /string/i.test(error.message));
  }
});

test("reuses a renderer without leaking document state", () => {
  const renderer = new Renderer({ footnotes: true });

  assert.match(renderer.toHtml("# Same\n\n# Same\n\nA[^a]\n\n[^a]: First"), /id="same-1"/);
  assert.equal(
    renderer.toHtml("# Same\n\n[local][ref]\n\n[^b]: Unused"),
    toHtml("# Same\n\n[local][ref]\n\n[^b]: Unused", { footnotes: true }),
  );
});

test("validates reusable renderer options at construction", () => {
  assert.throws(
    () => new Renderer({ taskList: true }),
    (error) => error instanceof TypeError && /unknown option.*taskList/i.test(error.message),
  );
});

test("maps typed options to the Rust surface", () => {
  assert.equal(toHtml("x^2^", { superscript: true }), "<p>x<sup>2</sup></p>\n");
  assert.match(
    toHtml("| Short | Long |\n| -- | ------ |", { tableColgroup: true }),
    /<col class="col-1">/,
  );
  assert.match(
    toHtml("| A | B |\n| --- | --- |\n| merged ||", { mergedTableCells: true }),
    /colspan="2"/,
  );
  assert.equal(
    toHtml("Term\n: Definition", { definitionLists: true }),
    '<dl class="ox-definition-list">\n<dt>Term</dt>\n<dd>Definition</dd>\n</dl>\n',
  );
  assert.equal(toHtml("// private note", { lineComments: true }), "");
  assert.throws(
    () => toHtml("text", { renderPolicy: "invalid" }),
    /renderPolicy must be either 'untrusted' or 'trusted'/,
  );
});

test("requires trusted rendering before allowHtml passes raw HTML through", () => {
  assert.equal(
    toHtml("<i>content</i>", { allowHtml: true }),
    "<p>&lt;i&gt;content&lt;/i&gt;</p>\n",
  );
  assert.equal(
    toHtml("<i>content</i>", { renderPolicy: "trusted", allowHtml: true }),
    "<p><i>content</i></p>\n",
  );
});

test("rejects unknown option keys across every public render entry point", () => {
  const highlighter = {
    codeToHtml() {
      return "<pre><code></code></pre>\n";
    },
  };
  const calls = [
    () => toHtml("text", { taskList: true }),
    () => toHtmlBuffer("text", { taskList: true }),
    () => transform("text", { footnote: true }),
    () => toHtmlWithHighlighter("text", highlighter, { theme: "dark" }, { taskList: true }),
    () => transformWithHighlighter("text", highlighter, { theme: "dark" }, { footnote: true }),
  ];

  for (const call of calls) {
    assert.throws(
      call,
      (error) =>
        error instanceof TypeError && /unknown option.*(?:taskList|footnote)/i.test(error.message),
    );
  }
});

test("selects the musl optional package on Alpine-style Linux", () => {
  const entry = new URL("../index.mjs", import.meta.url).href;
  const script = `
    Object.defineProperty(process, 'platform', { value: 'linux' })
    Object.defineProperty(process, 'arch', { value: 'arm64' })
    Object.defineProperty(process, 'report', {
      value: { getReport: () => ({ header: {} }) },
    })
    const { toHtml } = await import(${JSON.stringify(entry)})
    try {
      toHtml('text')
    }
    catch (error) {
      if (error instanceof Error && error.message.includes('ferromark-linux-arm64-musl')) {
        process.exit(0)
      }
    }
    process.exit(1)
  `;
  const result = spawnSync(process.execPath, ["--input-type=module", "--eval", script], {
    encoding: "utf8",
  });

  assert.equal(result.status, 0, result.stderr);
});

test("maps every supported native platform and rejects unsupported targets", () => {
  const targets = [
    ["darwin", "arm64", undefined, "darwin-arm64"],
    ["darwin", "x64", undefined, "darwin-x64"],
    ["linux", "arm64", "gnu", "linux-arm64-gnu"],
    ["linux", "arm64", "musl", "linux-arm64-musl"],
    ["linux", "x64", "gnu", "linux-x64-gnu"],
    ["linux", "x64", "musl", "linux-x64-musl"],
    ["win32", "arm64", undefined, "win32-arm64-msvc"],
    ["win32", "x64", undefined, "win32-x64-msvc"],
  ];

  for (const [platform, arch, libc, expected] of targets) {
    assert.equal(nativeTarget(platform, arch, libc), expected);
  }
  assert.equal(nativeTarget("linux", "x64"), "linux-x64-gnu");
  assert.throws(
    () => nativeTarget("linux", "riscv64", "gnu"),
    /ferromark does not support linux\/riscv64/,
  );
  assert.throws(() => nativeTarget("freebsd", "x64"), /ferromark does not support freebsd\/x64/);
});

/** Loader helper contents on a system without a musl loader. */
const noLoaderHelper = () => "";

/** Loader helper contents on a musl system, where it is the loader itself. */
const muslLoaderHelper = () => "musl libc (x86_64)\nVersion 1.2.5\n";

test("detects the Linux C library and assumes gnu without evidence of musl", () => {
  const glibcReport = {
    header: { glibcVersionRuntime: "2.39" },
    sharedObjects: ["/lib/libc.so.6"],
  };
  const muslReport = { header: {}, sharedObjects: ["/lib/ld-musl-x86_64.so.1"] };

  assert.equal(linuxLibc(glibcReport, noLoaderHelper), "gnu");
  assert.equal(linuxLibc(muslReport, noLoaderHelper), "musl");
  // A report without a glibc runtime version describes a musl host.
  assert.equal(linuxLibc({ header: {} }, noLoaderHelper), "musl");

  // Without a report the loader helper is the only remaining evidence.
  assert.equal(linuxLibc(undefined, muslLoaderHelper), "musl");
  assert.equal(linuxLibc(undefined, noLoaderHelper), "gnu");
  assert.equal(nativeTarget("linux", "x64", linuxLibc(undefined, noLoaderHelper)), "linux-x64-gnu");
  assert.equal(
    nativeTarget("linux", "x64", linuxLibc(undefined, muslLoaderHelper)),
    "linux-x64-musl",
  );
});

/**
 * Loads the package on a Linux x64 host whose diagnostic report runs `body`,
 * and reports what the loader observed.
 *
 * @param body Statements the stubbed `getReport()` runs.
 */
function loaderWithStubbedReport(body) {
  const entry = new URL("../index.mjs", import.meta.url).href;
  const script = `
    Object.defineProperty(process, 'platform', { value: 'linux' })
    Object.defineProperty(process, 'arch', { value: 'x64' })
    let excludeNetworkWhileCollecting
    Object.defineProperty(process, 'report', {
      value: {
        excludeNetwork: false,
        getReport() {
          excludeNetworkWhileCollecting = this.excludeNetwork
          ${body}
        },
      },
    })
    const { toHtml } = await import(${JSON.stringify(entry)})
    let outcome = 'rendered'
    try {
      toHtml('text')
    }
    catch (error) {
      // A host without the selected binary still ran the detection.
      outcome = error.message
    }
    console.log(JSON.stringify({
      excludeNetworkWhileCollecting,
      excludeNetworkAfter: process.report.excludeNetwork,
      outcome,
    }))
  `;
  const result = spawnSync(process.execPath, ["--input-type=module", "--eval", script], {
    encoding: "utf8",
  });

  assert.equal(result.status, 0, result.stderr);
  return JSON.parse(result.stdout);
}

test("excludes network interfaces from the loader's diagnostic report", () => {
  const collected = loaderWithStubbedReport("return { header: { glibcVersionRuntime: '2.39' } }");

  assert.equal(collected.excludeNetworkWhileCollecting, true);
  assert.equal(collected.excludeNetworkAfter, false);
});

test("restores the report setting when collecting it fails", () => {
  const failed = loaderWithStubbedReport("throw new Error('report collection failed')");

  assert.equal(failed.excludeNetworkWhileCollecting, true);
  assert.equal(failed.excludeNetworkAfter, false);
  // The loader falls back to the loader helper instead of surfacing that error.
  assert.doesNotMatch(failed.outcome, /report collection failed/);
  assert.match(failed.outcome, /^(?:rendered|ferromark could not load)/);
});

test("selects the gnu package on Linux without a diagnostic report", async (t) => {
  const fixture = await mkdtemp(path.join(tmpdir(), "ferromark-loader-libc-"));
  const entry = path.join(fixture, "index.mjs");
  t.after(() => rm(fixture, { force: true, recursive: true }));

  await Promise.all([
    copyFile(new URL("../index.mjs", import.meta.url), entry),
    copyFile(
      new URL("../native-target.mjs", import.meta.url),
      path.join(fixture, "native-target.mjs"),
    ),
  ]);

  const script = `
    Object.defineProperty(process, 'platform', { value: 'linux' })
    Object.defineProperty(process, 'arch', { value: 'x64' })
    Object.defineProperty(process, 'report', { value: undefined })
    const { toHtml } = await import(${JSON.stringify(pathToFileURL(entry).href)})
    try {
      toHtml('text')
    }
    catch (error) {
      console.error(error.message)
    }
  `;
  const result = spawnSync(process.execPath, ["--input-type=module", "--eval", script], {
    encoding: "utf8",
  });

  assert.equal(result.status, 0, result.stderr);
  // Only the loader helper may still select musl, as it does on Alpine.
  const loaderHelper = await readFile("/usr/bin/ldd", "latin1").catch(() => "");
  const expected = loaderHelper.includes("musl") ? "musl" : "gnu";
  assert.match(result.stderr, new RegExp(`ferromark-linux-x64-${expected}`));
});

test("does not collect a diagnostic report on non-Linux platforms", () => {
  const entry = new URL("../index.mjs", import.meta.url).href;
  const script = `
    Object.defineProperty(process, 'platform', { value: 'darwin' })
    Object.defineProperty(process, 'arch', { value: 'arm64' })
    Object.defineProperty(process, 'report', {
      value: { getReport: () => { throw new Error('diagnostic report collected') } },
    })
    const { toHtml } = await import(${JSON.stringify(entry)})
    try {
      toHtml('text')
    }
    catch (error) {
      if (error instanceof Error && error.message === 'diagnostic report collected') {
        process.exit(1)
      }
    }
  `;
  const result = spawnSync(process.execPath, ["--input-type=module", "--eval", script], {
    encoding: "utf8",
  });

  assert.equal(result.status, 0, result.stderr);
});

test("wraps native dynamic-loader failures with platform guidance", async (t) => {
  const target = currentNativeTarget();
  const packageName = `ferromark-${target}`;
  const binaryName = `ferromark.${target}.node`;
  const fixture = await mkdtemp(path.join(tmpdir(), "ferromark-loader-"));
  const packageDir = path.join(fixture, "node_modules", packageName);
  const entry = path.join(fixture, "index.mjs");
  t.after(() => rm(fixture, { force: true, recursive: true }));

  await mkdir(packageDir, { recursive: true });
  await Promise.all([
    copyFile(new URL("../index.mjs", import.meta.url), entry),
    copyFile(
      new URL("../native-target.mjs", import.meta.url),
      path.join(fixture, "native-target.mjs"),
    ),
    writeFile(
      path.join(packageDir, "package.json"),
      JSON.stringify({ name: packageName, main: binaryName }),
    ),
    writeFile(path.join(packageDir, binaryName), "not a native addon"),
  ]);

  const fixtureModule = await import(pathToFileURL(entry).href);
  assert.throws(
    () => fixtureModule.toHtml("text"),
    (error) => {
      assert.ok(error instanceof Error);
      assert.match(error.message, new RegExp(binaryName.replaceAll(".", "\\.")));
      assert.match(error.message, new RegExp(`${process.platform}/${process.arch}`));
      assert.match(error.message, /ERR_DLOPEN_FAILED/);
      assert.equal(error.cause?.code, "ERR_DLOPEN_FAILED");
      if (process.platform === "linux") {
        assert.match(error.message, /glibc 2\.17|musl runtime/);
      } else if (process.platform === "win32") {
        assert.match(error.message, /Visual C\+\+ Redistributable/);
      } else {
        assert.match(error.message, /quarantine or code-signing policy/);
      }
      return true;
    },
  );
});

test("retains both missing-native lookup causes", async (t) => {
  const fixture = await mkdtemp(path.join(tmpdir(), "ferromark-loader-missing-"));
  const entry = path.join(fixture, "index.mjs");
  t.after(() => rm(fixture, { force: true, recursive: true }));

  await Promise.all([
    copyFile(new URL("../index.mjs", import.meta.url), entry),
    copyFile(
      new URL("../native-target.mjs", import.meta.url),
      path.join(fixture, "native-target.mjs"),
    ),
  ]);

  const fixtureModule = await import(pathToFileURL(entry).href);
  assert.throws(
    () => fixtureModule.toHtml("text"),
    (error) => {
      assert.ok(error instanceof Error);
      assert.ok(error.cause instanceof AggregateError);
      assert.equal(error.cause.errors.length, 2);
      const [localCause, optionalCause] = error.cause.errors;
      assert.equal(localCause?.code, "MODULE_NOT_FOUND");
      assert.equal(optionalCause?.code, "MODULE_NOT_FOUND");
      return true;
    },
  );
});

test("loads the package by name from CommonJS", () => {
  const require = createRequire(import.meta.url);
  const { toHtml: cjsToHtml } = require("ferromark");

  assert.equal(cjsToHtml("# CommonJS"), '<h1 id="commonjs">CommonJS</h1>\n');
});

test("composes with a synchronous Ferriki-compatible highlighter", () => {
  const calls = [];
  const highlighter = {
    codeToHtml(code, options) {
      calls.push({ code, options });
      return '<pre class="ferriki"><code>safe</code></pre>\n';
    },
  };

  const html = toHtmlWithHighlighter("```rust\nconst x = 1\n```", highlighter, {
    theme: "github-dark",
  });

  assert.equal(html, '<pre class="ferriki"><code>safe</code></pre>\n');
  assert.deepEqual(calls, [
    {
      code: "const x = 1\n",
      options: { lang: "rust", theme: "github-dark" },
    },
  ]);
});

test("falls back to escaped code when highlighting fails", () => {
  const failures = [];
  const failure = new Error("unsupported language");
  const highlighter = {
    codeToHtml() {
      throw failure;
    },
  };

  const html = toHtmlWithHighlighter("```unknown\n<tag>\n```", highlighter, {
    theme: "github-dark",
    onHighlightError(error, context) {
      failures.push({ error, context });
    },
  });

  assert.equal(html, '<pre><code class="language-unknown">&lt;tag&gt;\n</code></pre>\n');
  assert.deepEqual(failures, [{ error: failure, context: { lang: "unknown" } }]);
  assert.equal(
    toHtmlWithHighlighter("```unknown\n<tag>\n```", highlighter, { theme: "github-dark" }),
    html,
  );
});

test("uses fallbackLanguage and preserves transform metadata when highlighting fails", () => {
  const calls = [];
  const failure = new Error("unsupported language");
  const highlighter = {
    codeToHtml(code, options) {
      calls.push({ code, options });
      throw failure;
    },
  };
  const failures = [];
  const result = transformWithHighlighter("```\n<tag>\n```\n\n# After", highlighter, {
    theme: "github-dark",
    fallbackLanguage: "plaintext",
    onHighlightError(error, context) {
      failures.push({ error, context });
    },
  });

  assert.match(result.html, /<pre><code>&lt;tag&gt;\n<\/code><\/pre>/);
  assert.match(result.html, /<h1 id="after">After<\/h1>/);
  assert.deepEqual(result.headings, [{ level: 1, id: "after", text: "After" }]);
  assert.deepEqual(calls, [
    {
      code: "<tag>\n",
      options: { lang: "plaintext", theme: "github-dark" },
    },
  ]);
  assert.deepEqual(failures, [{ error: failure, context: { lang: "plaintext" } }]);
});

test("validates and surfaces highlighter error observers", () => {
  const highlighter = {
    codeToHtml() {
      throw new Error("highlight failed");
    },
  };

  assert.throws(
    () =>
      toHtmlWithHighlighter("```js\ncode\n```", highlighter, {
        theme: "dark",
        onHighlightError: "invalid",
      }),
    /onHighlightError must be a function/,
  );
  assert.throws(
    () =>
      transformWithHighlighter("```js\ncode\n```", highlighter, {
        theme: "dark",
        onHighlightError() {
          throw new Error("observer failed");
        },
      }),
    /observer failed/,
  );
});

test("surfaces invalid highlighter return values from the native callback", () => {
  const highlighter = {
    codeToHtml() {
      return { html: "<pre>wrong shape</pre>" };
    },
  };

  assert.throws(
    () => toHtmlWithHighlighter("```js\ncode\n```", highlighter, { theme: "dark" }),
    (error) => error instanceof Error,
  );
});

test("transform returns html, headings, and front matter", () => {
  const result = transform("---\ntitle: X\n---\n# Top\n\n## Sub `code`\n", { frontMatter: true });

  assert.equal(result.frontMatter, "title: X\n");
  assert.match(result.html, /<h1 id="top">Top<\/h1>/);
  assert.deepEqual(result.headings, [
    { level: 1, id: "top", text: "Top" },
    { level: 2, id: "sub-code", text: "Sub code" },
  ]);
});

test("transform extracts TOML-style front matter", () => {
  const result = transform('+++\ntitle = "TOML"\n+++\n# Top', { frontMatter: true });

  assert.equal(result.frontMatter, 'title = "TOML"\n');
  assert.match(result.html, /<h1 id="top">Top<\/h1>/);
});

test("transform omits ids when headingIds is disabled", () => {
  const result = transform("# Top", { headingIds: false });

  assert.equal(result.headings.length, 1);
  assert.equal(result.headings[0].id, undefined);
  assert.equal(result.headings[0].text, "Top");
});

test("linkBasePath uses v2 site URL routing", () => {
  const html = toHtml("[in](/guide) [out](https://e.com/) ![img](/i.png)", {
    linkBasePath: "/docs",
  });

  assert.match(html, /<a href="\/docs\/guide">/);
  assert.match(html, /<a href="https:\/\/e.com\/">/);
  assert.match(html, /<img src="\/docs\/i.png"/);
});

test("linkBasePath joins Markdown links with or without a trailing slash", () => {
  for (const linkBasePath of ["/docs", "/docs/"]) {
    const html = toHtml("[guide](/guide.md)", {
      linkBasePath,
    });

    assert.match(html, /<a href="\/docs\/guide\/index.html">guide<\/a>/);
  }
});

test("an empty linkBasePath keeps root-absolute links root-absolute", () => {
  for (const linkBasePath of ["", "/"]) {
    const html = toHtml("[guide](/guide.md) [page](/page) [rel](./other.md)", {
      linkBasePath,
    });

    assert.match(html, /<a href="\/guide\/index.html">guide<\/a>/);
    assert.match(html, /<a href="\/page">page<\/a>/);
    assert.match(html, /<a href="\.\.\/other\/index.html">rel<\/a>/);
  }
});

test("highlighter receives fence meta as Shiki-style __raw", () => {
  const calls = [];
  const highlighter = {
    codeToHtml(code, options) {
      calls.push(options);
      return '<pre class="hl">x</pre>\n';
    },
  };

  const result = transformWithHighlighter(
    '```ts {1-3} title="Example"\ncode\n```\n\n```ts\ncode\n```',
    highlighter,
    { theme: "github-dark" },
  );

  assert.match(result.html, /class="hl"/);
  assert.deepEqual(calls, [
    { lang: "ts", theme: "github-dark", meta: { __raw: '{1-3} title="Example"' } },
    { lang: "ts", theme: "github-dark" },
  ]);
});

function currentNativeTarget() {
  const report = process.report?.getReport?.();
  const libc = report?.header?.glibcVersionRuntime ? "gnu" : "musl";
  const key =
    process.platform === "linux"
      ? `${process.platform}-${process.arch}-${libc}`
      : `${process.platform}-${process.arch}`;
  const targets = {
    "darwin-arm64": "darwin-arm64",
    "darwin-x64": "darwin-x64",
    "linux-arm64-gnu": "linux-arm64-gnu",
    "linux-arm64-musl": "linux-arm64-musl",
    "linux-x64-gnu": "linux-x64-gnu",
    "linux-x64-musl": "linux-x64-musl",
    "win32-arm64": "win32-arm64-msvc",
    "win32-x64": "win32-x64-msvc",
  };
  const target = targets[key];
  assert.ok(target, `test requires a supported native target, received ${key}`);
  return target;
}

test("rejects removed v1 options instead of silently ignoring them", () => {
  for (const key of ["tableColumnWidths", "indentedCodeBlocks"]) {
    assert.throws(() => toHtml("text", { [key]: true }), /unknown option/);
    assert.throws(() => new Renderer({ [key]: true }), /unknown option/);
    assert.throws(() => transform("text", { [key]: true }), /unknown option/);
  }
});

test("preserves the untrusted URL boundary across ordinary and hooked rendering", () => {
  const source = "[x](javascript:alert%281%29) ![x](data:text/html,bad) <script>x</script>";
  const highlighter = { codeToHtml: () => "<pre>trusted</pre>" };
  for (const html of [
    toHtml(source),
    toHtmlWithHighlighter(source, highlighter, { theme: "dark" }),
  ]) {
    assert.doesNotMatch(html, /(?:href|src)="(?:javascript|data):/);
    assert.doesNotMatch(html, /<script>/);
  }
});

test("metadata IDs agree with v2 heading output and reset between documents", () => {
  const source = "# A *title*!\n\n# A title!\n\n## Explicit {#custom}";
  const result = transform(source, { headingAttributes: true });
  assert.deepEqual(
    result.headings.map((h) => h.id),
    ["a-title", "a-title-1", "custom"],
  );
  for (const heading of result.headings) assert.ok(result.html.includes(`id="${heading.id}"`));
  assert.equal(transform("# A title!").headings[0].id, "a-title");
});

test("metadata deduplicates generated suffixes and repeated explicit IDs", () => {
  const result = transform("# a\n\n# a\n\n# a-1 {#a-1}\n\n# a-1 {#a-1}\n\n# b {#b}\n\n# b {#b}", {
    headingAttributes: true,
  });
  const ids = ["a", "a-1", "a-1-1", "a-1-2", "b", "b-1"];

  assert.deepEqual(
    result.headings.map((heading) => heading.id),
    ids,
  );
  for (const id of ids) assert.ok(result.html.includes(`id="${id}"`));
});

test("a reusable renderer recovers after a bounded-depth parse error", () => {
  const renderer = new Renderer();
  assert.throws(() => renderer.toHtml(`${"> ".repeat(150)}deep`), /nest|depth/i);
  assert.equal(renderer.toHtml("recovered"), "<p>recovered</p>\n");
});

test("a reusable renderer returns independent strings from its kept buffer", () => {
  // `Renderer.toHtml` copies out of an output buffer it keeps between calls,
  // so a later, shorter document must neither shorten nor overwrite a string
  // returned earlier.
  const long = `# Title\n\n${"Some *prose* with a [link](/u).\n\n".repeat(200)}`;
  const short = "short";
  const renderer = new Renderer();
  const first = renderer.toHtml(long);
  const second = renderer.toHtml(short);
  const third = renderer.toHtml(long);

  assert.equal(first, toHtml(long));
  assert.equal(second, "<p>short</p>\n");
  assert.equal(third, first);
  assert.equal(renderer.toHtmlBuffer(short).toString(), second);
  assert.equal(first, toHtml(long));
});

test("deeply nested inline brackets throw instead of killing the process", () => {
  // Issue #349: these calls used to take the whole process down with a
  // stack overflow, which is not a panic and so cannot be caught or
  // reported as a JavaScript error. Reaching the assertions is the result.
  const brackets = `${"[".repeat(20_000)}a${"]".repeat(20_000)}`;
  const images = `${"![".repeat(20_000)}a${"](u)".repeat(20_000)}`;

  assert.throws(() => toHtml(brackets), /nest|depth/i);
  assert.throws(() => toHtmlBuffer(brackets), /nest|depth/i);
  assert.throws(() => toHtml(images), /nest|depth/i);

  const renderer = new Renderer();
  assert.throws(() => renderer.toHtml(brackets), /nest|depth/i);
  assert.equal(renderer.toHtml("recovered"), "<p>recovered</p>\n");
});

test("deeply nested emphasis throws instead of killing the process", () => {
  // Issue #371: the parse survived this one — pairing is iterative — and
  // the renderer then walked a 10,000-level tree and overflowed the stack,
  // which takes the whole process down with no JavaScript error. Reaching
  // the assertions is the result.
  const stars = `${"*".repeat(20_000)}a${"*".repeat(20_000)}`;
  const underscores = `${"_".repeat(20_000)}a${"_".repeat(20_000)}`;

  assert.throws(() => toHtml(stars), /nest|depth/i);
  assert.throws(() => toHtmlBuffer(stars), /nest|depth/i);
  assert.throws(() => toHtml(underscores), /nest|depth/i);

  // A run with nothing to close it nests nothing and still renders.
  assert.equal(toHtml(`${"*".repeat(2000)}a`), `<p>${"*".repeat(2000)}a</p>\n`);

  const renderer = new Renderer();
  assert.throws(() => renderer.toHtml(stars), /nest|depth/i);
  assert.equal(renderer.toHtml("recovered"), "<p>recovered</p>\n");
});

// The assertions exercise every equivalent rendering entry point.
// eslint-disable-next-line max-statements
test("supports optional marked text, inline notes, and reference policy", () => {
  const source = "==Text==^[a *note*] [ref]\n\n[ref]: /url";
  const options = { highlight: true, inlineFootnotes: true, allowLinkRefs: false };
  const html = toHtml(source, options);
  assert.match(html, /<mark>Text<\/mark>/);
  assert.match(html, /a <em>note<\/em>/);
  assert.match(html, /\[ref\]: \/url/);
  assert.equal(toHtmlBuffer(source, options).toString(), html);
  const renderer = new Renderer(options);
  assert.equal(renderer.toHtml("plain"), "<p>plain</p>\n");
  assert.equal(renderer.toHtml(source), html);
  assert.equal(renderer.toHtml(source), html);
  assert.equal(transform(source, options).html, html);
  const highlighter = { codeToHtml: () => "<pre>code</pre>" };
  assert.equal(toHtmlWithHighlighter(source, highlighter, { theme: "dark" }, options), html);
  assert.equal(
    transformWithHighlighter(source, highlighter, { theme: "dark" }, options).html,
    html,
  );
  assert.equal(toHtml("==Text== ^[note]"), "<p>==Text== ^[note]</p>\n");
});
