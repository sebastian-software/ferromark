// Runs generated modules: TypeScript compiles their JSX against a small
// element factory, and Node imports the result. This checks what the module
// does, not only what it looks like.
import assert from "node:assert/strict";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { after, before, test } from "node:test";
import { pathToFileURL } from "node:url";
// TypeScript is a development dependency of the workspace root. It only
// compiles the JSX of the generated modules for this test.
// eslint-disable-next-line import/no-extraneous-dependencies
import ts from "typescript";

import { compileJsx } from "../index.mjs";

const runtime = `
export const Fragment = Symbol("Fragment");
export function h(type, props, ...children) {
  return { type, props: props ?? {}, children };
}
export function render(node) {
  if (node == null || typeof node === "boolean") return "";
  if (Array.isArray(node)) return node.map(render).join("");
  if (typeof node !== "object") return String(node);
  const { type, props, children } = node;
  if (type === Fragment) return render(children);
  if (typeof type === "function") return render(type({ ...props, children }));
  if (typeof type !== "string") throw new Error("Element type is invalid");
  const attributes = Object.entries(props)
    .filter(([, value]) => typeof value === "string")
    .map(([name, value]) => " " + name + '="' + value + '"')
    .join("");
  return "<" + type + attributes + ">" + render(children) + "</" + type + ">";
}
`;

const providerModule = `
import { h } from "./runtime.mjs";
const wrapper = ({ children }) => h("section", { "data-wrapper": "provider" }, children);
const Provided = () => h("span", null, "provider-owned");
const Button = () => h("span", null, "provider-button");
export function useMDXComponents() {
  return { wrapper, Provided, Button };
}
`;

const layoutModule = `
import { h } from "./runtime.mjs";
export default function Layout({ children }) {
  return h("main", { "data-layout": "imported" }, children);
}
export function Button() {
  return h("span", null, "import-owned");
}
`;

/** A wrapper passed in `props.components`, which outranks the provider's. */
function articleWrapper({ children }) {
  return { type: "article", props: {}, children };
}

/** @type {string} */
let directory;
let sequence = 0;

before(async () => {
  directory = await mkdtemp(path.join(tmpdir(), "ferromark-jsx-module-"));
  await Promise.all([
    writeFile(path.join(directory, "runtime.mjs"), runtime),
    writeFile(path.join(directory, "provider.mjs"), providerModule),
    writeFile(path.join(directory, "layout.mjs"), layoutModule),
  ]);
});

after(async () => {
  await rm(directory, { force: true, recursive: true });
});

/**
 * Compiles a document to a module, appends caller code, and imports it.
 * @param source MDX source.
 * @param options Module options.
 * @param appended Code a caller adds after the module.
 */
async function load(source, options = {}, appended = "") {
  const result = compileJsx(source, {
    format: "mdx",
    output: "module",
    providerImportSource: "./provider.mjs",
    ...options,
  });
  const { outputText } = ts.transpileModule(result.code + appended, {
    compilerOptions: {
      jsx: ts.JsxEmit.React,
      jsxFactory: "h",
      jsxFragmentFactory: "Fragment",
      module: ts.ModuleKind.ESNext,
      target: ts.ScriptTarget.ESNext,
    },
  });
  sequence += 1;
  const file = path.join(directory, `page-${sequence}.mjs`);
  await writeFile(file, `import { h, Fragment } from "./runtime.mjs";\n${outputText}`);
  const [page, { render }] = await Promise.all([
    import(pathToFileURL(file).href),
    import(pathToFileURL(path.join(directory, "runtime.mjs")).href),
  ]);
  return { page, render, result };
}

test("the provider wrapper surrounds the content when no layout is authored", async () => {
  const { page, render } = await load("# Title\n\nText");
  assert.equal(
    render(page.default({})),
    '<section data-wrapper="provider"><h1 id="title">Title</h1><p>Text</p></section>',
  );
  assert.match(render(page.default({ components: { wrapper: articleWrapper } })), /^<article>/);
});

for (const [name, source, marker] of [
  [
    "named function declaration",
    'export default function Layout({ children }) { return <main data-layout="named">{children}</main> }\n\ncontent',
    'data-layout="named"',
  ],
  [
    "anonymous function declaration",
    'export default function ({ children }) { return <main data-layout="anonymous">{children}</main> }\n\ncontent',
    'data-layout="anonymous"',
  ],
  [
    "expression",
    'export default ({ children }) => <article data-layout="expression">{children}</article>;\n\ncontent',
    'data-layout="expression"',
  ],
  [
    "imported default",
    'import Layout from "./layout.mjs";\nexport default Layout;\n\ncontent',
    'data-layout="imported"',
  ],
  [
    "re-exported default",
    'export { default } from "./layout.mjs";\n\ncontent',
    'data-layout="imported"',
  ],
]) {
  test(`an authored ${name} layout replaces the provider wrapper`, async () => {
    const { page, render } = await load(source);
    const view = render(page.default({}));
    assert.ok(view.includes(marker), view);
    assert.ok(view.includes("<p>content</p>"), view);
    assert.ok(!view.includes("data-wrapper"), view);
  });
}

test("a module binding wins over the provider, and the provider fills the rest", async () => {
  const { page, render } = await load(
    'import { Button } from "./layout.mjs";\n\n<Button />\n\n<Provided />\n\n<α />\n\n<props.Slot />\n',
  );
  const view = render(
    page.default({
      Slot: () => "route-prop",
      components: { α: () => "unicode-owned" },
    }),
  );
  assert.ok(view.includes("import-owned"), view);
  assert.ok(!view.includes("provider-button"), view);
  assert.ok(view.includes("provider-owned"), view);
  assert.ok(view.includes("unicode-owned"), view);
  assert.ok(view.includes("route-prop"), view);
});

test("an undefined component is reported by name", async () => {
  const { page, render } = await load("<Missing />\n\n<kit.Input />\n");
  assert.throws(
    () => render(page.default({})),
    /Expected component `Missing` to be defined: import it, pass it in `components`, or provide it\./,
  );
  assert.throws(
    () => render(page.default({ components: { Missing: () => "ok" } })),
    /Expected object `kit` to be defined/,
  );
  assert.throws(
    () => render(page.default({ components: { Missing: () => "ok", kit: {} } })),
    /Expected component `kit.Input` to be defined/,
  );
});

test("a caller appends its own exports around MDXContent", async () => {
  const appended = `
import { h as _h } from "./runtime.mjs";
export const toc = ["usage"];
export default function Route(props) {
  return _h("div", { "data-title": frontmatter.title }, _h(MDXContent, props));
}
`;
  const { page, render, result } = await load(
    'export const frontmatter = { title: "Authored" }\n\n## Usage\n',
    { defaultExport: false, reservedBindings: ["_h", "Route"] },
    appended,
  );
  // The caller reads these lists to decide which exports it still has to add.
  assert.deepEqual(result.exports, ["frontmatter"]);
  assert.deepEqual(result.bindings, ["frontmatter"]);
  assert.deepEqual(page.frontmatter, { title: "Authored" });
  assert.deepEqual(page.toc, ["usage"]);
  assert.match(render(page.default({})), /^<div data-title="Authored"><section/);
});
