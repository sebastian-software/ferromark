# Current Ferromark comparison

A native Markdown-to-HTML comparison of Ferromark main at
`a6e9906f7b4a01355d336f209fd12534796419df` and the initial v2 core at
`d1481ca7687e94d30e56f473d3044a175dc6c9a6`. The main revision was checked on
GitHub before preparation; the older local working branch was not measured.

## Scope and matching

Both workers use Rust 1.95.0, generic AArch64 code generation, the system
allocator, optimization level 3, fat LTO, one codegen unit, and panic abort.
They are separate binaries: each engine retains the registry versions and
checksums from its own source lockfile. No library code is patched.

CommonMark and GFM profiles use trusted HTML, heading IDs, no external-link
target attributes, and no footnotes. GFM tag filtering remains enabled. The
v2 renderer's additional bare-URL scanner is disabled because GFM autolinks
are already parsed into link nodes. XHTML void elements and hard breaks are
configured to match the main renderer. These are matched library profiles,
not a comparison of the projects' different default security policies.

Sixteen cases are declared in `make_corpus.py` before timing: nine synthetic
diagnostics, six unchanged existing benchmark fixtures, and the actual main
README. Repeated 10 KB diagnostics are synthetic, not real-world documents.
The README is a diagnostic, excluded by its declared origin from the rotating
primary groups. The two rotating groups traverse all 11 CommonMark and all
four primary GFM documents, respectively, with distinct input contents and
allocations. Each document occurs once per traversal.

All three lifecycle modes must return identical output within each engine.
Cross-engine admission first checks byte equality, then a narrow HTML
serialization normalization outside the timer: entity spelling, void-tag
slashes, checkbox boolean attributes, attribute order, and formatting newlines
between block tags. Text, link destinations, IDs, classes, and preformatted
whitespace remain significant. A mismatch is retained as a diagnostic, not
silently removed or included in aggregate claims. `test_runner.py` guards this
boundary. Original outputs are archived.

MDX is not timed here: main's segmentation/rendering contract differs from the
new core's MDX AST and island output. A comparable MDX application workload
needs its own contract.

## Lifecycles

| Mode | Main | v2 |
| --- | --- | --- |
| `fresh` | `to_html_with_options` | New source-sized arena, parser/AST, HTML renderer, owned output |
| `owned` | Persistent `Renderer::render` | Persistent arena and HTML renderer, new parser/AST, `render` |
| `reuse` | Persistent `Renderer::render_into` and output buffer | Persistent arena and HTML renderer, new parser/AST, `render_borrowed` |

The complete parser constructor/prepass, parsing, rendering, output consumption,
and per-document destruction/reset are inside the timer. Reused state is warmed
outside it. Owned output is released each iteration; borrowed output is consumed
before resetting the arena. No AST, rendered HTML, or content-result cache is
reused. A fresh parser is created for every v2 document in every mode.

This deliberately measures the available public APIs. For example, v2's fresh
renderer consumes owned options, requiring option clones, while main borrows
options. Their owned-output capacity strategies also differ. These setup and
allocation costs are included rather than modifying either library to equalize
them. The `reuse` lane shows the effect of retaining both scratch and output.

Filesystem reads, process startup, output verification, HTML normalization,
JSON reporting, and compilation are outside measured windows.

## Protocol and reproduction

`prepare.py MAIN_SOURCE FORK_SOURCE BUILD_DIR` builds workers from frozen source
exports. Supply the two pinned revisions above, not moving working directories.
Export the v2 revision with `git archive`; obtain main from its pinned GitHub
source archive. Preserve each source's `Cargo.toml`, `Cargo.lock`, source trees,
workspace crate manifests, fixtures, and licenses. Cargo dependencies must be
available locally; the preparation build uses `--offline` and rejects lock drift.

```sh
python3 -m unittest discover -s benchmarks/current-comparison -p 'test_*.py'
rustfmt +1.95 --edition 2024 --check benchmarks/current-comparison/worker.rs
python3 benchmarks/current-comparison/prepare.py MAIN_SOURCE FORK_SOURCE BUILD_DIR
python3 benchmarks/current-comparison/make_corpus.py MAIN_SOURCE CORPUS.json
python3 benchmarks/current-comparison/run.py BUILD_DIR CORPUS.json VERIFY_DIR --verify-only
python3 benchmarks/current-comparison/run.py BUILD_DIR CORPUS.json RESULTS_DIR
python3 benchmarks/current-comparison/publish.py RESULTS_DIR REPORT_DIR
```

Use new build/result directories. Stop builds and other benchmarks before timing.
Three process rounds each collect three paired windows per case/lifecycle.
Every worker gets a 75 ms warmup; measured windows last at least 75 ms, checked
after batches of 32 complete traversals. Pair order alternates and job order is
shuffled with a fixed seed. Results retain every elapsed duration and iteration
count. Summary ratios are medians of paired main-time/v2-time ratios; the range
and the three round medians expose variability. Collection times are also shown
per document for readability, without changing the ratio.

This is a local macOS/Apple Silicon screen, not a cross-platform guarantee.
Power/load observations are captured; unavailable thermal probes are recorded
as unavailable. The checked source revisions, worker/binary/lock hashes, frozen
inputs, original outputs, and compressed samples accompany the generated report.

Fixture and README provenance is in the corpus. The copied Ferromark inputs may
be used under the MIT license in `LICENSE-FERROMARK-MIT`.
