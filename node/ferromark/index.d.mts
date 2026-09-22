import type { Buffer } from "node:buffer";

export type RenderPolicy = "untrusted" | "trusted";

// oxlint-disable-next-line typescript/consistent-type-definitions -- preserve public declaration merging
export interface Options {
  tableAttributes?: boolean;
  headingAttributes?: boolean;
  wikiLinks?: boolean;
  cjkEmphasis?: boolean;
  /** Recognize MDX syntax; does not compile or execute JavaScript. */
  mdx?: boolean;
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
}

// oxlint-disable-next-line typescript/consistent-type-definitions -- preserve public declaration merging
export interface CodeHighlighter {
  /** The returned HTML is written verbatim. Escape every untrusted value. */
  codeToHtml(
    code: string,
    options: { lang: string; theme: string; meta?: { __raw: string } },
  ): string;
}

// oxlint-disable-next-line typescript/consistent-type-definitions -- preserve public declaration merging
export interface HighlightOptions {
  theme: string;
  fallbackLanguage?: string;
  /** Observe a highlighter exception before escaped-code fallback is used. */
  onHighlightError?: (error: unknown, context: { lang: string }) => void;
}

export declare function toHtml(markdown: string, options?: Options): string;

/** Render UTF-8 HTML directly into a Node.js Buffer. */
export declare function toHtmlBuffer(markdown: string, options?: Options): Buffer;

/** Reusable Markdown renderer with fixed options. */
export declare class Renderer {
  constructor(options?: Options);
  /** Render one document while retaining parser scratch allocations. */
  toHtml(markdown: string): string;
  /** Render UTF-8 HTML directly into a Node.js Buffer. */
  toHtmlBuffer(markdown: string): Buffer;
}

/**
 * Render code blocks with a trusted synchronous highlighter.
 * Highlighter exceptions fall back to ferromark's escaped code-block output
 * and can be observed with `onHighlightError`.
 */
// The four arguments are the stable public API shape.
// eslint-disable-next-line max-params
export declare function toHtmlWithHighlighter(
  markdown: string,
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
export declare function transform(markdown: string, options?: Options): TransformResult;

/**
 * `transform` with code blocks rendered by a trusted synchronous highlighter.
 * Highlighter exceptions fall back to ferromark's escaped code-block output
 * and can be observed with `onHighlightError`.
 */
// The four arguments are the stable public API shape.
// eslint-disable-next-line max-params
export declare function transformWithHighlighter(
  markdown: string,
  highlighter: CodeHighlighter,
  highlightOptions: HighlightOptions,
  options?: Options,
): TransformResult;
