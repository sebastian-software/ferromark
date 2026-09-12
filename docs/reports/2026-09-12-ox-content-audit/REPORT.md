# Ox Content benchmark and implementation audit

Date: September 12, 2026. This is a diagnostic investigation, not a replacement
for the published native comparison suite or a parser optimization proposal.

The generated [measurement tables](RESULTS.md) contain the final timings,
run-to-run stability, isolated scanner results and allocation counts.

## Scope and source revisions

- Ferromark main: `66892cfa2639c3ecb3e049583db746c75354fbda`, package 0.9.0.
- Historical Ferromark: crates.io 0.7.0, checksum
  `f26bc33597a3dede0e3c54e0a8ca200c7e94a6ecc69b4d4092896937ee1fa5dd`.
- Ox Content current main: `026d1859d1c35e5fb1ea65e7e855b428a918b9bb`.
  Its `crates/` tree is byte-identical to the previously tested v3.2.0 revision
  `5c97078779cf099245a78aacca8fa7cc5e993316`.
- Published table update: `2df1ef6d0dadb9f8fc1226c6f9de94ebc5cd9d52`,
  August 30. The historical runner differs from today's runner only in two
  documentation paths. This establishes the call sites and input, not the exact
  engine/build artifact used by the historical CI run.

Primary sources:
[deployed performance page](https://ubugeeei-prod.github.io/ox-content/performance/),
[historical runner](https://github.com/ubugeeei-prod/ox-content/blob/2df1ef6d0dadb9f8fc1226c6f9de94ebc5cd9d52/benchmarks/native-competitors/src/main.rs),
[historical lockfile](https://github.com/ubugeeei-prod/ox-content/blob/2df1ef6d0dadb9f8fc1226c6f9de94ebc5cd9d52/benchmarks/native-competitors/Cargo.lock),
[current conformance runner](https://github.com/ubugeeei-prod/ox-content/blob/026d1859d1c35e5fb1ea65e7e855b428a918b9bb/tools/benchmarks/native-competitors/src/conformance.rs).

## What the public comparison measures

The page still displays the August 30 snapshot: native Ox 7,445 versus Ferromark
4,389 operations/second on its large input (1.70x), and 329 versus 207 on its huge
input (1.59x). The reported runner is an AMD EPYC Linux machine, not this audit's
Apple machine. Those historical absolute numbers cannot be reproduced here.

Ferromark is measured natively, through `ferromark::to_html`, with **0.7.0** in
the lockfile. There is no Node wrapper penalty in that row. Ox's native row does
fresh arena allocation, parsing, creation of an HTML renderer, owned HTML
rendering and destruction. It is not a cached-AST or render-only comparison.
Ferromark correctly does not appear in the parse-only table.

The corpus is one 497-byte ASCII sample repeated 1, 10, 100 and 2,150 times,
with two newlines between copies. It contains headings, emphasis, lists, a code
fence, a block quote, a pipe table, a link and an image. It is not a diverse
48 KB/1 MB corpus; repetitions preserve essentially the same syntax mix and
repeat heading names. No MDX, JavaScript compilation, component rendering,
networking or site generation is timed in these native rows.

The option sets differ materially. Ferromark defaults enable tables, heading IDs,
strikethrough, task lists and callouts, and apply the untrusted rendering policy.
Ox parser defaults disable tables; its renderer adds heading IDs, URL autolinking
and external-link attributes. The sample's pipe table becomes ordinary paragraph
text in Ox and a real table in Ferromark. The upstream runner even asserts the
absence of `<table>` in Ox output. Thus the public ranking does not isolate
implementation speed for the same requested Markdown dialect.

The runner uses five warmup calls and 100/50/20/5 timed calls for the four sizes,
with seven repeated measurements and selection by median throughput. Engine
order is fixed. At the displayed rates, a large-input Ox measurement lasts only
about 2.7 ms and a huge-input one about 15 ms. This is much shorter than our
alternating measurement windows. It is not proof of incorrect results, but it
weakens the page's suggestion that relative rankings are stable across hosts.
The throughput calculation divides by 1024 squared, so the displayed MB/s is
actually MiB/s.

A reproducibility issue also exists in current main: after moving the benchmark
under `tools/`, its manifest still points to `../../crates/...`, resolving to
nonexistent `tools/crates`. `cargo metadata --offline` reproduces the failure.
The JS harness warns and omits native rows when the build fails. This does not
invalidate the earlier measurements; it means a clean current checkout cannot
refresh these rows with the advertised command without correcting the paths.

## CommonMark score audit

We applied **Ox's own normalizer** to both actual and expected HTML, using all
652 examples from our checked-in CommonMark fixture. The normalizer intentionally
removes heading IDs and link target/rel attributes, collapses whitespace outside
preformatted content, and canonicalizes entity and URL escaping. These scores
must not be confused with our stricter native workload admission rules.

With upstream's resolved `html-escape` 0.2.15, both Ferromark versions reproduce
579/652 (88.8%) under defaults. Changing to CommonMark syntax plus trusted HTML
raises both to 651/652. The remaining example is 26 (`&#0;`): that dependency
version leaves it encoded instead of producing the replacement character.

With our checked-in `html-escape` 0.2.14, both versions reach **652/652** under
CommonMark plus trusted HTML, as does Ox. Their defaults produce 580/652 because
HTML escaping and other intentional defaults differ from the specification.
The published 88.8% is therefore not a fair statement of Ferromark's available
CommonMark support. Even 0.7 exposes the necessary configuration. The page's
promise to use each engine's most spec-faithful configuration does not match its
Ferromark call site.

The 0.2.15 behavior is a separate dependency-upgrade finding. It is outside our
current lockfile and was not patched as part of this investigation. Initial
exploratory results using that dependency are kept separately in `pilot/`.

## Controlled measurement protocol

All final timing modes run in one standalone native Rust executable. They use
Rust 1.97.1, opt-level 3, fat LTO, one codegen unit, panic abort, the repository's Apple M1/NEON target flags and the system allocator.
The original audit described the target as generic; subsequent build review
found that Cargo inherited `.cargo/config.toml` from the repository working
directory. All audit competitors inherited the same flags. The follow-up
optimization experiments build outside the repository to explicitly use a
generic CPU target. The Ferromark direct dependency versions match
our lockfile, and the complete diagnostic Cargo.lock is archived. This removes
dependency resolution as a variable between old and current Ferromark.

Host: Apple M1 Pro, macOS 26.6.2, AC power. No concurrent builds or other benchmark
processes were started by this audit during final timing runs. This is a shared
developer machine, not an isolated Linux runner.

For each case/mode: 150 ms warmup, eleven 50 ms measurement windows, sixteen
fresh renders per batch, rotating engine order between windows. Three separate
process runs; reported value is the median of their window medians. Startup,
JSON, normalization and input creation are outside timing. Owned HTML and fresh
per-document parse state are destroyed inside timing. No AST or arena is reused.
`ox-cm-reuse` retains renderer scratch, while still returning a fresh owned
HTML string. The supplementary `reuse` experiment also measures Ferromark
with its public reusable `Renderer` API under the same options. Allocation instrumentation and scanner experiments use separate
executions and do not contribute to these timing values.

`cm` means tables off, trusted/CommonMark rendering, heading IDs **on for both**
so Ox's mandatory slug work is also done by Ferromark. `tables` changes only
that syntax option relative to `cm`. Both disable automatic URL linking and
external-link decoration. `default` preserves each engine's API defaults.
`ox-cm-grow` changes only arena construction from `for_source_len` to `new`;
`ox-cm-reuse` changes only renderer scratch retention. `ferro-cm-noids` isolates
our optional heading-ID work and is not the matched comparator for Ox headings.

## Implementation comparison

### Memory and representation

Ferromark emits block events, collects/resolves inline marks and renders events
without building an AST. Its buffers are reused within a document; the reusable
`Renderer` API can additionally retain parser storage across documents. This
audit keeps the public fresh-document convenience path used upstream.

Ox constructs a borrowing AST in a bump arena. AST node slots are at most 32
bytes, with larger variants behind arena pointers. Nodes deliberately need no
destructors, enforced by compile-time assertions, so dropping a document does
not recursively walk its tree. The recommended arena reserves eight times the
source length with a 16 KiB floor. It also preallocates HTML output at roughly
twice the source size. This exchanges reserved memory for fewer system allocator
calls; "AST" does not automatically imply individually allocated heap objects.

Sources: [arena](https://github.com/ubugeeei-prod/ox-content/blob/026d1859d1c35e5fb1ea65e7e855b428a918b9bb/crates/ox_content_allocator/src/lib.rs),
[AST invariants](https://github.com/ubugeeei-prod/ox-content/blob/026d1859d1c35e5fb1ea65e7e855b428a918b9bb/crates/ox_content_ast/src/lib.rs),
[renderer setup](https://github.com/ubugeeei-prod/ox-content/blob/026d1859d1c35e5fb1ea65e7e855b428a918b9bb/crates/ox_content_renderer/src/html/renderer.rs),
Ferromark [ADR-0001](../../arch/ADR-0001-core-architecture-streaming-no-ast.md)
and [ADR-0002](../../arch/ADR-0002-inline-parsing-three-phase.md).

### Scanning and plain text

Both implementations use SIMD and skip runs of ordinary text. Ox uses two
16-entry nibble lookup tables to classify many marker bytes together, with NEON
on ARM and SSSE3/AVX2 on x86. Ferromark's current generic ByteSet compares each
needle against a vector and ORs the masks. On ARM it then searches a matching
chunk scalarly to locate the first byte. These are materially different search
strategies, not a distinction between SIMD and no SIMD.

Ox's no-marker inline path constructs exactly one borrowed text node. Soft
newlines can stay inside that node, avoiding a separate node for each line.
Ferromark's three-phase machinery retains CommonMark precedence handling through
separate resolution and event emission. Both have shortcuts, but they pay
different per-block/per-fragment costs. The simple-text and inline-heavy case
measurements locate useful workloads for further optimization; source inspection
alone does not assign an exact percentage to each mechanism.

Sources: [Ox classifier](https://github.com/ubugeeei-prod/ox-content/blob/026d1859d1c35e5fb1ea65e7e855b428a918b9bb/crates/ox_content_parser/src/parser/inline/scan.rs),
[inline path](https://github.com/ubugeeei-prod/ox-content/blob/026d1859d1c35e5fb1ea65e7e855b428a918b9bb/crates/ox_content_parser/src/parser/inline.rs),
Ferromark `src/byte_search.rs`, `src/inline/mod.rs`, `src/inline/marks.rs`.

### Rendering, headings and containers

Ox reuses three small buffers for heading text, slug and ID generation, and uses
compact strings for short heading keys. It does a structural document scan to
size heading bookkeeping and avoids collecting a TOC unless one is requested.
Our `ferro-cm-noids` control shows the cost of optional heading work, but disabling
it would give us less work than Ox on those inputs and is not the main comparison.

Ox has SIMD/SWAR escaping and bulk-copy paths. Ferromark combines its ByteSet
short-string scanner with memchr for longer escapes. Code-fence timing includes
block parsing, output construction and escaping, so its result does not isolate
the escaper by itself.

For simple single-line list items Ox bypasses recursive block parsing, while
nested/multiline items still build a stripped source plus source mappings.
Ferromark's event-based container handling wins on our nested-list fixture.
Ox tables create AST rows/cells and parse each cell's inline content; Ferromark
streams the corresponding events and wins on the actual-table fixture. These
source differences are plausible explanations supported by workload localization,
not proof that removing one particular AST allocation yields the full gap.

Sources: [Ox list handling](https://github.com/ubugeeei-prod/ox-content/blob/026d1859d1c35e5fb1ea65e7e855b428a918b9bb/crates/ox_content_parser/src/parser/list/item_source.rs),
[tables](https://github.com/ubugeeei-prod/ox-content/blob/026d1859d1c35e5fb1ea65e7e855b428a918b9bb/crates/ox_content_parser/src/parser/table.rs),
[escaping](https://github.com/ubugeeei-prod/ox-content/blob/026d1859d1c35e5fb1ea65e7e855b428a918b9bb/crates/ox_content_renderer/src/html/escape.rs).

## CPU sampling and allocation evidence

Two-second macOS `sample` captures localize the large-input work to Ferromark's
inline parser, event rendering, paragraph processing, escaping and heading ID
tracking. Ox's captures concentrate on parsing, inline parsing, escaping, marker
search and list handling. These are sampled stacks with inlined functions, not
an additive accounting of exact phase time. On the tiny input, system allocation
and free routines are prominent for fresh Ferromark. The harness clock itself
also appears prominently at this scale, so the profiles are diagnostic only.

The separately instrumented system allocator confirms the design tradeoff:
Ox makes fewer system allocation calls on the large document but requests more
bytes. With the tiny input, retaining Ox's renderer scratch removes most of its
per-call allocation traffic. See the generated counts rather than interpreting
arena-node creation as a separate global allocation for every AST node. Counts
exclude renderer construction only in explicitly reused modes. They do not
measure peak resident memory.

The isolated scanner experiment validates both implementations against every
byte at each offset in lengths 1 through 64, then measures equal byte sets. It
supports nibble classification as a concrete experiment: long runs benefit,
while some sub-vector inputs favor our scalar path. No parser was patched to
adopt that classifier, so the end-to-end gain remains unmeasured.

## What follows from this audit

The [follow-up implementation experiments](../2026-09-12-ox-inspired-optimizations/REPORT.md)
now test the scanner, fragmentation, allocation and rendering ideas below.
They include retained production changes, rejected alternatives and end-to-end
ARM measurements. The recommendations here record the audit's starting point.

The supplementary reuse control reverses the tiny-document ordering: with
both public reusable renderers, Ferromark is faster on that input. Fresh calls
are approximately tied there. Therefore the earlier suite's reused-Ox versus
fresh-Ferromark short-document result must not be generalized to all API usage.
The large-document Ox lead remains with reuse on both sides.

Ox is a meaningful competitor. The historical public comparison uses an old
Ferromark version and mismatched options, but its lead is not entirely explained
by those choices. On the repeated upstream sample it remains ahead under matched
settings. Our existing table/list strengths and Ox's prose/code/heading strengths
can coexist; neither corpus alone establishes a universal winner.

The first optimization experiment worth trying is nibble-table byte
classification with scalar thresholds for short spans, behind the existing byte
search API. It needs exhaustive equivalence checks, CommonMark/GFM/MDX tests,
long-delimiter/resource-limit cases and end-to-end measurements on both ARM and
x86. Scanner microbenchmarks alone are not permission to replace the implementation.

The next useful areas are paragraph/soft-break fragmentation, heading scratch
allocation, and code-block rendering. Retain the event architecture unless a
controlled experiment demonstrates a better design; an arena AST is not necessary
to adopt these narrower techniques.

For our Ox adapter, test `Allocator::for_source_len` on all existing cases and
repeat the affected measured runs before changing published numbers. The arena
control here is approximately neutral on the long sample but helps the tiny
input. Renderer scratch reuse has a much larger effect on the tiny input; our
existing adapter already permits that reuse and correctly keeps owned output.

MDX is supported by both projects, but these measurements say nothing about MDX
performance. A future MDX comparison needs the same output contract: parsing or
preserving JSX/expressions is different work from compiling JavaScript,
executing components or generating a site.
