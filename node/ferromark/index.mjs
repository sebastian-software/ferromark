import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
import { types } from "node:util";

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
  "headingOffset",
  "headingIdPrefix",
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

// The facade stays one module: the loader tests run a copy of it with only
// native-target.mjs beside it, so the options reader cannot move out.
/* eslint-disable max-lines */

/**
 * An options object, read the way napi-rs reads `Options` and packed into the
 * plain arguments of the private native entries (`node/native/src/packed.rs`).
 *
 * For an `Options` argument, napi-rs gets each of the 30 fields once, in their
 * declaration order in `node/native/src/lib.rs`, with an ordinary property
 * get: inherited properties, getters and proxy traps all take part. It
 * converts each value before it gets the next field. `undefined` leaves a
 * field unset, and any other value must be a primitive of the field's type,
 * or the call throws. The reader below makes the same gets in the same order
 * and stops where napi-rs stops, so a getter or proxy sees the same accesses.
 *
 * `renderPolicy` (bit 0) and the boolean fields (bits 1 to 26, in declaration
 * order, as `unpack` numbers them) take one bit each of `set` (present) and
 * `on` (its value; `'trusted'` for `renderPolicy`). `headingOffset`,
 * `headingIdPrefix` and `linkBasePath` keep their values. The native side
 * rebuilds `Options` and resolves it with the code the object path runs, so
 * later checks and their errors are shared.
 *
 * napi-rs builds each conversion error from the value itself. For a value it
 * would reject, `rejected` holds a null-prototype object with just that field,
 * and the caller passes it to the object-taking entry. That entry fails on the
 * same field with the same value, and so with the same error, because the
 * fields before it are absent from `rejected` and cannot fail.
 */
class PackedOptions {
  set = 0;
  on = 0;
  /** @type {number | undefined} */
  headingOffset;
  /** @type {string | undefined} */
  headingIdPrefix;
  /** @type {string | undefined} */
  linkBasePath;
  /** @type {import('./index.mjs').Options | undefined} */
  rejected;
  /** @type {string | undefined} */
  unknownPolicy;

  /** @param {import('./index.mjs').Options} options Validated options. */
  constructor(options) {
    if (this.read(options) && this.unknownPolicy !== undefined) {
      // napi-rs converts any string for `renderPolicy`, and only rejects an
      // unknown one after it has read every field.
      this.reject("renderPolicy", this.unknownPolicy);
    }
  }

  /**
   * One get per field, in declaration order; `&&` stops at the first value
   * napi-rs rejects, as napi-rs does.
   * @param {import('./index.mjs').Options} options Validated options.
   * @returns {boolean} Whether napi-rs converts every field.
   */
  // One short-circuit per field keeps napi-rs's order in plain sight, and each
  // get stays a named load V8 caches; a loop over names would make them keyed.
  // eslint-disable-next-line complexity
  read(options) {
    return (
      this.policy(options.renderPolicy) &&
      this.flag("allowHtml", 1 << 1, options.allowHtml) &&
      this.flag("tables", 1 << 2, options.tables) &&
      this.flag("mergedTableCells", 1 << 3, options.mergedTableCells) &&
      this.flag("tableColgroup", 1 << 4, options.tableColgroup) &&
      this.flag("tableColumnNames", 1 << 5, options.tableColumnNames) &&
      this.flag("tableAttributes", 1 << 6, options.tableAttributes) &&
      this.flag("strikethrough", 1 << 7, options.strikethrough) &&
      this.flag("superscript", 1 << 8, options.superscript) &&
      this.flag("subscript", 1 << 9, options.subscript) &&
      this.flag("taskLists", 1 << 10, options.taskLists) &&
      this.flag("autolinkLiterals", 1 << 11, options.autolinkLiterals) &&
      this.flag("disallowedRawHtml", 1 << 12, options.disallowedRawHtml) &&
      this.flag("footnotes", 1 << 13, options.footnotes) &&
      this.flag("highlight", 1 << 14, options.highlight) &&
      this.flag("inlineFootnotes", 1 << 15, options.inlineFootnotes) &&
      this.flag("allowLinkRefs", 1 << 16, options.allowLinkRefs) &&
      this.flag("frontMatter", 1 << 17, options.frontMatter) &&
      this.flag("headingIds", 1 << 18, options.headingIds) &&
      this.number("headingOffset", options.headingOffset) &&
      this.string("headingIdPrefix", options.headingIdPrefix) &&
      this.flag("headingAttributes", 1 << 19, options.headingAttributes) &&
      this.flag("math", 1 << 20, options.math) &&
      this.flag("callouts", 1 << 21, options.callouts) &&
      this.flag("definitionLists", 1 << 22, options.definitionLists) &&
      this.flag("lineComments", 1 << 23, options.lineComments) &&
      this.flag("wikiLinks", 1 << 24, options.wikiLinks) &&
      this.flag("cjkEmphasis", 1 << 25, options.cjkEmphasis) &&
      this.flag("mdx", 1 << 26, options.mdx) &&
      this.string("linkBasePath", options.linkBasePath)
    );
  }

