# Markdown ecosystem comparison

Independent Markdown-to-HTML pairs extend the feature inventory:

| Track | Engines | Output and lifecycle |
| --- | --- | --- |
| Native | Local Ferromark v2 and `markdown` 1.0.0 (markdown-rs) | Owned UTF-8 HTML; Ferromark fresh/reused arena and renderer, markdown-rs fresh public HTML call in both modes |
| Node | Local Ferromark Node bindings and micromark 4.0.2 | Public APIs returning JavaScript strings; Ferromark `toHtml` / reusable `Renderer.toHtml`, micromark fresh public call in both modes |

These pairs use the same frozen 57-document broad corpus as the
[six-engine native harness](../native-comparison/README.md). They have their own
build/runtime contracts, so their numbers are not pooled with archived results.
Both native engines use the system allocator, one Rust compiler and release
recipe (optimization level 3, fat LTO, one codegen unit, generic CPU, aborting
panics). Node uses the local release-node addon (unwinding panics, no PGO) and
one Node runtime. The Node track includes the public wrapper and N-API string
conversion costs, together with JavaScript allocation and GC. It is not an
isolated measurement of language overhead.

The CommonMark profile disables optional syntax. The extension profile enables
only tables, strikethrough and task lists. It is not full GFM. Raw HTML and link
protocols pass through. Bare URL autolinking, tag filtering, footnotes, MDX,
frontmatter, math, heading IDs, callouts and other renderer extras remain off.
Micromark installs only the three required GFM syntax/HTML extensions. Competitors retain configured parser/processor objects in both lifecycle modes;
each timed call still parses and returns a fresh HTML string.

## Reproduce

From the repository root, with Rust 1.95 and Node 24:

```sh
npm ci --ignore-scripts --prefix benchmarks/markdown-ecosystem
RUSTFLAGS='-C target-cpu=generic' cargo build --release --locked \
  --manifest-path benchmarks/markdown-ecosystem/native/Cargo.toml
(cd node && pnpm install --frozen-lockfile && pnpm build:native)
python3 benchmarks/markdown-ecosystem/run.py native /private/tmp/markdown-native-run
python3 benchmarks/markdown-ecosystem/run.py node /private/tmp/markdown-node-run
```

The normal Node build has PGO disabled; leave `FERROMARK_PGO` unset. An equivalent
local macOS arm64 build is `cargo build -p ferromark-node --profile release-node
--locked`, followed by copying `target/release-node/libferromark_node.dylib` to
`node/ferromark/ferromark.darwin-arm64.node`. Other targets should use the package
build script to select the correct addon name.

Use `--verify-only` for output checks, or `--corpus` / `--binary` for explicit
inputs. Output directories must be new. Run the two timing tracks sequentially
on an otherwise quiet machine. Defaults are three process rounds, six rotating
engine-order windows of 40 ms, and 60 ms warmup per engine/document/lifecycle.
Select a Node competitor with `--competitor marked`, `markdown-it`, `remark`,
`showdown`, or `commonmark`; the default remains `micromark`.
commonmark.js uses CommonMark only on every input in both engines. The other
Node adapters use the per-document CommonMark or extension profile.

Native windows consume 32 complete document cycles before checking time; Node
windows check time after each cycle. Native consumption uses UTF-8 byte lengths;
Node consumption uses UTF-16 string lengths without a timed UTF-8 conversion.
Input file reads, process startup/imports, verification, and normalization are
outside the timed boundary. Native output destruction occurs in the timed call;
Node reclamation follows the runtime's GC schedule.

The shared native option guards exercise CommonMark, extension toggles, raw
HTML, reference isolation, literal code whitespace and disabled renderer extras.
Every document is verified in both lifecycles twice before timing, and again
before/after each process's timing windows. All raw HTML is retained. Only exact
or conservative serialization-equivalent output pairs enter the score; heading
IDs and code whitespace are never normalized away. All 57 documents are timed
and retained, including disagreements as diagnostics.

The result archives retain inputs, outputs, behavior guards, every timing
window, lockfiles, worker/addon hashes, source hashes, runtime, local Git status,
and host observations. Aggregation uses the median of each process round's
medians, then an equal-document geometric mean of competitor time / Ferromark
time on the agreeing set. Values above 1 mean higher Ferromark throughput.
Shared-workstation results do not establish statistical significance or a
universal ranking. Source attribution and licenses remain in the original
[broad corpus](../broad-comparison/README.md).

```sh
node --test benchmarks/markdown-ecosystem/test-node-adapters.mjs
python3 -m unittest discover -s benchmarks/markdown-ecosystem -p 'test_*.py'
cargo fmt --manifest-path benchmarks/markdown-ecosystem/native/Cargo.toml --check
cargo clippy --manifest-path benchmarks/markdown-ecosystem/native/Cargo.toml \
  --locked -- -D warnings
```

Measurements of the recorded main revision are in
[the clean-core report](../../docs/reports/2026-09-30-markdown-ecosystem-main/README.md).
The publisher generates its website section and homepage JSON from the same
archived aggregates; `publish.py --check` rejects drift.

## Node adapter contracts and project coverage

