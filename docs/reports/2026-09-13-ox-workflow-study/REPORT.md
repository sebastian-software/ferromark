# Ox Content: implementation differences and benchmark state audit

Date: September 13, 2026. This investigation changes no production parser code.

The measured Ox implementation does real parsing and rendering on each call.
Code inspection, fresh-renderer controls, CPU samples, and state-isolation
checks found no reuse of a previous document's AST or completed HTML. Ox keeps
its lead when both engines use fresh renderers and when both retain their
available renderer scratch. The evidence supports a real implementation
advantage on these documents, not an unexplained result-cache advantage.

The existing overview nevertheless needed a clearer lifecycle disclosure:
its Ferromark row uses `to_html_with_options`, while Ox's native adapter retains
an `HtmlRenderer`. Those are different integration choices. They must be visible
beside the table rather than inferred from archived adapter code.

[Generated diagnostic results](RESULTS.md) show every lifecycle, document, and
input-identity control. [Reproduction](REPRODUCE.md) describes all artifacts.
These experiments are separate from the
[published full-field run](../2026-09-13-workflow-engine-field/REPORT.md).

## Source and scope

Ferromark source is `0026d4c66eeb4d8a580c15d96fd89976ca9f833a`, with the same
production implementation as the earlier measured source `9ad6481`.
Ox is the exact pinned release revision
`5c97078779cf099245a78aacca8fa7cc5e993316`, package version 3.2.0.
`build.json` and `audit-provenance.json` preserve source and executable hashes.

The input is the same frozen twelve-file documentation collection. Both engines
enable trusted CommonMark, tables, strikethrough, and task lists. Ox's mandatory
generated heading IDs remain on. Ferromark is measured with IDs both off and on;
the latter includes navigation work but does not claim identical ID spelling.
All original content passes the established workload review. The fresh/reused
variants produce exactly the same HTML within each engine and option set.

## What can survive a call?

The [actual adapter](../../../benchmarks/native-pipeline-comparison/engines/ox-content/main.rs)
creates `Allocator::new()` and `Parser::with_options(...).parse()` inside every
render call. The parser is consumed by `parse`; the document and arena are
dropped before the adapter returns. The call is not parse-only, AST reuse,
`render_borrowed`, or incremental rendering.

The pinned source makes the lifetime boundaries explicit:

| State | Lifetime and reset | Implication |
| --- | --- | --- |
| Parser reference maps, bracket/link probes, and last-closer cache | Constructed empty for each parser; root and sub-parser constructors initialize them | Memoization avoids repeated work within a document; it cannot serve a later document |
| AST and growing bump arena | Constructed and destroyed for every Markdown call | The timed operation includes AST construction and arena release |
| Heading maps, counters, and scratch strings | Renderer capacities persist; document state is cleared in `prepare_render` | Reuse saves allocations, but heading collisions and other document state must reset |
| HTML output | `render` moves the `String` out with `mem::take`; the workload drops it | The next call cannot return that previous output buffer as a cached result |
| Entity lookup table and fixed pattern finders | Initialized once; contain HTML entities and fixed search patterns | Process warmup includes this initialization; these are independent of document contents |
| Incremental renderer APIs | Separate entry points, never called by the adapter | They do not participate in the measured row |

