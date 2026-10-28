import type { Buffer } from "node:buffer";

export type RenderPolicy = "untrusted" | "trusted";

// oxlint-disable-next-line typescript/consistent-type-definitions -- keeps this public config structurally extensible
export interface TypographyOptions {
  /** One explicit supported language code; regional variants are not inferred. */
  language:
    | "cs"
    | "da"
    | "de"
    | "en"
    | "es"
    | "fi"
    | "fr"
    | "it"
    | "nb"
    | "nl"
    | "pl"
    | "pt"
    | "ru"
    | "sv"
    | "uk";
  /** Convert locale-aware dash sequences. Default: on. */
  dashes?: boolean;
  /** Convert three consecutive periods to an ellipsis. Default: on. */
  ellipses?: boolean;
}

/** One ordered, opt-in native transform pass. Passes run in array order. */
export type NativePassOptions =
  | {
      kind: "typography";
      language: TypographyOptions["language"];
      dashes?: boolean;
      ellipses?: boolean;
    }
  | { kind: "githubReferences"; repository: string }
  | { kind: "emojiShortcodes" };

// oxlint-disable-next-line typescript/consistent-type-definitions -- preserve public declaration merging
export interface Options {
  tableAttributes?: boolean;
  headingAttributes?: boolean;
  cjkEmphasis?: boolean;
  /** Recognize MDX syntax; does not compile or execute JavaScript. */
  mdx?: boolean;
  /** Parse `{#id .class}` directly after an image. Default: off. */
  imageAttributes?: boolean;
  /** Attach a separate `: Caption` line to a standalone image. Default: off. */
  imageCaptions?: boolean;
  /** Enable shared key/value attributes on headings, links, images, tables, and figures. */
  extendedAttributes?: boolean;
  /** Enable `[inline Markdown]{attributes}` spans. */
  bracketedSpans?: boolean;
  /** Attach a separate `: Author` line after a block quote. Default: off. */
  blockquoteAttributions?: boolean;
  /** Enable `++inserted text++`. Default: off, including the GFM profile. */
  insertions?: boolean;
  /** Preserve `<<…>>` as inline text and allow typography to map balanced pairs. Default: off. */
  guillemetDigraphs?: boolean;
  /** Output trust boundary. Default: `'untrusted'`; use `'trusted'` only for trusted Markdown. */
  renderPolicy?: RenderPolicy;
  /** Allow raw HTML in trusted output. Default: on; untrusted output always escapes it. */
  allowHtml?: boolean;
  /** Enable GFM pipe tables. Default: on. */
  tables?: boolean;
  /** Enable MultiMarkdown-style table column spans. Default: off; requires `tables`. */
  mergedTableCells?: boolean;
  /** Emit CSS-addressable colgroup elements. Default: off; requires `tables`. */
  tableColgroup?: boolean;
  /** Add header-derived CSS classes to colgroup elements. */
  tableColumnNames?: boolean;
  /** Enable GFM `~~strikethrough~~`. Default: on. */
  strikethrough?: boolean;
  /** Enable `^superscript^`. Default: off. */
  superscript?: boolean;
  /** Enable `~subscript~`. Default: off. */
  subscript?: boolean;
  /** Enable GFM task lists. Default: on. */
  taskLists?: boolean;
  /** Enable bare URL, `www`, and email autolinks. Default: off. */
  autolinkLiterals?: boolean;
  /** Filter GFM-disallowed raw HTML in trusted mode. Default: on; this is not a sanitizer. */
  disallowedRawHtml?: boolean;
  /** Enable `[^label]` footnotes. Default: off. */
  footnotes?: boolean;
  /** Enable `==marked text==`. Default: off. Independent of code highlighting. */
  highlight?: boolean;
  /** Enable `^[inline notes]`. Default: off; independent of `footnotes`. */
  inlineFootnotes?: boolean;
  /** Resolve link/image references and consume definitions. Default: on. */
  allowLinkRefs?: boolean;
  /** Extract a leading `---` or `+++` front-matter block. Default: off. */
  frontMatter?: boolean;
  /** Generate v2 heading IDs. Default: on. */
  headingIds?: boolean;
  /** Signed 32-bit integer heading shift; positive values move toward h6, clamped to h1–h6. Default: 0. */
  headingOffset?: number;
  /** Prefix heading IDs and generated permalink fragments. Safe characters: ASCII letters, digits, `_`, and `-`. */
  headingIdPrefix?: string;
  /** Enable `$inline$` and `$$display$$` math. Default: off. */
  math?: boolean;
  /** Enable GitHub-style blockquote callouts. Default: on. */
  callouts?: boolean;
  /** Enable PHP Markdown Extra definition lists. Default: off. */
  definitionLists?: boolean;
  /** Omit physical-line-start `//` source comments. Default: off. */
  lineComments?: boolean;
  /**
   * Prefix internal absolute link destinations (starting with `/`) with
   * this base path, for sites deployed under a subpath. Image sources and root-absolute raw HTML URLs also use this base.
   * Enables v2 site routing, including .md to index.html conversion. Default: unset.
   */
  linkBasePath?: string;
  /** Apply explicit locale-aware typography after parsing. Default: off. */
  typography?: TypographyOptions;
  /** Run native transform passes in the given order. Cannot be combined with `typography`. */
  passes?: NativePassOptions[];
  /** Wrap complete uppercase technical terms in `<abbr>` markup. Default: off. */
  autoAbbreviations?: boolean;
  /**
   * Exact, case-sensitive abbreviation overrides. A nonempty string sets the
   * escaped title, an empty string wraps without a title, and null suppresses
   * wrapping. Supplying this map does not enable autoAbbreviations.
   */
  abbreviations?: Record<string, string | null>;
  /**
   * Start from the Ferromark Flavored Markdown syntax profile: GFM plus opt-in
   * authoring syntax and technical abbreviation markup. Individual options
   * still override it. Default: unset.
   */
  preset?: "ffm";
}

