# Native MD4X diagnostics

Ferromark revision: `83a716a208299e47c54b980167393185868dd8f5`. MD4X source: `b2623bb757ededc9adbc2a422ad1828a0965b2bf` (release `0.0.29`).

Environment: Apple M1 Pro, macOS-26.6.2-arm64-arm-64bit-Mach-O.

Native Zig md_html with renderer_flags=0; fresh C-allocator output buffer; parser and output deallocated per call.

The released parser has a fixed extended dialect, without independent syntax switches. Outputs are diagnostic only; no equal-configuration throughput is admitted.

## Workload and capability evidence

Admitted workloads: 0/18. Excluded: commonmark/short, commonmark/2k, commonmark/5k, commonmark/10k, commonmark/50k, tables/tables-plain, gfm_overlap/tables-plain, tables/tables-links, gfm_overlap/tables-links, tables/tables-commonmark-inline, gfm_overlap/tables-commonmark-inline, strikethrough/features, task_lists/features, gfm_overlap/features, tables/neutral, strikethrough/neutral, task_lists/neutral, gfm_overlap/neutral.

Admitted renderer differences: none.

Normalized mismatches against the 652 stored CommonMark examples: Ferromark 0; MD4X 3. This is diagnostic evidence, not a conformance certification.

Original HTML, effective options, all eight feature-switch combinations, single-tilde and trusted-URL probes, and spec outputs are retained. Admission follows ARCH-COMP-002 independently of HTML fidelity. Fixed-dialect engines are excluded when the required switches cannot be matched; no parser source is patched to remove native behavior.

For configurable engines, CommonMark disables extensions; table/strike/task lanes enable only their named syntax; GFM overlap enables those three together. Raw HTML and arbitrary URL schemes are preserved. Bare autolinks, tag filtering, footnotes, math, heading IDs, typography and MDX stay off. No full-GFM or MDX-throughput claim is made.

## Reproduction

See the engine adapter README. Metadata records upstream hashes, compiler versions, build commands, dependency locks, local source hashes and executable hashes. The archive retains raw evidence; report generation rechecks every timed window and recomputes summaries.

```sh
python3 benchmarks/native-pipeline-comparison/report.py docs/reports/2026-09-11-native-md4x --check
```
