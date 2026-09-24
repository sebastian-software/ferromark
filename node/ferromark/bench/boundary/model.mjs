// Splits each public call into input, core, output, fixed and residual time,
// and derives the per-candidate savings. Everything is paired per round.

export const apiNames = ["toHtml", "Renderer.toHtml", "toHtmlBuffer", "Renderer.toHtmlBuffer"];
export const parts = ["input", "core", "output", "fixed", "residual"];

/** Median of a list of numbers. */
export function median(values) {
  const sorted = values.toSorted((left, right) => left - right);
  const middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 === 1 ? sorted[middle] : (sorted[middle - 1] + sorted[middle]) / 2;
}

/** Applies `transform` to every value of `object`. */
export function mapValues(object, transform) {
  return Object.fromEntries(Object.entries(object).map(([key, value]) => [key, transform(value)]));
}

/** Turns per-lane round lists into one lane-to-value object per round. */
export function byRound(rounds) {
  return rounds.noop.map((_, index) => mapValues(rounds, (values) => values[index]));
}

// The model for one round, as [total, input, core, output, fixed, call floor].
// Input: `len` over the free-call floor. Output: the exact HTML (`probe.html`,
// `probe.htmlBuffer`) over the method-call floor. Core: the Rust loops, with
// `Renderer::new` (setup) moved to fixed. Fixed: the call floor, plus setup for
// one-shot calls.
function roundParts(lane) {
  const input = lane.len - lane.noop;
  const text = lane.html - lane.methodNoop;
  const buffer = lane.htmlBuffer - lane.methodNoop;
  const fresh = lane.coreFresh - lane.coreSetup;
  const oneShot = lane.noop + lane.coreSetup;
  return {
    "Renderer.toHtml": [lane.rendererToHtml, input, lane.coreReuse, text, lane.methodNoop],
    "Renderer.toHtmlBuffer": [
      lane.rendererToHtmlBuffer,
      input,
      lane.coreReuse,
      buffer,
      lane.methodNoop,
    ],
    toHtml: [lane.toHtml, input, fresh, text, oneShot, lane.noop],
    toHtmlBuffer: [lane.toHtmlBuffer, input, fresh, buffer, oneShot, lane.noop],
  };
}

function split(rounds, api) {
  const values = rounds.map((lane) => {
    const [total, input, core, output, fixed, call = lane.methodNoop] = roundParts(lane)[api];
    return { call, core, fixed, input, output, total };
  });
  const ns = Object.fromEntries(
    Object.keys(values[0]).map((key) => [key, median(values.map((round) => round[key]))]),
  );
  // Medians of the parts need not sum to the median total; the residual keeps
  // the shares summing to one.
  ns.residual = ns.total - ns.input - ns.core - ns.output - ns.fixed;
  const share = Object.fromEntries(parts.map((part) => [part, ns[part] / ns.total]));
  // The N-API boundary proper: both conversions plus the bare call.
  share.boundary = (ns.input + ns.output + ns.call) / ns.total;
  return { ns, share };
}

/** Per public API: median nanoseconds and shares of each part. */
export function attribution(rounds) {
  return Object.fromEntries(apiNames.map((api) => [api, split(rounds, api)]));
}

// Each candidate as [current path, candidate path, baseline API for the ratio].
const candidatePairs = {
  bufferCopyOutput: ["htmlBuffer", "htmlBufferCopy", "toHtmlBuffer"],
  bytesInput: ["len", "bytesLen", "toHtml"],
  cachedRenderer: ["toHtml", "rendererToHtml", "toHtml"],
  encodeIntoInput: ["len", "encodeIntoLen", "toHtml"],
  externalOutput: ["toHtml", "toHtmlExternal", "toHtml"],
  latin1Output: ["html", "htmlLatin1", "rendererToHtml"],
  singlePassInput: ["len", "lenSinglePass", "toHtml"],
};
export const candidateNames = Object.keys(candidatePairs);

/** Paired per-round savings (current minus candidate) and their share of the call. */
export function candidates(rounds) {
  const result = {};
  for (const [name, [current, candidate, baseline]] of Object.entries(candidatePairs)) {
    if (!(candidate in rounds[0])) continue;
    const savedNs = median(rounds.map((lane) => lane[current] - lane[candidate]));
    const relative = savedNs / median(rounds.map((lane) => lane[baseline]));
    result[name] = { baseline, relative, savedNs };
  }
  return result;
}
