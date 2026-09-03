import type { Buffer } from 'node:buffer'

export type RenderPolicy = 'untrusted' | 'trusted'

export interface Options {
  /** Output trust boundary. Default: `'untrusted'`; use `'trusted'` only for trusted Markdown. */
  renderPolicy?: RenderPolicy
  /** Parse raw inline and block HTML. Default: on; untrusted output still escapes it. */
  allowHtml?: boolean
  /** Resolve reference definitions and reference-style links. Default: on. */
  allowLinkRefs?: boolean
  /** Enable GFM pipe tables. Default: on. */
  tables?: boolean
  /** Enable MultiMarkdown-style table column spans. Default: off; requires `tables`. */
  mergedTableCells?: boolean
  /** Emit numeric table column-width hints. Default: off; requires `tables`. */
  tableColumnWidths?: boolean
  /** Enable GFM `~~strikethrough~~`. Default: on. */
  strikethrough?: boolean
  /** Enable `==highlight==`. Default: off. */
  highlight?: boolean
  /** Enable `^superscript^`. Default: off. */
  superscript?: boolean
  /** Enable `~subscript~`. Default: off. */
  subscript?: boolean
  /** Enable GFM task lists. Default: on. */
  taskLists?: boolean
  /** Enable bare URL, `www`, and email autolinks. Default: off. */
  autolinkLiterals?: boolean
  /** Filter GFM-disallowed raw HTML in trusted mode. Default: on; this is not a sanitizer. */
  disallowedRawHtml?: boolean
  /** Enable `[^label]` footnotes. Default: off. */
  footnotes?: boolean
  /** Enable Pandoc-style `^[note]` footnotes. Default: off. */
  inlineFootnotes?: boolean
  /** Extract a leading `---` or `+++` front-matter block. Default: off. */
  frontMatter?: boolean
  /** Generate GitHub-compatible heading IDs. Default: on. */
  headingIds?: boolean
  /** Enable `$inline$` and `$$display$$` math. Default: off. */
  math?: boolean
  /** Enable GitHub-style blockquote callouts. Default: on. */
  callouts?: boolean
  /** Enable PHP Markdown Extra definition lists. Default: off. */
  definitionLists?: boolean
  /** Omit physical-line-start `//` source comments. Default: off. */
  lineComments?: boolean
  /** Parse four-space indented code blocks. Default: on. */
  indentedCodeBlocks?: boolean
  /**
   * Prefix internal absolute link destinations (starting with `/`) with
   * this base path, for sites deployed under a subpath. Image sources and
   * autolinks are not rewritten. Default: unset.
   */
  linkBasePath?: string
}

export interface CodeHighlighter {
  /** The returned HTML is written verbatim. Escape every untrusted value. */
  codeToHtml(
    code: string,
    options: { lang: string; theme: string; meta?: { __raw: string } },
  ): string
}

export interface HighlightOptions {
  theme: string
  fallbackLanguage?: string
  /** Observe a highlighter exception before escaped-code fallback is used. */
  onHighlightError?: (error: unknown, context: { lang: string }) => void
}

export declare function toHtml(markdown: string, options?: Options): string

/** Render UTF-8 HTML directly into a Node.js Buffer. */
export declare function toHtmlBuffer(markdown: string, options?: Options): Buffer

/** Reusable Markdown renderer with fixed options. */
export declare class Renderer {
  constructor(options?: Options)
  /** Render one document while retaining parser scratch allocations. */
  toHtml(markdown: string): string
  /** Render UTF-8 HTML directly into a Node.js Buffer. */
  toHtmlBuffer(markdown: string): Buffer
}

/**
 * Render fenced code with a trusted synchronous highlighter.
 * Highlighter exceptions fall back to ferromark's escaped code-block output
 * and can be observed with `onHighlightError`.
 */
export declare function toHtmlWithHighlighter(
  markdown: string,
  highlighter: CodeHighlighter,
  highlightOptions: HighlightOptions,
  options?: Options,
): string

/** One document heading, in source order. */
export interface Heading {
  /** Heading level, 1-6. */
  level: number
  /** The generated slug; present when the `headingIds` option is enabled. */
  id?: string
  /** Plain heading text with inline markup and HTML tags removed. */
  text: string
}

/** Result of `transform`: HTML plus document metadata. */
export interface TransformResult {
  html: string
  /** Document headings for table-of-contents rendering. */
  headings: Heading[]
  /**
   * Raw front matter text (between the delimiters); present when the
   * `frontMatter` option is enabled and the document starts with a block.
   */
  frontMatter?: string
}

/** Render Markdown and return HTML together with headings and front matter. */
export declare function transform(markdown: string, options?: Options): TransformResult

/**
 * `transform` with fenced code rendered by a trusted synchronous highlighter.
 * Highlighter exceptions fall back to ferromark's escaped code-block output
 * and can be observed with `onHighlightError`.
 */
export declare function transformWithHighlighter(
  markdown: string,
  highlighter: CodeHighlighter,
  highlightOptions: HighlightOptions,
  options?: Options,
): TransformResult
