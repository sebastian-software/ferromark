import { types } from "node:util";

import { createNativeLoader } from "./native-target.mjs";

/** @type {() => NativeBindings} */
const nativeLoader = /** @type {() => NativeBindings} */ (
  createNativeLoader(import.meta.url, {
    name: "ferromark",
    packagePattern: "ferromark-{target}",
    binaryPattern: "ferromark.{target}.node",
  })
);

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
  "trackReferences",
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
  if ("trackReferences" in options) {
    throw new TypeError('unknown JSX option "trackReferences"');
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

/** @param {import('./index.mjs').JsxPreparationOptions | undefined} options Preparation choices. */
function nativePreparationOptions(options) {
  const format = options?.format;
  const trackReferences = options?.trackReferences;
  if (trackReferences !== undefined && typeof trackReferences !== "boolean") {
    throw new TypeError("trackReferences must be a boolean");
  }
  return { format, trackReferences };
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
  if ("trackReferences" in options) {
    throw new TypeError('unknown JSX render option "trackReferences"');
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

/** @param {import('./index.mjs').TextExtractionOptions | undefined} options Extraction settings. */
function validateTextExtractionOptions(options) {
  if (options == null) return;
  if (typeof options !== "object" || Array.isArray(options)) {
    throw new TypeError("text extraction options must be an object");
  }
  const keys = new Set([
    "includeBlockCode",
    "headingIds",
    "headingOffset",
    "headingIdPrefix",
    "omitTitleHeading",
    "callouts",
  ]);
  for (const key of Reflect.ownKeys(options)) {
    if (typeof key !== "string" || !keys.has(key)) {
      throw new TypeError(`unknown text extraction option "${String(key)}"`);
    }
    validateTextExtractionValue(key, Reflect.get(options, key));
  }
}

/** @param {string} key Extraction option name. @param {unknown} value Option value. */
function validateTextExtractionValue(key, value) {
  if (value === undefined) return;
  if (key === "headingOffset") {
    validateTextHeadingOffset(value);
    return;
  }
  const wantsString = key === "headingIdPrefix" || key === "omitTitleHeading";
  const valid = wantsString ? typeof value === "string" : typeof value === "boolean";
  if (!valid) {
    throw new TypeError(`${key} must be a ${wantsString ? "string" : "boolean"}`);
  }
}

/** @param {unknown} value Extraction heading level offset. */
function validateTextHeadingOffset(value) {
  if (
    typeof value !== "number" ||
    !Number.isInteger(value) ||
    value < -2_147_483_648 ||
    value > 2_147_483_647
  ) {
    throw new TypeError("headingOffset must be a signed 32-bit integer");
  }
}

/** @param {import('./index.mjs').TextExtraction} result Owned native extraction. */
function freezeTextExtraction(result) {
  for (const section of result.sections) {
    if (section.headingSpan != null) Object.freeze(section.headingSpan);
    if (section.sourceSpan != null) Object.freeze(section.sourceSpan);
    Object.freeze(section.textRange);
    Object.freeze(section);
  }
  Object.freeze(result.sections);
  if (result.omittedTitleHeadingSpan != null) Object.freeze(result.omittedTitleHeadingSpan);
  return Object.freeze(result);
}

class PreparedDocumentFacade {
  /** @type {NativePreparedJsxDocument} */
  #nativePrepared;

  /** @param {NativePreparedJsxDocument} nativePrepared Native prepared document. */
  constructor(nativePrepared) {
    this.#nativePrepared = nativePrepared;
    this.metadata = freezePreparedMetadata(nativePrepared.metadata);
    this.render = this.#render.bind(this);
    this.renderModule = this.#renderModule.bind(this);
    Object.freeze(this);
  }

  /** @returns {import('./index.mjs').CodePluginDiagnostic[]} Failed-fence diagnostics from the shared compiler. */
  takeDiagnostics() {
    return this.#nativePrepared.takeDiagnostics();
  }

  collectReferences() {
    return Object.freeze(
      this.#nativePrepared.collectReferences().map((reference) => {
        if (reference.span != null) Object.freeze(reference.span);
        if (reference.originalSpan != null) Object.freeze(reference.originalSpan);
        return Object.freeze(reference);
      }),
    );
  }

  /** @param {import('./index.mjs').TextExtractionOptions} [options] Static extraction and heading choices. */
  extractText(options) {
    validateTextExtractionOptions(options);
    return freezeTextExtraction(this.#nativePrepared.extractText(options));
  }

  /** @param {import('./index.mjs').MarkdownProjectionOptions} [options] Source projection choices. */
  projectMarkdown(options) {
    validateMarkdownProjectionOptions(options);
    const result = this.#nativePrepared.projectMarkdown(options);
    for (const diagnostic of result.diagnostics) {
      Object.freeze(diagnostic);
    }
    Object.freeze(result.diagnostics);
    if (result.omittedTitleHeadingSpan != null) {
      Object.freeze(result.omittedTitleHeadingSpan);
    }
    return Object.freeze(result);
  }

  /** @param {import('./index.mjs').JsxRenderOptions} [options] Render settings. @param {import('./index.mjs').CodeRendering} [renderCode] Trusted synchronous code hooks. */
  #render(options, renderCode) {
    validatePreparedRenderOptions(options, false);
    return this.#nativePrepared.render(options, codeDispatcher(renderCode));
  }

  /** @param {import('./index.mjs').JsxRenderModuleOptions} [options] Module and render settings. @param {import('./index.mjs').CodeRendering} [renderCode] Trusted synchronous code hooks. */
  #renderModule(options, renderCode) {
    validatePreparedRenderOptions(options, true);
    return this.#nativePrepared.renderModule(options, codeDispatcher(renderCode));
  }
}

