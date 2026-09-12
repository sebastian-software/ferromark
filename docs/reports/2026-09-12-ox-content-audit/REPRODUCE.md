# Reproduce the audit

This directory is a diagnostic archive. It does not change the official
benchmark suite, generated homepage figures or parser implementation.

`RESULTS.md` is generated from the compressed raw data:

```sh
python3 docs/reports/2026-09-12-ox-content-audit/make-results.py
```

Rebuild the standalone harness in a new directory (requires Git, Cargo and network
access for pinned sources/dependencies):

```sh
python3 docs/reports/2026-09-12-ox-content-audit/prepare.py /tmp/ox-audit-rerun
```

The recorded audit inherited Apple M1/NEON flags from the repository Cargo
configuration. To match them from another working directory, set
`RUSTFLAGS="-C target-cpu=apple-m1 -C target-feature=+neon"` for the build.
The follow-up optimization report uses a separately built generic target.

This checks out both source revisions. `--ferromark /path/to/checkout` may reuse
an existing checkout; the script verifies its relevant tracked files against the
pinned revision. `@FERROMARK@` placeholders in the archived manifest/scanner are
expanded by this script. Ox's published runner manifest has a broken relative
path; this standalone harness directly references the actual crate paths.

Run each command separately, with no builds or competing benchmarks running:

```sh
/tmp/ox-audit-rerun/harness/target/release/ox-audit conformance /tmp/ox-audit-rerun/ferromark/tests/spec.json
/tmp/ox-audit-rerun/harness/target/release/ox-audit verify
/tmp/ox-audit-rerun/harness/target/release/ox-audit matrix
/tmp/ox-audit-rerun/harness/target/release/ox-audit reuse
/tmp/ox-audit-rerun/harness/target/release/scan-audit
```

Capture stdout. Repeat `matrix`, `reuse` and `scan-audit` in three separate
processes to obtain `matrix-1.jsonl` through `matrix-3.jsonl`,
`reuse-1.jsonl` through `reuse-3.jsonl`, and `scans-1.jsonl` through
`scans-3.jsonl`. Capture `verify` as `output-verification.jsonl` and the spec
command as `conformance.jsonl`. The generator accepts uncompressed or gzip files.
Never overwrite the archived run with results from another host.

Allocation counting is deliberately a separate build:

```sh
cargo build --release --locked --manifest-path /tmp/ox-audit-rerun/harness/Cargo.toml --bin ox-audit --features allocations
/tmp/ox-audit-rerun/harness/target/release/ox-audit matrix
```

Capture that output as `allocations.jsonl`. Do not use this instrumented build
for timing. Restore the ordinary build before CPU profiling. On macOS,
`ox-audit profile ferro-cm large` (or `ox-cm`, or `short` as the last argument)
loops for eight seconds; attach `sample PID 2 1 -file output.txt` to capture a
two-second stack sample. The archive includes four diagnostic captures. CPU
sampling and allocation counting are excluded from the timing tables.

The `pilot/` conformance and baseline captures use html-escape 0.2.15 and
smallvec 1.16.1; final measurements use our locked 0.2.14 and 1.15.2 respectively.
The pilot reproduces the published conformance percentage but is not used for
final timing tables. Both exact lockfiles are retained. `upstream/` preserves
the August runner/lockfile and their MIT license; the fixture derives from that
runner. Source hashes, toolchain, CPU, and the measured binary hash are in
`metadata.json`. SHA256SUMS covers every archived file except itself.

Validation completed: release builds, all-target/all-feature harness checking,
three runs of each timing experiment, exhaustive scanner byte/offset equivalence,
18 matched-output comparisons, and five full CommonMark configurations. This
audit changes no production Rust files; the full repository preflight would be
required before opening a pull request.
