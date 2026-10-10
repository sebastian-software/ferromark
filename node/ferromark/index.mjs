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
  "cjkEmphasis",
  "mdx",
  "imageAttributes",
  "imageCaptions",
  "extendedAttributes",
  "bracketedSpans",
  "blockquoteAttributions",
  "insertions",
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
  "typography",
  "passes",
  "guillemetDigraphs",
  "autoAbbreviations",
  "abbreviations",
  "preset",
]);

/** @type {Array<[string, number]>} */
const packedBooleanFlags = [
  ["allowHtml", 1 << 1],
  ["tables", 1 << 2],
  ["mergedTableCells", 1 << 3],
  ["tableColgroup", 1 << 4],
  ["tableColumnNames", 1 << 5],
  ["tableAttributes", 1 << 6],
  ["strikethrough", 1 << 7],
  ["superscript", 1 << 8],
  ["subscript", 1 << 9],
  ["taskLists", 1 << 10],
  ["autolinkLiterals", 1 << 11],
  ["disallowedRawHtml", 1 << 12],
  ["footnotes", 1 << 13],
  ["highlight", 1 << 14],
  ["inlineFootnotes", 1 << 15],
  ["allowLinkRefs", 1 << 16],
  ["frontMatter", 1 << 17],
  ["headingIds", 1 << 18],
  ["headingAttributes", 1 << 19],
  ["math", 1 << 20],
  ["callouts", 1 << 21],
  ["definitionLists", 1 << 22],
  ["lineComments", 1 << 23],
  ["cjkEmphasis", 1 << 25],
  ["mdx", 1 << 26],
  ["imageAttributes", 1 << 27],
  ["imageCaptions", 1 << 28],
  ["extendedAttributes", 1 << 29],
  ["bracketedSpans", 1 << 30],
];

/** Read values the object entry receives unchanged when they are present. */
const objectPathValues = /** @type {const} */ ([
  "headingOffset",
  "headingIdPrefix",
  "linkBasePath",
  "typography",
  "passes",
  "abbreviations",
  "preset",
]);