// oxlint-disable-next-line typescript/consistent-type-definitions -- preserve public declaration merging
export interface CodeHighlighter {
  /** The returned HTML is written verbatim. Escape every untrusted value. */
  codeToHtml(
    code: string,
    options: { lang: string; theme: string; meta?: { __raw: string } },
  ): string;
}

/**
 * Trusted synchronous replacement of a whole code block. Return HTML for HTML
 * entry points or JSX for JSX entry points. Null/undefined uses the next renderer
 * or the built-in output; an empty string removes the block. Exceptions propagate.
 */
export type CodeBlockRenderer = (
  code: string,
  language?: string | null,
  meta?: string | null,
) => string | null | undefined;

/** Try callbacks in order until one returns a string. Promise results are rejected. */
export declare function composeCodeRenderers(
  ...renderers: readonly CodeBlockRenderer[]
): CodeBlockRenderer;

/** Adapt a highlighter to a whole-block HTML callback, preserving highlight error fallback. */
export declare function createHighlighterCodeRenderer(
  highlighter: CodeHighlighter,
  highlightOptions: HighlightOptions,
): CodeBlockRenderer;

/** Replace code blocks with trusted HTML. Escape untrusted values inside the callback. */
export declare function toHtmlWithCodeRenderer(
  markdown: string | Uint8Array,
  renderCode: CodeBlockRenderer,
  options?: Options,
): string;

/** Whole-block HTML rendering with the same metadata as `transform()`. */
export declare function transformWithCodeRenderer(
  markdown: string | Uint8Array,
  renderCode: CodeBlockRenderer,
  options?: Options,
): TransformResult;

// oxlint-disable-next-line typescript/consistent-type-definitions -- preserve public declaration merging
export interface HighlightOptions {
  theme: string;
  fallbackLanguage?: string;
  /** Observe a highlighter exception before escaped-code fallback is used. */
  onHighlightError?: (error: unknown, context: { lang: string }) => void;
}

/**
 * Render Markdown to HTML.
 *
 * Every function and `Renderer` method that takes `markdown` accepts a string
 * or UTF-8 bytes: a `Uint8Array`, which includes a Node.js `Buffer`. Bytes
 * render exactly like the string `Buffer.from(bytes.buffer, bytes.byteOffset,
 * bytes.byteLength).toString('utf8')` returns, so invalid UTF-8 becomes
 * U+FFFD. Other values throw a `TypeError` with code `ERR_INVALID_ARG_TYPE`.
 */