Sources: pinned [parser state and constructors](https://github.com/ubugeeei-prod/ox-content/blob/5c97078779cf099245a78aacca8fa7cc5e993316/crates/ox_content_parser/src/parser.rs),
[renderer entry/reset](https://github.com/ubugeeei-prod/ox-content/blob/5c97078779cf099245a78aacca8fa7cc5e993316/crates/ox_content_renderer/src/html/renderer.rs),
[entity table](https://github.com/ubugeeei-prod/ox-content/blob/5c97078779cf099245a78aacca8fa7cc5e993316/crates/ox_content_parser/src/parser/inline/entity.rs).

## Independent cache and state controls

`cache_run.py` compares every sequence output byte-for-byte with an independent
fresh-process result for that engine and API. Inputs include all twelve real
documents, nonce mutations, duplicate headings, changing reference targets,
entities, Unicode, tables, tasks, empty input, and a larger document.

Sequences contain repeated A, A–B–A, empty–A, large–A, and forward/reversed input
orders. One control overwrites a preallocated input string at the same address;
another keeps identical strings in distinct live allocations. Runtime pointer
assertions verify that these address conditions actually occur. The equal-size
Alpha/Bravo reference-link pair specifically challenges an address-and-length
cache that survives the document it belongs to.

Every control passed. Deliberately replacing outputs with the first cached
result fails the same verifier. `cache-inputs.json`, `cache-references.json.gz`,
and `cache-sequences.json.gz` retain complete source, HTML, and address evidence;
`cache-validation.json` records the counts and negative controls.

The timing sensitivity experiment cycles repeated input, the same text at
different addresses, and changing nonce text. Its modest variation does not
show an abrupt repeated-input shortcut. CPU captures independently show Ox
inside parsing, marker scanning, escaping, and rendering throughout the warmed
loop. Input `black_box` and owned-output `black_box` barriers are inside the
worker loop; the full input and native library implementation are runtime work.

These are falsifiable checks of this pinned native path, not a certification of
every engine version, possible cache, workload, or runtime. CPU instruction/data
caches, allocator caches, and branch prediction remain normal parts of a warmed
native measurement. Cold process startup and request-tail latency require
different experiments. The state controls start fresh processes to establish
output independence, not to publish cold-start timings.

## What Ox does differently

### Compact arena data and coarse ownership

Ox's AST is not a collection of separately allocated heap objects. Nodes borrow
source text, large variants sit behind arena pointers, and compile-time checks
keep `Node` at most 32 bytes and ensure that the node/document types require no
destructor walk. Releasing the arena reclaims its chunks together. Inline vector
capacity is estimated from content length; a plain-text block reserves exactly
one text node.

Ferromark avoids an AST but creates block events, inline marks, resolution
buffers, emission points, and inline events. Its fresh API recreates scratch;
`Renderer` preserves it. Avoiding an AST alone therefore does not establish
lower allocation or processing cost. The lifecycle experiment confirms a useful
reuse gain for Ferromark without eliminating Ox's lead. This does not isolate
the causal contribution of the arena itself or establish a universal memory win.

Sources: Ox [AST invariants](https://github.com/ubugeeei-prod/ox-content/blob/5c97078779cf099245a78aacca8fa7cc5e993316/crates/ox_content_ast/src/lib.rs),
[arena](https://github.com/ubugeeei-prod/ox-content/blob/5c97078779cf099245a78aacca8fa7cc5e993316/crates/ox_content_allocator/src/lib.rs),
[inline allocation sizing](https://github.com/ubugeeei-prod/ox-content/blob/5c97078779cf099245a78aacca8fa7cc5e993316/crates/ox_content_parser/src/parser/inline_helpers.rs);
Ferromark [renderer/session and event pipeline](../../../src/lib.rs),
[inline state](../../../src/inline/mod.rs), and the
[existing heap audit](../2026-09-12-ox-memory/REPORT.md).

### Less intermediate work for ordinary text

Ox walks inline input into nodes using local construct handling and a delimiter
stack. Plain text takes a single borrowed-node path. Eligible soft newlines stay
inside that text node instead of creating separate newline objects. Its forward
marker scans remember the next hit within the current inline parse.

Ferromark's three-phase parser collects marks, resolves precedence, builds
emission points, orders them, and emits events. It already has fast paths and
borrows contiguous paragraph input, but marked prose still traverses these
representations. On the actual twelve documents, pipeline counters and both CPU
captures identify inline parsing/rendering as the principal remaining target.
Visible out-of-line sorting is a small part of the profiles; removing a sort
alone is not supported as an explanation for the entire gap. Inlining means
these sample shares are localization evidence, not precise phase timers.

Sources: Ox [inline loop and soft-newline folding](https://github.com/ubugeeei-prod/ox-content/blob/5c97078779cf099245a78aacca8fa7cc5e993316/crates/ox_content_parser/src/parser/inline.rs),
[marker scan memoization](https://github.com/ubugeeei-prod/ox-content/blob/5c97078779cf099245a78aacca8fa7cc5e993316/crates/ox_content_parser/src/parser/inline/marker_scan.rs);
Ferromark [mark resolution and event emission](../../../src/inline/mod.rs).

### Specialized common-case paths

Ox constructs a simple single-line list-item paragraph directly, avoiding its
recursive block sub-parser. It borrows contiguous code/HTML bodies where source
normalization permits that, and its renderer skips optional TOC work when the
document has no TOC marker. These are concrete reductions in routine work,
not evidence that a specific shortcut explains a known percentage of this run.

Ferromark already adopted several relevant ideas in
[ARCH-EXP-020](../../arch/ARCH-EXP-020-ox-inspired-scanning-and-allocation.md) and
[ARCH-EXP-021](../../arch/ARCH-EXP-021-ox-corpus-line-scans-and-render-ranges.md):
NEON classification, source borrowing, heading reservation, faster line scans,
and bulk rendering of contiguous code/HTML ranges. Repeating those changes or
replacing the streaming architecture would miss the current evidence.

Source: Ox [single-line list path](https://github.com/ubugeeei-prod/ox-content/blob/5c97078779cf099245a78aacca8fa7cc5e993316/crates/ox_content_parser/src/parser/list/item_source.rs).

## Recommended next experiments

1. Reduce intermediate inline work on plain runs and safe soft-newline cases,
   preserving public event behavior, source ranges, hard breaks, and precedence.
   Confirm exact outputs before measuring the complete collection and broader
   specification/extension corpus.
2. Profile mark resolution and event construction more finely before selecting
   an ordering change. Earlier comparator/packed-key attempts were rejected;
   the current profiles do not justify repeating them unchanged.
3. Expose fresh and reusable integration choices consistently in future public
   comparisons. Keep scratch lifetime, output ownership, allocation traffic,
   requested heap, and whole-process RSS distinct.

No optimization is claimed or merged by this study. The next change should have
its own output guards, measured intervention, and memory evidence.
