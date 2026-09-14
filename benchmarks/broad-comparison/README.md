# Broad Markdown comparison

This experiment changes the corpus while retaining the **exact worker binaries**
from [the first comparison](../current-comparison/README.md). Ferromark main is
`a6e9906f7b4a01355d336f209fd12534796419df` (0.9.0); the v2 parser is the initial
OX-derived core `d1481ca7687e94d30e56f473d3044a175dc6c9a6` (2.0.0-dev.0).
It is not a moving-HEAD comparison or an optimization patch.

## Questions and selection

- Which engine is faster on short comments, ordinary prose, and technical docs?
- Does the answer change with input size or the public API's reuse lifecycle?
- How much of the real corpus produces equivalent HTML in the first place?

Select documents before output checks; freeze converted inputs before timing. Keep the previous 12 real
Ferromark documentation inputs intact as a bridge to the earlier workflow study.
Add external Markdown documents selected by source and size, encyclopedia prose
converted to Markdown, and explicitly authored comment examples. Excerpts are
identified as excerpts; no text is repeated or padded to hit a target size.
No document is chosen or removed because of a timing result.

The first encyclopedia converter draft used `[label](<URL>)` destinations.
Pre-timing validation found that the measured main build duplicates these as
additional autolinks. The final conversion uses ordinary `[label](URL)` with
parentheses percent-encoded; document/topic/excerpt selection is unchanged.
The 41-byte `guard-angle-link` reproducer remains in the corpus as a separate
diagnostic. This preparation adjustment is recorded rather than disguising the
initial converter draft as a frozen production dataset.

Each of the four encyclopedia topics also has an explicitly labeled full prose
adaptation: link labels are retained without destinations, emphasis markers are
removed, and headings become plain paragraphs. Lists and quotations retain their
structure. These cover prose with sparse markup alongside Wikipedia's dense
links; all original linked versions remain measured. No timing informed this
additional view. It is a separate content family, not four additional sources.

Sizes are UTF-8 input bytes, in half-open bins: <512 B, 512 B–2 KiB, 2–10 KiB,
10–50 KiB, 50–256 KiB, and >=256 KiB. Empty bins are gaps, not implicit coverage.
Comment examples are synthetic shapes of plausible posts, not actual GitHub
user comments. Converted encyclopedia material models prose; it does not claim
that Wikipedia stores or renders Markdown. Sources, revisions, licenses, and
transformations travel with every frozen input. Feature counts are simple input
descriptors, not a parser or a definition of semantic coverage.

The local and encyclopedia acquisition scripts produce compressed snapshots.
`make_corpus.py` merges these with the authored comments and writes the final
hash-checked manifest. Archived snapshots are authoritative for reproduction;
acquisition can require network access or the earlier report's frozen exports.
The source families have separate licenses; see the adjacent attribution files
and each case's `origin`. The comments and harness use this repository's MIT
license. The snapshot licenses do not change the parser's license.

## Matching and output checks

Reuse the existing worker and exact build options: Rust 1.95.0 / LLVM 22.1.2,
generic AArch64, system allocator, opt-level 3, fat LTO, one codegen unit, panic
abort, separate original dependency locks. Library sources are unchanged.

Both profiles generate heading IDs, trusted HTML, and matched link/hard-break
options. Encyclopedia prose uses CommonMark. Comments and technical docs use
GFM (including tag filtering); footnotes are disabled. The legacy collection is
now measured under this GFM profile, whereas the earlier workflow report used
CommonMark plus tables, strikethrough, and tasks. Its membership is the same;
profile, compiler, and lifecycle are not a reproduction of that older number.
MDX execution/segmentation, syntax highlighting, sanitization, I/O, JavaScript
bindings, and cold process startup are outside this native core comparison.

The three public API lifecycles are unchanged:

| Mode | Main | v2 |
| --- | --- | --- |
| `fresh` | `to_html_with_options` | New source-sized arena, parser/AST, renderer, owned output |
| `owned` | Persistent `Renderer::render` | Persistent arena/renderer, new parser/AST, owned output |
| `reuse` | Persistent `Renderer::render_into` + output buffer | Persistent arena/renderer, new parser/AST, borrowed output |

