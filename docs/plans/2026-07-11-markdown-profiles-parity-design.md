# Markdown profiles and feature-parity benchmark design

Status: approved for implementation
Date: 2026-07-11

## Context

Many Markdown documents use a small, predictable syntax surface. Ferromark can
avoid work for disabled features already, but callers must currently understand
and set every individual `Options` field. At the same time, the cross-parser
benchmark enables a hand-picked feature set without making the semantic overlap
explicit enough.

This design introduces a small hierarchy of opt-in user profiles and a separate
set of benchmark-only parity configurations. The two concepts intentionally do
not share names or constructors:

- profiles describe useful Ferromark behavior for users;
- parity configurations describe the exact common feature intersection between
  competing parsers.

`Options::default()` remains unchanged for backward compatibility.

## Alternatives considered

### One `Fast` profile

Rejected. The name describes an unstable performance outcome rather than a
syntax contract. It would not tell callers which Markdown constructs become
literal text or stop producing enhanced output.

### Automatic profile detection

Rejected as a public profile mechanism. Transparent internal fast paths remain
desirable, but an unconditional feature pre-scan adds work. Earlier Ferromark
candidate pre-scans cost roughly 2-4% on simple and mixed documents.

### Compile-time Cargo features

Rejected for these profiles. Cargo features would fragment builds and make
runtime selection and like-for-like benchmarks harder. The existing `Options`
path already lets disabled phases be skipped without changing the crate build.

### Three monotone runtime profiles

Accepted. `Essentials`, `Extended`, and `Full` form an understandable superset
relationship while leaving fine-grained `Options` overrides available.

## Public API

Add one public enum:

```rust
pub enum Profile {
    Essentials,
    Extended,
    Full,
}
```

Profiles convert into `Options`:

```rust
let options = Options::from(Profile::Essentials);
```

Callers can override individual fields afterward:

```rust
let options = Options {
    heading_ids: true,
    render_policy: RenderPolicy::Trusted,
    ..Options::from(Profile::Essentials)
};
```

There are no additional `fast`, `minimal`, `gfm`, or profile-specific helper
constructors. `RenderPolicy` stays orthogonal and every profile starts with the
secure `Untrusted` policy.

## Profile contracts

Every profile includes ordinary Markdown paragraphs, headings, emphasis,
strong emphasis, inline and block code, links and images, lists, blockquotes,
thematic breaks, and hard/soft breaks.

| Option | Essentials | Extended | Full |
| --- | :---: | :---: | :---: |
| Raw HTML parsing | off | on | on |
| Reference links | off | on | on |
| Tables | on | on | on |
| Strikethrough | on | on | on |
| Task lists | on | on | on |
| GFM disallowed raw HTML | off | on | on |
| Heading IDs | off | on | on |
| Callouts | off | on | on |
| Autolink literals | off | off | on |
| Footnotes | off | off | on |
| Front matter | off | off | on |
| Math | off | off | on |
| Highlight | off | off | on |
| Subscript | off | off | on |
| Superscript | off | off | on |

`Extended` deliberately matches the current `Options::default()` feature mix.
That relationship is tested, but `Options::default()` remains its own API and
is not implemented by calling the profile conversion.

`Essentials` and `Extended` are curated, stable contracts. New syntax features
remain off there until deliberately assigned. `Full` means all supported
features and therefore expands when new options land; those changes must be
called out in the changelog.

## Benchmark design

### Ferromark profile-cost matrix

Measure all three profiles on one Essentials-compatible corpus:

- `profiles/essentials`
- `profiles/extended`
- `profiles/full`

Because the shared corpus contains no disabled syntax, this matrix measures the
overhead of enabling unused feature paths. Each profile may additionally have a
representative capability corpus, but those results are reported independently
and are not ranked as if they performed identical work.

No speed claim is attached to the profile names. An Essentials performance
claim requires three repeated runs with a stable improvement.

### Cross-parser parity matrix

The comparison harness gets three explicit configurations:

1. `commonmark`
   - CommonMark syntax only
   - raw HTML preserved for comparable rendering work
   - no parser-specific extensions
2. `gfm-overlap`
   - CommonMark plus tables, strikethrough, and task lists
   - the established shared GFM subset
3. `extended-overlap`
   - GFM overlap plus footnotes, math, superscript, and callouts
   - every feature remains conditional on semantic fixture validation across all
     included parsers

The parity functions are benchmark infrastructure, not public `Profile`
conversions. Each function lists every enabled option explicitly so dependency
updates cannot silently alter the compared surface.

Ferromark's secure default remains a separate product benchmark. It must not be
called a parity comparison because pulldown-cmark does not provide the same URL
and raw-HTML trust boundary.

### Semantic guardrails

Every parity configuration has a feature-specific corpus and expected semantic
markers. Byte-identical HTML is not required where libraries use different
valid wrappers or attributes, but every parser must demonstrate the same
feature interpretation before a lane is labeled parity.

The harness also continues to:

- reuse output buffers where APIs allow;
- disable Ferromark-only output such as heading IDs in parity lanes;
- use identical input bytes and throughput accounting;
- pin dependency versions;
- report the compiler, commit, options, and corpus with published results.

## Testing and compatibility

- Unit tests assert the exact option mapping for each profile.
- A regression test asserts that `Extended` matches today's default feature
  mix without changing `Options::default()`.
- Behavior tests prove representative syntax is enabled and disabled at each
  boundary.
- `Full` has an explicit completeness test so new option fields require a
  conscious profile decision.
- Render-policy tests prove profiles never opt into trusted HTML.
- Benchmark semantic tests validate each feature before timing it.
- The full CommonMark suite and all-target Clippy remain merge gates.

Profile construction is infallible and introduces no new error type.

## Documentation

The README and homepage guide will include:

- the three-profile feature table;
- a short selection guide;
- an override example;
- the separation between syntax profiles and `RenderPolicy`;
- the three parity benchmark configurations and their exact scopes;
- the secure-default benchmark as a separate product measurement.

The performance roadmap links back to this design and tracks implementation and
measurement follow-up as checkboxes.

## Acceptance criteria

- `Options::default()` behavior is unchanged.
- The three profiles are monotone supersets.
- All profile mappings and representative behavior are tested.
- Profile-cost benchmarks compile and can be filtered independently.
- Cross-parser parity configurations are explicit and semantically guarded.
- Documentation never presents secure-default and trusted parity numbers as the
  same workload.
- The full test, formatting, Clippy, and benchmark smoke gates pass.
