export type RenderPolicy = 'untrusted' | 'trusted'

export interface Options {
  renderPolicy?: RenderPolicy
  allowHtml?: boolean
  allowLinkRefs?: boolean
  tables?: boolean
  strikethrough?: boolean
  highlight?: boolean
  superscript?: boolean
  subscript?: boolean
  taskLists?: boolean
  autolinkLiterals?: boolean
  disallowedRawHtml?: boolean
  footnotes?: boolean
  frontMatter?: boolean
  headingIds?: boolean
  math?: boolean
  callouts?: boolean
  lineComments?: boolean
}

export interface CodeHighlighter {
  /** The returned HTML is written verbatim. Escape every untrusted value. */
  codeToHtml(code: string, options: { lang: string; theme: string }): string
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
