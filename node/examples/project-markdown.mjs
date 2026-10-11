import assert from "node:assert/strict";

import { JsxCompiler } from "../ferromark/index.mjs";

// An offline compiler also prepares documents for repeated JSX output.
// Static projection itself never accesses this highlighter.
const compiler = new JsxCompiler({
  theme: { name: "example", settings: [{ settings: { foreground: "#111111" } }] },
  assets: { remote: false },
});
const source =
  "import Box from './box.js';\n\n# Guide\n\n<Box>**Static** [link](/guide) {dynamic}</Box>\n";
const prepared = compiler.prepare(source, { format: "mdx" });
const projected = prepared.projectMarkdown({ omitTitleHeading: "Guide" });
assert.ok(projected.markdown.includes("**Static** [link](/guide)"));
assert.ok(!projected.markdown.includes("{dynamic}"));
console.log(projected.markdown);
for (const diagnostic of projected.diagnostics) {
  console.log(diagnostic.code, diagnostic.start, diagnostic.end);
}