export declare function toHtml(markdown: string | Uint8Array, options?: Options): string;

/** Render UTF-8 HTML directly into a Node.js Buffer. */
export declare function toHtmlBuffer(markdown: string | Uint8Array, options?: Options): Buffer;

/** Reusable Markdown renderer with fixed options. */
export declare class Renderer {
  constructor(options?: Options);
  /** Render one document while retaining parser scratch allocations. */
  toHtml(markdown: string | Uint8Array): string;
  /** Render UTF-8 HTML directly into a Node.js Buffer. */
  toHtmlBuffer(markdown: string | Uint8Array): Buffer;
}

/**
 * Render code blocks with a trusted synchronous highlighter.
 * Highlighter exceptions fall back to ferromark's escaped code-block output
 * and can be observed with `onHighlightError`.
 */
// The four arguments are the stable public API shape.
// eslint-disable-next-line max-params
export declare function toHtmlWithHighlighter(
  markdown: string | Uint8Array,
  highlighter: CodeHighlighter,
  highlightOptions: HighlightOptions,
  options?: Options,
): string;

/** One document heading, in source order. */
// oxlint-disable-next-line typescript/consistent-type-definitions -- preserve public declaration merging
export interface Heading {
  /** Heading level, 1-6. */
  level: number;
  /** The generated slug; present when the `headingIds` option is enabled. */
  id?: string;
  /** Plain heading text with inline markup and HTML tags removed. */
  text: string;
}

/** Result of `transform`: HTML plus document metadata. */
// oxlint-disable-next-line typescript/consistent-type-definitions -- preserve public declaration merging
export interface TransformResult {
  html: string;
  /** Document headings for table-of-contents rendering. */
  headings: Heading[];
  /**
   * Raw front matter text (between the delimiters); present when the
   * `frontMatter` option is enabled and the document starts with a block.
   */
  frontMatter?: string;
}

/** Render Markdown and return HTML together with headings and front matter. */
export declare function transform(
  markdown: string | Uint8Array,
  options?: Options,
): TransformResult;

/**
 * `transform` with code blocks rendered by a trusted synchronous highlighter.
 * Highlighter exceptions fall back to ferromark's escaped code-block output
 * and can be observed with `onHighlightError`.
 */
// The four arguments are the stable public API shape.
// eslint-disable-next-line max-params
export declare function transformWithHighlighter(
  markdown: string | Uint8Array,
  highlighter: CodeHighlighter,
  highlightOptions: HighlightOptions,
  options?: Options,
): TransformResult;

/** Compile authored source as a program. Unlike HTML rendering, this is not an untrusted-content boundary. */
export type CompileJsxOptions = {
  /** Source grammar; defaults to Markdown. MDX enables strict JavaScript/JSX parsing. */
  format?: "md" | "mdx";
  /** Prefix generated Markdown tags (for example `_components.p`). Authored JSX is unchanged. */
  componentPrefix?: string;
  /** Map normalized callout kinds to JSX component identifiers. */
  calloutComponents?: Record<string, string>;
  /** Map fenced languages to JSX component identifiers, such as `mermaid: "Mermaid"`. */
  codeComponents?: Record<string, string>;
  /** Wrap highlighted fences in this component, with metadata props and rendered JSX children. */
  codeBlockComponent?: string;
  /** Omit the first top-level H1 only when its trimmed visible text equals this title. */
  omitTitleHeading?: string;
  /** `"body"` (the default) returns a JSX fragment with its parts; `"module"` returns a complete module. */
  output?: "body";
} & Omit<
  Options,
  | "mdx"
  | "renderPolicy"
  | "allowHtml"
  | "disallowedRawHtml"
  | "tableColgroup"
  | "tableColumnNames"
  | "linkBasePath"
  | "autoAbbreviations"
  | "abbreviations"
  | "preset"
>;

/**
 * Options for a complete MDX module. Generated Markdown elements are members of `_components`,
 * so `componentPrefix` is not available.
 */
