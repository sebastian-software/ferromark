# Source and build provenance

The measured v2 core is `7c887a2b` — the head of the segmented-definition-pass
branch (PR #339), nine code commits on top of `main` `e35e9f64` plus its report.
Ferromark v1 stays at `4e151415a15c67e9d3735f719b0bed3e25d8cff8`, OX at
`a71a58939ffe7f154117cea026f6d6e71a139393`, md4c at
`65c6c9d72cebd9a731aaa5597414ce04d9ea5de3`, Bun at
`76e9dcc6ad272a4fb1ee4a4dbbde4809201b71d1`, and pulldown-cmark at 0.13.4.
**These are the same five comparison pins as the preceding report**; only v2
moved. This reruns pinned versions; it does not claim to compare every library's
latest release.

The preceding report measured v2 at `c232d97debe24251a026d1bbc79725babe3da601`.
**137 commits separate that revision from `7c887a2b`, 44 of them `perf`**, so the
difference between the two reports is not the definition pass alone. The README
sets out what may and may not be attributed to the nine commits under review.

## Source audit

`harness/audit_sources.py` checks the exported sources against Git blobs or the
checksummed OX archive: 35 v1 files, 3,495 Bun files, 10 md4c files, 393 OX
files, and both native-support archives. **No mismatches.**

That script reports only 2 files for v2, and that is a coverage gap at this
revision rather than a result: its v2 path list is `crates`, and `23a212d8` /
`885e5b1c` consolidated the Rust components into the repository root package,
so only `Cargo.toml` and `Cargo.lock` matched. `audit_v2_sources.py` in this
directory runs the same `audit_git` helper over the paths v2 actually uses
(`src`, `Cargo.toml`, `Cargo.lock`) and verifies **182 v2 files with no
mismatches** ([source-audit-v2.json](source-audit-v2.json)). The harness script
is unmodified; its own output is archived unchanged as
[source-audit.json.gz](source-audit.json.gz).

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
- Final Cargo lock SHA-256: `14077badf244f9ba1de879d369f6418192286a4de3c4954cd628c1b859b64c2a`,
  shared by both builds.
- Default executable SHA-256: `82ba3edc500d9fd616df081cf80d867ca6e8ed0c5c686790ead58a5042479a69`.
- PGO executable SHA-256: `7eb9af2f4b3fefb32672582731a84641e3683301b152b5b57169ae213a699754`.
- Merged profile SHA-256: `85500d772c7f13a6a9db7742cd701c4905d515fb79a2674a788733fcca1add93`
  (900,120 bytes, 24 `.profraw` inputs — six engines × two profiles × two lifecycles).

### The archived lock could not be replayed verbatim, and why that is not a drift

The preceding report's `Cargo.lock` names five v2 path packages
(`ferromark` 2.0.0-dev.0 plus `ferromark_allocator`, `_ast`, `_parser`,
`_renderer`). At `7c887a2b` those are one package, `ferromark` 2.0.0-rc.1, so
`prepare.py --lockfile` fails: `--locked` forbids the one line Cargo must add.

This run therefore seeds the same archived lock through `--bun-lock`, which is
byte-identical seeding without `--locked`. Everything that protects the
comparison still holds:

- `CARGO_NET_OFFLINE=true` and `--offline`, so nothing could be fetched.
- `prepare.py` builds the allowed registry union from the v1, v2, OX and Bun
  locks and **fails the build** if Cargo resolves any registry package outside
  it, or any checksum that disagrees (`prepare.py` lines 798–802). It did not
  fail.
- Checked independently afterwards: **all 67 registry packages are identical to
  the preceding report's lock in name, version and checksum** — none added, none
  removed. The entire lock delta is the five v2 path packages collapsing into
  one, which carry no checksum and are audited as Git blobs instead.

The resolved lock is archived as [Cargo.lock](Cargo.lock) and is the one both
builds used.

## Measurement conditions

Both builds completed before any timing began, and no `rustc` or `cargo` process
existed from then until the last run finished. The three runs executed back to
back from one script with no other work on the machine.

| Run | Build | Documents | Start (UTC) | End | Load before | Load after |
| --- | --- | ---: | --- | --- | ---: | ---: |
| Broad | default | 57 | 14:05:58 | 14:17:09 | 7.96 | 6.84 |
| Held-out | default | 28 | 14:17:09 | 14:22:50 | 6.84 | 7.84 |
| Held-out | PGO | 28 | 14:22:51 | 14:28:31 | 7.84 | 5.53 |

The load came from a macOS Spotlight reindex, not from this work. It is higher
than the preceding report's 2.69 → 4.54, so absolute nanoseconds are not
comparable across the two reports and this document does not compare them. The
published ratios are between engines measured in the same rotating windows of
the same process, which is what the load cancels out of; the objective check is
that the three process rounds agree to within 0.009 on every validation row, and
that the 57-document run took 11m11s against the preceding report's 11m06s.
Thermal telemetry is unavailable on this host; that is not proof that no
throttling occurred. Raw per-round windows are retained in `samples.json.gz`.

## Reproduction

Use the existing source caches or run this report's `restore.py` in a new empty
temporary cache directory — it writes next to itself, so never run it inside the
repository. From the repository root:

```sh
python3 docs/reports/2026-09-16-native-segments/harness/prepare.py /private/tmp/native-segments-replay \
  --bun-source /private/tmp/native-replay-cache/bun \
  --bun-native-cache /private/tmp/native-replay-cache/native \
  --md4c-source /private/tmp/native-replay-cache/md4c \
  --ox-archive /private/tmp/native-replay-cache/ox.tar.gz \
  --ferromark-v1-source . --ferromark-v2-source . \
  --ferromark-v2-revision 7c887a2b \
  --worker docs/reports/2026-09-16-native-segments/harness/worker.rs \
  --bun-lock docs/reports/2026-09-16-native-segments/Cargo.lock --compile
python3 -m unittest discover -s docs/reports/2026-09-16-native-segments/harness -p 'test_*.py'
python3 docs/reports/2026-09-16-native-segments/harness/run.py \
  /private/tmp/native-segments-replay/build.json \
  docs/reports/2026-09-16-native-segments/corpus.json.gz \
  /private/tmp/native-segments-replay-results \
  --rounds 3 --samples 6 --window-ms 40 --warmup-ms 60
python3 docs/reports/2026-09-16-native-segments/harness/report.py /private/tmp/native-segments-replay-results
```

The PGO pair adds the training corpus and the round-3 split, and measures both
executables on the held-out half only:

```sh
python3 benchmarks/optimization-rounds/make_corpus.py \
  /private/tmp/native-segments-training-corpus.json.gz --include-scanner-diagnostics
python3 docs/reports/2026-09-16-native-segments/harness/prepare.py /private/tmp/native-segments-pgo-replay \
  ... same source arguments ... --compile \
  --bun-lock docs/reports/2026-09-16-native-segments/Cargo.lock \
  --pgo --pgo-training-corpus /private/tmp/native-segments-training-corpus.json.gz \
  --pgo-training-filter docs/reports/2026-09-16-arm-round-3/harness/filter-broad-train.txt \
  --pgo-measurement-filter docs/reports/2026-09-16-arm-round-3/harness/filter-broad-test.txt
# then run.py each build with
#   --filter-file docs/reports/2026-09-16-arm-round-3/harness/filter-broad-test.txt
```

The training corpus generated here has SHA-256
`272c832aa36774bb337cc607aff552d7ddfe7efe190083454a6c276cf735a089` (150 cases);
74 of them are selected for training and the harness asserts that none of them
is in the measured half. `make_corpus.py` regenerates it from the repository.

The shared repository contains both Ferromark histories. Supply the pinned
commits and new build/result paths; preparation never edits the source checkouts.
Engine sources and corpus inputs retain their original licenses and attribution.
This is Bun's native Markdown core, not its JavaScript API.
