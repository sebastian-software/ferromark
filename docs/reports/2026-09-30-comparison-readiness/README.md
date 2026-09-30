# Local comparison readiness — 2026-09-30

All **eight Native and six Node.js competitors** built and passed the existing
option/output guards on the local Mac with the refreshed release pins. A short
execution test also completed every engine's timing loop in both lifecycles.
This is a readiness report, not a performance campaign. Blacksmith was not
started, and no homepage figures or historical reports were updated.

The tested code revision was
[`acd3bcfd2ebcbb6dcec67c7c0419b22df4c2c651`](https://github.com/sebastian-software/ferromark/commit/acd3bcfd2ebcbb6dcec67c7c0419b22df4c2c651).
Later commits add runbooks, skill guidance and these retained records;
they do not change the tested benchmark code.
See the [version inventory and refresh procedure](../../../benchmarks/manual-comparison/dependency-readiness.md)
for the checked latest stable releases and upstream changes.

## Host and build

Apple M1 Ultra, 64 GiB RAM, macOS 27.0, native arm64 processes outside Rosetta.
Node.js 24.21.0, npm 11.14.0, Python 3.12.14, CMake 4.4.3, Apple clang 21.0.0.
The additional Rust adapters and Node addon use the committed Rust 1.95.0;
the shared native bundle uses Bun's required nightly-2026-07-20
(rustc 1.99.0-nightly, LLVM 22.1.8). Builds use the existing release recipes,
without PGO. Source directories and executables were rebuilt in a fresh output
directory; previously built benchmark executables were not reused.

Current competitor sources come from the committed restoration script and
current dedicated native campaign lock. Registry fetching and compilation use
locked resolution; the shared native compilation remains offline. The first
refresh attempt compiled but was rejected by the registry guard because the
historical seed did not cover the new dependency resolution. The final run
uses the Cargo-generated current lock without relaxing that guard.

The [source audit](source-audit.json.gz) found zero mismatches, including 3,495
Bun files, 395 OX-Content files, 189 local v2 files and both native support
archives. [Native build metadata](native-build.json),
[additional native adapters](ecosystem-build.json),
[restored pins and archive hashes](restore.json), and
[the readiness record](readiness.json) retain versions, compiler flags,
source/adapter/lock hashes, and executable/addon hashes. Absolute temporary
paths in metadata describe this execution; binaries and caches are not retained.

## Commands and results

Preparation and verification ran from the clean committed worktree, with
`NODE_OPTIONS` and `RUSTUP_TOOLCHAIN` unset and the native Python interpreter
selected through `FERROMARK_BENCH_PYTHON`:

```sh
./scripts/benchmark-comparison doctor
./scripts/benchmark-comparison prepare /private/tmp/ferromark-latest-competitors-20260930-c
./scripts/benchmark-comparison verify /private/tmp/ferromark-latest-competitors-20260930-c
"$FERROMARK_BENCH_PYTHON" diagnostic-smoke.py /private/tmp/ferromark-latest-competitors-20260930-c
```

The retained [diagnostic script](diagnostic-smoke.py) runs the existing harnesses
with **one process round, one sample, a 10 ms window and 10 ms warmup**. It
executes all 57 inputs, fresh/reuse calls, and the shared native lane's rotating
batches. It audits coverage, positive iterations, window duration and consumed
output checksums: **2,988 engine windows passed**. Each shortened ecosystem
pair was also rejected by the actual publication validator. Official publication
requires three rounds, six samples, 40 ms windows, and 60 ms warmup.

Raw execution records live under [diagnostic/](diagnostic/) and output
SHA-256/UTF-8/UTF-16 descriptors under [verification/](verification/), compressed
with deterministic gzip. Full HTML remains in the local output directory and
can be reproduced from the pinned sources and frozen inputs.
No summary speed factors are retained. These short timings are unsuitable for
performance claims or platform comparisons. The frozen corpus and parser
profiles are unchanged; cmark/commonmark.js still use CommonMark for both sides.

Every adapter passed its guards, but HTML equality is classified separately:

| Track | Engine | Exact | Serialization equivalent | Heading ID only | Other |
| --- | --- | ---: | ---: | ---: | ---: |
| Native | pulldown-cmark | 14 | 43 | 0 | 0 |
| Native | md4c | 19 | 33 | 0 | 5 |
| Native | Bun MD | 16 | 40 | 0 | 1 |
| Native | OX-Content | 16 | 0 | 39 | 2 |
| Native | markdown-rs | 19 | 38 | 0 | 0 |
| Native | Comrak | 19 | 38 | 0 | 0 |
| Native | cmark | 20 | 37 | 0 | 0 |
| Native | cmark-gfm | 19 | 38 | 0 | 0 |
| Node.js | micromark | 19 | 37 | 0 | 1 |
| Node.js | marked | 38 | 18 | 0 | 1 |
| Node.js | markdown-it | 19 | 37 | 0 | 1 |
| Node.js | remark / unified | 0 | 56 | 0 | 1 |
| Node.js | Showdown | 0 | 31 | 0 | 26 |
| Node.js | commonmark.js | 20 | 37 | 0 | 0 |

The existing scoring contracts and agreement exclusions remain unchanged.
Passing execution does not make disagreeing documents eligible for factors.
The historical v1 control was exercised inside the native bundle but is not a
homepage competitor.

Repository checks passed: workspace formatting, all-feature locked Rust tests,
Clippy with warnings denied, and all benchmark builds; 52 native harness tests,
19 manual workflow tests, 15 ecosystem Python tests (one additional fixture
skipped), six Node adapter tests, two real-addon loader tests, 75 script Node
tests and 45 script Python tests. Script formatting, actionlint, wrapper shell
syntax and both homepage publishers' `--check` also passed. Compressed build,
verification and Rust check logs are retained in [logs/](logs/).

`npm outdated --json` returned `{}` for all direct Node benchmark dependencies.
[npm audit](npm-audit.json) retains Showdown's three moderate advisories with no
available fixed stable release, as documented in the readiness procedure.
The application lockfile, upstream attribution and selected homepage values
are unchanged. This local run does not establish compatibility on Linux,
Intel Macs, or Blacksmith; those platforms need their own verification before
an official measurement.

Recheck every retained diagnostic window with `python3 validate-diagnostic.py`.
Verify retained file integrity with `shasum -a 256 -c SHA256SUMS` from this
report directory. The full local output remains outside the repository at the
path above for further diagnosis.
