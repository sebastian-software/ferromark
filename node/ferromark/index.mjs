import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";

import { linuxLibc, nativeTarget as resolveNativeTarget } from "./native-target.mjs";

const require = createRequire(import.meta.url);

const optionKeys = new Set([
  "renderPolicy",
  "allowHtml",
  "tableAttributes",
  "headingAttributes",
  "wikiLinks",
  "cjkEmphasis",
  "mdx",
  "tables",
  "mergedTableCells",
  "tableColgroup",
  "tableColumnNames",
  "strikethrough",
  "superscript",
  "subscript",
  "taskLists",
  "autolinkLiterals",
  "disallowedRawHtml",
  "footnotes",
  "highlight",
  "inlineFootnotes",
  "allowLinkRefs",
  "frontMatter",
  "headingIds",
  "math",
  "callouts",
  "definitionLists",
  "lineComments",
  "linkBasePath",
]);

/** @param {import('./index.mjs').Options | null | undefined} options Options to validate. */
function validateOptions(options) {
  if (options == null) {
    return;
  }
  if (typeof options !== "object" && typeof options !== "function") {
    throw new TypeError("options must be an object");
  }
  for (const key of Reflect.ownKeys(options)) {
    if (typeof key !== "string" || !optionKeys.has(key)) {
      throw new TypeError(`unknown option "${String(key)}"`);
    }
  }
}

/**
 * @param {string} markdown Markdown source to render.
 * @param {import('./index.mjs').Options} [options] Rendering options.
 */
export function toHtml(markdown, options) {
  validateOptions(options);
  return loadNative().toHtml(markdown, options);
}

/**
 * Render UTF-8 HTML into a Node.js Buffer.
 * @param {string} markdown Markdown source to render.
 * @param {import('./index.mjs').Options} [options] Rendering options.
 * @returns {import('node:buffer').Buffer} Rendered HTML as a UTF-8 buffer.
 */
export function toHtmlBuffer(markdown, options) {
  validateOptions(options);
  return loadNative().toHtmlBuffer(markdown, options);
}

/** Reusable Markdown renderer with fixed options. */
export class Renderer {
  /** @type {NativeRendererSession} */
  #native;

  /** @param {import('./index.mjs').Options} [options] Rendering options. */
  constructor(options) {
    validateOptions(options);
    const NativeRenderer = loadNative().Renderer;
    this.#native = new NativeRenderer(options);
  }

  /** @param {string} markdown Markdown source to render. */
  toHtml(markdown) {
    return this.#native.toHtml(markdown);
  }

  /** @param {string} markdown Markdown source to render. @returns {import('node:buffer').Buffer} Rendered HTML. */
  toHtmlBuffer(markdown) {
    return this.#native.toHtmlBuffer(markdown);
  }
}

/**
 * @param {string} markdown Markdown source to transform.
 * @param {import('./index.mjs').Options} [options] Transformation options.
 */
export function transform(markdown, options) {
  validateOptions(options);
  return loadNative().transform(markdown, options);
}

/**
 * @param {import('./index.mjs').CodeHighlighter} highlighter Synchronous code highlighter.
 * @param {import('./index.mjs').HighlightOptions} highlightOptions Highlighter options.
 */
function highlighterRenderer(highlighter, highlightOptions) {
  if (!highlighter || typeof highlighter.codeToHtml !== "function") {
    throw new TypeError("highlighter must provide a synchronous codeToHtml method");
  }
  if (!highlightOptions || typeof highlightOptions.theme !== "string") {
    throw new TypeError("highlightOptions.theme must be a string");
  }
  if (
    highlightOptions.onHighlightError != null &&
    typeof highlightOptions.onHighlightError !== "function"
  ) {
    throw new TypeError("highlightOptions.onHighlightError must be a function");
  }

  const fallbackLanguage = highlightOptions.fallbackLanguage ?? "text";
  const onHighlightError = highlightOptions.onHighlightError;
  /**
   * @param {string} code Code block source.
   * @param {string | null | undefined} language Code language.
   * @param {string | null | undefined} meta Code block metadata.
   */
  return (code, language, meta) => {
    const lang = language ?? fallbackLanguage;
    try {
      return highlighter.codeToHtml(code, {
        lang,
        theme: highlightOptions.theme,
        ...(meta ? { meta: { __raw: meta } } : {}),
      });
    } catch (error) {
      onHighlightError?.(error, { lang });
      return null;
    }
  };
}

/**
 * @param {string} markdown Markdown source to render.
 * @param {import('./index.mjs').CodeHighlighter} highlighter Synchronous code highlighter.
 * @param {import('./index.mjs').HighlightOptions} highlightOptions Highlighter options.
 * @param {import('./index.mjs').Options} [options] Rendering options.
 */
// The four arguments are the stable public API shape.
// eslint-disable-next-line max-params
export function toHtmlWithHighlighter(markdown, highlighter, highlightOptions, options) {
  validateOptions(options);
  const render = highlighterRenderer(highlighter, highlightOptions);
  return loadNative().toHtmlWithRenderer(markdown, options, render);
}

/**
 * @param {string} markdown Markdown source to transform.
 * @param {import('./index.mjs').CodeHighlighter} highlighter Synchronous code highlighter.
 * @param {import('./index.mjs').HighlightOptions} highlightOptions Highlighter options.
 * @param {import('./index.mjs').Options} [options] Transformation options.
 */