export type CompileJsxModuleOptions = {
  output: "module";
  /** Module that exports `useMDXComponents`. Its components rank below `props.components`. */
  providerImportSource?: string;
  /** Source file name, recorded as the source of the source map. */
  filename?: string;
  /** Export `MDXContent` as the default export. Set `false` to append your own. Defaults to `true`. */
  defaultExport?: boolean;
  /**
   * Names you declare in code you add to the module. A component reference to one of them uses
   * that binding instead of the provider, and an authored declaration of the same name is an error.
   */
  reservedBindings?: string[];
} & Omit<CompileJsxOptions, "output" | "componentPrefix">;

/** A trusted hook may replace an entire code block with JSX. Exceptions propagate. */
export type JsxCodeRenderer = CodeBlockRenderer;

/** Heading metadata with its original UTF-8 byte range, including heading syntax. */
export type JsxHeading = {
  start: number;
  end: number;
} & Heading;

export type JsxResult = {
  /** JSX fragment expression; contains no framework imports or module scaffolding. */
  body: string;
  /** Preserved authored module statements; offsets refer to original UTF-8 source bytes. */
  esm: Array<{ value: string; start: number; end: number }>;
  /** Authored JSX component root identifiers in first-reference order. */
  components: string[];
  /** Generated Markdown intrinsic names, including names under componentPrefix. */
  elements: string[];
  codeBlocks: Array<{ code: string; language?: string; meta?: string }>;
  headings: JsxHeading[];
  frontMatter?: string;
  frontMatterSpan?: { start: number; end: number };
  frontMatterKind?: "yaml" | "toml";
  /** Original UTF-8 range of the matching top-level H1 omitted from the body. */
  omittedTitleHeadingSpan?: { start: number; end: number };
  /** Zero-based positions; columns count UTF-16 code units. */
  mappings: Array<{
    generatedLine: number;
    generatedColumn: number;
    sourceLine: number;
    sourceColumn: number;
  }>;
};

/** A version 3 source map from module code to the Markdown/MDX source. */
export type JsxModuleMap = {
  version: 3;
  /** The `filename` option, or an empty name. */
  sources: string[];
  sourcesContent: string[];
  names: string[];
  /** Zero-based lines; columns count UTF-16 code units. */
  mappings: string;
};

export type JsxModuleResult = {
  /**
   * A complete ES module that still contains JSX: authored ESM in document order, the content
   * function, and `MDXContent`. It imports no framework; compile it with your JSX transform.
   */
  code: string;
  map: JsxModuleMap;
  /** Names the authored ESM exports. An authored default export becomes the layout and is not listed. */
  exports: string[];
  /** Names the authored ESM binds at the top level: imports, exported declarations, a named default export. */
  bindings: string[];
  codeBlocks: Array<{ code: string; language?: string; meta?: string }>;
  headings: JsxHeading[];
  frontMatter?: string;
  frontMatterSpan?: { start: number; end: number };
  frontMatterKind?: "yaml" | "toml";
  /** Original UTF-8 range of the matching top-level H1 omitted from the module. */
  omittedTitleHeadingSpan?: { start: number; end: number };
};

export declare function compileJsx(
  markdown: string | Uint8Array,
  options: CompileJsxModuleOptions,
  renderCode?: JsxCodeRenderer,
): JsxModuleResult;
export declare function compileJsx(
  markdown: string | Uint8Array,
  options?: CompileJsxOptions,
  renderCode?: JsxCodeRenderer,
): JsxResult;

/** Parser and native transform settings fixed when a document is prepared. */
type JsxRenderOptionKey =
  | "componentPrefix"
  | "calloutComponents"
  | "codeComponents"
  | "codeBlockComponent"
  | "omitTitleHeading"
  | "headingIds"
  | "headingOffset"
  | "headingIdPrefix"
  | "callouts";
export type JsxPreparationOptions = {
  /** Capture reference provenance before native passes. Default: false; required for collection with passes. */
  trackReferences?: boolean;
} & Omit<CompileJsxOptions, JsxRenderOptionKey | "output">;

