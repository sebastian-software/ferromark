import { createRequire } from 'node:module'
import process from 'node:process'
import { fileURLToPath } from 'node:url'

import { nativeTarget as resolveNativeTarget } from './native-target.mjs'

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
 * Render UTF-8 HTML into a Node.js Buffer.
 * @param {string} markdown
 * @param {import('./index.mjs').Options} [options]
 * @returns {import('node:buffer').Buffer}
 */
export function toHtmlBuffer(markdown, options) {
  validateOptions(options)
  return loadNative().toHtmlBuffer(markdown, options)
}

/** Reusable Markdown renderer with fixed options. */
export class Renderer {
  /** @type {NativeRendererSession} */
  #native

  /** @param {import('./index.mjs').Options} [options] */
  constructor(options) {
    validateOptions(options)
    const NativeRenderer = loadNative().Renderer
    this.#native = new NativeRenderer(options)
  }

  /** @param {string} markdown */
  toHtml(markdown) {
    return this.#native.toHtml(markdown)
  }

  /** @param {string} markdown @returns {import('node:buffer').Buffer} */
  toHtmlBuffer(markdown) {
    return this.#native.toHtmlBuffer(markdown)
  }
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
  if (
    highlightOptions.onHighlightError != null
    && typeof highlightOptions.onHighlightError !== 'function'
  ) {
    throw new TypeError('highlightOptions.onHighlightError must be a function')
  }

  const fallbackLanguage = highlightOptions.fallbackLanguage ?? 'text'
  const onHighlightError = highlightOptions.onHighlightError
  /**
   * @param {string} code
   * @param {string | null | undefined} language
   * @param {string | null | undefined} meta
   */
  return (code, language, meta) => {
    const lang = language ?? fallbackLanguage
    try {
      return highlighter.codeToHtml(code, {
        lang,
        theme: highlightOptions.theme,
        ...(meta ? { meta: { __raw: meta } } : {}),
      })
    }
    catch (error) {
      onHighlightError?.(error, { lang })
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
 * @typedef {(code: string, language?: string | null, meta?: string | null) => string | null} NativeFencedCodeRenderer
 * @typedef {{
 *   toHtml(markdown: string): string
 *   toHtmlBuffer(markdown: string): import('node:buffer').Buffer
 * }} NativeRendererSession
 * @typedef {{
 *   Renderer: new (options?: import('./index.mjs').Options) => NativeRendererSession
 *   toHtml(markdown: string, options?: import('./index.mjs').Options): string
 *   toHtmlBuffer(
 *     markdown: string,
 *     options?: import('./index.mjs').Options,
 *   ): import('node:buffer').Buffer
 *   toHtmlWithRenderer(
 *     markdown: string,
 *     options: import('./index.mjs').Options | undefined,
 *     renderer: NativeFencedCodeRenderer,
 *   ): string
 *   transform(
 *     markdown: string,
 *     options?: import('./index.mjs').Options,
 *   ): import('./index.mjs').TransformResult
 *   transformWithRenderer(
 *     markdown: string,
 *     options: import('./index.mjs').Options | undefined,
 *     renderer: NativeFencedCodeRenderer,
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
    if (hasErrorCode(error, 'ERR_DLOPEN_FAILED')) {
      throw nativeBinaryLoadError(filename, 'the local package', target, error)
    }
    if (!hasErrorCode(error, 'MODULE_NOT_FOUND')) {
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
    if (hasErrorCode(error, 'ERR_DLOPEN_FAILED')) {
      throw nativeBinaryLoadError(filename, `the optional package ${packageName}`, target, error)
    }
    if (!hasErrorCode(error, 'MODULE_NOT_FOUND')) {
      throw error
    }
    throw new Error(
      `ferromark could not load the optional native package ${packageName} for ${process.platform}/${process.arch}`,
      { cause: new AggregateError([localError, error], `No native binary found for ${target}`) },
    )
  }
}

/** @param {unknown} error @param {string} code */
function hasErrorCode(error, code) {
  return error instanceof Error && 'code' in error && error.code === code
}

/**
 * @param {string} filename
 * @param {string} source
 * @param {string} target
 * @param {unknown} cause
 */
function nativeBinaryLoadError(filename, source, target, cause) {
  return new Error(
    `ferromark could not load native binary ${filename} from ${source} for ${process.platform}/${process.arch} (ERR_DLOPEN_FAILED). ${nativeLoadHint(target)}`,
    { cause },
  )
}

/** @param {string} target */
function nativeLoadHint(target) {
  if (target.endsWith('-gnu')) {
    return 'Check that glibc 2.17 or newer is available and that no required shared library is missing.'
  }
  if (target.endsWith('-musl')) {
    return 'Check that the musl runtime is compatible and that no required shared library is missing.'
  }
  if (target.startsWith('win32-')) {
    return 'Install or repair the Microsoft Visual C++ Redistributable and verify the binary architecture.'
  }
  return 'Check the macOS version and binary architecture, and whether quarantine or code-signing policy blocked the addon.'
}

function nativeTarget() {
  const report = process.platform === 'linux'
    ? /** @type {{ header?: { glibcVersionRuntime?: string } }} */ (
        process.report?.getReport?.()
      )
    : undefined
  return resolveNativeTarget(
    process.platform,
    process.arch,
    report?.header?.glibcVersionRuntime,
  )
}
