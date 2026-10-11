# Static text projection on prepared native documents

Issue #522 needs reusable search text and section boundaries from the same tree
Ferromark renders. A read-only core `Document::extract_text()` and bulk Node
`prepared.extractText()` own this interpretation. Additional work stays opt-in;
parsing, passes, and existing render output are unchanged.

The contract is static content, not arbitrary MDX runtime DOM. Parsed prose,
link labels, image alt text, inline code, optional block code, literal math, and
footnote bodies contribute text. Raw HTML nodes, expressions, ESM, attributes,
and hidden raw text elements are omitted. Stable separators preserve block,
list, and table boundaries; unknown inline runtime content introduces a space
rather than inventing a joined word. The [guide](../text-extraction.md) owns the
complete consumer policy.

Final anchors use JSX planning, including omitted top-level title behavior,
footnote and authored element namespace claims, heading prefix, level offset,
and duplicate IDs. The heading label helper is shared verbatim with JSX output.
Preparation metadata remains a snapshot and is not reused as a final plan.
Sections have nonoverlapping text ranges and parent indexes rather than repeated
subtree text, leaving index construction and ranking with the consumer.

Original UTF-8 source spans and normalized UTF-8 text ranges are separate fields.
Source section ranges are best-effort boundaries, not reverse mappings of
normalized text. Empty generated spans and reordered headings do not fabricate
provenance. Owned, immutable results preserve prepared-tree lifetimes and permit
repeatable extraction without a cache or JavaScript traversal callbacks.

Criterion records extraction separately from parsing; extraction never runs a
highlight hook. Acceptance fixtures compare final IDs with native JSX output,
cover Unicode byte slicing and explicit static-content policies, and verify that
repeated extraction leaves rendering unchanged.