Object.freeze(PreparedDocumentFacade.prototype);

/** @param {NativePreparedJsxDocument} nativePrepared Native prepared document. */
function preparedFacade(nativePrepared) {
  return new PreparedDocumentFacade(nativePrepared);
}

/** @param {import('./index.mjs').MarkdownProjectionOptions | undefined} options Projection choices. */
function validateMarkdownProjectionOptions(options) {
  if (options === undefined) {
    return;
  }
  if (options === null || typeof options !== "object" || Array.isArray(options)) {
    throw new TypeError("projectionOptions must be an object");
  }
  for (const key of Reflect.ownKeys(options)) {
    if (key !== "frontMatter" && key !== "omitTitleHeading") {
      throw new TypeError(`unknown Markdown projection option "${String(key)}"`);
    }
  }
  validateMarkdownProjectionValues(options);
}

/** @param {import('./index.mjs').MarkdownProjectionOptions} options Projection choices. */
function validateMarkdownProjectionValues(options) {
  if (
    options.frontMatter !== undefined &&
    options.frontMatter !== "preserve" &&
    options.frontMatter !== "omit"
  ) {
    throw new TypeError('frontMatter must be "preserve" or "omit"');
  }
  if (options.omitTitleHeading !== undefined && typeof options.omitTitleHeading !== "string") {
    throw new TypeError("omitTitleHeading must be a string");
  }
}

/**
 * Prepare a document without loading highlighting themes or grammars.
 * @param {string | Uint8Array} markdown Authored Markdown or MDX source.
 * @param {import('./index.mjs').JsxPreparationOptions} [preparationOptions] Parser and native pass settings.
 * @returns {import('./index.mjs').PreparedJsxDocument} An immutable handle for extraction and rendering.
 */
export function prepareDocument(markdown, preparationOptions) {
  validateJsxPreparationOptions(preparationOptions);
  const preparation = nativePreparationOptions(preparationOptions);
  return preparedFacade(loadNative().prepareDocument(markdown, preparationOptions, preparation));
}

/**
 * Compile trusted authored Markdown or MDX to framework-neutral JSX.
 * @param {string | Uint8Array} markdown Authored source.
 * @param {import('./index.mjs').CompileJsxOptions | import('./index.mjs').CompileJsxModuleOptions} [options] Syntax and JSX options.
 * @param {import('./index.mjs').CodeRendering} [renderCode] Trusted synchronous code hooks.
 * @returns {import('./index.mjs').JsxResult | import('./index.mjs').JsxModuleResult} A JSX body with its parts, or a module with `output: "module"`.
 */
export function compileJsx(markdown, options, renderCode) {
  const module = validateJsxOptions(options);
  const dispatcher = codeDispatcher(renderCode);
  const addon = loadNative();
  return module
    ? addon.compileJsxModule(markdown, options, options, dispatcher)
    : addon.compileJsx(markdown, options, options, dispatcher);
}

/** Names of code plugin factories compiled into the loaded addon. */
export function availableCodePlugins() {
  return loadNative().availableCodePlugins();
}

/** Reusable HTML and JSX compiler with explicitly selected Rust plugins. */
export class CodeCompiler {
  /** @type {NativeCodeCompilerSession} */
  #native;

  /** @param {import('./index.mjs').CodeCompilerOptions} [options] Native plugin configuration. */
  constructor(options = {}) {
    if (options == null || typeof options !== "object" || Array.isArray(options)) {
      throw new TypeError("code compiler options must be an object");
    }
    this.#native = new (loadNative().CodeCompiler)(JSON.stringify(options));
  }

