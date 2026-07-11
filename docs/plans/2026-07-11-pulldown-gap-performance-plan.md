# Performance plan: widen the `pulldown-cmark` gap

Status: ready to execute  
Baseline commit: `d5f32d8` (`ferromark` 0.3.0)  
Date: 2026-07-11

## Verdict

Yes, there is still credible headroom.

For the secure default path, a further **2-5% throughput gain** on the 50 KB
fixture looks achievable without changing the public architecture. A **5-8%
gain** is plausible if one of the structural experiments around paragraph or
inline-event handling succeeds. More than that probably requires a deeper
parser/rendering integration and should not be planned as a short sequence of
micro-optimizations.

The most important conclusion is that the remaining gap will not come from one
obvious byte-scanner tweak. Previous work has already removed many such costs.
The current opportunities are concentrated in:

1. fixed per-document allocation and setup cost,
2. paragraph and inline intermediate buffers/events,
3. repeated scans in the secure URL/raw-HTML rendering path, and
4. measurement quality at differences below 2%.

## Current evidence

### Published baseline

The README currently reports the following non-PGO results on Apple Silicon:

| Fixture | ferromark | pulldown-cmark | ferromark lead |
| --- | ---: | ---: | ---: |
| CommonMark 5 KB | 259.6 MiB/s | 254.5 MiB/s | 2.0% |
| CommonMark 50 KB | 280.5 MiB/s | 275.2 MiB/s | 1.9% |

### Local confirmation after updating `main`

An isolated two-parser Criterion run was used because the four-parser harness
currently fails to link the locally configured md4c checkout. Both parsers used
reusable output buffers, GFM tables/strikethrough/task lists, 80 samples, a
5-second measurement window, and a 3-second warm-up.

Environment: Apple Silicon, `rustc 1.95.0-nightly` (2026-01-29), LLVM 22.1.0.
These numbers confirm direction; they should not replace the published numbers
because the compiler and machine state are not pinned to the publication run.

| Fixture | ferromark | pulldown-cmark | relative result |
| --- | ---: | ---: | ---: |
| CommonMark 5 KB | 245.91 MiB/s | 246.64 MiB/s | ferromark -0.3% |
| CommonMark 20 KB | 278.06 MiB/s | 265.92 MiB/s | ferromark +4.6% |
| CommonMark 50 KB | 267.64 MiB/s | 262.91 MiB/s | ferromark +1.8% |

The 5 KB confidence intervals overlap. Treat that row as parity, not as a
confirmed regression. The size curve nevertheless suggests fixed setup and
allocation cost: ferromark is strongest at 20 KB, but its advantage is consumed
by fixed cost at 5 KB.

### Important methodology issue: security semantics differ

The current cross-parser options disable ferromark heading IDs and callouts, but
leave `RenderPolicy::Untrusted` enabled. The fixture contains raw HTML and URLs.
Consequently:

- ferromark escapes raw HTML and validates URL schemes;
- `pulldown-cmark::html::push_html` preserves raw HTML;
- the parsers are not doing exactly the same rendering work.

A closer trusted-input lane produced:

| CommonMark 50 KB lane | Throughput |
| --- | ---: |
| ferromark, trusted rendering | 272.68 MiB/s |
| pulldown-cmark | 257.85 MiB/s |

That is a 5.8% ferromark lead in that run. The secure default was about 1.9%
slower than the trusted ferromark lane in adjacent runs. This does **not** mean
the README should switch to trusted mode. It means we need two explicitly named
lanes:

- **secure default**: the product-relevant default path;
- **closest rendering semantics**: trusted raw HTML for a more direct parser
  comparison.

Output equivalence still needs to be classified because the libraries can use
different, valid HTML spellings.

### Profile snapshot

A 10-second macOS `sample` profile was collected from a 40-second 50 KB run with
debug symbols and fat LTO. The counts below are directional exclusive samples,
not cycle-accurate attribution; inlining makes large functions look broader.