Parsing, prepasses, rendering, output consumption, per-document destruction and
arena reset are timed. Reused state is warmed. No result or AST is reused.
Fresh v2 includes cloning owned renderer options; public allocation strategies
are retained. File reads, verification, normalization, IPC, and startup are
outside the timer. Small comments therefore measure warm-process rendering,
not the latency of launching a CLI or receiving a web request.

All three modes must agree byte for byte within each engine. Cross-engine
checks use the previous narrow serialization normalizer: entity spelling,
attribute order, checkbox booleans, void-tag slashes, and formatting newlines
between block tags. Content, links, classes, IDs, and literal whitespace remain
significant. **Only exact or serialization-equivalent output is admitted to
the comparable aggregates.** Heading-ID-only differences get their own
diagnostic label; they are not normalized into admission. Other differences
remain visible, with original outputs and token diffs archived.
Non-void self-closing tags retain HTML's unclosed-element behavior in the
comparator. This small correction to the previous normalizer does not change
either worker binary or timed code.

The broad corpus also exposed Unicode URL spelling differences. The comparator
equates literal non-ASCII path/query/fragment characters with their UTF-8
percent encoding for HTTP(S)/relative URLs with ASCII authorities, following
the [URL Standard's percent encoding](https://url.spec.whatwg.org/#percent-encoded-bytes).
It does not decode existing escapes, alter ASCII reserved characters, rewrite
hosts, change destinations, or normalize heading IDs. This is a documented
serialization extension to the first comparison, guarded by negative tests.

Two complete rotating cycles are verified in every mode before timing. Each
timing worker must match the original HTML before and after its windows, and
every window's output-length checksum must match the verified document lengths.
These checks cover state transitions and repeated calls as well as first calls;
the checksum is a length guard, not a cryptographic content check inside the timer.

## Protocol and interpretation

Each document is measured individually. Fixed rotating collections additionally
visit each member once per traversal. Collection membership never changes after
output verification; any mismatch makes that collection's result diagnostic.
This prevents silently substituting an easier subset for the declared workload.

Three independent process rounds, each with three alternating paired windows,
use at least 75 ms per window and a 75 ms warmup per engine. Jobs are shuffled
with a fixed seed. Timed windows execute batches of 32 complete traversals.
Only one engine is timed at once. Every duration and iteration count is kept.

Per-document ratios are paired medians of `main time / v2 time`; **above 1
means v2 is faster**. Per-category and per-size ratios are geometric means of
these document ratios, giving each admitted document equal weight. These are
two views of the same cases and must not be added together. There is no claimed
production traffic distribution or single overall score. Excerpts from one
article are correlated examples, not independent source populations.

A practical lead means every paired ratio is >1.05 (v2) or <1/1.05 (main).
Everything else is labeled close/variable; this is a conservative screen, not
a statistical confidence interval. Round medians and min/max ranges accompany
each case. Local Apple Silicon findings are not a cross-platform guarantee.
This was an active desktop, with browser and backup activity observed before
the run; no concurrent compiler or other benchmark was present at that check.
It is not a machine-isolated latency measurement. Retained paired ranges and
round medians are therefore essential when judging small differences.

## Reproduction

Use the frozen source exports and `current-comparison/prepare.py` to rebuild
workers if needed. The original comparison's `build.json` records their hashes.

```sh
python3 -m unittest discover -s benchmarks/current-comparison -p 'test_*.py'
python3 -m unittest discover -s benchmarks/broad-comparison -p 'test_*.py'
python3 benchmarks/broad-comparison/make_corpus.py CORPUS.json
python3 benchmarks/broad-comparison/run.py BUILD_DIR CORPUS.json VERIFY_DIR --verify-only
# Stop builds and other benchmarks before timing. Use a new output directory.
python3 benchmarks/broad-comparison/run.py BUILD_DIR CORPUS.json RESULTS_DIR
python3 benchmarks/broad-comparison/publish.py RESULTS_DIR REPORT_DIR
```

The report contains a compressed frozen corpus, full HTML verification, paired
samples, lockfiles, build metadata, run configuration, CSV, and readable tables.
To inspect an individual input without regeneration, decompress `corpus.json.gz`
and find its named entry under `cases`.