  /** @param {unknown} value The `renderPolicy` value. */
  policy(value) {
    if (value === undefined) {
      return true;
    }
    if (typeof value !== "string") {
      return this.reject("renderPolicy", value);
    }
    if (value === "trusted" || value === "untrusted") {
      this.set |= 1;
      this.on |= value === "trusted" ? 1 : 0;
    } else {
      this.unknownPolicy = value;
    }
    return true;
  }

  /** @param {string} key Field name. @param {number} bit Its bit mask. @param {unknown} value Field value. */
  flag(key, bit, value) {
    if (typeof value !== "boolean") {
      return value === undefined || this.reject(key, value);
    }
    this.set |= bit;
    if (value) {
      this.on |= bit;
    }
    return true;
  }

  /** @param {"headingOffset"} key Field name. @param {unknown} value Field value. */
  number(key, value) {
    if (value !== undefined && typeof value !== "number") {
      return this.reject(key, value);
    }
    this[key] = value;
    return true;
  }

  /** @param {"headingIdPrefix" | "linkBasePath"} key Field name. @param {unknown} value Field value. */
  string(key, value) {
    if (value !== undefined && typeof value !== "string") {
      return this.reject(key, value);
    }
    this[key] = value;
    return true;
  }

  /** @param {string} key Field name. @param {unknown} value Rejected value. @returns {false} Always. */
  reject(key, value) {
    const rejected = Object.create(null);
    rejected[key] = value;
    this.rejected = rejected;
    return false;
  }
}

/**
 * Whether the addon accepts `markdown`: a string, or a `Uint8Array` (which
 * includes a `Buffer`) holding UTF-8.
 *
 * `types.isUint8Array` is a brand check, like the addon's own: it holds for
 * subclasses and arrays from other realms, and no getter, proxy trap or
 * patched prototype can change its answer.
 *
 * @param {unknown} markdown The Markdown argument.
 * @returns {markdown is string | Uint8Array} Whether the addon converts it.
 */
function isMarkdown(markdown) {
  return typeof markdown === "string" || types.isUint8Array(markdown);
}

/**
 * Packs validated options for the private native entries.
 *
 * An entry whose Markdown the addon rejects (see `isMarkdown`) skips this and
 * passes `options` to the object-taking native entry: the addon rejects the
 * Markdown before napi-rs gets any option, and so gets none.
 *
 * @param {import('./index.mjs').Options | null | undefined} options Validated options.
 * @returns {NativePackedOptions | NativeOptions} The packed arguments, or the
 *   argument for the object-taking entry instead: absent options, which cost
 *   napi-rs nothing to read, or a rejected value. Validated options are never
 *   an array, so `Array.isArray` tells the two apart.
 */
function packOptions(options) {
  if (options == null) {
    return options;
  }
  const packed = new PackedOptions(options);
  return (
    packed.rejected ?? [
      packed.set,
      packed.on,
      packed.headingOffset,
      packed.headingIdPrefix,
      packed.linkBasePath,
    ]
  );
}

/**
 * @param {string | Uint8Array} markdown Markdown source to render.
 * @param {import('./index.mjs').Options} [options] Rendering options.
 */
export function toHtml(markdown, options) {
  validateOptions(options);
  const addon = loadNative();
  const packed = isMarkdown(markdown) ? packOptions(options) : options;
  return Array.isArray(packed)
    ? addon.toHtmlPacked(markdown, ...packed)
    : addon.toHtml(markdown, packed);
}

/**
 * Render UTF-8 HTML into a Node.js Buffer.
 * @param {string | Uint8Array} markdown Markdown source to render.
 * @param {import('./index.mjs').Options} [options] Rendering options.
 * @returns {import('node:buffer').Buffer} Rendered HTML as a UTF-8 buffer.
 */
export function toHtmlBuffer(markdown, options) {
  validateOptions(options);
  const addon = loadNative();
  const packed = isMarkdown(markdown) ? packOptions(options) : options;
  return Array.isArray(packed)
    ? addon.toHtmlBufferPacked(markdown, ...packed)
    : addon.toHtmlBuffer(markdown, packed);
}

/** Reusable Markdown renderer with fixed options. */
export class Renderer {
  /** @type {NativeRendererSession} */
  #native;

  /** @param {import('./index.mjs').Options} [options] Rendering options. */
  constructor(options) {
    validateOptions(options);
    const NativeRenderer = loadNative().Renderer;
    const packed = packOptions(options);
    this.#native = Array.isArray(packed)
      ? NativeRenderer.withPackedOptions(...packed)
      : new NativeRenderer(packed);
  }

  /** @param {string | Uint8Array} markdown Markdown source to render. */
  toHtml(markdown) {
    return this.#native.toHtml(markdown);
  }