  /** @returns {import('./index.mjs').CodePluginDiagnostic[]} Drains diagnostics from this compiler and its prepared documents. */
  takeDiagnostics() {
    return this.#native.takeDiagnostics();
  }

  /**
   * Parses source and runs the configured native passes once.
   * @param {string | Uint8Array} markdown Authored source.
   * @param {import('./index.mjs').JsxPreparationOptions} [preparationOptions] Parser and transform settings.
   * @returns {import('./index.mjs').PreparedJsxDocument} An immutable handle with metadata and repeatable render methods.
   */
  prepare(markdown, preparationOptions) {
    validateJsxPreparationOptions(preparationOptions);
    const preparation = nativePreparationOptions(preparationOptions);
    const nativePrepared = this.#native.prepare(markdown, preparationOptions, preparation);
    return preparedFacade(nativePrepared);
  }

  /**
   * @param {import('./index.mjs').JsxCodeBlockRenderInput} input Standalone code and optional fence language and metadata.
   * @param {import('./index.mjs').CodeRendering} [hooks] Code hooks; only the highlighting stage applies.
   * @returns {import('./index.mjs').JsxCodeBlockRenderResult} Intrinsic JSX and metadata parsed by the native fence parser.
   */
  renderCodeBlock(input, hooks) {
    return this.#native.renderCodeBlock(validateJsxCodeBlockInput(input), codeDispatcher(hooks));
  }

  /**
   * @param {string | Uint8Array} markdown Trusted authored source.
   * @param {import('./index.mjs').CompileJsxOptions | import('./index.mjs').CompileJsxModuleOptions} [options] Syntax and JSX options.
   * @param {import('./index.mjs').CodeRendering} [renderCode] Optional code hooks.
   * @returns {import('./index.mjs').JsxResult | import('./index.mjs').JsxModuleResult} Highlighted JSX and metadata, or a module with `output: "module"`.
   */
  compile(markdown, options, renderCode) {
    const module = validateJsxOptions(options);
    const dispatcher = codeDispatcher(renderCode);
    return module
      ? this.#native.compileModule(markdown, options, options, dispatcher)
      : this.#native.compile(markdown, options, options, dispatcher);
  }

  /**
   * @param {string | Uint8Array} markdown Authored Markdown source.
   * @param {import('./index.mjs').Options} [options] Parser and rendering settings.
   */
  toHtml(markdown, options) {
    validateOptions(options);
    return this.#native.toHtml(markdown, options);
  }

  /**
   * @param {string | Uint8Array} markdown Authored Markdown source.
   * @param {import('./index.mjs').Options} [options] Parser and rendering settings.
   */
  transform(markdown, options) {
    validateOptions(options);
    return this.#native.transform(markdown, options);
  }
}

/**
 * @param {import('./index.mjs').CodeHighlighter} highlighter Synchronous code highlighter.
 * @param {import('./index.mjs').HighlightOptions} highlightOptions Highlighter options.
 */
export function createHighlighterCodeRenderer(highlighter, highlightOptions) {
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
      const html = highlighter.codeToHtml(code, {
        lang,
        theme: highlightOptions.theme,
        ...(meta ? { meta: { __raw: meta } } : {}),
      });
      if (typeof html !== "string") {
        throw new TypeError("highlighter.codeToHtml must return a string synchronously");
      }
      return html;
    } catch (error) {
      if (error instanceof TypeError && error.message.startsWith("highlighter.codeToHtml")) {
        throw error;
      }
      onHighlightError?.(error, { lang });
      return null;
    }
  };
}

/**
 * @param {import('./index.mjs').DiagramRenderer} renderer Diagram renderer to check.
 * @param {import('./index.mjs').DiagramOptions} diagramOptions Diagram options to check.
 * @returns {Required<Pick<import('./index.mjs').DiagramOptions, "language">> & import('./index.mjs').DiagramOptions} Checked options with the default language.
 */
function validateDiagramOptions(renderer, diagramOptions) {
  if (!renderer || typeof renderer.render !== "function") {
    throw new TypeError("renderer must provide a synchronous render method");
  }
  if (diagramOptions === null || typeof diagramOptions !== "object") {
    throw new TypeError("diagramOptions must be an object");
  }
  return validateDiagramFields(diagramOptions);
}

/**
 * @param {import('./index.mjs').DiagramOptions} diagramOptions Diagram options to check.
 * @returns {Required<Pick<import('./index.mjs').DiagramOptions, "language">> & import('./index.mjs').DiagramOptions} Checked options with the default language.
 */
