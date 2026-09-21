# Source and build provenance

The measured v2 core is `60602a5a78babff67d47b1dcf799d3ea1ca415f2` — the
release head `bffc89f6` plus the one commit that repairs its regression
([decision record](../../decisions/2026-09-21-lazy-tracker-cost.md)). The
[preceding run](../2026-09-21-native-release-fixed/README.md) measured `bffc89f6`
itself; this report's comparison baseline is the run before that,
[2026-09-16-native-segments](../2026-09-16-native-segments/README.md) at
`7c887a2b`, because the question is whether the fix restored that position.
Ferromark v1 stays at `4e151415a15c67e9d3735f719b0bed3e25d8cff8`, OX at
`a71a58939ffe7f154117cea026f6d6e71a139393`, md4c at
`65c6c9d72cebd9a731aaa5597414ce04d9ea5de3`, Bun at
`76e9dcc6ad272a4fb1ee4a4dbbde4809201b71d1`, and pulldown-cmark at 0.13.4.
**These are the same five comparison pins as the four preceding reports**;
only v2 moved. This reruns pinned versions; it does not claim to compare every
library's latest release.

Both Ferromark histories live in this repository, so `--ferromark-v1-source`
and `--ferromark-v2-source` both point at the repository root and `prepare.py`
exports the two pinned commits with `git archive`. Working-tree sources are
never built.

## Source caches

The comparison sources are the ones the preceding run restored a few hours
earlier with the archived `restore.py` (`restore.json`); the OX tarball, the
Bun mimalloc archive and the Highway archive carry the same SHA-256 values as
the original run.

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
report — and therefore the `7c887a2b` run — byte for byte. The measured
revision is the committed fix; no source was edited for this measurement.

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
- Default executable SHA-256: `f14fe577cc819f7ce24397972c2eb5cd2eb062b1c0d3bd5709b114f2c8a21890`.
- PGO executable SHA-256: `333592cf7eae675ab37eecf9ad52434ae7d111a278d338cfbdf2193240343785`.
- Merged profile SHA-256: `534fb8cef17078d2fc4a58941004b3b84d982a4545594e352c4fd8caabe0b229`
  (917,856 bytes, 24 `.profraw` inputs — six engines × two profiles × two lifecycles).

### The lock was seeded, not replayed with `--locked`

The `7c887a2b` report archived its lock with the v2 path package at
`ferromark 2.0.0-rc.1`; at `60602a5a` the package is `2.0.0-rc.2`, so
`prepare.py --lockfile` would fail on that one line exactly as it did for the
preceding run. This run therefore seeds that archived lock through
`--bun-lock`, as the preceding run did.
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
| Broad | default | 57 | 14:56:06 | 15:07:11 | 4.67 | 2.62 |
| Held-out | default | 28 | 15:07:14 | 15:12:52 | 2.65 | 3.06 |
| Held-out | PGO | 28 | 15:12:55 | 15:18:32 | 3.06 | 2.52 |

The 1-minute load average was 4.67 before the first run and
2.52 after the last, from background indexers, against 7.96 → 5.53
for the `7c887a2b` run this report compares positions with. Absolute
nanoseconds are therefore not comparable across the two reports and this
document does not compare them; the published ratios are between engines
measured in the same rotating windows of the same process. The objective
check is that the three process rounds agree to within 0.007 on every
validation row, and the 57-document run took 11m05s against 11m11s.
Thermal telemetry is unavailable on this host; that is not proof that no
throttling occurred. Raw per-round windows are retained in `samples.json.gz`.

## Reproduction

Use the existing source caches or run this report's `restore.py` in a new empty
temporary cache directory — it writes next to itself, so never run it inside the
repository. From the repository root:

```sh
python3 docs/reports/2026-09-21-native-release-fixed/harness/prepare.py /private/tmp/native-fixed-replay \
  --bun-source /private/tmp/native-replay-cache/bun \
  --bun-native-cache /private/tmp/native-replay-cache/native \
  --md4c-source /private/tmp/native-replay-cache/md4c \
  --ox-archive /private/tmp/native-replay-cache/ox.tar.gz \
  --ferromark-v1-source . --ferromark-v2-source . \
  --ferromark-v2-revision 60602a5a \
  --worker docs/reports/2026-09-21-native-release-fixed/harness/worker.rs \
  --bun-lock docs/reports/2026-09-21-native-release-fixed/Cargo.lock --compile
python3 -m unittest discover -s docs/reports/2026-09-21-native-release-fixed/harness -p 'test_*.py'
python3 docs/reports/2026-09-21-native-release-fixed/harness/run.py \
  /private/tmp/native-fixed-replay/build.json \
  docs/reports/2026-09-21-native-release-fixed/corpus.json.gz \
  /private/tmp/native-fixed-replay-results \
  --rounds 3 --samples 6 --window-ms 40 --warmup-ms 60
python3 docs/reports/2026-09-21-native-release-fixed/harness/report.py /private/tmp/native-fixed-replay-results
```

The PGO pair adds the training corpus and the round-3 split, and measures both
executables on the held-out half only:

```sh
python3 benchmarks/optimization-rounds/make_corpus.py \
  /private/tmp/native-fixed-training-corpus.json.gz --include-scanner-diagnostics
python3 docs/reports/2026-09-21-native-release-fixed/harness/prepare.py /private/tmp/native-fixed-pgo-replay \
  ... same source arguments ... --compile \
  --bun-lock docs/reports/2026-09-21-native-release-fixed/Cargo.lock \
  --pgo --pgo-training-corpus /private/tmp/native-fixed-training-corpus.json.gz \
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
