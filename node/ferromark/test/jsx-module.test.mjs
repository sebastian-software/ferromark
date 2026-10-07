import assert from "node:assert/strict";
import { test } from "node:test";

import { compileJsx, JsxCompiler } from "../index.mjs";

const provider = { format: "mdx", output: "module", providerImportSource: "docs/provider" };

test("module output is a complete MDX module with a source map", () => {
  const source = "---\ntitle: Guide\n---\n\n# Guide\n\n## Usage\n\n<Badge />\n";
  const result = compileJsx(source, {
    ...provider,
    filename: "guide.mdx",
    frontMatter: true,
    omitTitleHeading: "Guide",
  });
  assert.match(
    result.code,
    /^import \{useMDXComponents as _provideComponents\} from "docs\/provider";\n/,
  );
  assert.match(result.code, /\nfunction _createMdxContent\(props\) \{\n/);
  assert.match(result.code, /\nexport default function MDXContent\(props = \{\}\) \{\n/);
  assert.match(result.code, / {2}const \{Badge\} = _components;\n/);
  assert.match(result.code, /<_components.h2 id=\{"usage"\}>/);
  assert.ok(!result.code.includes("<_components.h1"));
  assert.ok(!result.code.includes("react"));
  assert.equal(result.body, undefined);
});

test("module output carries a source map and the document metadata", () => {
  const source = "---\ntitle: Guide\n---\n\n# Guide\n\n## Usage\n";
  const result = compileJsx(source, {
    ...provider,
    filename: "guide.mdx",
    frontMatter: true,
    omitTitleHeading: "Guide",
  });
  assert.deepEqual(result.map.sources, ["guide.mdx"]);
  assert.deepEqual(result.map.sourcesContent, [source]);
  assert.equal(result.map.version, 3);
  assert.deepEqual(result.map.names, []);
  assert.match(result.map.mappings, /^[a-z0-9+/,;]+$/i);
  assert.equal(result.frontMatter, "title: Guide\n");
  assert.deepEqual(
    result.headings.map(({ id }) => id),
    ["usage"],
  );
  assert.ok(result.omittedTitleHeadingSpan);
});

test("module output reports authored bindings and exports and rewrites the layout", () => {
  const result = compileJsx(
    'import Layout, { helper } from "./layout.js"\nexport const meta = { draft: true }\nexport default Layout\n\nText',
    provider,
  );
  assert.deepEqual(result.bindings, ["Layout", "helper", "meta"]);
  assert.deepEqual(result.exports, ["meta"]);
  assert.match(result.code, /\nconst MDXLayout = Layout\n/);
  assert.match(
    result.code,
    /return <MDXLayout \{\.\.\.props\}><_createMdxContent \{\.\.\.props\} \/><\/MDXLayout>;/,
  );
});

test("a caller can append its own default export without a naming pass", () => {
  const result = compileJsx("```mermaid\ngraph TD\n```\n\n<Chart />\n", {
    ...provider,
    codeComponents: { mermaid: "_Diagram" },
    defaultExport: false,
    reservedBindings: ["_Diagram", "_createRoute"],
  });
  assert.ok(!result.code.includes("export default"));
  assert.match(result.code, /\nfunction MDXContent\(props = \{\}\) \{\n/);
  // The caller binds `_Diagram`; only `Chart` comes from the components object.
  assert.match(result.code, / {2}const \{Chart\} = _components;\n/);
  assert.equal(result.codeBlocks[0].language, "mermaid");

  assert.throws(
    () =>
      compileJsx("export const _createRoute = 1\n\nText", {
        ...provider,
        reservedBindings: ["_createRoute"],
      }),
    /`_createRoute` at .* is reserved in MDX module output/,
  );
  assert.throws(
    () => compileJsx("export const MDXContent = 1\n\nText", provider),
    /`MDXContent` at .* is reserved/,
  );
  assert.throws(
    () => compileJsx("export default A\n\nexport { B as default }\n", provider),
    /only one default export/,
  );
});

test("module options are rejected where they do not apply", () => {
  assert.throws(() => compileJsx("text", { output: "html" }), /output must be "body" or "module"/);
  for (const [key, value] of [
    ["providerImportSource", "docs/provider"],
    ["filename", "page.mdx"],
    ["defaultExport", false],
    ["reservedBindings", ["a"]],
  ]) {
    assert.throws(
      () => compileJsx("text", { [key]: value }),
      new RegExp(`JSX option "${key}" requires output: "module"`),
    );
    assert.throws(
      () => compileJsx("text", { output: "body", [key]: value }),
      /requires output: "module"/,
    );
  }
  assert.throws(
    () => compileJsx("text", { output: "module", providerImportSource: "" }),
    /providerImportSource must be a nonempty string/,
  );
  assert.throws(
    () => compileJsx("text", { output: "module", componentPrefix: "custom" }),
    /module output sets the component prefix/,
  );
  assert.equal(typeof compileJsx("text", { output: "body" }).body, "string");
});

test("TypeScript in an MDX module block is named as the cause", () => {
  assert.throws(
    () => compileJsx("export const count: number = 1\n\n# Heading", provider),
    /TypeScript syntax is not supported in MDX module blocks/,
  );
});

test("module output works for plain Markdown and without a provider", () => {
  const result = compileJsx("# Title\n\nText", { output: "module" });
  assert.ok(!result.code.includes("import "));
  assert.match(result.code, /const \{wrapper: MDXLayout\} = props\.components \|\| \{\};/);
  assert.deepEqual(result.bindings, []);
  assert.deepEqual(result.exports, []);
  assert.deepEqual(result.map.sources, [""]);
});

test("JsxCompiler emits highlighted fences inside a module", () => {
  const compiler = new JsxCompiler({
    theme: {
      name: "module-theme",
      type: "dark",
      colors: { "editor.background": "#101010", "editor.foreground": "#f0f0f0" },
      tokenColors: [],
    },
    languages: [],
    assets: { remote: false },
  });
  const result = compiler.compile('```text title="notes.txt"\nplain\n```\n', {
    ...provider,
    codeBlockComponent: "_components.CodeBlock",
  });
  assert.match(result.code, /<_components\.CodeBlock code=\{"plain\\n"\}/);
  assert.match(
    result.code,
    / {2}if \(!_components\.CodeBlock\) _missingMdxReference\("CodeBlock", true\);\n/,
  );
  assert.match(result.code, /<_components\.pre /);
  assert.equal(typeof compiler.compile("text").body, "string");
});
