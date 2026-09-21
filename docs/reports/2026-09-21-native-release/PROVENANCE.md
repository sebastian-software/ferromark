# Source and build provenance

The measured v2 core is `bffc89f6034776da19ed24e40b3eb5ac33e50452` — `main`
after the final pre-release review, 33 commits past the preceding report's
`7c887a2b` (3 `perf`, 9 `fix`, the rest documentation, packaging and CI).
Ferromark v1 stays at `4e151415a15c67e9d3735f719b0bed3e25d8cff8`, OX at
`a71a58939ffe7f154117cea026f6d6e71a139393`, md4c at
`65c6c9d72cebd9a731aaa5597414ce04d9ea5de3`, Bun at
`76e9dcc6ad272a4fb1ee4a4dbbde4809201b71d1`, and pulldown-cmark at 0.13.4.
**These are the same five comparison pins as the three preceding reports**;
only v2 moved. This reruns pinned versions; it does not claim to compare every
library's latest release.

Both Ferromark histories live in this repository, so `--ferromark-v1-source`
and `--ferromark-v2-source` both point at the repository root and `prepare.py`
exports the two pinned commits with `git archive`. Working-tree sources are
never built.

## Source caches

macOS had purged the files of the preceding restoration from `/private/tmp`
(the directories survived, empty), so this run restored every comparison
source again with the archived `restore.py` into a fresh cache directory. The
Bun and md4c checkouts are the pinned commits above; the OX tarball, the Bun
mimalloc archive and the Highway archive verified against the same SHA-256
values as the original run (`restore.json`).

## Source audit

`harness/audit_sources.py` checks the exported sources against Git blobs or the
checksummed OX archive. Its v2 path list is still `crates`, which the
consolidated root package no longer uses, so `audit_v2_sources.py` runs the same
`audit_git` helper over `src`, `Cargo.toml` and `Cargo.lock` and writes
[source-audit-v2.json](source-audit-v2.json). The harness script is unmodified
and its own output is archived unchanged as
[source-audit.json.gz](source-audit.json.gz). Both report no mismatches; the
counts are in `checks/commands.json`.

All 342 HTML outputs and every agreement classification match the preceding
report byte for byte. No production parser or renderer source was edited for
this measurement.

## Build conditions

- Shared `nightly-2026-07-20` compiler (`rustc 1.99.0-nightly`, LLVM 22.1.8),
  required by the native Bun integration.
- Generic AArch64, optimization level 3, fat LTO, one codegen unit, panic abort.
- Original native mimalloc for Rust and redirected md4c C allocations.
- Default build `RUSTFLAGS`: `-C target-cpu=generic`; no PGO object in
  `build.json`.
- PGO build `RUSTFLAGS`: the same plus
  `-Cprofile-use=<merged> -Cllvm-args=-pgo-warn-missing-function`.
- Final Cargo lock SHA-256: `dcd99aa42554a095c9798bb3a70578f90512be17315af72d16767345797cb6d0`,
  shared by both builds.
- Default executable SHA-256: `fb0c5e10fd3b75837caa4eb5b624f44757a513144408b9d0ceb12ff2457f4e8b`.
- PGO executable SHA-256: `7b438c32b46dff7eda27f9d3c5eff4f5718af51503bebb913279fe3dbd42d0a8`.
- Merged profile SHA-256: `c97ad789c2d26a5ea70a26e4b56aad6ecae386146234648a60db03056e2f206e`
  (916,224 bytes, 24 `.profraw` inputs — six engines × two profiles × two lifecycles).

### The lock was seeded, not replayed with `--locked`

The preceding report archived its lock with the v2 path package at
`ferromark 2.0.0-rc.1`; at `bffc89f6` the package is `2.0.0-rc.2`, so
`prepare.py --lockfile` would fail on that one line exactly as it did last
time. This run therefore seeds the preceding report's lock through `--bun-lock`.
Everything that protects the comparison still holds:

- `CARGO_NET_OFFLINE=true` and `--offline`, so nothing could be fetched.
- `prepare.py` builds the allowed registry union from the v1, v2, OX and Bun
  locks and fails the build if Cargo resolves any registry package outside it
  or any checksum that disagrees. It did not fail.
- Checked independently afterwards: **all 67 registry packages are identical to
  the preceding report's lock in name, version and checksum** — none added,
  none removed. The entire lock delta is the v2 path package's version line,
  which carries no checksum and is audited as Git blobs instead.