| ferromark area | Samples | Approx. share |
| --- | ---: | ---: |
| `InlineParser::parse_with_options` | 1,787 | 21.8% |
| `RenderContext::render_block_event` | 1,015 | 12.4% |
| `escape_text_into` | 695 | 8.5% |
| `render_inline_content` | 490 | 6.0% |
| `memmove` | 387 | 4.7% |
| `BlockParser::match_containers` | 297 | 3.6% |
| `BlockParser::parse_paragraph_line` | 266 | 3.2% |
| `BlockParser::split_table_cells` | 149 | 1.8% |

The profile also shows smaller but actionable costs in URL escaping/safety
checks, label normalization, inline emit-point sorting, paragraph closing, and
allocator functions.

The `pulldown-cmark` profile is organized differently: block first-pass work,
iterator event production, scalar HTML escaping, and inline pass 1 dominate.
There is no single competitor function we can copy to gain 5%; ferromark's
opportunity is reducing its own intermediate work.

## Execution rules

Use these rules for every checkbox below.

- [ ] Run on a pinned Rust toolchain and record `rustc -Vv`, commit, machine,
      power mode, and benchmark command.
- [ ] Compare the candidate against an untouched baseline worktree using the
      same command, alternating baseline/candidate order when practical.
- [ ] Use at least 80 samples, 5 seconds measurement, and 3 seconds warm-up for
      screening; repeat promising results three times.
- [ ] Treat changes below 1% as noise unless all repeated confidence intervals
      and absolute medians agree.
- [ ] Keep a change only if its target lane improves and no primary fixture
      regresses by more than 1% reproducibly.
- [ ] Run `cargo test --all-targets` and the 652-example CommonMark suite before
      keeping a parser or renderer change.
- [ ] Check output behavior on raw HTML, unsafe URL schemes, tables, reference
      links, nested emphasis, and fenced code.
- [ ] Record rejected experiments here or in a successor experiment note so
      they are not rediscovered later.

Primary guardrails:

- secure-default CommonMark 5 KB, 20 KB, and 50 KB;
- trusted-semantics CommonMark 5 KB and 50 KB;
- reference-heavy `refs` and `refs_escaped`;
- table-heavy 5 KB;
- simple text and code-heavy fixtures.

## Work plan

### P0 — Make sub-2% claims reproducible

#### 0.1 Add a first-class two-parser benchmark lane

Why: the current comparison build requires md4c even when filtering to
ferromark and pulldown-cmark. On this machine it failed at link time with an
undefined `_md_html` symbol. That makes the most important comparison depend on
an unrelated C checkout.

- [x] Add a ferromark-versus-pulldown harness that does not build md4c or comrak.
- [x] Reuse output buffers with equivalent starting capacity in both lanes.
- [x] Keep dependency versions locked, especially `pulldown-cmark`.
- [x] Add named CommonMark, GFM-overlap, and extended-overlap trusted-parity
      groups while keeping secure-default measurements separate.
- [ ] Print or store input size, output size, option set, compiler version, and
      git commit with the result artifact.
- [x] Keep the four-parser harness for published broad comparisons.
- [ ] Separately fix or document the md4c `_md_html` link failure and validate
      the checkout SHA against the published `65c6c9d` pin.

Acceptance: one command can measure the two leading parsers without md4c and
reproduces the current relative ordering within 1 percentage point.

#### 0.2 Split benchmark claims by semantics

- [x] Name the current secure lane explicitly; do not call it identical work.
- [x] Add trusted raw-HTML lanes for closer output semantics.
- [x] Compare representative outputs and document intentional differences.
- [x] Add semantic assertions for links, raw HTML, tables, task
      lists, and reference links rather than relying only on output length.
- [ ] Publish replacement parity numbers only after the new lanes have three
      stable runs on the publication machine.

Acceptance: every published percentage states which security/rendering lane it
measures.

#### 0.3 Pin the performance toolchain

- [ ] Add a documented benchmark toolchain version; do not silently use the
      developer's current nightly.
- [ ] Run stable/MSRV and the benchmark toolchain as separate lanes if both are
      useful.
- [ ] Report PGO and non-PGO results separately.
- [ ] Save Criterion estimates or machine-readable summaries as artifacts.

