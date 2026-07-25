export type RenderPolicy = 'untrusted' | 'trusted'

export interface Options {
  renderPolicy?: RenderPolicy
  allowHtml?: boolean
  allowLinkRefs?: boolean
  tables?: boolean
  mergedTableCells?: boolean
  tableColumnWidths?: boolean
  strikethrough?: boolean
  highlight?: boolean
  superscript?: boolean
  subscript?: boolean
  taskLists?: boolean
  autolinkLiterals?: boolean
  disallowedRawHtml?: boolean
  footnotes?: boolean
  inlineFootnotes?: boolean
  frontMatter?: boolean
  headingIds?: boolean
  math?: boolean
  callouts?: boolean
  definitionLists?: boolean
  lineComments?: boolean
  indentedCodeBlocks?: boolean
  /**
   * Prefix internal absolute link destinations (starting with `/`) with
   * this base path, for sites deployed under a subpath. Image sources and
   * autolinks are not rewritten.
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
}

export declare function toHtml(markdown: string, options?: Options): string

/**
 * Render fenced code with a trusted synchronous highlighter.
 * Highlighter errors fall back to ferromark's escaped code-block output.
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
 * Highlighter errors fall back to ferromark's escaped code-block output.
 */
export declare function transformWithHighlighter(
  markdown: string,
  highlighter: CodeHighlighter,
  highlightOptions: HighlightOptions,
  options?: Options,
): TransformResult
