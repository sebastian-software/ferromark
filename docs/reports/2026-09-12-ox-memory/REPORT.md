# Native peak-heap audit with a growing Ox arena

September 12, 2026. Follow-up to the [optimization experiments](../2026-09-12-ox-inspired-optimizations/REPORT.md).
The primary comparison uses `Allocator::new()` for Ox, as requested for evaluating
memory demand. `Allocator::for_source_len()` is retained only as a diagnostic
control. Its speed-oriented reservation policy is not the baseline for a memory
advantage claim. No publication performance figures change.

## Findings

The report leads with mixed and plain documents in the 1 MiB and 8 MiB size
classes. Small cases remain controls, not the headline basis for a memory claim.
The [generated results](RESULTS.md) establish a peak-heap advantage for Ferromark
on the upstream mixed-document fixture and many structured Markdown cases, even
when Ox's arena grows on demand. They also show real counterexamples: tiny input,
empty input, a long plain paragraph and a long paragraph with one late marker
use fewer peak bytes with Ox. The large plain-text counterexample persists
at both added sizes. Ferromark's fresh path reserves `input.len() / 16` block-event
slots up front, independently of how many blocks the document actually has
(`src/lib.rs`, `render_to_writer_impl`). This is a concrete source of wasted
capacity on a huge single paragraph and a potential optimization target.
Avoiding an AST is therefore an architectural
choice, not a guarantee of lower memory consumption for every input.

A defensible statement must name the workload, versions, native API and measured
quantity. The upstream mixed-document rows support a claim about their lower
peak of live requested heap bytes. They do not support "always uses less memory
than AST parsers," "lowest RAM use" or a peak-RSS percentage.

Forty-five fresh-document cases were checked; forty-four produce matching normalized
HTML and appear in the primary table. The edge-case aggregate differs in table
recognition and is excluded; its inputs and differing outputs are preserved.
The corpus contains synthetic/repeated patterns and overlapping guard cases.
Do not turn its case count into a representative market-wide winning percentage.

For many documents, processing them sequentially does not add up the parser
peaks. Concurrent conversions and retaining HTML results can make the memory
budget grow; those service-level policies are separate from this per-operation
measurement. No claim about batch RSS or a concurrency workload is made here.

## What is counted

A wrapper around Rust's system allocator tracks successful `alloc`,
`alloc_zeroed`, `realloc` and `dealloc` operations. It maintains the sum of live
requested allocation sizes and records the maximum above the pre-operation
baseline. A successful reallocation replaces the old logical size with the new
size; a failed allocation does not increase the counters.

The measured scope contains native parsing, fresh renderer creation, its scratch,
Ox's fresh arena/AST, and production of a complete owned HTML string. Input
storage, immutable configuration, JSON, normalization and benchmark bookkeeping
are outside the scope. Each returned HTML buffer is included at the high-water
mark. After render returns, its capacity must equal the remaining counted bytes;
after dropping it, the live count must return to the scope baseline.

This is an incremental peak of **requested live Rust heap bytes**. It excludes
stack memory, allocator metadata/rounding/caches, physical page commitment and
any transient old-plus-new storage hidden inside the system `realloc` call.
It is not RSS, peak process RAM or total physical allocator consumption. A
reserved arena chunk contributes its full requested size even if not all pages
have been touched. The primary growing-arena comparison avoids the optional
source-length reservation but still includes each library's internal buffers
and growth policy.

Thirty observations per engine/case (ten fresh operations in each of three
separate processes) agree exactly. This tests reproducibility of allocation
behavior, not statistical stability of RSS. Allocation-counter overhead is not
used to make speed claims. Self-tests cover a live allocation outside the scope,
simultaneous allocations, zeroed allocation, growth, shrinkage, deallocation,
peak preservation and return to baseline.

## Equivalent work and source provenance

Ferromark is the retained implementation in PR #307 at
`c7fa85302d5a9b79b5b0188ef3d78facd1f0e8b7`; every core source hash matches the
prior production confirmation. Ox is the native core at
`026d1859d1c35e5fb1ea65e7e855b428a918b9bb` (3.2.0). Separate archived dependency
locks preserve Ferromark's smallvec 1.15.2 and Ox's standalone resolution to
1.16.1. No Node wrapper or MDX/JavaScript compilation is measured.

The existing Markdown corpus plus four large-document cases is used with
heading IDs enabled for both engines. Mixed documents repeat the upstream fixture
in whole units to the requested size class; plain documents contain exactly
one or eight MiB of text in one paragraph.
CommonMark, tables, strikethrough and task-list flags match per case. Raw HTML is
trusted on both sides; automatic bare-URL conversion and external-link
attributes are disabled. For the earlier guard cases, enabling heading IDs is
an intentional difference from the previous speed corpus. Cases using MDX and
the existing reused-renderer control are excluded from this fresh API comparison.

A warmup initializes one-time state, but its renderer, parse storage, arena and
HTML are all dropped. Every measured operation then creates new document state;
no cross-document scratch or arena reuse is hidden outside the counters. Both
engines preserve their normal internal buffer sizing; Ferromark is not
reconfigured specially for the memory test.

Output is normalized with Ox's pinned conformance normalizer outside the measured
scope. It ignores incidental HTML whitespace, heading IDs, target/rel attributes
and equivalent encoding spellings. This establishes comparable work for the
selected fixtures, not complete byte-level equivalence or dialect conformance.
The normalizer/text codec and upstream MIT notice are included.

Builds use the same Rust 1.97.1, generic AArch64 target, release optimization,
fat LTO, one codegen unit, panic abort and system allocator as the preceding
standalone experiment, outside the repository's Cargo configuration. Source and
binary hashes, lockfiles and build logs are archived. See [REPRODUCE.md](REPRODUCE.md).