/** @param {{ set: number, on: number }} packed Parsed flags. @param {object} target Rebuilt object. */
function copyPackedBooleanFlags(packed, target) {
  for (const [key, bit] of packedBooleanFlags) {
    if (packed.set & bit) Reflect.set(target, key, Boolean(packed.on & bit));
  }
}

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
 * For an `Options` argument, napi-rs gets each of the 41 fields once, in their
 * declaration order in `node/native/src/lib.rs`, with an ordinary property
 * get: inherited properties, getters and proxy traps all take part. It
 * converts each value before it gets the next field. `undefined` leaves a
 * field unset, and any other value must be a primitive of the field's type,
 * or the call throws. The reader below makes the same gets in the same order
 * and stops where napi-rs stops, so a getter or proxy sees the same accesses.
 *
 * `renderPolicy` (bit 0) and 29 boolean fields (bits 1 to 30 without the
 * retired bit 24, in declaration order, as `unpack` numbers them) take one bit each of `set` (present) and
 * `on` (its value; `'trusted'` for `renderPolicy`). Enabled blockquote
 * attributions, insertions, guillemet digraphs, abbreviation settings, and a
 * `preset` use the object entry so later option bits remain available.
 * `headingOffset`,
 * `headingIdPrefix`, `linkBasePath`, `typography` and `passes` keep their
 * values. The native side rebuilds `Options` and resolves it with the code the
 * object path runs, so later checks and their errors are shared.
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
  /** @type {string | undefined} */
  preset;
  /** @type {import('./index.mjs').TypographyOptions | null | undefined} */
  typography;
  /** @type {import('./index.mjs').NativePassOptions[] | null | undefined} */
  passes;
  /** @type {Record<string, string | null> | undefined} */
  abbreviations;
  /** @type {import('./index.mjs').Options | undefined} */
  rejected;
  /** @type {string | undefined} */
  unknownPolicy;
  /** @type {Array<[string, boolean]>} Set boolean fields that have no packed bit. */
  objectPathFlags = [];

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
      this.flag("cjkEmphasis", 1 << 25, options.cjkEmphasis) &&
      this.flag("mdx", 1 << 26, options.mdx) &&
      this.flag("imageAttributes", 1 << 27, options.imageAttributes) &&
      this.flag("imageCaptions", 1 << 28, options.imageCaptions) &&
      this.flag("extendedAttributes", 1 << 29, options.extendedAttributes) &&
      this.flag("bracketedSpans", 1 << 30, options.bracketedSpans) &&
      this.string("linkBasePath", options.linkBasePath) &&
      this.object("typography", options.typography) &&
      this.array("passes", options.passes) &&
      this.objectFlag("blockquoteAttributions", options.blockquoteAttributions) &&
      this.objectFlag("insertions", options.insertions) &&
      this.objectFlag("guillemetDigraphs", options.guillemetDigraphs) &&
      this.objectFlag("autoAbbreviations", options.autoAbbreviations) &&
      this.record("abbreviations", options.abbreviations) &&
      this.string("preset", options.preset)
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

  /**
   * Reads a boolean field that has no packed bit. Only `true` needs the object
   * entry; absent or `false` keeps the packed path and its bit allocation. A
   * set `false` still travels on the object path, where it can override a
   * `preset`.
   * @param {string} key Field name. @param {unknown} value Field value.
   */
  objectFlag(key, value) {
    if (typeof value !== "boolean") {
      return value === undefined || this.reject(key, value);
    }
    this.objectPathFlags.push([key, value]);
    return true;
  }

  /** Whether an enabled field requires the object-taking native entry. */
  get requiresObjectPath() {
    return (
      this.objectPathFlags.some(([, value]) => value) ||
      this.abbreviations !== undefined ||
      this.preset !== undefined
    );
  }

  /** Rebuilds values already read once, without repeating caller getters. */
  toObjectOptions() {
    const result = Object.create(null);
    if (this.set & 1) {
      result.renderPolicy = this.on & 1 ? "trusted" : "untrusted";
    }
    copyPackedBooleanFlags(this, result);
    for (const key of objectPathValues) {
      if (this[key] !== undefined) result[key] = this[key];
    }
    for (const [key, value] of this.objectPathFlags) result[key] = value;
    return result;
  }

  /** @param {"headingOffset"} key Field name. @param {unknown} value Field value. */
  number(key, value) {
    if (value !== undefined && typeof value !== "number") {
      return this.reject(key, value);
    }
    this[key] = value;
    return true;
  }

  /** @param {"headingIdPrefix" | "linkBasePath" | "preset"} key Field name. @param {unknown} value Field value. */
  string(key, value) {
    if (value !== undefined && typeof value !== "string") {
      return this.reject(key, value);
    }
    this[key] = value;
    return true;
  }

  /** @param {string} key Field name. @param {unknown} value The optional object value. */
  object(key, value) {
    if (value === null) {
      return this.reject(key, value);
    }
    this.typography = /** @type {import('./index.mjs').TypographyOptions | undefined} */ (value);
    return true;
  }

  /** @param {"passes"} key Field name. @param {unknown} value The optional array value. */
  array(key, value) {
    if (value !== undefined && !Array.isArray(value)) {
      return this.reject(key, value);
    }
    this[key] = value;
    return true;
  }

  /** @param {"abbreviations"} key Field name. @param {unknown} value The optional map value. */
  record(key, value) {
    if (value === undefined) {
      return true;
    }
    if (value === null) {
      return this.reject(key, value);
    }
    // napi-rs converts `Option<HashMap<...>>` by enumerating the supplied
    // value; several primitives therefore act as an empty map. Capture those
    // entries once so invalid values can take the object path and retain
    // napi-rs's `Options.abbreviations` error context without re-reading a
    // caller's getters.
    const entries = Object.entries(value);
    const abbreviations = Object.fromEntries(entries.filter(([, title]) => title !== undefined));
    if (
      entries.some(
        ([, title]) => title !== undefined && title !== null && typeof title !== "string",
      )
    ) {
      return this.reject(key, Object.fromEntries(entries));
    }
    this.abbreviations = /** @type {Record<string, string | null>} */ (abbreviations);
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
 *   argument for the object-taking entry instead: absent options, abbreviation
 *   options that need their object fields, or a rejected value. Validated
 *   options are never an array, so `Array.isArray` tells the two apart.
 */
function packOptions(options) {
  if (options == null) {
    return options;
  }
  const packed = new PackedOptions(options);
  return (
    packed.rejected ??
    (packed.requiresObjectPath
      ? packed.toObjectOptions()
      : [
          packed.set,
          packed.on,
          packed.headingOffset,
          packed.headingIdPrefix,
          packed.linkBasePath,
          packed.typography,
          packed.passes,
        ])
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

const jsxOnlyKeys = new Set([
  "format",
  "componentPrefix",
  "calloutComponents",
  "codeComponents",
  "codeBlockComponent",
  "omitTitleHeading",
  "output",
  "providerImportSource",
  "filename",
  "defaultExport",
  "reservedBindings",
]);
const jsxCodeBlockInputKeys = new Set(["code", "language", "meta"]);
/** Options that only shape a module. With body output they are an error, not ignored. */
const moduleOnlyKeys = ["providerImportSource", "filename", "defaultExport", "reservedBindings"];
const jsxPreparationKeys = new Set([
  "format",
  "tables",
  "mergedTableCells",
  "tableAttributes",
  "strikethrough",
  "superscript",
  "subscript",
  "taskLists",
  "autolinkLiterals",
  "footnotes",
  "highlight",
  "inlineFootnotes",
  "allowLinkRefs",
  "frontMatter",
  "headingAttributes",
  "math",
  "definitionLists",
  "lineComments",
  "cjkEmphasis",
  "imageAttributes",
  "imageCaptions",
  "extendedAttributes",
  "bracketedSpans",
  "blockquoteAttributions",
  "insertions",
  "guillemetDigraphs",
  "typography",
  "passes",
]);
const jsxRenderKeys = new Set([
  "componentPrefix",
  "calloutComponents",
  "codeComponents",
  "codeBlockComponent",
  "omitTitleHeading",
  "headingIds",
  "headingOffset",
  "headingIdPrefix",
  "callouts",
]);
const htmlOnlyKeys = new Set([
  "mdx",
  "renderPolicy",
  "allowHtml",
  "disallowedRawHtml",
  "tableColgroup",
  "tableColumnNames",
  "linkBasePath",
  "autoAbbreviations",
  "abbreviations",
  "preset",
]);

/**
 * Validate the syntax and JSX option names before loading the addon.
 * @param {import('./index.mjs').CompileJsxOptions | import('./index.mjs').CompileJsxModuleOptions} [options] Syntax and JSX options.
 * @returns {boolean} Whether the options select module output.
 */
function validateJsxOptions(options) {
  if (options == null) {
    return false;
  }
  if (typeof options !== "object" && typeof options !== "function") {
    throw new TypeError("options must be an object");
  }
  for (const key of Reflect.ownKeys(options)) {
    if (
      typeof key !== "string" ||
      htmlOnlyKeys.has(key) ||
      (!optionKeys.has(key) && !jsxOnlyKeys.has(key))
    ) {
      throw new TypeError(`unknown JSX option "${String(key)}"`);
    }
  }
  return selectsModuleOutput(options);
}

/**
 * Validate standalone code-block input before calling the native compiler.
 * @param {import('./index.mjs').JsxCodeBlockRenderInput} input Code, language, and optional fence metadata.
 * @returns {import('./index.mjs').JsxCodeBlockRenderInput} Validated input.
 */
function validateJsxCodeBlockInput(input) {
  if (input == null || typeof input !== "object" || Array.isArray(input)) {
    throw new TypeError("code block input must be an object");
  }
  assertJsxCodeBlockInputKeys(input);
  if (typeof input.code !== "string") {
    throw new TypeError("code block input.code must be a string");
  }
  assertOptionalCodeBlockString("language", input.language);
  assertOptionalCodeBlockString("meta", input.meta);
  return {
    code: input.code,
    ...(input.language === undefined ? {} : { language: input.language }),
    ...(input.meta === undefined ? {} : { meta: input.meta }),
  };
}

/**
 * @param {import('./index.mjs').JsxCodeBlockRenderInput} input Input to validate.
 */
function assertJsxCodeBlockInputKeys(input) {
  const unknown = Reflect.ownKeys(input).find(
    (key) => typeof key !== "string" || !jsxCodeBlockInputKeys.has(key),
  );
  if (unknown !== undefined) {
    throw new TypeError(`unknown code block input "${String(unknown)}"`);
  }
}

/** @param {"language" | "meta"} name Property name. @param {string | null | undefined} value Property value. */
function assertOptionalCodeBlockString(name, value) {
  if (value == null || typeof value === "string") {
    return;
  }
  throw new TypeError(`code block input.${name} must be a string or null`);
}

/**
 * Read the requested output and reject module options where they do not apply.
 * @param {import('./index.mjs').CompileJsxOptions | import('./index.mjs').CompileJsxModuleOptions} options Syntax and JSX options.
 * @returns {boolean} Whether the options select module output.
 */
function selectsModuleOutput(options) {
  const { output } = options;
  if (output !== undefined && output !== "body" && output !== "module") {
    throw new TypeError('output must be "body" or "module"');
  }
  if (output === "module") {
    return true;
  }
  for (const key of moduleOnlyKeys) {
    if (Reflect.get(options, key) !== undefined) {
      throw new TypeError(`JSX option "${key}" requires output: "module"`);
    }
  }
  return false;
}

/** @param {import('./index.mjs').JsxPreparationOptions | undefined} [options] Preparation settings. */
function validateJsxPreparationOptions(options) {
  if (options == null) {
    return;
  }
  if (typeof options !== "object" || Array.isArray(options)) {
    throw new TypeError("preparationOptions must be an object");
  }
  for (const key of Reflect.ownKeys(options)) {
    if (typeof key !== "string" || !jsxPreparationKeys.has(key)) {
      const category = preparationOptionCategory(key);
      throw new TypeError(`${category} "${String(key)}" cannot be used in preparationOptions`);
    }
  }
  if (options.format !== undefined && options.format !== "md" && options.format !== "mdx") {
    throw new TypeError('format must be "md" or "mdx"');
  }
}

/** @param {string | symbol} key An option key. @returns {string} Its rejected category. */
function preparationOptionCategory(key) {
  if (typeof key !== "string") {
    return "unknown preparation option";
  }
  if (jsxRenderKeys.has(key)) {
    return "render option";
  }
  if (moduleOnlyKeys.includes(key)) {
    return "module option";
  }
  return "unknown preparation option";
}

/** @param {string | symbol} key An option key. @param {boolean} module Whether module output is selected. */
function isPreparedRenderOption(key, module) {
  if (typeof key !== "string") {
    return false;
  }
  return jsxRenderKeys.has(key) || (module && moduleOnlyKeys.includes(key));
}

/** @param {import('./index.mjs').JsxRenderOptions | import('./index.mjs').JsxRenderModuleOptions | undefined} options Render settings. @param {boolean} module Whether this selects module output. */
function validatePreparedRenderOptions(options, module) {
  if (options == null) {
    return;
  }
  if (typeof options !== "object" || Array.isArray(options)) {
    throw new TypeError("renderOptions must be an object");
  }
  for (const key of Reflect.ownKeys(options)) {
    if (!isPreparedRenderOption(key, module)) {
      throw new TypeError(`unknown JSX render option "${String(key)}"`);
    }
  }
  if (
    options.componentPrefix !== undefined &&
    module &&
    options.componentPrefix !== "_components"
  ) {
    throw new TypeError('module output sets the component prefix to "_components"');
  }
}

/** @param {import('./index.mjs').JsxPreparedMetadata} metadata Native prepared metadata copy. */
function freezePreparedMetadata(metadata) {
  for (const key of ["esm", "codeBlocks", "outline"]) {
    const values = Reflect.get(metadata, key);
    if (Array.isArray(values)) {
      for (const item of values) {
        Object.freeze(item);
      }
      Object.freeze(values);
    }
  }
  if (metadata.frontMatterSpan != null) {
    Object.freeze(metadata.frontMatterSpan);
  }
  return Object.freeze(metadata);
}

/** @param {NativePreparedJsxDocument} nativePrepared Native prepared document. */
function preparedFacade(nativePrepared) {
  const metadata = freezePreparedMetadata(nativePrepared.metadata);
  return Object.freeze({
    metadata,
    /** @param {import('./index.mjs').JsxRenderOptions} [options] Render settings. @param {import('./index.mjs').JsxCodeRenderer} [renderCode] Trusted synchronous code override. */
    render(options, renderCode) {
      validatePreparedRenderOptions(options, false);
      if (renderCode !== undefined && typeof renderCode !== "function") {
        throw new TypeError("renderCode must be a synchronous function");
      }
      return nativePrepared.render(options, renderCode);
    },
    /** @param {import('./index.mjs').JsxRenderModuleOptions} [options] Module and render settings. @param {import('./index.mjs').JsxCodeRenderer} [renderCode] Trusted synchronous code override. */
    renderModule(options, renderCode) {
      validatePreparedRenderOptions(options, true);
      if (renderCode !== undefined && typeof renderCode !== "function") {
        throw new TypeError("renderCode must be a synchronous function");
      }
      return nativePrepared.renderModule(options, renderCode);
    },
  });
}

/**
 * Compile trusted authored Markdown or MDX to framework-neutral JSX.
 * @param {string | Uint8Array} markdown Authored source.
 * @param {import('./index.mjs').CompileJsxOptions | import('./index.mjs').CompileJsxModuleOptions} [options] Syntax and JSX options.
 * @param {import('./index.mjs').JsxCodeRenderer} [renderCode] Trusted synchronous JSX hook.
 * @returns {import('./index.mjs').JsxResult | import('./index.mjs').JsxModuleResult} A JSX body with its parts, or a module with `output: "module"`.
 */
export function compileJsx(markdown, options, renderCode) {
  const module = validateJsxOptions(options);
  if (renderCode !== undefined && typeof renderCode !== "function") {
    throw new TypeError("renderCode must be a synchronous function");
  }
  const addon = loadNative();
  return module
    ? addon.compileJsxModule(markdown, options, options, renderCode)
    : addon.compileJsx(markdown, options, options, renderCode);
}

/** Reusable compiler whose Ferriki highlighter lives in the native addon. */
export class JsxCompiler {
  /** @type {NativeJsxCompilerSession} */
  #native;

  /** @param {import('./index.mjs').JsxCompilerOptions} [options] Native highlighting and asset settings. */
  constructor(options = {}) {
    if (options == null || typeof options !== "object" || Array.isArray(options)) {
      throw new TypeError("JSX compiler options must be an object");
    }
    for (const key of Reflect.ownKeys(options)) {
      if (!["theme", "lineNumbers", "languages", "assets"].includes(String(key))) {
        throw new TypeError(`unknown JSX compiler option "${String(key)}"`);
      }
    }
    const NativeCompiler = loadNative().JsxCompiler;
    this.#native = new NativeCompiler(JSON.stringify(options));
  }

  /**
   * Parses source and runs the configured native passes once.
   * @param {string | Uint8Array} markdown Authored source.
   * @param {import('./index.mjs').JsxPreparationOptions} [preparationOptions] Parser and transform settings.
   * @returns {import('./index.mjs').PreparedJsxDocument} An immutable handle with metadata and repeatable render methods.
   */
  prepare(markdown, preparationOptions) {
    validateJsxPreparationOptions(preparationOptions);
    const format = preparationOptions?.format;
    const nativePrepared = this.#native.prepare(markdown, preparationOptions, { format });
    return preparedFacade(nativePrepared);
  }

  /**
   * @param {import('./index.mjs').JsxCodeBlockRenderInput} input Standalone code and optional fence language and metadata.
   * @returns {import('./index.mjs').JsxCodeBlockRenderResult} Intrinsic JSX and metadata parsed by the native fence parser.
   */
  renderCodeBlock(input) {
    return this.#native.renderCodeBlock(validateJsxCodeBlockInput(input));
  }

  /**
   * @param {string | Uint8Array} markdown Trusted authored source.
   * @param {import('./index.mjs').CompileJsxOptions | import('./index.mjs').CompileJsxModuleOptions} [options] Syntax and JSX options.
   * @param {import('./index.mjs').JsxCodeRenderer} [renderCode] Optional whole-fence override.
   * @returns {import('./index.mjs').JsxResult | import('./index.mjs').JsxModuleResult} Highlighted JSX and metadata, or a module with `output: "module"`.
   */
  compile(markdown, options, renderCode) {
    const module = validateJsxOptions(options);
    if (renderCode !== undefined && typeof renderCode !== "function") {
      throw new TypeError("renderCode must be a synchronous function");
    }
    return module
      ? this.#native.compileModule(markdown, options, options, renderCode)
      : this.#native.compile(markdown, options, options, renderCode);
  }
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
 * @typedef {import('./index.mjs').CompileJsxOptions | import('./index.mjs').CompileJsxModuleOptions} NativeJsxOptions
 * @typedef {{
 *   readonly metadata: import('./index.mjs').JsxPreparedMetadata
 *   render(options?: import('./index.mjs').JsxRenderOptions,
 *     renderCode?: import('./index.mjs').JsxCodeRenderer): import('./index.mjs').JsxResult
 *   renderModule(options?: import('./index.mjs').JsxRenderModuleOptions,
 *     renderCode?: import('./index.mjs').JsxCodeRenderer): import('./index.mjs').JsxModuleResult
 * }} NativePreparedJsxDocument
 * @typedef {{
 *   prepare(markdown: string | Uint8Array, options?: import('./index.mjs').JsxPreparationOptions,
 *     jsxOptions?: Pick<NativeJsxOptions, 'format'>): NativePreparedJsxDocument
 *   compile(markdown: string | Uint8Array, options?: NativeJsxOptions,
 *     jsxOptions?: NativeJsxOptions,
 *     renderCode?: import('./index.mjs').JsxCodeRenderer): import('./index.mjs').JsxResult
 *   compileModule(markdown: string | Uint8Array, options?: NativeJsxOptions,
 *     jsxOptions?: NativeJsxOptions,
 *     renderCode?: import('./index.mjs').JsxCodeRenderer): import('./index.mjs').JsxModuleResult
 *   renderCodeBlock(input: import('./index.mjs').JsxCodeBlockRenderInput): import('./index.mjs').JsxCodeBlockRenderResult
 * }} NativeJsxCompilerSession
 * @typedef {import('./index.mjs').Options | null | undefined} NativeOptions
 * @typedef {[
 *   set: number,
 *   on: number,
 *   headingOffset: number | undefined,
 *   headingIdPrefix: string | undefined,
 *   linkBasePath: string | undefined,
 *   typography: import('./index.mjs').TypographyOptions | null | undefined,
 *   passes: import('./index.mjs').NativePassOptions[] | null | undefined,
 * ]} NativePackedOptions
 * @typedef {[
 *   set: number,
 *   on: number,
 *   headingOffset: number | undefined,
 *   headingIdPrefix: string | undefined,
 *   linkBasePath: string | undefined,
 *   typography: import('./index.mjs').TypographyOptions | null | undefined,
 *   passes: import('./index.mjs').NativePassOptions[] | null | undefined,
 *   renderer: NativeFencedCodeRenderer,
 * ]} NativePackedRendererOptions
 * @typedef {{
 *   Renderer: {
 *     new (options?: NativeOptions): NativeRendererSession
 *     withPackedOptions(...options: NativePackedOptions): NativeRendererSession
 *   }
 *   JsxCompiler: { new (settings: string): NativeJsxCompilerSession }
 *   compileJsx(
 *     markdown: string | Uint8Array,
 *     options?: NativeJsxOptions,
 *     jsxOptions?: NativeJsxOptions,
 *     renderCode?: import('./index.mjs').JsxCodeRenderer,
 *   ): import('./index.mjs').JsxResult
 *   compileJsxModule(
 *     markdown: string | Uint8Array,
 *     options?: NativeJsxOptions,
 *     jsxOptions?: NativeJsxOptions,
 *     renderCode?: import('./index.mjs').JsxCodeRenderer,
 *   ): import('./index.mjs').JsxModuleResult
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
