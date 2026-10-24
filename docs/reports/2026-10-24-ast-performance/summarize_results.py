#!/usr/bin/env python3
"""Generate the report tables from the retained paired samples."""
import json
from pathlib import Path
import statistics

ROOT = Path(__file__).resolve().parent

def rows(run):
    return json.loads((ROOT / "raw" / run / "summary.json").read_text())

def delta(ratio):
    return f"{100 * (1 / ratio - 1):+.2f}%"

broad_lines = []
for run, label in [("head-v-initial", "Initial APIs"), ("head-v-promoted", "Accepted implementation")]:
    for mode in ["parse", "fresh"]:
        selected = [r for r in rows(run) if r["mode"] == mode and not r["case"].startswith("target-")]
        ratio = statistics.geometric_mean(r["baseline_over_candidate_median"] for r in selected)
        rounds = [statistics.geometric_mean(r["round_medians"][i] for r in selected) for i in range(3)]
        broad_lines.append(f"| {label} | {mode} | {len(selected)} | {delta(ratio)} | {' / '.join(delta(r) for r in rounds)} |")

final = {(r["case"], r["mode"]): r for r in rows("head-v-promoted")}
target_lines = []
for name in dict.fromkeys(r["case"] for r in rows("head-v-promoted") if r["case"].startswith("target-")):
    target_lines.append(f"| {name.removeprefix('target-')} | {delta(final[name, 'parse']['baseline_over_candidate_median'])} | {delta(final[name, 'fresh']['baseline_over_candidate_median'])} |")

confirm_lines = []
for r in rows("promoted-confirm"):
    ns = r["candidate_ns_per_document"] - r["baseline_ns_per_document"]
    confirm_lines.append(f"| {r['case']} | {r['mode']} | {delta(r['baseline_over_candidate_median'])} | {ns:+.2f} ns |")

control_lines = []
for r in rows("control"):
    control_lines.append(f"| {r['case']} | {r['mode']} | {delta(r['baseline_over_candidate_median'])} |")