// The four arguments are the stable public API shape.
// eslint-disable-next-line max-params
export function transformWithHighlighter(markdown, highlighter, highlightOptions, options) {
  validateOptions(options);
  const render = highlighterRenderer(highlighter, highlightOptions);
  return loadNative().transformWithRenderer(markdown, options, render);
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
let native;

/** @returns {NativeBindings} Loaded native bindings. */
// This loader keeps local and optional-package fallback diagnostics together.
// eslint-disable-next-line max-statements, sonarjs/cognitive-complexity
function loadNative() {
  if (native) {
    return native;
  }

  const target = nativeTarget();
  const filename = `ferromark.${target}.node`;
  let localError;
  try {
    native = /** @type {NativeBindings} */ (
      require(fileURLToPath(new URL(filename, import.meta.url)))
    );
    return native;
  } catch (error) {
    if (hasErrorCode(error, "ERR_DLOPEN_FAILED")) {
      throw nativeBinaryLoadError(filename, "the local package", target, error);
    }
    if (!hasErrorCode(error, "MODULE_NOT_FOUND")) {
      throw error;
    }
    localError = error;
  }

  const packageName = `ferromark-${target}`;
  try {
    native = /** @type {NativeBindings} */ (require(packageName));
    return native;
  } catch (error) {
    if (hasErrorCode(error, "ERR_DLOPEN_FAILED")) {
      throw nativeBinaryLoadError(filename, `the optional package ${packageName}`, target, error);
    }
    if (!hasErrorCode(error, "MODULE_NOT_FOUND")) {
      throw error;
    }
    // Preserve both local and optional lookup failures for actionable diagnostics.
    /* eslint-disable preserve-caught-error -- AggregateError intentionally retains both lookup failures. */
    throw new Error(
      `ferromark could not load the optional native package ${packageName} for ${process.platform}/${process.arch}`,
      {
        // oxlint-disable-next-line preserve-caught-error -- preserve both lookup failures
        cause: new AggregateError([localError, error], `No native binary found for ${target}`),
      },
    );
    /* eslint-enable preserve-caught-error */
  }
}

/** @param {unknown} error Error to inspect. @param {string} code Expected error code. */
function hasErrorCode(error, code) {
  return error instanceof Error && "code" in error && error.code === code;
}

/**
 * @param {string} filename Native binary filename.
 * @param {string} source Load source description.
 * @param {string} target Native package target.
 * @param {unknown} cause Original loader error.
 */
// The four values keep dynamic-loader diagnostics actionable.
// eslint-disable-next-line max-params
function nativeBinaryLoadError(filename, source, target, cause) {
  return new Error(
    `ferromark could not load native binary ${filename} from ${source} for ${process.platform}/${process.arch} (ERR_DLOPEN_FAILED). ${nativeLoadHint(target)}`,
    { cause },
  );
}

/** @param {string} target Native package target. */
function nativeLoadHint(target) {
  if (target.endsWith("-gnu")) {
    return "Check that glibc 2.17 or newer is available and that no required shared library is missing.";
  }
  if (target.endsWith("-musl")) {
    return "Check that the musl runtime is compatible and that no required shared library is missing.";
  }
  if (target.startsWith("win32-")) {
    return "Install or repair the Microsoft Visual C++ Redistributable and verify the binary architecture.";
  }
  return "Check the macOS version and binary architecture, and whether quarantine or code-signing policy blocked the addon.";
}

/** @returns {string} Native package target for the current runtime. */
function nativeTarget() {
  const libc =
    process.platform === "linux" ? linuxLibc(diagnosticReport(), readLoaderHelper) : undefined;
  return resolveNativeTarget(process.platform, process.arch, libc);
}

/**
 * Collect the diagnostic report that identifies the Linux C library.
 *
 * Network interfaces are excluded: enumerating them can stall for seconds in
 * containers with slow DNS or many interfaces, and the loader only reads the
 * report header and the list of shared objects. The previous setting is
 * restored so an application's own reports keep their configured content.
 *
 * `excludeNetwork` is missing from the installed Node.js typings; Node.js has
 * supported it since v13.12, and an unknown property is simply ignored.
 *
 * @returns {import('./native-target.mjs').DiagnosticReport | undefined} Report, if available.
 */
function diagnosticReport() {
  const report = /** @type {{ excludeNetwork?: boolean, getReport(): object } | undefined} */ (
    process.report
  );
  if (typeof report?.getReport !== "function") {
    return;
  }
  const excludeNetwork = report.excludeNetwork;
  try {
    report.excludeNetwork = true;
    return /** @type {import('./native-target.mjs').DiagnosticReport} */ (report.getReport());
  } catch {
    // A runtime that refuses to collect a report leaves only the loader helper.
  } finally {
    // Restored on both paths, so a failed collection cannot leave the process
    // writing reports without network interfaces.
    report.excludeNetwork = excludeNetwork;
  }
}

/** @returns {string} Loader helper contents, or an empty string when unreadable. */
function readLoaderHelper() {
  try {
    // The loader helper is a binary on musl systems, where it is the loader
    // itself, so its bytes are read without UTF-8 decoding.
    return readFileSync("/usr/bin/ldd", "latin1");
  } catch {
    return "";
  }
}
