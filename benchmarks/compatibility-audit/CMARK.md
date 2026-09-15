# Offline cmark differential oracle

This bounded harness compares the Ferromark compatibility worker with pinned
official C reference implementations. It retains every raw output and classifies
pairs with the existing conservative comparator as `exact`,
`serialization-equivalent`, `heading-id-only`, or `other`. Only the first two
are suitable for strict aggregation; the harness never replaces the vendored
CommonMark or GFM specification fixtures.

The checked-in corpus has 106 deterministic cases (82 CommonMark and 24 GFM),
including nested block/inline combinations, the six requested tilde binding
reproductions, line endings, Unicode/NUL, raw HTML, tables, tasks, and
autolinks. `cmark_fixtures.py --check` verifies that
`cmark-fixtures.json` exactly matches its generator.

## Pinned sources and offline build

The source trees must already exist locally and be clean at these exact
revisions:

```text
commonmark/cmark       bb3678d7a73cb02d35c8876ecd097072636200a8  (0.31.1)
github/cmark-gfm       587a12bb54d95ac37241377e6ddc93ea0e45439b  (0.29.0.gfm.13)
```

Build them into a new temporary directory without installing a system package:

```sh
python3 benchmarks/compatibility-audit/cmark_build.py \
  --cmark-source /private/tmp/ferromark-cmark-0311 \
  --cmark-gfm-source /private/tmp/ferromark-cmark-gfm-13 \
  --build-root /private/tmp/ferromark-v2-correctness-fixes/cmark-build-final \
  --metadata /private/tmp/ferromark-v2-correctness-fixes/cmark-build-final/metadata.json
```

The builder verifies Git revisions and clean trees, records the exact CMake
configure/build commands, actual configured compiler versions, binary SHA-256
values, and logs. It uses `cmake` from `PATH`; alternatively pass
`--cmake /path/to/cmake`. The recorded run used an existing temporary CMake at
`/private/tmp/ferromark-cmark-tools/lib/python3.14/site-packages/cmake/data/bin/cmake`.
No reference source or binary is copied into the Rust workspace.

## Reproduction

The worker metadata must describe the exact worker binary being tested. The
runner verifies its SHA-256 before processing any input. Reference binary
paths and hashes must likewise come from `cmark_build.py` metadata.
Generate a fresh worker and its metadata with the [spec audit commands](README.md)
when testing another checkout. Each oracle input runs in a separate worker
process with a ten-second deadline, so crashes and partial responses cannot
hang the remaining corpus. This is a correctness comparison, not a benchmark.

```sh
python3 benchmarks/compatibility-audit/cmark_fixtures.py --check
python3 benchmarks/compatibility-audit/cmark_oracle.py \
  --worker /private/tmp/ferromark-v2-compat-audit/repro-build/target/release/compatibility-audit-worker \
  --worker-metadata /private/tmp/ferromark-v2-correctness-fixes/audit-final/metadata.json \
  --cmark /private/tmp/ferromark-v2-correctness-fixes/cmark-build-final/cmark/src/cmark \
  --cmark-gfm /private/tmp/ferromark-v2-correctness-fixes/cmark-build-final/cmark-gfm/src/cmark-gfm \
  --cmark-build-metadata /private/tmp/ferromark-v2-correctness-fixes/cmark-build-final/metadata.json \
  --output /private/tmp/ferromark-v2-correctness-fixes/cmark-oracle-final \
  --fail-on-differences
```

Without `--fail-on-differences`, exit 0 means the diagnostic run completed;
it does not mean outputs agree. Errors return 2. Strict mode returns 1 for
any difference beyond admitted serialization, including heading IDs. The
first correction batch returned 1: six heading-ID-only differences and seven
other cases remained. The [second correction batch](../../docs/reports/2026-09-14-reference-compatibility/README.md)
records the fixes and explicit profiles; the historical results remain unchanged.

CommonMark runs cmark with `--unsafe` so raw HTML matches the configured worker.
GFM also uses `--unsafe` and enables `table`, `strikethrough`, `autolink`,
`tasklist`, and `tagfilter`,
while omitting footnotes to match the worker profile. cmark-gfm predates the
current GFM website, so GFM differences are diagnostic candidates rather than
a current website oracle. Heading IDs, raw HTML policy, Unicode URL spelling,
and all other differences remain visible in `results.json`.

Primary provenance: [commonmark/cmark](https://github.com/commonmark/cmark),
[cmark 0.31.1 release](https://github.com/commonmark/cmark/releases), and
[github/cmark-gfm](https://github.com/github/cmark-gfm).

## Extended delimiter corpus

Generate 256 deterministic combinations, then repeat the oracle command with
`--fixtures /tmp/cmark-bindings.json` and a fresh output directory:

```sh
python3 benchmarks/compatibility-audit/cmark_binding_fixtures.py /tmp/cmark-bindings.json
```

The combinations cross single/double tildes and 32 inline shapes with paragraph,
emphasis, link-label, and block-quote contexts. They include link destinations
and titles, code spans, HTML attributes, escaped tildes, nested emphasis, and
whitespace boundaries. These authored inputs are MIT licensed with the repo.
The original 106 outputs and 254 additional reference outputs are asserted
by `cargo test -p ferromark_renderer --test cmark_regressions --locked`, without
requiring a C toolchain during ordinary workspace tests. Two additional cases
assert the GFM prohibition of nested links instead of the pinned oracle's
invalid nested anchors. They remain `other` in the raw 256-case comparison,
so its generic strict run intentionally returns 1; the 106-case strict run
returns 0. The separate live
oracle still verifies the exact pinned reference binaries.