- **marked 18.0.14:** retained `Marked` instance, synchronous `parse`; its public
  URL tokenizer override disables bare-URL autolinking in the extension lane.
- **markdown-it 15.0.2:** CommonMark preset, HTML enabled, linkify and typographer
  off; enable table/strikethrough rules and markdown-it-task-lists 2.1.1 only in
  the extension lane. Trusted link protocols pass through. Its `<s>` tags and
  plugin classes remain in verification and may exclude documents from scoring.
- **remark 15.0.1:** frozen processor with remark-rehype 11.1.2 and
  rehype-stringify 10.0.1; `processSync` returns the HTML string. Three pinned
  micromark/mdast extensions enable tables, strikethrough, and task lists.
  Raw HTML passes through without an extra HTML parsing/sanitizing stage.
- **Showdown 2.1.0:** retained Converter and public `makeHtml`; heading IDs,
  ellipsis conversion, metadata, literal autolinking, and renderer extras off.
  Tables, strikethrough, and task lists follow the profile. Its Markdown dialect
  and styled task markup remain unchanged; only agreeing outputs enter scoring.
- **commonmark.js 0.31.2:** retained Parser and HtmlRenderer, smart punctuation
  and safe rendering off. It has no GFM extensions, so both engines parse the
  entire frozen input corpus with CommonMark only.

The selection includes direct HTML converters, plugin/token parsers, the
remark/unified AST pipeline, and the JavaScript CommonMark reference. The
[website feature guide](../../homepage/app/routes/guide/feature-comparison.mdx)
links these projects, React Markdown and MDX, and identifies native candidates
that have no archived throughput result yet.

Showdown's current published version has moderate npm advisories with no
registry fix. It is a benchmark-only dependency, receives frozen local inputs,
and is never shipped with Ferromark or its homepage. Metadata handling is off.
The lockfile is retained so this older project's measured implementation remains
identifiable rather than silently replaced.

## Retain the expanded Node runs

Run each pair sequentially, then verify and compress the raw results. This
archival command requires the original micromark metadata as a source/addon
baseline and rejects any change to those measured sources or addon binaries.
The expanded runs must use a committed adapter revision; the command retains
that revision's adapter sources and checks their recorded hashes.

```sh
for engine in marked markdown-it remark showdown commonmark; do
  python3 benchmarks/markdown-ecosystem/run.py node "/private/tmp/node-comparison-$engine" --competitor "$engine"
done
python3 benchmarks/markdown-ecosystem/archive-node.py --input-prefix /private/tmp/node-comparison-
python3 benchmarks/markdown-ecosystem/publish.py
```

## Complete native and Linux values

`prepare-native.py` builds Comrak 0.55.0 together with Ferromark and markdown-rs,
and two separate C API workers pinned to cmark 0.31.2 and cmark-gfm
0.29.0.gfm.13. The process dispatcher runs before the worker command loop; no
process startup or dispatcher code is timed. C libraries and adapters use clang
`-O3` and system malloc, with no PGO, allocator overrides, or cross-language LTO.
C HTML is consumed with `strlen` and freed inside timing. Rust owned HTML uses
its byte length and is destroyed inside timing. cmark runs CommonMark only in
both engines on all 57 documents. Comrak and cmark-gfm use the shared extension
subset. Competitors create parser/AST/output state per call in both schedules;
their reuse column retains configuration only.

```sh
python3 benchmarks/markdown-ecosystem/prepare-native.py /private/tmp/ecosystem-build
for engine in comrak cmark cmark-gfm; do
  python3 benchmarks/markdown-ecosystem/run.py native "/private/tmp/ecosystem-$engine" \
    --competitor "$engine" --binary /private/tmp/ecosystem-build/worker \
    --build-metadata /private/tmp/ecosystem-build/build.json
  python3 benchmarks/markdown-ecosystem/archive-results.py \
    "/private/tmp/ecosystem-$engine" "docs/reports/2026-09-30-ecosystem-platforms/macos-arm64/native-$engine"
done
ECOSYSTEM_BUILD=/private/tmp/ecosystem-build python3 -m unittest discover \
  -s benchmarks/markdown-ecosystem -p 'test_native_adapters.py'
```

The [Linux workflow](../../.github/workflows/markdown-ecosystem.yml) runs all ten
ecosystem pairs on GitHub-hosted Ubuntu 24.04 x86-64 VMs. Each pair alternates on
one VM; CPU models and hosts may differ between pairs. Install/build steps finish
before verification and timing. Download its per-pair artifacts, then run
`archive-results.py` for each raw directory and a `linux-x86-64/<track>-<engine>`
destination. It rejects incomplete/duplicate windows, altered core/adapter
provenance, invalid output agreement, consumption checksums, or aggregate drift.
Raw corpus, outputs, behavior guards, timing windows, locks, host metadata, and
committed adapters remain available per pair. This is independent steady-state
evidence, with no shared-host ranking or significance claim.

After all thirteen runs are archived, `publish_values.py` generates the new report
and homepage JSON. Run `publish.py` to refresh the benchmark guide; both support
`--check`. Earlier raw archives remain unchanged.
