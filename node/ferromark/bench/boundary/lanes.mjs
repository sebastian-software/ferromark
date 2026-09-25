// The timed lanes. Each lane runs `k` units: `k` calls, or for the core loops
// one call of `k` iterations. Every lane is its own loop, so V8 optimizes each
// call site separately, and every result lands in `sink`.

import { representation } from "./corpus.mjs";

let sink;

/** The last lane result, read once so no lane result is dead. */
export function lastResult() {
  return sink;
}

/**
 * Whether the addon's exports take Markdown as UTF-8 bytes (a `Uint8Array`).
 * An addon built before they did lacks the public bytes lanes, and its
 * `bytesInput` and `encodeIntoInput` candidates time its `boundaryBytesLen`
 * prototype instead, so one script can time a build from before and one from
 * after.
 * @param native The loaded addon.
 */
export function acceptsBytes(native) {
  try {
    return native.boundaryLen(new Uint8Array([0x61])) === 1;
  } catch {
    return false;
  }
}

/** Per-document state shared by verification and the lanes. */
export function documentState(native, document) {
  const { markdown } = document;
  const probe = new native.BoundaryProbe(markdown);
  const accepted = acceptsBytes(native);
  return {
    acceptsBytes: accepted,
    bytes: Buffer.from(markdown, "utf8"),
    // The input conversion of UTF-8 bytes: the exports' own, or on an addon
    // from before they took bytes, its `boundaryBytesLen` prototype.
    convertBytes: accepted ? native.boundaryLen : native.boundaryBytesLen,
    encoder: new TextEncoder(),
    html: probe.html(),
    markdown,
    native,
    outputAscii: probe.htmlIsAscii,
    outputBytes: probe.htmlByteLength,
    outputRepresentation: representation(native.toHtml(markdown)),
    probe,
    renderer: new native.Renderer(),
    scratch: new Uint8Array(markdown.length * 3 + 16),
  };
}

/**
 * Whether the addon renders one-shot calls without options on a kept renderer
 * and has `boundaryToHtmlFresh`, the one-shot path from before, as a reference.
 * An addon built before lacks it, and the script then skips its checks and
 * lanes and splits `toHtml` as that addon runs it, so one script can time a
 * build from before and one from after the change.
 * @param native The loaded addon.
 */
export function hasKeptDefault(native) {
  return typeof native.boundaryToHtmlFresh === "function";
}

/**
 * Whether the addon has napi-rs's `String` conversion as a reference next to
 * the exports' single pass. An addon built before the exports switched lacks
 * it, and the script then skips its checks and lane, so one script can time a
 * build from before and one from after the switch.
 * @param native The loaded addon.
 */
export function hasNapiStringReference(native) {
  return typeof native.boundaryLenNapiString === "function";
}

// The public exports, exactly as the package calls them.
function publicLanes({ markdown, native, renderer }) {
  return {
    rendererToHtml(k) {
      for (let i = 0; i < k; i++) sink = renderer.toHtml(markdown);
    },
    rendererToHtmlBuffer(k) {
      for (let i = 0; i < k; i++) sink = renderer.toHtmlBuffer(markdown);
    },
    toHtml(k) {
      for (let i = 0; i < k; i++) sink = native.toHtml(markdown);
    },
    toHtmlBuffer(k) {
      for (let i = 0; i < k; i++) sink = native.toHtmlBuffer(markdown);
    },
  };
}

// The same public exports with the document as UTF-8 bytes, as a caller that
// read the file into a `Buffer` passes it. Only an addon that takes bytes has
// these lanes.
function bytesLanes({ acceptsBytes: accepted, bytes, native, renderer }) {
  if (!accepted) return {};
  return {
    rendererToHtmlBytes(k) {
      for (let i = 0; i < k; i++) sink = renderer.toHtml(bytes);
    },
    toHtmlBufferBytes(k) {
      for (let i = 0; i < k; i++) sink = native.toHtmlBuffer(bytes);
    },
    toHtmlBytes(k) {
      for (let i = 0; i < k; i++) sink = native.toHtml(bytes);
    },
  };
}

// Call floors, input and output conversion.
function boundaryLanes({ markdown, native, outputBytes, probe }) {
  return {
    echo(k) {
      for (let i = 0; i < k; i++) sink = native.boundaryEcho(markdown);
    },
    html(k) {
      for (let i = 0; i < k; i++) sink = probe.html();
    },
    htmlBuffer(k) {
      for (let i = 0; i < k; i++) sink = probe.htmlBuffer();
    },
    len(k) {
      for (let i = 0; i < k; i++) sink = native.boundaryLen(markdown);
    },
    make(k) {
      for (let i = 0; i < k; i++) sink = native.boundaryMake(outputBytes);
    },
    methodNoop(k) {
      for (let i = 0; i < k; i++) sink = probe.noop();
    },
    noop(k) {
      for (let i = 0; i < k; i++) sink = native.boundaryNoop();
    },
  };
}