function validateDiagramFields(diagramOptions) {
  const {
    language = "mermaid",
    colorScheme,
    idPrefix = "fm-mermaid",
    onRenderError,
  } = diagramOptions;
  if (typeof language !== "string" || language === "") {
    throw new TypeError("diagramOptions.language must be a nonempty string");
  }
  if (colorScheme != null && !["light", "dark", "auto"].includes(colorScheme)) {
    throw new TypeError('diagramOptions.colorScheme must be "light", "dark", or "auto"');
  }
  validateIdPrefix(idPrefix);
  if (onRenderError != null && typeof onRenderError !== "function") {
    throw new TypeError("diagramOptions.onRenderError must be a function");
  }
  return { language, colorScheme, idPrefix, onRenderError };
}

/** @param {unknown} idPrefix SVG namespace prefix to check. */
function validateIdPrefix(idPrefix) {
  if (typeof idPrefix !== "string" || !/^[A-Za-z][\w-]*$/u.test(idPrefix)) {
    throw new TypeError(
      "diagramOptions.idPrefix must start with an ASCII letter and contain only letters, digits, - or _",
    );
  }
}

/**
 * Adapt a diagram renderer, such as `new Renderer()` from `@ferrigura/core`,
 * to a whole-block code renderer for HTML and JSX output. Ferromark only calls
 * `render(text, options)`; it does not import or pin the diagram package.
 * Each fence gets the SVG namespace `<idPrefix>-<UTF-8 start offset>`, as in
 * the native integration.
 * @param {import('./index.mjs').DiagramRenderer} renderer Synchronous diagram renderer.
 * @param {import('./index.mjs').DiagramOptions} [diagramOptions] Diagram options.
 * @returns {import('./index.mjs').CodeBlockRenderer} A whole-block renderer returning `{ html }`.
 */
export function createDiagramCodeRenderer(renderer, diagramOptions = {}) {
  const { language, colorScheme, idPrefix, onRenderError } = validateDiagramOptions(
    renderer,
    diagramOptions,
  );
  // Code hooks share the public (code, language, meta, block) shape.
  // eslint-disable-next-line max-params
  return (code, codeLanguage, _meta, block) => {
    if (codeLanguage !== language) return null;
    const location = blockLocation(block);
    try {
      const options = {
        ...(colorScheme == null ? {} : { colorScheme }),
        ...(location == null ? {} : { id: `${idPrefix}-${location.start}` }),
      };
      const html = renderer.render(code, Object.keys(options).length === 0 ? undefined : options);
      if (typeof html !== "string") {
        throw new TypeError("diagram renderers must return an SVG string");
      }
      return { html };
    } catch (error) {
      onRenderError?.(error, { language, ...location });
      return null;
    }
  };
}

/**
 * Adapt a Shiki-compatible highlighter, such as one from `@ferriki/core`, to
 * the highlighting stage. Ferromark keeps its code block wrapper, line
 * annotations and code block component and inserts one fragment per line.
 * @param {import('./index.mjs').TokenHighlighter} highlighter Synchronous highlighter.
 * @param {import('./index.mjs').LineHighlightOptions} highlightOptions Theme and error options.
 * @returns {import('./index.mjs').CodeLineHighlighter} A highlighting-stage hook.
 */
export function createLineHighlighter(highlighter, highlightOptions) {
  const { theme, themes, onHighlightError } = validateLineHighlightOptions(
    highlighter,
    highlightOptions,
  );
  const themeOptions =
    themes == null
      ? { theme: /** @type {string} */ (theme) }
      : { themes, defaultColor: /** @type {const} */ (false) };
  // Code hooks share the public (code, language, meta, block) shape.
  // eslint-disable-next-line max-params
  return (code, language, _meta, block) => {
    if (language == null || language === "") return null;
    /** @type {{ tokens?: import('./index.mjs').HighlightToken[][], style?: unknown }} */
    const captured = {};
    try {
      highlighter.codeToHtml(code, {
        lang: language,
        ...themeOptions,
        transformers: [
          {
            name: "ferromark:lines",
            tokens(tokens) {
              captured.tokens = tokens;
            },
            pre(node) {
              captured.style = node?.properties?.style;
            },
          },
        ],
      });
      if (!Array.isArray(captured.tokens)) {
        throw new TypeError("highlighter did not expose tokens to transformers");
      }
      return highlightedLines(captured.tokens, captured.style, themes != null);
    } catch (error) {
      onHighlightError?.(error, { lang: language, ...blockLocation(block) });
      return null;
    }
  };
}

/**
 * @param {import('./index.mjs').TokenHighlighter} highlighter Highlighter to check.
 * @param {import('./index.mjs').LineHighlightOptions} options Options to check.
 */