/** Render-time choices for a prepared JSX body. */
export type JsxRenderOptions = Pick<CompileJsxOptions, JsxRenderOptionKey>;

/** Render-time choices for a prepared MDX module. */
export type JsxRenderModuleOptions = JsxRenderOptions &
  Pick<
    CompileJsxModuleOptions,
    "providerImportSource" | "filename" | "defaultExport" | "reservedBindings"
  >;

/** Snapshot metadata computed during preparation without JSX body rendering. */
export type JsxPreparedMetadata = Readonly<{
  /** Top-level authored module statements and their original UTF-8 ranges. */
  esm: ReadonlyArray<Readonly<{ value: string; start: number; end: number }>>;
  /** Fenced code descriptors in source order. */
  codeBlocks: ReadonlyArray<Readonly<{ code: string; language?: string; meta?: string }>>;
  /** Preparation-time outline, before render-time title omission and final ID planning. */
  outline: ReadonlyArray<Readonly<JsxHeading>>;
  frontMatter?: string;
  frontMatterSpan?: Readonly<{ start: number; end: number }>;
  frontMatterKind?: "yaml" | "toml";
}>;

/** Source-preserving Markdown projection choices; no JavaScript is evaluated. */
export type MarkdownProjectionOptions = {
  /** Preserve recognized front matter by default, or omit its delimiters and body. */
  frontMatter?: "preserve" | "omit";
  /** Omit the first top-level H1 only if its display text matches this title. */
  omitTitleHeading?: string;
};

/** A loss or omission at an original half-open UTF-8 byte range `[start, end)`. */
export type MarkdownProjectionDiagnostic = Readonly<{
  code:
    | "esm-omitted"
    | "jsx-wrapper-unwrapped"
    | "expression-omitted"
    | "self-closing-element-omitted"
    | "raw-html-omitted"
    | "unsupported-jsx-omitted"
    | "raw-text-element-omitted"
    | "front-matter-omitted"
    | "title-heading-omitted";
  message: string;
  start: number;
  end: number;
}>;

/** Static Markdown copied from original source, with explicit omission diagnostics. */
export type MarkdownProjectionResult = Readonly<{
  /** Original Markdown and code; native pass changes are not serialized. */
  markdown: string;
  diagnostics: readonly MarkdownProjectionDiagnostic[];
  omittedTitleHeadingSpan?: Readonly<{ start: number; end: number }>;
}>;

/** One source occurrence, or a reference created/changed by native passes. */
export type DocumentReference = Readonly<{
  kind: "link" | "image" | "autolink" | "mdxAttribute";
  destination: string;
  /** Markdown label or image alt text, when available. */
  text?: string;
  title?: string;
  /** Authored MDX tag name, including custom components. */
  element?: string;
  /** Literal MDX href/src attribute name. */
  attribute?: "href" | "src";
  /** Half-open original UTF-8 byte range; absent for transformed references. */
  span?: Readonly<{ start: number; end: number }>;
  provenance: "source" | "transformed";
  /** Related original occurrence for a changed retained node; never a generated value span. */
  originalSpan?: Readonly<{ start: number; end: number }>;
}>;

/** Asset-free preparation using the same syntax and passes as CodeCompiler.prepare. */
export declare function prepareDocument(
  markdown: string | Uint8Array,
  preparationOptions?: JsxPreparationOptions,
): PreparedJsxDocument;

/** Static text projection choices; heading choices must match render options for matching anchors. */
export type TextExtractionOptions = Readonly<{
  /** Include fenced and indented code (default true). Inline code is always retained. */
  includeBlockCode?: boolean;
  headingIds?: boolean;
  headingOffset?: number;
  headingIdPrefix?: string;
  omitTitleHeading?: string;
  callouts?: boolean;
}>;

/** Half-open UTF-8 byte range. Source spans and normalized text ranges are distinct. */
export type TextByteRange = Readonly<{ start: number; end: number }>;