Acceptance: a later run can explain compiler-caused drift rather than treating
it as a parser regression.

### P1 — Recover low-risk headroom first

#### 1.1 A/B the 0.3 renderer integration

Why: 0.3 added a generic `RenderContext`, renderer state, and fenced-code
branches. The default path is statically specialized with
`DisabledFencedCodeRenderer`, so it may compile away completely—but this has not
yet been demonstrated against the 0.2 baseline under one protocol.

- [ ] Compare `e59fa30` (0.2 release merge) with `d5f32d8` using the new
      two-parser harness and identical toolchain.
- [ ] Run default 5/20/50 KB plus code-heavy default rendering.
- [ ] Inspect generated code/profile only if a repeatable regression is at least
      0.7%.
- [ ] If present, isolate generic context size, `Option` checks, and fenced-code
      state fields one at a time.
- [ ] Preserve zero-cost default rendering as an explicit benchmark guardrail.

Expected gain: 0-2%.  
Stop condition: no reproducible regression; do not refactor generic code based
on appearance alone.

#### 1.2 Audit eager `InlineParser` allocations

Why: `InlineParser::new()` creates many `Vec::with_capacity` buffers, including
buffers for disabled or uncommon extensions. This is a fixed cost per document
and is consistent with parity at 5 KB but a lead at 20 KB.

- [ ] Add an allocation-count benchmark for 5 KB, 20 KB, and 50 KB.
- [ ] Record allocation count and allocated bytes for parser construction,
      block parse, inline parse, and render separately where possible.
- [ ] Classify every preallocated inline buffer as common-default, conditional,
      or rare.
- [ ] Test lazy `Vec::new()` for rare/disabled paths in small independent groups.
- [ ] Test lower initial capacities for buffers that almost never grow.
- [ ] Test `SmallVec` only for one measured buffer at a time; watch stack size
      and initialization cost.
- [ ] Keep previous successful preallocation for common event/mark buffers unless
      allocation evidence proves a better threshold.

Expected gain: 1-4% at 5 KB, 0-1.5% at 50 KB.  
Acceptance: fewer allocations and a reproducible 5 KB win without losing more
than 1% at 20/50 KB.

#### 1.3 Reduce secure URL rendering passes

Why: the default fixture pays URL UTF-8 conversion, entity detection/decoding,
scheme validation, URL encoding, and HTML escaping. The profile shows URL safety
and escaping as visible costs, and the trusted/default delta bounds the total
security-path opportunity at roughly 1.9% on this fixture.

- [ ] Add separate safe-ASCII, entity-containing, Unicode, relative, and unsafe
      scheme URL benchmarks.
- [ ] Measure `is_safe_url`, destination escaping, and the combined render path.
- [ ] Test an ASCII byte fast path for scheme classification with a Unicode
      fallback; preserve all security tests.
- [ ] Test sharing one scan result between safety classification and destination
      escaping instead of independently searching the same bytes.
- [ ] Test entity decoding only when `&` is present and keep the current safe
      fast path intact.
- [ ] Fuzz or property-test the optimized classifier against the current
      implementation before replacement.

Expected gain: 0.5-1.5% on secure-default 50 KB; larger on link-heavy input.  
Acceptance: byte-for-byte identical output for the security corpus and no unsafe
scheme bypass.

#### 1.4 Avoid paragraph reassembly for contiguous source ranges

Why: block parsing emits per-line `Text` plus `SoftBreak` events. Rendering then
copies the paragraph into `ParagraphState` before inline parsing. The profile
shows paragraph parsing, paragraph close, `ParagraphState::add_text`, and
`memmove`; normal fixture paragraphs are usually contiguous in the source.

- [ ] Measure the percentage of paragraphs/headings that are source-contiguous
      and require no tab/container normalization.
- [ ] Add a block event or metadata form that can represent one contiguous
      paragraph range.
- [ ] Parse/render that borrowed slice directly, preserving embedded newlines as
      soft breaks.
- [ ] Retain the current buffer fallback for container indentation, tab
      expansion, tables, and other transformed content.
