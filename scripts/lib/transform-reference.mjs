import { createRequire } from "node:module";
import remarkEmoji from "remark-emoji";
import remarkGemoji from "remark-gemoji";
import remarkSmartypants from "remark-smartypants";
import { runPluginReference } from "./plugin-reference-runner.mjs";

const require = createRequire(import.meta.url);
const remarkTypograf = require("@mavrin/remark-typograf");

const plugins = {
  emoji: remarkEmoji,
  gemoji: remarkGemoji,
  smartypants: remarkSmartypants,
  typograf: remarkTypograf,
};

export async function runTransformReference(fixture) {
  const plugin = plugins[fixture.reference];

  if (!plugin) {
    throw new Error("Unsupported transform reference: " + fixture.reference);
  }

  return runPluginReference({
    source: fixture.source,
    plugin,
    options: fixture.options,
    parserPlugins: fixture.parserPlugins ?? ["remark-gfm"],
  });
}