/** A heading plus its text up to the next heading, in document order. */
export type TextSection = Readonly<{
  heading: string;
  level: number;
  /** Index of the nearest earlier section with a lower effective heading level. */
  parentIndex?: number;
  id?: string;
  /** Original heading syntax, when its provenance is available. */
  headingSpan?: TextByteRange;
  /** Original heading through next heading/document end; absent for unavailable/reordered spans. */
  sourceSpan?: TextByteRange;
  /** UTF-8 byte range in TextExtraction.text, including the heading. Encode before slicing in JS. */
  textRange: TextByteRange;
  /** The normalized text in textRange; does not repeat descendant sections. */
  text: string;
}>;

/** Immutable owned text analysis; raw HTML, ESM, expressions, and runtime MDX output are omitted. */
export type TextExtraction = Readonly<{
  text: string;
  sections: readonly TextSection[];
  omittedTitleHeadingSpan?: TextByteRange;
}>;

/** Immutable native document handle returned by `prepareDocument` or `CodeCompiler.prepare`. */
export type PreparedJsxDocument = Readonly<{
  /** Metadata snapshot computed from the prepared AST without rendering JSX. */
  metadata: JsxPreparedMetadata;
  /** Ordered references. With native passes, preparation must set trackReferences: true or this throws. */
  collectReferences(): readonly DocumentReference[];
  /** Bulk static text and final sections, without rendering/highlighting or rerunning passes. */
  extractText(options?: TextExtractionOptions): TextExtraction;
  /** Project original source without parsing again, rendering, highlighting, or running passes. */
  projectMarkdown(options?: MarkdownProjectionOptions): MarkdownProjectionResult;
  /** Render a body. Repeated calls do not mutate the prepared tree or rerun passes. */
  render(options?: JsxRenderOptions, renderCode?: JsxCodeRenderer): JsxResult;
  /** Render a complete module. Module and body choices can be varied per call. */
  renderModule(options?: JsxRenderModuleOptions, renderCode?: JsxCodeRenderer): JsxModuleResult;
}>;

/** Selects an initialized Rust plugin from this addon's compiled factory set. */
export type NativeCodePluginConfig = {
  name: string;
  /** Engine-specific, JSON-serializable settings passed once to the Rust factory. */
  options?: Readonly<Record<string, unknown>>;
};

export type CodeCompilerOptions = {
  /** Ordered plugins. Empty by default; no engine or assets are initialized. */
  plugins?: readonly NativeCodePluginConfig[];
  /** Enable line numbers; HTML also enables VitePress-compatible code annotations. */
  lineNumbers?: boolean;
};

/** Names compiled into this addon. Installing an npm package does not extend this set. */
export declare function availableCodePlugins(): string[];

/** Reusable native HTML/JSX compiler. Rust plugins run without per-block JavaScript calls. */
export declare class CodeCompiler {
  constructor(options?: CodeCompilerOptions);
  toHtml(markdown: string | Uint8Array, options?: Options): string;
  transform(markdown: string | Uint8Array, options?: Options): TransformResult;
  prepare(
    markdown: string | Uint8Array,
    preparationOptions?: JsxPreparationOptions,
  ): PreparedJsxDocument;
  /** Standalone intrinsic code output consults highlighting plugins only. */
  renderCodeBlock(input: JsxCodeBlockRenderInput): JsxCodeBlockRenderResult;
  compile(
    markdown: string | Uint8Array,
    options: CompileJsxModuleOptions,
    renderCode?: JsxCodeRenderer,
  ): JsxModuleResult;
  compile(
    markdown: string | Uint8Array,
    options?: CompileJsxOptions,
    renderCode?: JsxCodeRenderer,
  ): JsxResult;
}

/** Input to `CodeCompiler.renderCodeBlock`; `language` and `meta` use fence syntax. */
export type JsxCodeBlockRenderInput = {
  code: string;
  language?: string | null;
  meta?: string | null;
};

/** Standalone intrinsic JSX and metadata parsed with the native fence parser. */
export type JsxCodeBlockRenderResult = {
  /** Intrinsic `<pre><code>` JSX, including the document fence's trailing newline. */
  jsx: string;
  language?: string;
  title?: string;
  label?: string;
  /** Effective setting after the compiler default and fence metadata are applied. */
  lineNumbers: boolean;
};