// The Rust core, `k` iterations inside one call.
function coreLanes({ markdown, native }) {
  return {
    ...(hasKeptDefault(native) && {
      coreDefault(k) {
        sink = native.boundaryCoreOnly(markdown, k, "default");
      },
    }),
    coreFresh(k) {
      sink = native.boundaryCoreOnly(markdown, k, "fresh");
    },
    coreReuse(k) {
      sink = native.boundaryCoreOnly(markdown, k, "reuse");
    },
    coreSetup(k) {
      sink = native.boundaryCoreOnly("", k, "setup");
    },
  };
}

// The input conversion of UTF-8 bytes (`convertBytes`), for a caller that holds
// bytes and for one that encodes its string into a kept scratch buffer first.
function bytesInputLanes({ bytes, convertBytes, encoder, markdown, scratch }) {
  return {
    bytesLen(k) {
      for (let i = 0; i < k; i++) sink = convertBytes(bytes);
    },
    encodeIntoLen(k) {
      for (let i = 0; i < k; i++) {
        const { written } = encoder.encodeInto(markdown, scratch);
        sink = convertBytes(scratch.subarray(0, written));
      }
    },
  };
}

// Prototypes of the reductions README.md describes.
function candidateLanes(state) {
  const { markdown, native, outputAscii, probe } = state;
  const lanes = {
    ...bytesInputLanes(state),
    htmlBufferCopy(k) {
      for (let i = 0; i < k; i++) sink = probe.htmlBufferCopy();
    },
    toHtmlExternal(k) {
      for (let i = 0; i < k; i++) sink = native.boundaryToHtmlExternal(markdown);
    },
  };
  if (outputAscii) {
    lanes.htmlLatin1 = (k) => {
      for (let i = 0; i < k; i++) sink = probe.htmlLatin1();
    };
  }
  if (hasNapiStringReference(native)) {
    lanes.lenNapiString = (k) => {
      for (let i = 0; i < k; i++) sink = native.boundaryLenNapiString(markdown);
    };
  }
  if (hasKeptDefault(native)) {
    lanes.toHtmlFresh = (k) => {
      for (let i = 0; i < k; i++) sink = native.boundaryToHtmlFresh(markdown);
    };
  }
  return lanes;
}

/** Every per-document lane. */
export function documentLanes(state) {
  return {
    ...publicLanes(state),
    ...bytesLanes(state),
    ...boundaryLanes(state),
    ...coreLanes(state),
    ...candidateLanes(state),
  };
}

export const trusted = { renderPolicy: "trusted" };
export const empty = {};

/**
 * Whether the addon has the packed options entries the facade calls since it
 * packs options itself. An older addon lacks them, and the script then skips
 * the packed lanes.
 * @param native The loaded addon.
 */
export function hasPackedOptions(native) {
  return typeof native.boundaryOptionsPacked === "function";
}

// Options as the facade passes them now: packed into plain arguments. Only
// an addon with the packed entries has these lanes.
function packedLanes(native) {
  if (!hasPackedOptions(native)) return {};
  return {
    optionsPackedNone(k) {
      for (let i = 0; i < k; i++) sink = native.boundaryOptionsPacked(0, 0);
    },
    optionsPackedTrusted(k) {
      for (let i = 0; i < k; i++) sink = native.boundaryOptionsPacked(1, 1);
    },
  };
}

// The package facade (index.mjs), loaded against this addon: its validation,
// option packing and the export it calls.
function facadeLanes(facade) {
  return {
    facadeRendererTrusted(k) {
      for (let i = 0; i < k; i++) sink = new facade.Renderer(trusted);
    },
    facadeToHtmlEmpty(k) {
      for (let i = 0; i < k; i++) sink = facade.toHtml("");
    },
    facadeToHtmlEmptyOptions(k) {
      for (let i = 0; i < k; i++) sink = facade.toHtml("", empty);
    },
    facadeToHtmlTrusted(k) {
      for (let i = 0; i < k; i++) sink = facade.toHtml("", trusted);
    },
  };
}

/** Document-independent costs: call floors, options and renderer setup. */
export function fixedLanes(native, facade) {
  const probe = new native.BoundaryProbe("");
  const { coreSetup } = coreLanes({ markdown: "", native });
  const { methodNoop, noop } = boundaryLanes({ markdown: "", native, outputBytes: 0, probe });
  return {
    coreSetup,
    methodNoop,
    noop,
    optionsEmpty(k) {
      for (let i = 0; i < k; i++) sink = native.boundaryOptions(empty);
    },
    optionsNone(k) {
      for (let i = 0; i < k; i++) sink = native.boundaryOptions();
    },
    optionsTrusted(k) {
      for (let i = 0; i < k; i++) sink = native.boundaryOptions(trusted);
    },
    rendererConstruct(k) {
      for (let i = 0; i < k; i++) sink = new native.Renderer();
    },
    rendererConstructTrusted(k) {
      for (let i = 0; i < k; i++) sink = new native.Renderer(trusted);
    },
    toHtmlEmpty(k) {
      for (let i = 0; i < k; i++) sink = native.toHtml("");
    },
    toHtmlEmptyOptions(k) {
      for (let i = 0; i < k; i++) sink = native.toHtml("", empty);
    },
    toHtmlTrusted(k) {
      for (let i = 0; i < k; i++) sink = native.toHtml("", trusted);
    },
    ...packedLanes(native),
    ...facadeLanes(facade),
  };
}