- [ ] Extend the same mechanism to headings only after paragraph results are
      stable.

Expected gain: 1-3% on 20/50 KB, plus fewer copies/allocations.  
Acceptance: at least 1% repeatable 50 KB improvement, neutral reference-heavy
and table-heavy results, full CommonMark correctness.

### P2 — Structural inline work

#### 2.1 Produce and reuse one inline feature summary

Why: inline parsing scans text/marks in several stages to answer related
questions: special characters, `<`, bracket presence, emphasis/tilde markers,
and extension candidates.

- [ ] Instrument bytes visited per paragraph by the inline stages.
- [ ] Have mark collection return a compact summary bitset/counters.
- [ ] Use that summary to gate HTML/autolink, bracket, emphasis, tilde, and
      extension resolution without new pre-scans.
- [ ] Remove only scans proven redundant by the instrumentation.
- [ ] Benchmark simple, mixed, HTML-heavy, refs-heavy, and delimiter-heavy input.

Expected gain: 1-3% on mixed 50 KB.  
Stop condition: discard if the summary adds work to the plain-text fast path or
repeats the failed "extra pre-check" pattern.

#### 2.2 Add a render sink to bypass `InlineEvent` materialization

Why: inline resolution builds/sorts emit points, materializes `InlineEvent`s,
then immediately dispatches them into `HtmlWriter`. Buffers are reused and prior
allocation work already helped, so the opportunity is eliminating a pass rather
than tuning capacity again.

- [ ] Define the smallest internal sink abstraction that can receive resolved
      inline events.
- [ ] Keep the public event path if it is needed by tests or future APIs.
- [ ] Add an HTML sink that writes directly after all precedence/range decisions
      are complete.
- [ ] Preserve image-alt collection, raw HTML policy, footnote numbering, link
      safety, and suppression ordering.
- [ ] Compare instruction/profile share, event count, allocation count, and
      throughput.

Expected gain: 2-5%, medium implementation risk.  
Acceptance: at least 2% on 50 KB or a substantial allocation reduction with no
5 KB regression. Otherwise revert.

#### 2.3 Specialize only proven option profiles

- [ ] Measure branch/profile cost for default GFM, CommonMark-only, and all
      extensions.
- [ ] Consider an internal feature mask or specialized entry point only if the
      compiler does not already fold constant options.
- [ ] Keep one implementation of parsing rules; specialization must select
      phases, not fork grammar logic.

Expected gain: unknown.  
Stop condition: no specialization without generated-code or profile evidence.

### P3 — Block-path experiments after P1/P2

#### 3.1 Reduce block-event volume without a second block pass

Previous full streaming/list-buffered work regressed by 27-28% because it needed
two block passes. Do not repeat that design.

- [ ] Count block events by type and bytes copied per fixture.
- [ ] Prototype aggregated paragraph/table-row events within the existing single
      parse pass.
- [ ] Explore rendering reference-insensitive blocks early only if forward link
      references remain correct without a pre-scan.
- [ ] Keep unresolved/ref-sensitive paragraphs buffered when definitions can
      occur later.
- [ ] Reject any design that adds an unconditional pre-scan; prior candidate
      pre-scans cost about 2-4% on simple/mixed documents.

Expected gain: 2-6% if event aggregation works; high uncertainty.

#### 3.2 Re-profile container and table paths

- [ ] Add list-heavy, blockquote-heavy, and table-heavy profile runs.
- [ ] Investigate `match_containers` only in those focused runs; it is about 3.6%
      of the mixed 50 KB sample, so its theoretical global ceiling is limited.
- [ ] Investigate table cell splitting/copying and delimiter-row detection as one
      workstream.
- [ ] Prefer algorithmic event/copy reduction over another cursor-vs-slice
      rewrite; the earlier slice-indent experiment did not improve throughput.

Expected gain: fixture-specific 2-5%; likely below 1% on the mixed 50 KB corpus.

### P4 — Compiler-level tuning, kept separate from source wins

- [ ] Establish non-PGO source baseline first.
- [ ] Build a representative PGO training corpus rather than only the 50 KB
      fixture.