function validateLineHighlightOptions(highlighter, options) {
  if (!highlighter || typeof highlighter.codeToHtml !== "function") {
    throw new TypeError("highlighter must provide a synchronous codeToHtml method");
  }
  if (options === null || typeof options !== "object") {
    throw new TypeError("highlightOptions must be an object");
  }
  const { theme, themes, onHighlightError } = options;
  const pair = themePair(themes);
  if ((theme == null) === (pair == null) || (theme != null && typeof theme !== "string")) {
    throw new TypeError(
      "highlightOptions needs either a theme name or themes: { light, dark } with theme names",
    );
  }
  if (onHighlightError != null && typeof onHighlightError !== "function") {
    throw new TypeError("highlightOptions.onHighlightError must be a function");
  }
  return { theme, themes: pair, onHighlightError };
}

/**
 * @param {unknown} themes Candidate light and dark theme names.
 * @returns {{ light: string, dark: string } | undefined} The checked pair.
 */
function themePair(themes) {
  if (themes == null || typeof themes !== "object") return;
  const { light, dark } = /** @type {{ light?: unknown, dark?: unknown }} */ (themes);
  return typeof light === "string" && typeof dark === "string" ? { light, dark } : undefined;
}

/**
 * @param {import('./index.mjs').HighlightToken[][]} tokens Highlighted tokens per line.
 * @param {unknown} style The highlighter's `<pre>` style.
 * @param {boolean} dual Whether light and dark themes were requested.
 * @returns {import('./index.mjs').HighlightedCode} Line fragments and block colors.
 */
function highlightedLines(tokens, style, dual) {
  const lines = tokens.map((line) =>
    line
      .map((token) => {
        if (token.content === "") return "";
        const css = dual ? dualTokenStyle(token) : singleTokenStyle(token);
        const text = escapeHtml(token.content);
        return css === "" ? text : `<span style="${escapeAttribute(css)}">${text}</span>`;
      })
      .join(""),
  );
  const colors = preColors(typeof style === "string" ? style : "");
  return dual
    ? {
        lines,
        foreground: colors["--shiki-light"],
        background: colors["--shiki-light-bg"],
        darkForeground: colors["--shiki-dark"],
        darkBackground: colors["--shiki-dark-bg"],
      }
    : { lines, foreground: colors.color, background: colors["background-color"] };
}

/** @param {string} style CSS declarations. @returns {Record<string, string>} */
function preColors(style) {
  /** @type {Record<string, string>} */
  const colors = {};
  for (const declaration of style.split(";")) {
    const separator = declaration.indexOf(":");
    if (separator > 0) {
      colors[declaration.slice(0, separator).trim().toLowerCase()] = declaration
        .slice(separator + 1)
        .trim();
    }
  }
  return colors;
}

const FONT_ITALIC = 1;
const FONT_BOLD = 2;
const FONT_UNDERLINE = 4;
const FONT_STRIKETHROUGH = 8;

/** @param {import('./index.mjs').HighlightToken} token Single-theme token. */
function singleTokenStyle(token) {
  const declarations = [];
  if (token.color) declarations.push(`color:${token.color}`);
  const fontStyle = token.fontStyle ?? 0;
  if (fontStyle & FONT_ITALIC) declarations.push("font-style:italic");
  if (fontStyle & FONT_BOLD) declarations.push("font-weight:bold");
  const decoration = textDecoration(fontStyle);
  if (decoration != null) declarations.push(`text-decoration:${decoration}`);
  return declarations.join(";");
}

/**
 * Mirrors the native Ferriki JSX integration so both paths emit the same fragments.
 * @param {import('./index.mjs').HighlightToken} token Dual-theme token.
 */
function dualTokenStyle(token) {
  const light = token.variants?.light ?? {};
  const dark = token.variants?.dark ?? {};
  const declarations = [];
  if (light.color) declarations.push(`color:${light.color}`, `--shiki-light:${light.color}`);
  if (dark.color) declarations.push(`--shiki-dark:${dark.color}`);
  declarations.push(...dualFontDeclarations(light.fontStyle ?? 0, dark.fontStyle ?? 0));
  return declarations.join(";");
}

/**
 * @param {number} lightStyle Light theme font style flags.
 * @param {number} darkStyle Dark theme font style flags.
 * @returns {string[]} Font declarations with light and dark variables.
 */
