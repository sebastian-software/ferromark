import { createRequire } from 'node:module'
import process from 'node:process'
import { fileURLToPath } from 'node:url'

const require = createRequire(import.meta.url)

const optionKeys = new Set([
  'renderPolicy',
  'allowHtml',
  'allowLinkRefs',
  'tables',
  'mergedTableCells',
  'tableColumnWidths',
  'strikethrough',
  'highlight',
  'superscript',
  'subscript',
  'taskLists',
  'autolinkLiterals',
  'disallowedRawHtml',
  'footnotes',
  'inlineFootnotes',
  'frontMatter',
  'headingIds',
  'math',
  'callouts',
  'definitionLists',
  'lineComments',
  'indentedCodeBlocks',
  'linkBasePath',
])

/** @param {import('./index.mjs').Options | null | undefined} options */
function validateOptions(options) {
  if (options == null) {
    return
  }
  if (typeof options !== 'object' && typeof options !== 'function') {
    throw new TypeError('options must be an object')
  }
  for (const key of Reflect.ownKeys(options)) {
    if (typeof key !== 'string' || !optionKeys.has(key)) {
      throw new TypeError(`unknown option "${String(key)}"`)
    }
  }
}

/**
 * @param {string} markdown
 * @param {import('./index.mjs').Options} [options]
 */
export function toHtml(markdown, options) {
  validateOptions(options)
  return loadNative().toHtml(markdown, options)
}

/**
 * @param {string} markdown
 * @param {import('./index.mjs').Options} [options]
 */
export function transform(markdown, options) {
  validateOptions(options)
  return loadNative().transform(markdown, options)
}

/**
 * @param {import('./index.mjs').CodeHighlighter} highlighter
 * @param {import('./index.mjs').HighlightOptions} highlightOptions
 */
function highlighterRenderer(highlighter, highlightOptions) {
  if (!highlighter || typeof highlighter.codeToHtml !== 'function') {
    throw new TypeError('highlighter must provide a synchronous codeToHtml method')
  }
  if (!highlightOptions || typeof highlightOptions.theme !== 'string') {
    throw new TypeError('highlightOptions.theme must be a string')
  }

  const fallbackLanguage = highlightOptions.fallbackLanguage ?? 'text'
  /**
   * @param {string} code
   * @param {string | null | undefined} language
   * @param {string | null | undefined} meta
   */
  return (code, language, meta) => {
    try {
      return highlighter.codeToHtml(code, {
        lang: language ?? fallbackLanguage,
        theme: highlightOptions.theme,
        ...(meta ? { meta: { __raw: meta } } : {}),
      })
    }
    catch {
      return null
    }
  }
}

/**
 * @param {string} markdown
 * @param {import('./index.mjs').CodeHighlighter} highlighter
 * @param {import('./index.mjs').HighlightOptions} highlightOptions
 * @param {import('./index.mjs').Options} [options]
 */
export function toHtmlWithHighlighter(markdown, highlighter, highlightOptions, options) {
  validateOptions(options)
  const render = highlighterRenderer(highlighter, highlightOptions)
  return loadNative().toHtmlWithRenderer(markdown, options, render)
}

/**
 * @param {string} markdown
 * @param {import('./index.mjs').CodeHighlighter} highlighter
 * @param {import('./index.mjs').HighlightOptions} highlightOptions
 * @param {import('./index.mjs').Options} [options]
 */
export function transformWithHighlighter(markdown, highlighter, highlightOptions, options) {
  validateOptions(options)
  const render = highlighterRenderer(highlighter, highlightOptions)
  return loadNative().transformWithRenderer(markdown, options, render)
}

/**
 * @typedef {(code: string, language?: string | null, meta?: string | null) => string | null} NativeRenderer
 * @typedef {{
 *   toHtml(markdown: string, options?: import('./index.mjs').Options): string
 *   toHtmlWithRenderer(
 *     markdown: string,
 *     options: import('./index.mjs').Options | undefined,
 *     renderer: NativeRenderer,
 *   ): string
 *   transform(
 *     markdown: string,
 *     options?: import('./index.mjs').Options,
 *   ): import('./index.mjs').TransformResult
 *   transformWithRenderer(
 *     markdown: string,
 *     options: import('./index.mjs').Options | undefined,
 *     renderer: NativeRenderer,
 *   ): import('./index.mjs').TransformResult
 * }} NativeBindings
 */

/** @type {NativeBindings | undefined} */
let native

/** @returns {NativeBindings} */
function loadNative() {
  if (native) {
    return native
  }

  const target = nativeTarget()
  const filename = `ferromark.${target}.node`
  let localError
  try {
    native = /** @type {NativeBindings} */ (
      require(fileURLToPath(new URL(filename, import.meta.url)))
    )
    return native
  }
  catch (error) {
    if (!(error instanceof Error) || !('code' in error) || error.code !== 'MODULE_NOT_FOUND') {
      throw error
    }
    localError = error
  }

  const packageName = `ferromark-${target}`
  try {
    native = /** @type {NativeBindings} */ (require(packageName))
    return native
  }
  catch (error) {
    if (!(error instanceof Error) || !('code' in error) || error.code !== 'MODULE_NOT_FOUND') {
      throw error
    }
    throw new Error(
      `ferromark could not load the optional native package ${packageName} for ${process.platform}/${process.arch}`,
      { cause: new AggregateError([localError, error], `No native binary found for ${target}`) },
    )
  }
}

function nativeTarget() {
  const report = /** @type {{ header?: { glibcVersionRuntime?: string } }} */ (
    process.report?.getReport?.()
  )
  const libc = report?.header?.glibcVersionRuntime ? 'gnu' : 'musl'
  const key = process.platform === 'linux'
    ? `${process.platform}-${process.arch}-${libc}`
    : `${process.platform}-${process.arch}`
  /** @type {Record<string, string>} */
  const targets = {
    'darwin-arm64': 'darwin-arm64',
    'darwin-x64': 'darwin-x64',
    'linux-arm64-gnu': 'linux-arm64-gnu',
    'linux-arm64-musl': 'linux-arm64-musl',
    'linux-x64-gnu': 'linux-x64-gnu',
    'linux-x64-musl': 'linux-x64-musl',
    'win32-arm64': 'win32-arm64-msvc',
    'win32-x64': 'win32-x64-msvc',
  }

  const target = targets[key]
  if (!target) {
    throw new Error(`ferromark does not support ${process.platform}/${process.arch}`)
  }
  return target
}
