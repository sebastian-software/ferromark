// The timed lanes. Each lane runs `k` units: `k` calls, or for the core loops
// one call of `k` iterations. Every lane is its own loop, so V8 optimizes each
// call site separately, and every result lands in `sink`.

import { representation } from "./corpus.mjs";

let sink;

/** The last lane result, read once so no lane result is dead. */
export function lastResult() {
  return sink;
}

/** Per-document state shared by verification and the lanes. */
export function documentState(native, document) {
  const { markdown } = document;
  const probe = new native.BoundaryProbe(markdown);
  return {
    bytes: Buffer.from(markdown, "utf8"),
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

function encodeIntoLength({ encoder, native, scratch }, markdown) {
  const { written } = encoder.encodeInto(markdown, scratch);
  return native.boundaryBytesLen(scratch.subarray(0, written));
}

const utf8 = (buffer) => buffer.toString("utf8");

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

function outputChecks({ html, markdown, native, outputAscii, probe, renderer }) {
  return {
    "Renderer.toHtml": renderer.toHtml(markdown) === html,
    "Renderer.toHtmlBuffer": utf8(renderer.toHtmlBuffer(markdown)) === html,
    ...(hasKeptDefault(native) && {
      boundaryToHtmlFresh: native.boundaryToHtmlFresh(markdown) === html,
    }),
    boundaryToHtmlExternal: native.boundaryToHtmlExternal(markdown) === html,
    htmlBuffer: utf8(probe.htmlBuffer()) === html,
    htmlBufferCopy: utf8(probe.htmlBufferCopy()) === html,
    htmlLatin1: !outputAscii || probe.htmlLatin1() === html,
    toHtml: native.toHtml(markdown) === html,
    toHtmlBuffer: utf8(native.toHtmlBuffer(markdown)) === html,
  };
}

/**
 * Whether the addon has napi-rs's `String` conversion as a reference next to
 * the exports' single pass. An addon built before the exports switched lacks
 * it, and the script then skips its checks and lane, so one script can time a
 * build from before and one from after the switch.
 * @param native The loaded addon.
 */
function hasNapiStringReference(native) {
  return typeof native.boundaryLenNapiString === "function";
}

function inputChecks({ bytes, markdown, native }) {
  if (!hasNapiStringReference(native)) return {};
  const converted = native.boundaryInputBytes(markdown);
  return {
    boundaryInputBytes:
      converted.equals(bytes) && converted.equals(native.boundaryNapiStringBytes(markdown)),
    boundaryLenNapiString: native.boundaryLenNapiString(markdown) === bytes.length,
  };
}

function partChecks(state, inputBytes) {
  const { bytes, markdown, native, outputBytes } = state;
  const threeOutputs = (outputBytes * 3) >>> 0;
  return {
    boundaryBytesLen: native.boundaryBytesLen(bytes) === inputBytes,
    boundaryEcho: native.boundaryEcho(markdown) === markdown,
    boundaryLen: native.boundaryLen(markdown) === inputBytes,
    boundaryMake: native.boundaryMake(outputBytes).length === outputBytes,
    ...(hasKeptDefault(native) && {
      coreDefault: native.boundaryCoreOnly(markdown, 3, "default") === threeOutputs,
    }),
    coreFresh: native.boundaryCoreOnly(markdown, 3, "fresh") === threeOutputs,
    coreReuse: native.boundaryCoreOnly(markdown, 3, "reuse") === threeOutputs,
    coreSetup: native.boundaryCoreOnly("", 3, "setup") === 3,
    encodeIntoLen: encodeIntoLength(state, markdown) === inputBytes,
    ...inputChecks(state),
  };
}

/** Output equality comes before timing: every lane must produce what it claims. */
export function verify(document, state) {
  const checks = { ...outputChecks(state), ...partChecks(state, document.inputBytes) };
  const failed = Object.keys(checks).filter((name) => !checks[name]);
  if (failed.length > 0) {
    throw new Error(`${document.name}: output mismatch in ${failed.join(", ")}`);
  }
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

// Prototypes of the reductions README.md describes.
function candidateLanes({ bytes, encoder, markdown, native, outputAscii, probe, scratch }) {
  const lanes = {
    bytesLen(k) {
      for (let i = 0; i < k; i++) sink = native.boundaryBytesLen(bytes);
    },
    encodeIntoLen(k) {
      for (let i = 0; i < k; i++) {
        const { written } = encoder.encodeInto(markdown, scratch);
        sink = native.boundaryBytesLen(scratch.subarray(0, written));
      }
    },
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
    ...boundaryLanes(state),
    ...coreLanes(state),
    ...candidateLanes(state),
  };
}

const trusted = { renderPolicy: "trusted" };
const empty = {};

/**
 * Whether the addon has the packed options entries the facade calls since it
 * packs options itself. An older addon lacks them, and the script then skips
 * the packed lanes.
 * @param native The loaded addon.
 */
function hasPackedOptions(native) {
  return typeof native.boundaryOptionsPacked === "function";
}

/**
 * The option lanes must agree before they are timed: packed and object options
 * resolve alike, and the facade renders as the object-taking export does.
 */
export function verifyFixed(native, facade) {
  const markdown = "<i>raw</i> x^2^";
  try {
    facade.toHtml(markdown, trusted);
  } catch (error) {
    throw new Error(
      "The facade does not work with this addon; pass --facade with the package directory " +
        "of the checkout the addon was built from",
      { cause: error },
    );
  }
  const checks = {
    ...(hasPackedOptions(native) && {
      optionsPackedNone: native.boundaryOptionsPacked(0, 0) === native.boundaryOptions(empty),
      optionsPackedTrusted: native.boundaryOptionsPacked(1, 1) === native.boundaryOptions(trusted),
    }),
    facadeRendererTrusted:
      new facade.Renderer(trusted).toHtml(markdown) ===
      new native.Renderer(trusted).toHtml(markdown),
    facadeToHtmlEmpty: facade.toHtml(markdown) === native.toHtml(markdown),
    facadeToHtmlEmptyOptions: facade.toHtml(markdown, empty) === native.toHtml(markdown, empty),
    facadeToHtmlTrusted: facade.toHtml(markdown, trusted) === native.toHtml(markdown, trusted),
  };
  const failed = Object.keys(checks).filter((name) => !checks[name]);
  if (failed.length > 0) {
    throw new Error(`fixed lanes: output mismatch in ${failed.join(", ")}`);
  }
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