text = '''# AST API performance investigation

This follow-up compares the APIs from #563, #564 and #565 with the previous
revision, then checks two changes retained in the implementation. The original
[alt-content opt-in characterization](../2026-10-24-ast-provenance/README.md)
compares off/on within the initial implementation; it is a different question.

## Scope and method

Baseline: `b4e4377a541a15aa72559cda1ab26fc8e3f322e8`.
Candidate: the local implementation reconstructed by applying patches 01, 02
and 05 in order. Patches 03 and 04 are retained experiments and are not part of
the accepted code. All stages use identical registry versions from Cargo.lock,
Rust 1.95, the `jsx` Cargo feature, opt-level 3, fat LTO, one codegen unit,
`panic=abort`, and `-C target-cpu=generic`.

The repository's existing optimization-round worker and runner perform exact
HTML, AST Debug (including spans), and child-count comparisons before timing
and around every measured batch. All comparisons passed. The [worker](worker.rs)
only adapts the historical harness to the current option fields and adds a
`strict-mdx` profile; it does not change the timed loops. The
[builder](build_pair.py) uses the existing frozen-source builder, enables `jsx`
in both engines and copies the current transforms workspace member.

The final corpus contains the existing 57 mixed documents and eight authored
diagnostics: prose, plain/rich image labels, callouts, strict MDX, nesting,
fragments and deeper nesting beyond the tag stack's inline capacity. The initial
run predates the eighth diagnostic. Existing document provenance is retained
in each compressed corpus and in the original frozen SIMD report.

`parse` reuses a source-sized arena and resets it after each parse. `fresh`
creates/drops an arena and renderer and includes HTML output. These measure
public default behavior; `image_alt_content` and diagnostic collection stay off.
The two binaries alternate in each pair; case order is shuffled deterministically.
Full runs use three rounds, three pairs and 30 ms windows. Focused confirmations
use three rounds, five pairs and 100 ms windows. A/A uses the identical old
binary in both slots, with three rounds, three pairs and 50 ms windows.

Host: macOS 27.0.1, arm64, AC power. CPU-model and thermal probes are unavailable
in the sandbox, and host activity is not controlled. Host observations, exact
sampling configuration, source/binary hashes, samples and verification output
are retained under [raw/](raw/). This is local diagnostic evidence, not a
cross-platform throughput claim.

## Accepted changes

1. Strict MDX tag matching stores source-backed names, with an empty source
   slice identifying each fragment. It reconstructs a conflicting opener's span
   through the existing tag scanner only on failure, using the expression memo.
   The tag-stack container returns from 392 to 264 bytes on this target,
   matching the old parser's footprint. [Layout probe](layout.rs),
   [result](layout.txt). Runtime differences from this change alone were mixed
   and below 1%; its demonstrated benefit is storage.
2. Ordinary `Parser::parse()` keeps its former configuration/completion control
   flow. The new diagnostic entry point owns the helpers it needs. This removes
   extra helper calls from the ordinary path. Some checks are intentionally
   repeated between the two entry points to preserve that path's layout.

The completion-inline experiment improved some focused cases, but did not
remove all short-input differences. Adding another inline hint for configuration
checks had mixed results and regressed some cases. Neither hint is retained.
The accepted path restores the original structure rather than relying on them.

The additional related-span tests cover fragments, multiline openers and `<`/`>`
inside JavaScript attributes and quoted strings. Existing JavaScript validation,
container source mapping and nesting limits remain in effect.

## Broad comparison with the old revision

Values are changes in per-document latency; positive means slower. Each corpus
entry has equal weight in the geometric mean. These small values do not prove
strict neutrality, particularly given the control variation below.

| Implementation | Stage | Documents | Latency change | Per-round changes |
| --- | --- | ---: | ---: | --- |
'''+ '\n'.join(broad_lines)+'''

## Authored diagnostics, final full run

| Workload | Parse latency change | Fresh parse + HTML change |
| --- | ---: | ---: |
'''+ '\n'.join(target_lines)+'''

## Longer confirmation of notable cases

The short-comment differences persist at roughly a few nanoseconds. The rich
image workload parses faster but its complete parse/render is around 2% slower.
The volcanic-article result varies across rounds; the longer confirmation does
not retain the short run's roughly 3.5% end-to-end difference. Strict-MDX changes
are comparable to the measured A/A variation. No claim of exact neutrality for
every individual input follows from the broad mean.

The absolute changes below subtract the separately computed median times;
percentage changes use the median paired ratio, so they need not match exactly.

| Case | Stage | Latency change | Median time difference |
| --- | --- | ---: | ---: |
'''+ '\n'.join(confirm_lines)+'''

## Same-binary A/A control

| Case | Stage | Apparent latency change |
| --- | --- | ---: |
'''+ '\n'.join(control_lines)+'''

## Remaining costs and limits

`Image` still grows from 64 to 72 bytes on this 64-bit target, even when alt
retention is off. Removing that slot would require a different metadata storage
API. Retaining alt children remains opt-in and has real allocation/parse costs;
this work does not make that additional analysis free. Callout inspection still
borrows the AST, and diagnostic collection still runs only when requested.

The broad default result is close to the old parser on this host, while narrow
inputs retain small differences. No x86-64 run, process-RSS measurement or Node
boundary benchmark was performed. Functional validation covers the complete
Rust workspace and the no-default-features build.

## Reproduction and retained evidence

Create a temporary source archive of the baseline with Cargo.toml, Cargo.lock,
rust-toolchain.toml, README.md, LICENSE, src, tests, benches, examples,
node/native and transforms. Apply [patch 01](patches/01-initial-api.patch) to
reconstruct the initial APIs; apply patches 02 and 05 for the accepted runtime
sources. Keep a separate unmodified baseline archive. Build and run with:

```sh
python3.12 docs/reports/2026-10-24-ast-performance/build_pair.py \\
  /tmp/old /tmp/candidate /tmp/ast-build --verify-baseline
python3.12 benchmarks/optimization-rounds/run.py /tmp/ast-build \\
  docs/reports/2026-10-24-ast-performance/corpus.json.gz /tmp/ast-run \\
  --rounds 3 --pairs 3 --window-ms 30 --modes parse fresh
```

The small layout probe links against the build's SmallVec dependency using the
same panic strategy and fat LTO. It models Span as its two u32 fields. Raw JSON
files compressed by the archiver decompress byte-for-byte to the originals;
[SHA256SUMS.json](SHA256SUMS.json) records artifact integrity. The other
experiments and their patches remain available to explain the acceptance
choice. The Python summarizer generates these tables from retained estimates;
benchmark values were not entered by hand.
'''
validation = json.loads((ROOT / "validation.json").read_text())
text += "\n## Validation\n\n"
text += f"{validation['tests_passed']:,} tests passed across {validation['test_suites']} suites, with no failures or ignored tests.\n\n"
text += "```sh\n" + "\n".join(check['command'] for check in validation['checks']) + "\n```\n"
text += "\nEvery command exited successfully. [Validation record](validation.json); [patch replay](reproduction.json).\n"
(ROOT / "README.md").write_text(text)