  /** @param {string | Uint8Array} markdown Markdown source to render. @returns {import('node:buffer').Buffer} Rendered HTML. */
  toHtmlBuffer(markdown) {
    return this.#native.toHtmlBuffer(markdown);
  }
}

/**
 * @param {string | Uint8Array} markdown Markdown source to transform.
 * @param {import('./index.mjs').Options} [options] Transformation options.
 */
export function transform(markdown, options) {
  validateOptions(options);
  const addon = loadNative();
  const packed = isMarkdown(markdown) ? packOptions(options) : options;
  return Array.isArray(packed)
    ? addon.transformPacked(markdown, ...packed)
    : addon.transform(markdown, packed);
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
 * @param {string | Uint8Array} markdown Markdown source to render.
 * @param {import('./index.mjs').CodeHighlighter} highlighter Synchronous code highlighter.
 * @param {import('./index.mjs').HighlightOptions} highlightOptions Highlighter options.
 * @param {import('./index.mjs').Options} [options] Rendering options.
 */
// The four arguments are the stable public API shape.
// eslint-disable-next-line max-params
export function toHtmlWithHighlighter(markdown, highlighter, highlightOptions, options) {
  validateOptions(options);
  const render = highlighterRenderer(highlighter, highlightOptions);
  const addon = loadNative();
  const packed = isMarkdown(markdown) ? packOptions(options) : options;
  return Array.isArray(packed)
    ? addon.toHtmlWithRendererPacked(markdown, ...packed, render)
    : addon.toHtmlWithRenderer(markdown, packed, render);
}

/**
 * @param {string | Uint8Array} markdown Markdown source to transform.
 * @param {import('./index.mjs').CodeHighlighter} highlighter Synchronous code highlighter.
 * @param {import('./index.mjs').HighlightOptions} highlightOptions Highlighter options.
 * @param {import('./index.mjs').Options} [options] Transformation options.
 */
// The four arguments are the stable public API shape.
// eslint-disable-next-line max-params
export function transformWithHighlighter(markdown, highlighter, highlightOptions, options) {
  validateOptions(options);
  const render = highlighterRenderer(highlighter, highlightOptions);
  const addon = loadNative();
  const packed = isMarkdown(markdown) ? packOptions(options) : options;
  return Array.isArray(packed)
    ? addon.transformWithRendererPacked(markdown, ...packed, render)
    : addon.transformWithRenderer(markdown, packed, render);
}

/**
 * @typedef {(code: string, language?: string | null, meta?: string | null) => string | null} NativeFencedCodeRenderer
 * @typedef {{
 *   toHtml(markdown: string | Uint8Array): string
 *   toHtmlBuffer(markdown: string | Uint8Array): import('node:buffer').Buffer
 * }} NativeRendererSession
 * @typedef {import('./index.mjs').Options | null | undefined} NativeOptions
 * @typedef {[
 *   set: number,
 *   on: number,
 *   headingOffset: number | undefined,
 *   headingIdPrefix: string | undefined,
 *   linkBasePath: string | undefined,
 * ]} NativePackedOptions
 * @typedef {[
 *   set: number,
 *   on: number,
 *   headingOffset: number | undefined,
 *   headingIdPrefix: string | undefined,
 *   linkBasePath: string | undefined,
 *   renderer: NativeFencedCodeRenderer,
 * ]} NativePackedRendererOptions
 * @typedef {{
 *   Renderer: {
 *     new (options?: NativeOptions): NativeRendererSession
 *     withPackedOptions(...options: NativePackedOptions): NativeRendererSession
 *   }
 *   toHtml(markdown: string | Uint8Array, options?: NativeOptions): string
 *   toHtmlPacked(markdown: string | Uint8Array, ...options: NativePackedOptions): string
 *   toHtmlBuffer(markdown: string | Uint8Array, options?: NativeOptions): import('node:buffer').Buffer
 *   toHtmlBufferPacked(
 *     markdown: string | Uint8Array,
 *     ...options: NativePackedOptions
 *   ): import('node:buffer').Buffer
 *   toHtmlWithRenderer(
 *     markdown: string | Uint8Array,
 *     options: NativeOptions,
 *     renderer: NativeFencedCodeRenderer,
 *   ): string
 *   toHtmlWithRendererPacked(
 *     markdown: string | Uint8Array,
 *     ...options: NativePackedRendererOptions
 *   ): string
 *   transform(
 *     markdown: string | Uint8Array,
 *     options?: NativeOptions,
 *   ): import('./index.mjs').TransformResult
 *   transformPacked(
 *     markdown: string | Uint8Array,
 *     ...options: NativePackedOptions
 *   ): import('./index.mjs').TransformResult
 *   transformWithRenderer(
 *     markdown: string | Uint8Array,
 *     options: NativeOptions,
 *     renderer: NativeFencedCodeRenderer,
 *   ): import('./index.mjs').TransformResult
 *   transformWithRendererPacked(
 *     markdown: string | Uint8Array,
 *     ...options: NativePackedRendererOptions
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
