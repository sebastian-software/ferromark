# ADR-0025: Validate and reproduce static Mermaid output

- Status: Accepted
- Date: 2026-10-27
- Extends [ADR-0024](ADR-0024-native-code-plugins.md).

## Context

The optional Ferrigura adapter already renders Mermaid in Rust. Issue [#531](https://github.com/sebastian-software/ferromark/issues/531)
also requires safe inline SVG, repeatable prerendering, and source-located code
fallback. Stateful IDs, raw engine SVG, and a default throwing policy did not
meet that contract. The engine choice remains the existing pinned Ferrigura
integration; exact mermaid.js layout parity is outside this adapter's contract.

## Decision

Derive IDs from a caller namespace and the original fence's byte offset. Use a
constant default namespace rather than compiler construction order. Reusing a
compiler, preparing a document, or rendering HTML before JSX cannot change IDs.
Applications combining documents supply distinct namespaces. Freeze the engine's
implicit clock at the Unix epoch so date-dependent diagrams remain reproducible.

Default to automatic light/dark SVG and escaped code fallback. Retain an explicit
throwing policy for applications that require it. Both policies record failures
with the complete fence's original UTF-8 range. Diagnostics can be drained from
the Rust adapter, Node compiler, or shared prepared handle. A failed fence must
not prevent subsequent diagrams from rendering under the fallback policy.

Parse engine SVG as XML, allow only static SVG elements and attributes, validate
CSS tokens, and serialize text and attributes again before emitting HTML or
converting to JSX. Reject scripts, event attributes, foreign namespaces,
`foreignObject`, images, animation elements, processing instructions, DTDs,
non-local references, and CSS resource-loading constructs. Local fragment URLs
and embedded base64 font data are allowed; neither requires a network request.
The policy applies regardless of the authored HTML trust setting. Independent
application callbacks retain their separate trusted-output contract.

Use the existing XML parser and pinned `cssparser` tokenizer rather than matching
CSS URLs with strings or regular expressions. CSS escapes, comments, quoted URLs
and nested functions need token semantics. This introduces a version-scoped
MPL-2.0 exception for the unmodified tokenizer in optional Mermaid builds only.
Custom bundles retain its license and identify its exact source archive in the
notices manifest. The family license allow-list and engine-free standard build
remain unchanged.

## Consequences and validation

Unsupported SVG constructs cause a diagnostic and ordinary code output, even if
Ferrigura can draw them. Embedded links to other pages and image-based diagrams
are deliberately unavailable under this policy. This is a bounded static SVG
integration, not a sanitizer for arbitrary application HTML.

Five shared fixtures cover flowchart, sequence, state, class and ER output.
Pinned SHA-256 values test complete SVG bytes, including fonts and CSS, across
Rust test processes and installed Node bundles. Existing CI runs them on Linux,
macOS and Windows; musl bundle verification uses the same expectations. Tests
also cover repeated/combined diagrams, invalid and unknown types, UTF-8 spans,
prepared ownership, strict errors, and active SVG/CSS rejection.

The adapter, tokenizer, and fonts remain optional. Parser snapshots and
conformance output are unchanged. No rendering performance claim is made.
