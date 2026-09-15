# Source and build provenance

The measured v2 core is `c232d97debe24251a026d1bbc79725babe3da601` on `codex/v2`, including
the document-wide footnote fix and completed parser/renderer refactors.
Ferromark v1 stays at `4e151415a15c67e9d3735f719b0bed3e25d8cff8`, OX at
`a71a58939ffe7f154117cea026f6d6e71a139393`, md4c at
`65c6c9d72cebd9a731aaa5597414ce04d9ea5de3`, Bun at
`76e9dcc6ad272a4fb1ee4a4dbbde4809201b71d1`, and pulldown-cmark at 0.13.4.
These are the same comparison pins as the preceding ARM report. This reruns
pinned versions; it does not claim to compare every library's latest release.

All engines were rebuilt into one executable, with the original adapters and
engine options. The source audit checks 35 v1 files, 389 v2 files, 3,495 Bun
files, 10 md4c files, and 393 OX files against Git blobs or the checksummed OX
archive. It also checks the native-support archives. There are no mismatches.
The 342 HTML outputs and all agreement classifications match the preceding
ARM report byte for byte. No production parser/renderer changes were made for
this measurement.

## Build conditions

- Shared `nightly-2026-07-20` compiler, required by the native Bun integration.
  Product tests separately use the supported MSRV and stable compilers.
- Generic AArch64, optimization level 3, fat LTO, one codegen unit, panic abort,
  and Bun's line-table debug information.
- Original native mimalloc for Rust and redirected md4c C allocations.
- Same registry versions and checksums as the preceding ARM report; no registry
  dependency was added or removed. Build ran offline from the old comparison
  lock and checked the resolved dependency identities against its allowed union.
- Final Cargo lock SHA-256: `c207bd5aa59eed548e98b03907fe01071480050fed0fe2869b47b58386910040`.
- Executable SHA-256: `466e2662a6ad717a3975d56945c0431a616935a24649cf143cbf2c7e8cf3e5c8`.
- Native-support archive checksums are unchanged. Compiled native library hashes
  are recorded for this rebuild, rather than assumed identical across builds.

Fresh/reuse lifecycles and timing loops are unchanged. Host probes and round
ordering are recorded in `run.json`; unavailable thermal telemetry is not proof
that no throttling occurred. No local builds, tests, or archive compression run
during the measurements. Raw samples preserve all process rounds.

## Reproduction

Use the existing source caches or run this report's `restore.py` in a new
empty temporary cache directory. From the repository root:

```sh
python3 docs/reports/2026-09-15-release-native/harness/prepare.py /private/tmp/native-release-replay \
  --bun-source /private/tmp/native-replay-cache/bun \
  --bun-native-cache /private/tmp/native-replay-cache/native \
  --md4c-source /private/tmp/native-replay-cache/md4c \
  --ox-archive /private/tmp/native-replay-cache/ox.tar.gz \
  --ferromark-v1-source . --ferromark-v2-source . \
  --ferromark-v2-revision c232d97debe24251a026d1bbc79725babe3da601 \
  --worker docs/reports/2026-09-15-release-native/harness/worker.rs \
  --lockfile docs/reports/2026-09-15-release-native/Cargo.lock --compile
python3 docs/reports/2026-09-15-release-native/harness/run.py \
  /private/tmp/native-release-replay/build.json \
  docs/reports/2026-09-15-release-native/corpus.json.gz /private/tmp/native-release-replay-results
python3 docs/reports/2026-09-15-release-native/harness/report.py /private/tmp/native-release-replay-results
```

The shared repository contains both Ferromark histories. Supply the pinned
commits and new build/result paths; preparation never edits the source checkouts.
Engine sources and corpus inputs retain their original licenses and attribution.
This is Bun's native Markdown core, not its JavaScript API.
