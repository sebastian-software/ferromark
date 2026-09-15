# Document-wide footnote collection

## Status and behavior

The candidate corrects label discovery in MDX, lists, block quotes, nested
footnote bodies, and raw HTML. See the [scope decision](../../decisions/2026-09-15-footnote-scope.md).
The owner accepts the targeted cost when footnotes are used. The fix is retained;
a separate follow-up measures disabled and enabled-but-unused paths more closely.

The old scan missed real container definitions and accepted fake definitions
inside raw HTML. `regressions-before.txt` records the two initially failing
end-to-end tests. The final suite passes 831 Rust tests, Clippy with warnings
denied, formatting, and every benchmark build. Existing snapshots and official
specification baselines remain unchanged. The MDX span test no longer needs an
extra global definition to make nested references resolve.

## Measurement

Compare the unchanged core at `c06c5de` with the final candidate patch. The frozen
baseline worker was built during the previous URL refactor and has the same core
source as that commit; its metadata records the working-tree parent. Intermediate
CI-only commits do not affect these core comparisons.

Use Rust 1.95, generic CPU, fat LTO, one codegen unit, and the default allocator on
Apple Silicon. All equivalent-output runs check exact HTML, debug AST including
spans, root children, and output lengths before and after timing. Corrected-output
cases are checked by regression tests rather than admitted into equality timing.
No coverage instrumentation or test/build workload runs during final timing.

- Full frozen corpus: 57 documents, each measured separately and in rotating
  batches, three rounds, five alternating pairs, 40 ms windows.
- Targeted corpus: six authored cases, three rounds, seven alternating pairs,
  50 ms windows. The disabled case turns off both footnotes and link references;
  the enabled-unused case contains ordinary Markdown without footnotes.
- Full corpus aggregate: **−0.45% fresh / −0.12% reused**. Individual cases are
  shown in [per-document.md](per-document.md); the largest positive median is
  +2.13%. This establishes no full-suite regression on this host, not an overall
  speedup or a guarantee for every platform.
- Footnote stress case: **+10.95% / +10.21%**. It repeats the same short reference
  and definition 64 times. Correct structural discovery costs more than the old
  physical-line scan on this deliberately definition-heavy input.
- Disabled case: **+2.44% / +2.70%**; enabled-unused: **+1.54% / +1.39%**. Neither
  enters structural collection. These measurements still include generated-code
  and layout effects; skipping collection is not proof of zero runtime cost.
- Root link references: **−1.43% / −0.67%**; fenced definition decoys:
  **−3.87% / −4.49%**. Collection no longer allocates plain paragraph nodes.

## Attempts

Keep the raw samples and patches for every attempt. `structural` simply shares
one grammar and was rejected for its roughly 27% footnote stress regression.
`body-skip` avoids collecting from footnote bodies without any `]:` marker, but
still costs roughly 13%. `lean` also omits temporary paragraph nodes and avoids
an intermediate normalized footnote label. Its full-corpus result was −0.05% /
+0.04%. The final candidate additionally accepts definition candidates immediately
after a flow component's opening tag, with a blocking public-output regression.

`tables.md` summarizes all retained measurement rounds; run directories contain
raw samples, verification data, corpus bytes, build metadata, and host details.
Intermediate results do not replace the final candidate measurement.

## Coverage

The final footnote candidate alone measures 93.89% lines (12,134 / 12,923;
789 missed), on this ARM host, excluding the separately tested N-API crate.
`prepass.rs` and `reference/collect.rs` each reach 100% line coverage;
`footnote.rs` reaches 98.42%. No new coverage exclusions were added.

Four independent public-output TOC tests increase `html/toc.rs` from 78.52% to
91.28% lines and the combined workspace to **94.05%** (12,154 / 12,923; 769
missed). The combined instrumented run passes **830 executable tests**;
the separate ordinary test checks also pass five doctests (**835 total**). These totals include
the retained footnote implementation together with the TOC tests.
The before/after totals are in `coverage-summary.json` and
`coverage-with-toc-summary.json`. This is local line coverage, not branch coverage
or a claim that architecture-specific x86 paths ran on ARM.

## Platform CI

[Run 34970712075](https://github.com/sebastian-software/ferromark/actions/runs/34970712075)
passes all 24 jobs at `31ebb31`, before the local footnote candidate. This verifies
Rust on Linux/macOS/Windows with MSRV and stable, eight native Node targets,
Node 22/24 and the consumer floor, coverage, contracts, standards, advisories,
licenses, and homepage build. Musl targets are build/artifact checks; the matrix
does not claim runtime tests for those two targets. Re-run CI with the retained
candidate before release.

Windows checkout preserves the historical reserved `nul.md` input in Git but
sparse-excludes it from Windows working trees. Git's path validation is disabled
only within the checkout action so that the excluded index entry is accepted.
RustSec builds its tool with stable independently of the project's pinned MSRV.
The Rust test job runs serially after one concurrent macOS complexity guard
reported x8.4 for a fourfold input increase; the assertion remains x8.0.
