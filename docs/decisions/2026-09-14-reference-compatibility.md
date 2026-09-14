# Closing the remaining reference-compatibility gaps

The user requested the remaining correctness fixes and explicitly prioritized
correctness over speed. This follows the first correction batch and its
seven cmark/cmark-gfm oracle discrepancies. These are intentional semantic
changes, not output-preserving optimization experiments.

## Decisions

- Resolve GFM strikethrough with the correct inline boundaries, whitespace
  flanking, and run-pairing rules. A delimiter inside a link destination,
  title, code span, or HTML tag must not close an outer strike. Existing
  explicit subscript configuration retains its single-tilde precedence.
- Adopt cmark-compatible UTF-8 percent serialization for non-ASCII URL
  attribute bytes. This is a consistent URI spelling policy, not a claim that
  the previous Unicode spelling was universally invalid. Preserve existing
  percent escapes, reserved delimiters, valid IPv6 syntax, and visible text;
  do not perform IDNA rewriting.
- Treat one leading UTF-8 BOM as an input encoding marker, matching cmark.
  Preserve embedded BOM characters and original byte offsets, including
  inputs that also require NUL replacement. This is an explicit input policy
  beyond the original finite CommonMark fixtures.
- Add explicit CommonMark and formal-extension GFM option constructors.
  They disable renderer-added heading IDs, TOC, callouts, bare-URL linking,
  external-link attributes, and fence metadata cleanup. The GFM pair enables
  the specified extensions and renderer tagfilter without adding footnotes.
  Existing convenience presets retain their feature defaults. CommonMark
  0.31.2 remains the core grammar; do not reintroduce older comment rules to
  match two historical GFM website examples.

## Review and acceptance

Keep the specification fixtures and historical results immutable. The two renderer
snapshots for Unicode autolinks and Unicode TOC destinations intentionally change
only URL attribute spelling to UTF-8 percent encoding; visible text and heading
IDs remain unchanged. Their before/after output was reviewed. The 106
original oracle inputs become continuous Rust regressions against their pinned
reference output, alongside the separate specification suites. An additional
256 deterministic tilde/inline combinations probe the failure families beyond
the five originally discovered examples. Preserve initial and final raw
comparisons and the exact reference/build metadata.

Review normal, hooked, reused, and incremental renderer paths for the new
options. Check original spans for BOM/NUL handling and scalar/SIMD URL escaping
agreement. Run workspace tests, formatting, Clippy, benchmark compilation, and
the frozen-spec and cmark oracle gates. Any performance measurement must use
declared profiles and verify equivalent output before timing.

These changes do not add an MDX compiler or runtime. The existing MDX syntax
capture and static rendering remain explicitly bounded features.

## Oracle review

The expanded corpus exposed a harness mismatch: cmark-gfm also needs `--unsafe`
to pass ordinary raw HTML while the enabled GFM tagfilter handles its prohibited
tags. Preserve both the original and aligned baseline records. The original
106-case corpus has no output changes from this invocation correction.

Two expanded cases (`binding-1-31-link`, `binding-2-31-link`) expose nested anchors
in the pinned cmark-gfm output. Follow the [GFM links rule](https://github.github.io/gfm/#links),
which excludes links inside links at any depth, and retain the innermost link.
Keep those two oracle differences visible; the Rust regression explicitly
asserts the spec-correct HTML instead. Thus acceptance is 106 reference matches,
254/256 expanded reference matches, and two reviewed spec assertions. The
generic strict oracle must still reject these two mismatches.