function dualFontDeclarations(lightStyle, darkStyle) {
  const declarations = [];
  for (const [flag, property, on] of /** @type {const} */ ([
    [FONT_ITALIC, "font-style", "italic"],
    [FONT_BOLD, "font-weight", "bold"],
  ])) {
    if ((lightStyle | darkStyle) & flag) {
      const lightValue = lightStyle & flag ? on : "normal";
      const darkValue = darkStyle & flag ? on : "normal";
      declarations.push(
        `${property}:${lightValue}`,
        `--shiki-light-${property}:${lightValue}`,
        `--shiki-dark-${property}:${darkValue}`,
      );
    }
  }
  const lightDecoration = textDecoration(lightStyle) ?? "none";
  const darkDecoration = textDecoration(darkStyle) ?? "none";
  if (lightDecoration !== "none" || darkDecoration !== "none") {
    declarations.push(
      `text-decoration:${lightDecoration}`,
      `--shiki-light-text-decoration:${lightDecoration}`,
      `--shiki-dark-text-decoration:${darkDecoration}`,
    );
  }
  return declarations;
}

/** @param {number} fontStyle TextMate font style flags. */
function textDecoration(fontStyle) {
  const underline = (fontStyle & FONT_UNDERLINE) !== 0;
  const strike = (fontStyle & FONT_STRIKETHROUGH) !== 0;
  if (underline && strike) return "underline line-through";
  if (underline) return "underline";
  return strike ? "line-through" : undefined;
}

/** @param {string} value Text content. */
function escapeHtml(value) {
  return value.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;");
}

/** @param {string} value Attribute value. */
function escapeAttribute(value) {
  return escapeHtml(value).replaceAll('"', "&quot;").replaceAll("'", "&#39;");
}

/**
 * @param {import('./index.mjs').CodeBlockLocation | undefined} block Location argument.
 * @returns {import('./index.mjs').CodeBlockLocation | undefined} A plain copy.
 */
function blockLocation(block) {
  return block == null ? undefined : { start: block.start, end: block.end };
}

/**
 * Compose synchronous whole-code-block renderers in priority order.
 * A string or `{ html }` handles the block (including an empty string); null or undefined declines it.
 * @param {...import('./index.mjs').CodeBlockRenderer} renderers Trusted render callbacks.
 * @returns {import('./index.mjs').CodeBlockRenderer} A reusable callback for HTML or JSX entry points.
 */
export function composeCodeRenderers(...renderers) {
  for (const renderer of renderers) {
    if (typeof renderer !== "function") {
      throw new TypeError("code renderers must be synchronous functions");
    }
  }
  // Code hooks share the public (code, language, meta, block) shape.
  // eslint-disable-next-line max-params
  return (code, language, meta, block) => {
    for (const renderer of renderers) {
      const output = checkedRenderResult(renderer(code, language, meta, block));
      if (output != null) return output;
    }
    return null;
  };
}

/**
 * Compose code renderers and code hooks in priority order. Within each stage the
 * first hook that returns a result handles the block; whole-block replacements
 * precede highlighting, as in the Rust render hooks.
 * @param {...import('./index.mjs').CodeRendering} items Functions or hook objects.
 * @returns {import('./index.mjs').CodeHooks} One hook object for HTML or JSX entry points.
 */
export function composeCodeHooks(...items) {
  const hooks = items.map((item) => checkedCodeHooks(item));
  const renderers = hooks.flatMap((hook) => (hook.renderCodeBlock ? [hook.renderCodeBlock] : []));
  const highlighters = hooks.flatMap((hook) =>
    hook.highlightCodeBlock ? [hook.highlightCodeBlock] : [],
  );
  return Object.freeze({
    renderCodeBlock: composeCodeRenderers(...renderers),
    // Code hooks share the public (code, language, meta, block) shape.
    // eslint-disable-next-line max-params
    highlightCodeBlock(code, language, meta, block) {
      for (const highlighter of highlighters) {
        const output = checkedHighlightResult(highlighter(code, language, meta, block));
        if (output != null) return output;
      }
      return null;
    },
  });
}

/**
 * @param {unknown} item A code renderer function or hook object.
 * @returns {import('./index.mjs').CodeHooks} Checked hooks.
 */
function checkedCodeHooks(item) {
  if (typeof item === "function") {
    return { renderCodeBlock: /** @type {import('./index.mjs').CodeBlockRenderer} */ (item) };
  }
  if (item === null || typeof item !== "object" || Array.isArray(item)) {
    throw new TypeError("code hooks must be a function or an object with hook functions");
  }
  const hooks = /** @type {Record<string | symbol, unknown>} */ (item);
  for (const key of Reflect.ownKeys(hooks)) {
    checkedCodeHook(key, hooks[key]);
  }
  return /** @type {import('./index.mjs').CodeHooks} */ (item);
}

/**
 * @param {string | symbol} key Hook name.
 * @param {unknown} hook Hook value.
 */
function checkedCodeHook(key, hook) {
  if (key !== "renderCodeBlock" && key !== "highlightCodeBlock") {
    throw new TypeError(`unknown code hook "${String(key)}"`);
  }
  if (hook != null && typeof hook !== "function") {
    throw new TypeError(`${key} must be a synchronous function`);
  }
}

