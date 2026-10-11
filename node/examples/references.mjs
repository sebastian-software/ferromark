import { prepareDocument } from "../ferromark/index.mjs";

const source = "[Guide][guide] ![Logo](./logo.svg)\n\n[guide]: /guide\n";
const document = prepareDocument(source);
for (const reference of document.collectReferences()) {
  console.log(reference.kind, reference.destination, reference.span);
  // Apply application-specific route or asset rules to each destination.
}

const transformed = prepareDocument("[Hello :smile:](/hello)", {
  passes: [{ kind: "emojiShortcodes" }],
  trackReferences: true,
});
console.log(transformed.collectReferences()[0].provenance); // transformed
