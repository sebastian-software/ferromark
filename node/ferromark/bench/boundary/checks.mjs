// Output checks that run before any lane is timed: every lane must produce
// what it claims, so a lane that measures the wrong thing fails loudly.

import {
  empty,
  hasKeptDefault,
  hasNapiStringReference,
  hasPackedOptions,
  trusted,
} from "./lanes.mjs";

function encodeIntoLength({ convertBytes, encoder, scratch }, markdown) {
  const { written } = encoder.encodeInto(markdown, scratch);
  return convertBytes(scratch.subarray(0, written));
}

const utf8 = (buffer) => buffer.toString("utf8");

function outputChecks(state) {
  const { bytes, html, markdown, native, outputAscii, probe, renderer } = state;
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
    ...(state.acceptsBytes && {
      "Renderer.toHtml(bytes)": renderer.toHtml(bytes) === html,
      "toHtml(bytes)": native.toHtml(bytes) === html,
      "toHtmlBuffer(bytes)": utf8(native.toHtmlBuffer(bytes)) === html,
    }),
  };
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
    boundaryEcho: native.boundaryEcho(markdown) === markdown,
    boundaryLen: native.boundaryLen(markdown) === inputBytes,
    boundaryMake: native.boundaryMake(outputBytes).length === outputBytes,
    bytesLen: state.convertBytes(bytes) === inputBytes,
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
