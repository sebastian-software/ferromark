# Runtime feature and profile study

This harness measures runtime options in one frozen native binary. It separates
the cost of enabling an option on plain prose from processing a dense synthetic
example of that option, then compares candidate configurations on mixed documents.
No parser or renderer implementation is changed by the study.

```sh
python3 benchmarks/runtime-profiles/prepare.py /tmp/runtime-profile-build
python3 benchmarks/runtime-profiles/make_cases.py /tmp/runtime-profile-cases.json
python3 benchmarks/runtime-profiles/run.py /tmp/runtime-profile-build \
  /tmp/runtime-profile-cases.json /tmp/runtime-profile-results
python3 benchmarks/runtime-profiles/summarize.py /tmp/runtime-profile-results \
  /tmp/runtime-profile-tables
```

Output directories must be new. `prepare.py --revision COMMIT` selects a committed
core; the default is HEAD. It archives source, uses the pinned Rust 1.95 toolchain
and registry versions, and builds with optimization level 3, fat LTO, one codegen
unit, and `target-cpu=generic`. The runtime worker is adapted from the existing
[optimization worker](../optimization-rounds/worker.rs), and the runner reuses its
checked protocol and checksum validation. No application logging, file reads,
JSON parsing, or IPC is inside the timed interval.

## Workloads and comparisons

- **37 feature toggles:** parser and renderer options at approximately 300 B,
  4 KiB, and 64 KiB, each on plain prose and repeated active syntax. Actual byte
  lengths are recorded; complete syntax blocks are never truncated to hit a size.
  Frontmatter occurs only once; the TOC probe has one marker and many headings.
  These are mechanism probes, not representative frequency estimates.
- **Malformed frontmatter:** a valid opener with no closing delimiter, at all
  three sizes. The full input remains ordinary Markdown.
- **13 mixed documents:** four authored comment shapes, five unchanged project
  documents, and four Wikipedia-derived inputs, 37 B to 113,609 B. Selection is
  fixed in `make_cases.py` before timing. Five profile recipes run on each input.
- **Four profile ablations:** broader parser options versus selected use-case
  options on the exact same input, requiring identical HTML and AST. Renderer
  policy stays identical. These isolate the cost of unused parser extensions.

The comparison recipes are defined in `make_cases.py`; unspecified fields use
the explicit CommonMark parser and renderer profiles. Dependencies such as
`tables`, `table_colgroup`, `heading_ids`, and code annotation syntax are enabled
on both sides when needed to isolate an individual toggle. `gfm` is preset
metadata, not a substitute for the individual syntax flags.

These are candidate recipes, not new public presets. Their output requirements
must be chosen before comparing performance: turning off tables or link handling
can change the document. The `comments` recipe explicitly escapes raw HTML;
`gfm-spec` instead applies the GFM tagfilter. Neither recipe silently substitutes
the other's HTML policy. MDX is exercised on authored MDX, not arbitrary corpus
documents with framework-specific syntax. Nesting protection remains enabled.

Original input origins and licenses are carried into `corpus.json.gz`. See the
[project-document attribution](../broad-comparison/licenses/ATTRIBUTION.md) and
[Wikipedia attribution](../broad-comparison/WIKIPEDIA-ATTRIBUTION.md). Synthetic
probes and authored comments use the repository's MIT license. Timed HTML is a
mechanical transformation of those inputs; debug ASTs retain source excerpts.

## Measurement contract

All configurations run through four entry-point lifecycles:

| Stage | Work inside the timer |
| --- | --- |
| `parse` | Parse into a retained arena, consume AST through `black_box`, reset arena |
| `render` | Render a previously parsed AST into a retained output buffer |
| `fresh` | Construct arena and owned renderer options, parse, render owned HTML, destroy objects |
| `reuse` | Parse into a retained arena, render borrowed HTML with retained renderer, reset arena |

Parser options are cloned per parse in every relevant stage. Renderer options
are constructed/cloned per call only in `fresh`; the other renderer stages use
existing renderer objects. Each stage uses the same contract on both sides.
Fresh and retained results answer different integration questions and cannot be
added or averaged as if they were the same API lifecycle. Arena capacity from
verification is reserved memory, not a global-allocation or peak-RSS measurement.

Before timing, all four lifecycles must produce identical HTML and AST for each
configuration. Every active feature probe must change HTML, except explicitly
identified no-op fields. Same-output ablations and malformed-frontmatter controls
must preserve exact HTML and AST across configurations. Plain probes record
equality; options such as `source_spans` legitimately affect even plain prose.
This is a measurement integrity check, not an independent conformance oracle.

Each case/stage runs two process rounds, three alternating-order pairs per round,
5 ms warmup per worker, and 20 ms per timed window by default. Case/stage order
is shuffled deterministically. Worker startup is excluded. Output checksums and
post-timing output checks must match. Renderer-toggle parse-only controls run at
300 B; their redundant larger parse-only stages are omitted.

`summary.json` and CSV report the median of paired **on/off time ratios**,
absolute median nanoseconds, pair ranges, and separate round medians. Above 1
means more time with the option enabled; below 1 means less time. Active probes
often produce different ASTs or different amounts of HTML, so their ratios are
cost observations, not same-output optimization wins. Profile rows compare with
strict CommonMark; the `commonmark` self-comparison is an explicit noise control.
Ablations use broad options as `off` and tailored options as `on`.

Small differences and overlapping ranges are inconclusive on a shared workstation.
Repeat selected cases with `--filter REGEX --rounds 3 --pairs 3 --window-ms 50`.
Three input sizes describe scaling on these shapes; they do not prove worst-case
algorithmic complexity. Real-source feature mixes and repeated synthetic snippets
must remain separate in summaries.

Frozen inputs, full verification output, samples, build identity, and host state
are retained in the result directory. `--verify-only` exercises all semantic gates
without collecting timings.

`make_interactions.py OUTPUT.json` creates six additional same-output probes for
renderer URL detection after parser GFM autolinking. Pass that corpus to `run.py`
in the same way. `summarize.py` generates the complete main-study tables; partial
confirmation and interaction runs retain their machine-readable summaries.

Run `python3 -m unittest discover -s benchmarks/runtime-profiles -p 'test_*.py'`
to check the semantic gates, output-drift checks, and paired-ratio aggregation.
