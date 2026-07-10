import { createRequire } from 'node:module'
import process from 'node:process'
import { fileURLToPath } from 'node:url'

const require = createRequire(import.meta.url)

/**
 * @param {string} markdown
 * @param {import('./index.mjs').Options} [options]
 */
export function toHtml(markdown, options) {
  return loadNative().toHtml(markdown, options)
}

/**
 * @param {string} markdown
 * @param {import('./index.mjs').CodeHighlighter} highlighter
 * @param {import('./index.mjs').HighlightOptions} highlightOptions
 * @param {import('./index.mjs').Options} [options]
 */
export function toHtmlWithHighlighter(markdown, highlighter, highlightOptions, options) {
  if (!highlighter || typeof highlighter.codeToHtml !== 'function') {
    throw new TypeError('highlighter must provide a synchronous codeToHtml method')
  }
  if (!highlightOptions || typeof highlightOptions.theme !== 'string') {
    throw new TypeError('highlightOptions.theme must be a string')
  }

  const fallbackLanguage = highlightOptions.fallbackLanguage ?? 'text'
  /** @param {string} code @param {string | null | undefined} language */
  const render = (code, language) => {
    try {
      return highlighter.codeToHtml(code, {
        lang: language ?? fallbackLanguage,
        theme: highlightOptions.theme,
      })
    }
    catch {
      return null
    }
  }

  return loadNative().toHtmlWithRenderer(markdown, options, render)
}

/**
 * @typedef {{
 *   toHtml(markdown: string, options?: import('./index.mjs').Options): string
 *   toHtmlWithRenderer(
 *     markdown: string,
 *     options: import('./index.mjs').Options | undefined,
 *     renderer: (code: string, language?: string | null) => string | null,
 *   ): string
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
    throw new Error(
      `ferromark does not include a native binary for ${process.platform}/${process.arch} (${target})`,
      { cause: error },
    )
  }
}

function nativeTarget() {
  const key = `${process.platform}-${process.arch}`
  /** @type {Record<string, string>} */
  const targets = {
    'darwin-arm64': 'darwin-arm64',
    'darwin-x64': 'darwin-x64',
    'linux-arm64': 'linux-arm64-gnu',
    'linux-x64': 'linux-x64-gnu',
    'win32-arm64': 'win32-arm64-msvc',
    'win32-x64': 'win32-x64-msvc',
  }

  const report = /** @type {{ header?: { glibcVersionRuntime?: string } }} */ (
    process.report?.getReport?.()
  )
  if (process.platform === 'linux' && !report?.header?.glibcVersionRuntime) {
    throw new Error('ferromark currently supports glibc Linux builds; musl is not supported')
  }

  const target = targets[key]
  if (!target) {
    throw new Error(`ferromark does not support ${process.platform}/${process.arch}`)
  }
  return target
}
