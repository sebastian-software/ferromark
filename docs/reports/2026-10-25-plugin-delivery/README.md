# Native plugin delivery and performance

This follow-up fixes the repository and Rust-package CI failures after #567,
adds installable custom plugin bundles, and characterizes the installed addon.
The engine remains pinned to published Ferrigura 0.2.0: crates.io and upstream
main still report that version. The separate label-parser replacement has no
published release to adopt yet.

## Delivery and validation

- Release Please's name-only lockfile updater changes registry Ferromark when
  it bumps the workspace package. The production correction now preserves base
  registry blocks and updates qualified local references. Tests exercise the
  real upstream updater and the actual correction CLI, plus five coordinated
  release transitions validated by Cargo and pnpm.
- The workspace contract includes the private adapter and fixture. Rust package
  rehearsal resolves its temporary core patch in a disposable workspace before
  packaging with `--locked`; both extracted archives compile their own targets
  and an isolated consumer runs successfully. Failed commands expose log tails.
- All four plugin selections install from their tarball in a clean project,
  offline, without install scripts or Rust. The package includes one platform
  binary, matching declarations, source/binary/archive hashes, and dependency
  notices including Ferrigura font attribution. Registry publishing is disabled
  on custom tarballs; standard package distribution and dependencies are preserved.
- Local checks pass: 1,429 Rust tests in 103 suites, 222 Node tests, 88 repository
  contract tests, 54 Python tests, workspace Clippy/format/benchmark builds,
  strict rustdoc, engine-free Clippy, Node lint/types/format, packaging, and clean
  installation. Audit has no high/critical findings and one existing moderate.

The [bundle workflow](../../../.github/workflows/plugin-bundles.yml) builds the
seven existing targets and tests native installs where a matching host exists,
including x64 Alpine musl. ARM64 musl remains cross-built without a native
runtime test. The [guide](../../plugin-bundles.md) describes installation.

## Measured boundary

Apple M1 Ultra, macOS arm64, Node.js 24.19.0, pinned Rust 1.95.0,
`release-node`, PGO disabled. Measurements run after compilation stops, on a
shared developer host without thermal or power controls. The installed bundle
is loaded in a fresh process for each case and repetition. Each process warms
five renders and records eleven batches of at least 20 ms. Three processes run
per case, rotating/reversing case order. GC runs outside timed batches.

The ordinary fixture is 2,498 bytes with 20 headings and 20 non-Mermaid code
blocks. Its HTML is checked equal across the regular API, an empty compiler,
and an initialized but unused Ferrigura plugin. Diagram fixtures separately
characterize a four-node flowchart, a 31-node chain, and Markdown with three
diagrams. They are different work from rendering plain code; no speed ratio
between those outputs is claimed.

## HTML-only metadata optimization

`CodeCompiler.toHtml()` previously called `transform()` and collected heading
and frontmatter metadata that its caller never received. It now skips that
collection while `transform()` retains it. Output parity is tested with heading
prefixes/offsets, typography, frontmatter, and footnotes.

Warm time per ordinary document, in microseconds. Ranges are the minimum and
maximum of the three process medians, not confidence intervals:

| HTML path, Ferrigura-capable binary | Before | After |
| --- | ---: | ---: |
| Regular `toHtml` | 8.79 (8.46–9.01) | 8.51 (8.51–8.57) |
| Empty `CodeCompiler` | 13.68 (13.41–14.59) | 9.61 (9.56–9.96) |
| Compiler with unused Ferrigura | 14.06 (13.56–14.87) | 9.71 (9.66–9.81) |

The compiler path improves on this fixture but still costs about 1.2 µs more
than the regular API. It is not universally performance-neutral. JSX was not
changed by this optimization; ordinary JSX remains around 139 µs here.

## Diagram rendering and memory

Final Ferrigura-only bundle, reused compiler. Times include Markdown parsing,
engine rendering, output allocation, and for JSX, SVG conversion:

| Fixture | Warm HTML, ms | Warm JSX, ms | HTML / JSX process RSS, MiB |
| --- | ---: | ---: | ---: |
| Small flowchart | 0.270 | 0.385 | 60.3 / 60.3 |
| 31-node flowchart | 1.441 | 1.786 | 60.7 / 60.8 |
| Markdown and three diagrams | 0.651 | 1.072 | 60.6 / 60.9 |

Ferrigura initialization is about 1.1–1.3 ms in these runs. Ordinary Markdown
with an unused plugin has around 60.8 MiB HTML process RSS, compared with
58.8 MiB for the empty compiler in the same binary. These are whole-process
values including Node/V8, returned strings, native code and caches, not Rust
heap attribution or an isolated engine allocation measurement. Peak RSS and
phase snapshots, first render and addon-load times are retained in the raw
results. This is not a leak, capacity, or cross-platform benchmark.

## Payload

Local macOS arm64 builds, bytes. Compressed tarballs also include facade,
declarations, provenance, and notices:

| Plugins | Native binary | Installable `.tgz` |
| --- | ---: | ---: |
| None | 2,436,208 | 1,238,668 |
| Ferriki | 5,848,144 | 3,046,074 |
| Ferrigura | 15,434,624 | 7,773,867 |
| Both | 18,531,856 | 9,427,833 |

The optional engine/font/layout payload is substantial, and remains outside
the standard addon. Binary size does not predict process RSS or render time.

## Evidence

[Before](before.json.gz), [engine-free](engine-free.json.gz), and
[final Ferrigura](ferrigura.json.gz) results contain every batch, environment,
fixture hashes, and measured binary/archive identities. The before binary uses
the runtime from merged #567; only tooling was being developed at that point.
Its bundle predates the explicit compiler metadata field; the pinned workspace
compiler is recorded by the harness rather than the temporary consumer's
global toolchain.

[Readable summaries](summary.json), [bundles](bundles.json), [release rehearsal](release-rehearsal.json),
[Rust archives](rust-packages.json), compressed logs,
[source hashes and validation](validation.json), and `SHA256SUMS` retain the
remaining evidence. No registry credentials or uploads were exercised, and
other-platform workflow execution is separate from this local report.