/** @param {unknown} output Whole-block renderer result. */
function checkedRenderResult(output) {
  if (output == null) return null;
  if (typeof output === "string") return output;
  if (
    typeof output === "object" &&
    typeof (/** @type {{ html?: unknown }} */ (output).html) === "string"
  ) {
    return { html: /** @type {{ html: string }} */ (output).html };
  }
  throw new TypeError(
    "code renderers must return a string, { html }, null, or undefined synchronously",
  );
}

/** @param {unknown} output Highlighting hook result. */
function checkedHighlightResult(output) {
  if (output == null) return null;
  const value = /** @type {import('./index.mjs').HighlightedCode} */ (output);
  if (
    typeof output !== "object" ||
    !Array.isArray(value.lines) ||
    !value.lines.every((line) => typeof line === "string")
  ) {
    throw new TypeError(
      "code highlighters must return { lines } with string fragments, null, or undefined synchronously",
    );
  }
  /** @type {import('./index.mjs').HighlightedCode} */
  const result = { lines: value.lines };
  for (const key of /** @type {const} */ ([
    "foreground",
    "background",
    "darkForeground",
    "darkBackground",
  ])) {
    if (value[key] != null) {
      if (typeof value[key] !== "string") {
        throw new TypeError(`highlighted ${key} must be a string`);
      }
      result[key] = value[key];
    }
  }
  return result;
}

/**
 * @param {import('./index.mjs').CodeRendering} rendering Function or hooks.
 * @returns {NativeCodeDispatcher} The native callback.
 */
function requiredCodeDispatcher(rendering) {
  if (rendering === undefined) {
    throw new TypeError("code renderers must be synchronous functions or code hooks");
  }
  return /** @type {NativeCodeDispatcher} */ (codeDispatcher(rendering));
}

/**
 * Turn public code rendering into the single native dispatcher.
 * @param {import('./index.mjs').CodeRendering | undefined} rendering Function or hooks.
 * @returns {NativeCodeDispatcher | undefined} The native callback.
 */
function codeDispatcher(rendering) {
  if (rendering === undefined) return;
  const { renderCodeBlock, highlightCodeBlock } = composeCodeHooks(rendering);
  // Code hooks share the public (code, language, meta, block) shape.
  // eslint-disable-next-line max-params
  return (code, language, meta, block) => {
    const location = { start: block.start, end: block.end };
    const hook = block.stage === "render" ? renderCodeBlock : highlightCodeBlock;
    return hook?.(code, language, meta, location) ?? null;
  };
}

/**
 * Replace code blocks with trusted HTML through a synchronous renderer.
 * @param {string | Uint8Array} markdown Markdown source.
 * @param {import('./index.mjs').CodeRendering} renderCode Trusted code hooks.
 * @param {import('./index.mjs').Options} [options] Rendering options.
 * @returns {string} Rendered HTML.
 */
export function toHtmlWithCodeRenderer(markdown, renderCode, options) {
  validateOptions(options);
  const render = requiredCodeDispatcher(renderCode);
  const addon = loadNative();
  const packed = isMarkdown(markdown) ? packOptions(options) : options;
  return Array.isArray(packed)
    ? addon.toHtmlWithRendererPacked(markdown, ...packed, render)
    : addon.toHtmlWithRenderer(markdown, packed, render);
}

/**
 * Replace code blocks with trusted HTML and retain document metadata.
 * @param {string | Uint8Array} markdown Markdown source.
 * @param {import('./index.mjs').CodeRendering} renderCode Trusted code hooks.
 * @param {import('./index.mjs').Options} [options] Rendering options.
 * @returns {import('./index.mjs').TransformResult} HTML, headings, and front matter.
 */
export function transformWithCodeRenderer(markdown, renderCode, options) {
  validateOptions(options);
  const render = requiredCodeDispatcher(renderCode);
  const addon = loadNative();
  const packed = isMarkdown(markdown) ? packOptions(options) : options;
  return Array.isArray(packed)
    ? addon.transformWithRendererPacked(markdown, ...packed, render)
    : addon.transformWithRenderer(markdown, packed, render);
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
  const render = /** @type {NativeCodeDispatcher} */ (
    codeDispatcher(createHighlighterCodeRenderer(highlighter, highlightOptions))
  );
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
  const render = /** @type {NativeCodeDispatcher} */ (
    codeDispatcher(createHighlighterCodeRenderer(highlighter, highlightOptions))
  );
  const addon = loadNative();
  const packed = isMarkdown(markdown) ? packOptions(options) : options;
  return Array.isArray(packed)
    ? addon.transformWithRendererPacked(markdown, ...packed, render)
    : addon.transformWithRenderer(markdown, packed, render);
}

