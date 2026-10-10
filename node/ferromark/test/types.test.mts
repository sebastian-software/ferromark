import type {
  CodeHighlighter,
  JsxPreparedMetadata,
  Options,
  PreparedJsxDocument,
} from "../index.mjs";

import {
  compileJsx,
  JsxCompiler,
  Renderer,
  toHtml,
  toHtmlBuffer,
  toHtmlWithHighlighter,
} from "../index.mjs";

const options: Options = {
  renderPolicy: "untrusted",
  tables: true,
  mergedTableCells: true,
  tableColgroup: true,
  tableAttributes: true,
  definitionLists: true,
  lineComments: true,
  headingAttributes: true,
  headingOffset: 1,
  headingIdPrefix: "docs-",
  cjkEmphasis: true,
  mdx: true,
  imageAttributes: true,
  imageCaptions: true,
  extendedAttributes: true,
  bracketedSpans: true,
  blockquoteAttributions: true,
  guillemetDigraphs: true,
  autoAbbreviations: true,
  abbreviations: {
    API: "Application Programming Interface",
    GraphQL: "",
    XYZ: null,
  },
};
const highlighter: CodeHighlighter = {
  codeToHtml: (code, { lang, theme }) => `${lang}:${theme}:${code}`,
};

toHtml("# Typed", options);
const output: Buffer = toHtmlBuffer("# Buffered", options);
output.toString("utf8");
const renderer = new Renderer(options);
renderer.toHtml("# Reused");
const reusedOutput: Buffer = renderer.toHtmlBuffer("# Buffered and reused");
reusedOutput.toString("utf8");
toHtmlWithHighlighter("```ts\nconst typed = true\n```", highlighter, {
  theme: "github-dark",
  onHighlightError(_error, { lang }) {
    const _upperCaseLanguage = lang.toUpperCase();
  },
});

toHtml("==text==^[note]", { highlight: true, inlineFootnotes: true, allowLinkRefs: false });

// Markdown as UTF-8 bytes: any Uint8Array, including a Buffer.
toHtml(new TextEncoder().encode("# Bytes"), options);
const bytesOutput: Buffer = toHtmlBuffer(Buffer.from("# Buffered bytes"));
bytesOutput.toString("utf8");
renderer.toHtml(Buffer.from("# Reused bytes"));
renderer.toHtmlBuffer(new Uint8Array(0));
toHtmlWithHighlighter(Buffer.from("```ts\nconst typed = true\n```"), highlighter, {
  theme: "github-dark",
});
// @ts-expect-error -- an ArrayBuffer must be wrapped in a Uint8Array first
toHtml(new ArrayBuffer(8));

const jsx = compileJsx("# Typed", { format: "mdx", componentPrefix: "_components" }, () => null);
const jsxBody: string = jsx.body;
compileJsx(jsxBody);
const jsxCompiler = new JsxCompiler({
  theme: { name: "custom", settings: [{ settings: { foreground: "#000000" } }] },
  languages: [{ name: "custom", scopeName: "source.custom", patterns: [] }],
  assets: { remote: false },
});
const highlightedJsx: string = jsxCompiler.compile("# Typed", {
  codeBlockComponent: "CodeBlock",
}).body;
const standaloneCode = jsxCompiler.renderCodeBlock({
  code: 'const value = "typed";\n',
  language: "ts",
  meta: 'title="typed.ts" :line-numbers=4',
});
const standaloneMarkup: string = standaloneCode.jsx;
const standaloneLanguage: string | undefined = standaloneCode.language;
const standaloneTitle: string | undefined = standaloneCode.title;
const standaloneLabel: string | undefined = standaloneCode.label;
const standaloneLineNumbers: boolean = standaloneCode.lineNumbers;
compileJsx(
  `${standaloneMarkup}${standaloneLanguage ?? ""}${standaloneTitle ?? ""}${standaloneLabel ?? ""}${standaloneLineNumbers}`,
);
// @ts-expect-error -- standalone code input requires its code string
jsxCompiler.renderCodeBlock({ language: "ts" });
// @ts-expect-error -- standalone code input does not accept renderer options
jsxCompiler.renderCodeBlock({ code: "value", codeBlockComponent: "CodeBlock" });
compileJsx(highlightedJsx);
const preparedJsx: PreparedJsxDocument = jsxCompiler.prepare("# Typed", {
  format: "mdx",
  frontMatter: true,
  passes: [{ kind: "emojiShortcodes" }],
});
const preparedMetadata: JsxPreparedMetadata = preparedJsx.metadata;
const preparedBody = preparedJsx.render({ componentPrefix: "_components", headingIds: false });
const preparedModule = preparedJsx.renderModule({
  providerImportSource: "docs/provider",
  filename: "typed.mdx",
});
compileJsx(`${preparedBody.body}${preparedModule.code}${preparedMetadata.outline.length}`);
// @ts-expect-error -- heading settings are render-time choices
jsxCompiler.prepare("# Typed", { headingOffset: 1 });
// @ts-expect-error -- native passes are fixed during preparation
preparedJsx.render({ passes: [] });
const jsxModule = compileJsx("# Typed", {
  format: "mdx",
  output: "module",
  providerImportSource: "docs/provider",
  filename: "typed.mdx",
  defaultExport: false,
  reservedBindings: ["createRoute"],
});
const moduleCode: string = jsxModule.code;
const moduleMappings: string = jsxModule.map.mappings;
const moduleNames: string[] = [...jsxModule.exports, ...jsxModule.bindings];
compileJsx(moduleCode + moduleMappings + moduleNames.join(""));
const highlightedModule: string = jsxCompiler.compile("# Typed", { output: "module" }).code;
const explicitBody: string = compileJsx(highlightedModule, { output: "body" }).body;
compileJsx(explicitBody);
// @ts-expect-error -- a module has no separate body
const moduleKey: keyof typeof jsxModule = "body";
compileJsx(moduleKey);
// @ts-expect-error -- module output owns the component prefix
compileJsx("# Typed", { output: "module", componentPrefix: "_components" });
// @ts-expect-error -- module options need `output: "module"`
compileJsx("# Typed", { providerImportSource: "docs/provider" });
// @ts-expect-error -- JSX compilation does not promise HTML sanitization
compileJsx("# Typed", { renderPolicy: "untrusted" });
// @ts-expect-error -- callbacks are synchronous
compileJsx("```js\nvalue\n```", {}, async () => {
  await Promise.resolve();
  return "<Code />";
});
