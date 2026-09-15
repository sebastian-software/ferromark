# Optional writing feature experiment

This experiment tests opt-in marked text, inline notes, and disabling reference
links. The baseline is the v2 integration commit `6fded76`.

```sh
python3 benchmarks/runtime-profiles/prepare.py /tmp/writing-before --revision 6fded76
python3 benchmarks/runtime-profiles/prepare.py /tmp/writing-after --working-tree
python3 benchmarks/optional-writing/run.py /tmp/writing-before /tmp/writing-after /tmp/writing-results
```

Output directories must not exist. `--working-tree` freezes current core files,
including new source files. Each build records the baseline revision, whether
working files were used, a source-tree digest, binary digest, lockfile identity,
compiler, and build flags. Both builds use Rust 1.95, optimized release builds,
fat LTO, one codegen unit, and the generic target CPU. Dependencies remain locked
to the original registry versions. For a committed candidate, omit
`--working-tree` and use `--revision COMMIT`.

The runtime worker conditionally compiles the new options only when the frozen
core exposes them. All other runtime configuration and lifecycle code is shared.
The existing paired worker protocol checks timed checksums and re-verifies HTML
and AST after timing. Arena capacity may grow during warmup and is not an output
correctness assertion.

## Workloads and interpretation

Five fixed repository documents, seven repeated synthetic shapes, and sparse
documents with one mark and one inline note at approximately
300 B, 4 KiB, and 64 KiB cover prose, mixed Markdown, literal equals/carets,
active marks, active inline notes, reference definitions, and malformed markers.
Whole snippets are repeated without cutting syntax; actual byte lengths are
recorded. Inputs are archived with each run. All inputs are repository-owned MIT
text or the attributed upstream changelog fixture.

Every input compares the old core to the new core with writing extensions off,
requiring exact HTML and debug AST equality. Additional comparisons enable each
option, both options, or disable reference links. Unused-option controls require
identical HTML; AST equality is reported separately because literal markers can
split text nodes when extension scanning runs. Active syntax must change HTML.
Disabling references on real documents is also allowed to change output and is
reported as a dialect cost, not equivalent-output acceleration.

Each comparison runs parse-only, retained parse/render, and fresh parse/render.
See [runtime study](../runtime-profiles/README.md) for the lifecycle definitions.
The default is seven alternating-order pairs with 10 ms warmup and 50 ms timing
windows per side. Job order uses a fixed shuffle seed. Parsing configuration,
reading inputs, verification, serialization, and IPC are outside the timers.

Ratios are candidate/baseline or enabled/disabled time: above 1 is slower.
`summary.json` reports medians and full paired ranges; `results.json.gz` retains
raw samples, HTML, debug AST, and allocation-capacity observations. Do not call
small workstation differences improvements. Repeat runs in independent processes;
inspect each workload and outliers instead of only averaging ratios.

The retention target is no reproducible material slowdown with extensions off.
A repeated slowdown above roughly 5% on a representative workload warrants
investigation. Enabled-but-unused costs and marker-heavy corner cases must be
reported separately, even when the disabled path meets that target.

Use `--profile commonmark` for the core default profile, and `--filter REGEX`
to repeat selected case/stage/comparison rows. `report.py REPORT_DIRECTORY`
generates tables from archived `gfm/`, `confirmation/`, and `commonmark/` runs.