The resolved lock is archived as [Cargo.lock](Cargo.lock) and is the one both
builds used.

## Measurement conditions

Both builds completed before any timing began, and no `rustc`, `cargo` or
`clang` process existed from then until the last run finished: every run was
gated on `pgrep -x` for all three. The three runs executed back to back from one
script with no other work on the machine.

| Run | Build | Documents | Start (UTC) | End | Load before | Load after |
| --- | --- | ---: | --- | --- | ---: | ---: |
| Broad | default | 57 | 13:23:17 | 13:34:22 | 4.49 | 4.56 |
| Held-out | default | 28 | 13:34:25 | 13:40:03 | 4.56 | 2.95 |
| Held-out | PGO | 28 | 13:40:07 | 13:45:43 | 2.96 | 1.80 |

The 1-minute load average fell from 4.49 to
1.80 over the session; it came from
background indexers, not from this work, and is lower than the preceding
report's 7.96 → 5.53. Absolute nanoseconds are therefore not comparable across
the two reports and this document does not compare them. The published ratios
are between engines measured in the same rotating windows of the same process,
which is what the load cancels out of; the objective check is that the three
process rounds agree to within 0.004 on every validation row, and that the
57-document run took 11m05s against the preceding report's 11m11s. The
comparison-engine absolute times reproduce the preceding run to within a few
percent, which is why the loss of v2's relative position is attributed to v2
and not to the host. Thermal telemetry is unavailable on this host; that is not
proof that no throttling occurred. Raw per-round windows are retained in
`samples.json.gz`.

## Reproduction

Use the existing source caches or run this report's `restore.py` in a new empty
temporary cache directory — it writes next to itself, so never run it inside the
repository. From the repository root:

```sh
python3 docs/reports/2026-09-21-native-release/harness/prepare.py /private/tmp/native-release-replay \
  --bun-source /private/tmp/native-replay-cache/bun \
  --bun-native-cache /private/tmp/native-replay-cache/native \
  --md4c-source /private/tmp/native-replay-cache/md4c \
  --ox-archive /private/tmp/native-replay-cache/ox.tar.gz \
  --ferromark-v1-source . --ferromark-v2-source . \
  --ferromark-v2-revision bffc89f6 \
  --worker docs/reports/2026-09-21-native-release/harness/worker.rs \
  --bun-lock docs/reports/2026-09-21-native-release/Cargo.lock --compile
python3 -m unittest discover -s docs/reports/2026-09-21-native-release/harness -p 'test_*.py'
python3 docs/reports/2026-09-21-native-release/harness/run.py \
  /private/tmp/native-release-replay/build.json \
  docs/reports/2026-09-21-native-release/corpus.json.gz \
  /private/tmp/native-release-replay-results \
  --rounds 3 --samples 6 --window-ms 40 --warmup-ms 60
python3 docs/reports/2026-09-21-native-release/harness/report.py /private/tmp/native-release-replay-results
```

The PGO pair adds the training corpus and the round-3 split, and measures both
executables on the held-out half only:

```sh
python3 benchmarks/optimization-rounds/make_corpus.py \
  /private/tmp/native-release-training-corpus.json.gz --include-scanner-diagnostics
python3 docs/reports/2026-09-21-native-release/harness/prepare.py /private/tmp/native-release-pgo-replay \
  ... same source arguments ... --compile \
  --bun-lock docs/reports/2026-09-21-native-release/Cargo.lock \
  --pgo --pgo-training-corpus /private/tmp/native-release-training-corpus.json.gz \
  --pgo-training-filter docs/reports/2026-09-16-arm-round-3/harness/filter-broad-train.txt \
  --pgo-measurement-filter docs/reports/2026-09-16-arm-round-3/harness/filter-broad-test.txt
# then run.py each build with
#   --filter-file docs/reports/2026-09-16-arm-round-3/harness/filter-broad-test.txt
```

The training corpus generated here has SHA-256
`272c832aa36774bb337cc607aff552d7ddfe7efe190083454a6c276cf735a089` (150 cases),
byte-identical to the preceding report's; 74 of them are selected for training
and the harness asserts that none of them is in the measured half.
`make_corpus.py` regenerates it from the repository.

Engine sources and corpus inputs retain their original licenses and attribution.
This is Bun's native Markdown core, not its JavaScript API.