/**
 * @typedef {(code: string, language: string | null | undefined, meta: string | null | undefined,
 *   block: { stage: "render" | "highlight", start: number, end: number }) =>
 *   string | { html: string } | import('./index.mjs').HighlightedCode | null} NativeCodeDispatcher
 * @typedef {{
 *   toHtml(markdown: string | Uint8Array): string
 *   toHtmlBuffer(markdown: string | Uint8Array): import('node:buffer').Buffer
 * }} NativeRendererSession
 * @typedef {import('./index.mjs').CompileJsxOptions | import('./index.mjs').CompileJsxModuleOptions} NativeJsxOptions
 * @typedef {{
 *   readonly metadata: import('./index.mjs').JsxPreparedMetadata
 *   collectReferences(): import('./index.mjs').DocumentReference[]
 *   extractText(options?: import('./index.mjs').TextExtractionOptions): import('./index.mjs').TextExtraction
 *   projectMarkdown(options?: import('./index.mjs').MarkdownProjectionOptions): import('./index.mjs').MarkdownProjectionResult
 *   render(options?: import('./index.mjs').JsxRenderOptions,
 *     renderCode?: NativeCodeDispatcher): import('./index.mjs').JsxResult
 *   renderModule(options?: import('./index.mjs').JsxRenderModuleOptions,
 *     renderCode?: NativeCodeDispatcher): import('./index.mjs').JsxModuleResult
 *   takeDiagnostics(): import('./index.mjs').CodePluginDiagnostic[]
 * }} NativePreparedJsxDocument
 * @typedef {{
 *   prepare(markdown: string | Uint8Array, options?: import('./index.mjs').JsxPreparationOptions,
 *     preparationOptions?: Pick<import('./index.mjs').JsxPreparationOptions, 'format' | 'trackReferences'>): NativePreparedJsxDocument
 *   compile(markdown: string | Uint8Array, options?: NativeJsxOptions,
 *     jsxOptions?: NativeJsxOptions,
 *     renderCode?: NativeCodeDispatcher): import('./index.mjs').JsxResult
 *   compileModule(markdown: string | Uint8Array, options?: NativeJsxOptions,
 *     jsxOptions?: NativeJsxOptions,
 *     renderCode?: NativeCodeDispatcher): import('./index.mjs').JsxModuleResult
 *   renderCodeBlock(input: import('./index.mjs').JsxCodeBlockRenderInput,
 *     renderCode?: NativeCodeDispatcher): import('./index.mjs').JsxCodeBlockRenderResult
 *   toHtml(markdown: string | Uint8Array, options?: import('./index.mjs').Options): string
 *   transform(markdown: string | Uint8Array, options?: import('./index.mjs').Options): import('./index.mjs').TransformResult
 *   takeDiagnostics(): import('./index.mjs').CodePluginDiagnostic[]
 * }} NativeCodeCompilerSession
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
 *   renderer: NativeCodeDispatcher,
 * ]} NativePackedRendererOptions
 * @typedef {{
 *   Renderer: {
 *     new (options?: NativeOptions): NativeRendererSession
 *     withPackedOptions(...options: NativePackedOptions): NativeRendererSession
 *   }
 *   CodeCompiler: { new (settings: string): NativeCodeCompilerSession }
 *   availableCodePlugins(): string[]
 *   prepareDocument(markdown: string | Uint8Array, options?: import('./index.mjs').JsxPreparationOptions,
 *     preparationOptions?: Pick<import('./index.mjs').JsxPreparationOptions, 'format' | 'trackReferences'>): NativePreparedJsxDocument
 *   compileJsx(
 *     markdown: string | Uint8Array,
 *     options?: NativeJsxOptions,
 *     jsxOptions?: NativeJsxOptions,
 *     renderCode?: NativeCodeDispatcher,
 *   ): import('./index.mjs').JsxResult
 *   compileJsxModule(
 *     markdown: string | Uint8Array,
 *     options?: NativeJsxOptions,
 *     jsxOptions?: NativeJsxOptions,
 *     renderCode?: NativeCodeDispatcher,
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
 *     renderer: NativeCodeDispatcher,
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
 *     renderer: NativeCodeDispatcher,
 *   ): import('./index.mjs').TransformResult
 *   transformWithRendererPacked(
 *     markdown: string | Uint8Array,
 *     ...options: NativePackedRendererOptions
 *   ): import('./index.mjs').TransformResult
 * }} NativeBindings
 */

/** @returns {NativeBindings} Loaded native bindings. */
function loadNative() {
  return nativeLoader();
}
