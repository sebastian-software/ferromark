import assert from "node:assert/strict";

import { prepareDocument } from "../ferromark/index.mjs";

const prepared = prepareDocument(
  "# Guide\n\nVisible **prose** and `inline code`.\n\n## Setup\n\nInstall the package.\n",
);
const extracted = prepared.extractText({ includeBlockCode: false });
assert.equal(extracted.sections[1].parentIndex, 0);
assert.equal(extracted.sections[1].id, "setup");
assert.equal(extracted.sections[1].text, "Setup\n\nInstall the package.");
for (const section of extracted.sections) {
  const bytes = Buffer.from(extracted.text);
  assert.equal(
    bytes.subarray(section.textRange.start, section.textRange.end).toString(),
    section.text,
  );
}
