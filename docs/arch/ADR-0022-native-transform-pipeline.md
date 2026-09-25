# ADR-0022: Add an optional native transform pipeline

- Status: Accepted
- Date: 2026-09-25
- Parent: #394; contract decisions confirmed in #399; foundation tracked by #401

## Context

Callers can mutate Ferromark's public AST, but there is no shared contract for
ordered passes, arena allocations, pass errors, text runs or source ranges after
edits. The selected typography, GitHub-reference and emoji capabilities need
one Rust extension boundary that can also support caller-defined passes. Core
parser and renderer users must not acquire a transform dependency or runtime
work unless they opt in.

The AST is arena-backed, single-threaded and deliberately compact. Its public
`Span` and `Text` types are exhaustive; adding per-node provenance would change
the API and grow every text node. Renderer autolinks already have one
recognizer with boundary and punctuation rules that transform passes can reuse.

## Decision

### Crate boundary

Add the published `ferromark-transforms` crate as an explicit Cargo workspace
member. It depends on the root `ferromark` package; the core package never
depends on transforms. Core-only callers therefore do not compile or run the
pipeline. The crates share the root product version. Release Please updates the
explicit workspace member and lockfile together. When publishing is enabled,
the core crate must be published before the transform crate, whose registry
dependency has an exact version and whose development manifest also has a local
path.

The crate contains `TransformPass`, `TransformPipeline`,
`TransformContext`, text-run and text-range helpers, URL protection helpers,
and an external custom-pass example. Rust and Node built-in configuration is a
later slice; this foundation adds no JavaScript callbacks or built-in passes.

### Pass lifecycle and errors

- A pipeline owns heterogeneous pass implementations and calls them in the
  order supplied by the caller. It does not reorder passes or detect conflicts.
- One pass instance may be reused for multiple documents. Its method borrows
  the document, source and arena for one call, so a safe implementation cannot
  retain those references in its longer-lived pass state. The pipeline stores
  no document or arena reference.
- Passes are synchronous and run on the thread that owns the document. No
  `Send` or `Sync` contract is added.
- The first error stops the pipeline and reports the pass's zero-based index,
  stable name and original error. A pass can have partially mutated the AST
  before failing; the pipeline does not roll back. The caller discards or
  reparses that document.
- A pass validates its own configuration before making its first mutation.
  There is no cross-pass preflight or transaction boundary.
- The pipeline operates on complete `Document` values. It makes no transform
  equivalence guarantee for incremental or provisional fragments.

### Text runs and replacement

`text_runs` exposes maximal adjacent `Text` siblings within one node slice. A
run provides the coalesced decoded value, its individual segment spans and a
helper for replacing a UTF-8 byte range across the run. Runs do not cross
formatting nodes. A pass that needs context across emphasis, links or other
ordinary inline markup traverses those children in source order and carries
that context itself; choosing which nested content is prose remains the pass's
policy. Code, math, raw HTML, MDX payloads, link destinations, titles and other
non-text fields are never visited by these helpers.

Replacing text keeps the source range for the text it replaces. A replacement
across segments gets the bounding range of source nodes it overlaps; unchanged
prefixes and suffixes keep the bounding spans of their contributing nodes. A
zero-width insertion receives `Span::empty()`. When one decoded text node is
split around an edit, each resulting part retains that node's complete source
span. Character-accurate source maps are not provided.

### URL protection

URL recognition is explicit work. The transform crate never scans URLs
automatically: a URL-sensitive pass calls the context helper when it needs
protected ranges. The default helper uses the renderer's existing
`http://`/`https://` recognition, including its word-boundary and trailing
punctuation rules. A pass can supply custom prefixes matching its renderer
configuration. Neither parsing nor rendering calls the new range helper, so
ordinary processing has no additional URL scan.

### Provenance

No marker or side table distinguishes generated content. Replacement text keeps
the source span of the replaced content. Pure insertion uses `Span::empty()`.
Consumers must treat an empty span as “no source range available” and cannot
infer that every such node was generated. Revisit this only when a concrete
consumer needs that distinction.

### Derived data and terminology

Passes run before renderer metadata, heading IDs and outlines are computed, so
those consumers observe the transformed document. Rust AST operations are
called **passes**. The existing Node `transform()` name remains reserved for
Markdown-to-HTML conversion; this crate does not change it.

## Consequences

Consumers can write custom Rust passes without copying the AST into a second
owned representation. Ordered failures and source-range behavior are
inspectable and testable. Text-run helpers cover fragmented adjacent nodes
while leaving formatting and protected-content traversal under each pass's
control.

The new package adds a second published Cargo archive to the product. The
release workflow must publish `ferromark` before `ferromark-transforms`, and
crates.io Trusted Publishing must be configured for the new package before its
first release. The publish workflow remains a separate gated change until that
external publish action is explicitly approved.

## Validation

- Pipeline tests cover empty output-equivalence, caller order, first-error
  stopping and identity, source-span preservation, insertion spans, invalid
  UTF-8 offsets, URL boundaries, pipeline reuse and arena reset.
- An isolated packaged consumer compiles against both extracted crate archives
  and runs parse → custom pass → render.
- `cargo package --locked` and `cargo check --all-targets` run for each archive.
- The Release Please rehearsal checks one coordinated version entry, concrete
  workspace member versions, exact internal dependency versions and lockfile
  entries.
- No parser or renderer call site invokes URL range detection. Normal-path
  benchmark evidence remains a release gate if URL scanning is ever made
  automatic.
