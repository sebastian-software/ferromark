# Source and build provenance

The measured v2 core is `e93394ee4c2d5e3022eaf3fd0c3b542bcd5eb97a` from
`sebastian/macos-arm-performance-e17e95`, including both final dense-escape
fixes. Ferromark v1 remains `4e151415a15c67e9d3735f719b0bed3e25d8cff8`.
Original OX is `a71a58939ffe7f154117cea026f6d6e71a139393`, md4c is
`65c6c9d72cebd9a731aaa5597414ce04d9ea5de3`, Bun native is
`76e9dcc6ad272a4fb1ee4a4dbbde4809201b71d1`, and pulldown-cmark is 0.13.4.
All engines were rebuilt in one new executable. No production parser or renderer
source was modified for this rerun.

The previous temporary source caches had been removed. `restore.py` retrieves
only the original pinned public sources; `restore.json` records URLs and hashes.
Bun is a new sparse, blob-filtered Git checkout; the materialized source is
checked against its tracked blobs. The OX, mimalloc, and Highway tarballs match
the original archived SHA-256 values exactly. Their parser and native-support
sources were independently checked again after extraction and workspace setup.

The source audit checks 35 v1 files, 373 v2 files, 3495 Bun files,
10 md4c files, and 393 OX files, plus all files in the native-support
archives. There are no mismatches. See `source-audit.json.gz`.

## Build conditions

- The same pinned `nightly-2026-07-20` Rust toolchain for every Rust engine.
- Generic AArch64, optimization level 3, fat LTO, one codegen unit, panic abort,
  and Bun's line-table debug information.
- Clang `-O3`; md4c C99 and Highway C++23; no cross-language LTO or PGO.
- Bun's original mimalloc for Rust and redirected md4c C allocations, in one
  process. This is a controlled allocator comparison, not system malloc.
- The exact shared Cargo lock from the preceding matched-flags comparison:
  `c207bd5aa59eed548e98b03907fe01071480050fed0fe2869b47b58386910040`. Build used `--offline --locked`; registry dependencies did not change.
- Fresh/reused lifecycles, explicit flags, and timed loops are unchanged.
  The workstation is shared; CPU, power, load, and thermal probe results are
  preserved in `run.json`. No builds, tests, or archive compression ran during
  the timed measurements.

The host was on AC power throughout. macOS did not expose thermal/performance
warning telemetry to the probe; the recorded error is not evidence that no
throttling occurred. Round ordering controls and raw timings remain available
to assess variability.

The executable hash is `d68d1975b6a4b9bbcf5c8ef15bcf4b3a7d89c366274b05d2c9675cf2b8cf42dd`. The native libraries, adapters, source
snapshots, and dependency identities are recorded in `build.json`. The
[prior provenance](../2026-09-14-native-matched/PROVENANCE.md) explains the
common dependency resolution and original standalone Bun integration. This is
Bun's native Markdown core, not JavaScript-facing `Bun.markdown.html()`.

The precheck compares all 342 actual HTML outputs with the previous run:
every output and agreement classification is byte-identical. Both suites'
checks and source audits finished before timing. `ParseError`'s new API shape
is accepted as part of v2; no compatibility shim or different parse path was
added to the worker.

## Reproduction

Start with an empty temporary cache directory, copy `restore.py` there, and run
it to restore the pinned sources (network access required). Then, from the
repository root, run:

```sh
python3 docs/reports/2026-09-15-native-arm/harness/prepare.py /private/tmp/native-replay-build \
  --bun-source /private/tmp/native-replay-cache/bun \
  --bun-native-cache /private/tmp/native-replay-cache/native \
  --md4c-source /private/tmp/native-replay-cache/md4c \
  --ox-archive /private/tmp/native-replay-cache/ox.tar.gz \
  --ferromark-v1-source ../ferromark --ferromark-v2-source . \
  --worker docs/reports/2026-09-15-native-arm/harness/worker.rs \
  --lockfile docs/reports/2026-09-15-native-arm/Cargo.lock --compile
python3 docs/reports/2026-09-15-native-arm/harness/run.py \
  /private/tmp/native-replay-build/build.json \
  docs/reports/2026-09-15-native-arm/corpus.json.gz /private/tmp/native-replay-results
python3 docs/reports/2026-09-15-native-arm/harness/report.py /private/tmp/native-replay-results
```

Sources need the pinned commits available locally; the build and result
paths must be new. Benchmark inputs and outputs retain their existing source
licenses and attribution; third-party engine licenses remain unchanged.