- [ ] Report PGO gains separately for ferromark and pulldown-cmark.
- [ ] Inspect code size and instruction-cache effects of fat LTO and
      `codegen-units = 1`.
- [ ] Test `target-cpu=native` only as a local/deployment lane, never as the
      portable published baseline.

## Do not repeat without new profile evidence

The repository already records the following neutral or negative experiments:

- full two-pass/list-buffered block streaming: 27-28% slower;
- candidate/full pre-scans before parsing: roughly 2-4% overhead on simple/mixed;
- simple inline streaming fast path: no meaningful gain;
- slice-based indentation scanning: no gain/slight regression;
- extra `memchr` escape pre-scan: no gain;
- index-based autolink filtering: no gain;
- skip-code-span pre-check: no gain;
- precomputed or cached nested reference candidates: regressions/unstable;
- delimiter-stack reference-resolution rewrites: did not beat the kept path;
- repeated label-normalization and local binary-search micro-tweaks: neutral or
  regressive;
- earlier NEON/SIMD escape and label variants: no stable improvement.

Also preserve the successful work already in place: reusable buffers, event
preallocation, no-mark/bracket fast paths, unstable emit-point sorting, deferred
link-ref materialization, contiguous ref-definition parsing, URL/title fast
paths, and skipping inline-link resolution without a `](` candidate.

## Recommended execution order

- [ ] 0.1 Two-parser harness
- [ ] 0.2 Semantic lanes and output classification
- [ ] 0.3 Toolchain/result pinning
- [ ] 1.1 0.2-versus-0.3 default-path A/B
- [ ] 1.2 Allocation census and lazy rare buffers
- [ ] 1.3 Secure URL pass reduction
- [ ] 1.4 Contiguous paragraph rendering
- [ ] Re-profile and update the expected-return ranking
- [ ] 2.1 Inline feature-summary experiment
- [ ] 2.2 Direct inline render sink
- [ ] 3.1 Aggregated block-event experiment
- [ ] 3.2 Focused container/table work
- [ ] PGO/compiler lane only after source-level results are stable

## Target outcome

The next milestone should not be framed as one universal percentage. Use this
scorecard:

- secure default 5 KB: consistently ahead of pulldown-cmark, not parity;
- secure default 50 KB: **at least 4% ahead** over three publication runs;
- trusted-semantics 50 KB: **at least 6% ahead** over three publication runs;
- no primary fixture regression above 1%;
- 652/652 CommonMark examples and the full test suite remain green;
- every kept/rejected experiment has reproducible evidence.

Reaching 4% on the secure 50 KB lane appears realistic from the combined
allocation, security-pass, and paragraph-copy opportunities. Treat 6-8% as a
stretch target that requires at least one structural inline/block change to
land successfully.

## Profile and feature-parity extension

The approved design is documented in
[`2026-07-11-markdown-profiles-parity-design.md`](./2026-07-11-markdown-profiles-parity-design.md).
It adds a bounded product/DX workstream without changing `Options::default()`.

### Public profiles

- [x] Add the monotone `Profile::{Essentials, Extended, Full}` API.
- [x] Keep security orthogonal through `RenderPolicy`.
- [x] Make Essentials cover everyday Markdown plus tables, strikethrough, and
      task lists.
- [x] Make Extended add reference links, raw HTML parsing, heading IDs, and
      callouts, matching the current default feature mix.
- [x] Make Full add all remaining supported syntax features.
- [x] Add exact mapping, behavior-boundary, default-compatibility, and Full
      completeness tests.
- [x] Benchmark all profiles on the same Essentials-compatible corpus before
      making a speed claim.

### Cross-parser parity configurations

- [x] Add an explicit CommonMark-only parity lane.
- [x] Add a GFM-overlap lane with tables, strikethrough, and task lists.
- [x] Add an extended-overlap lane with only semantically verified shared
      features.
- [x] Keep Ferromark secure-default results separate from trusted rendering
      parity.
- [x] Document every parser flag and semantic exception per lane.
- [x] Validate representative output semantics before timing each lane.
