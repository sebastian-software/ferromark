import assert from "node:assert/strict";

import { prepareDocument } from "../ferromark/index.mjs";

// Preparation and static projection do not load highlighting assets.
const source =
  "import Box from './box.js';\n\n# Guide\n\n<Box>**Static** [link](/guide) {dynamic}</Box>\n";
const prepared = prepareDocument(source, { format: "mdx" });
const projected = prepared.projectMarkdown({ omitTitleHeading: "Guide" });
assert.ok(projected.markdown.includes("**Static** [link](/guide)"));
assert.ok(!projected.markdown.includes("{dynamic}"));
console.log(projected.markdown);
for (const diagnostic of projected.diagnostics) {
  console.log(diagnostic.code, diagnostic.start, diagnostic.end);
}
