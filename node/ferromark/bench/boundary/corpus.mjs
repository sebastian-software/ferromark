// Loads the frozen broad corpus and describes how V8 stores each input.

import { readFileSync } from "node:fs";
import { serialize } from "node:v8";
import { gunzipSync } from "node:zlib";

// Spans every size bin and all three input representations.
const smokeDocuments = new Set([
  "comment-ack",
  "comment-unicode",
  "wiki-chess-lead",
  "vite-docs-api-plugin",
  "wiki-chess-article-body",
]);

/** Selects the corpus documents to measure, optionally widened to two-byte storage. */
export function loadDocuments({ corpus, filter, smoke, twoByte }) {
  const { cases } = JSON.parse(gunzipSync(readFileSync(corpus)).toString("utf8"));
  const pattern = filter === undefined ? undefined : new RegExp(filter, "u");
  const selected = cases.filter((entry) =>
    pattern ? pattern.test(entry.name) : !smoke || smokeDocuments.has(entry.name),
  );
  if (selected.length === 0) throw new Error("No corpus document matches the selection");
  return selected.map((entry) => {
    // Widening keeps the content and stores it as UTF-16: a slice of a
    // two-byte string stays two-byte.
    const markdown = twoByte ? `${entry.input}Ā`.slice(0, -1) : entry.input;
    return {
      category: entry.category,
      content: contentClass(markdown),
      inputBytes: Buffer.byteLength(markdown),
      markdown,
      name: entry.name,
      representation: representation(markdown),
      sizeBin: entry.size_bin,
    };
  });
}

/**
 * The characters a string holds, which decide the storage V8 can choose.
 * @param text String to classify.
 */
export function contentClass(text) {
  // UTF-8 and UTF-16 lengths agree only for ASCII; Latin-1 round-trips only
  // characters up to U+00FF.
  if (Buffer.byteLength(text) === text.length) return "ascii";
  return Buffer.from(text, "latin1").toString("latin1") === text ? "latin1" : "wide";
}

/**
 * V8's actual storage: its serializer tags one-byte strings '"' and two-byte 'c'.
 * @param text String to inspect.
 */
export function representation(text) {
  return serialize(text)[2] === 0x22 ? "one-byte" : "two-byte";
}
