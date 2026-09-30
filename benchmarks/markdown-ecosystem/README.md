# Markdown ecosystem comparison

Independent Markdown-to-HTML pairs extend the feature inventory:

| Track | Engines | Output and lifecycle |
| --- | --- | --- |
| Native | Local Ferromark v2 and `markdown` 1.0.0 (markdown-rs) | Owned UTF-8 HTML; Ferromark fresh/reused arena and renderer, markdown-rs fresh public HTML call in both modes |
| Node | Local Ferromark Node bindings and micromark 4.0.3 | Public APIs returning JavaScript strings; Ferromark `toHtml` / reusable `Renderer.toHtml`, micromark fresh public call in both modes |

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
before/after each process's timing windows. All raw HTML is retained. New scores
include every one of the 57 frozen documents, including different outputs.
Exact or conservative serialization-equivalent output counts are descriptive
metadata, not a scoring filter or a conformance gate; heading IDs and code
whitespace are never normalized away. Different syntax and rendering features
remain documented alongside performance. Historical schema-1/2 reports retain
their original equivalent-output scoring.

The result archives retain inputs, outputs, behavior guards, every timing
window, lockfiles, worker/addon hashes, source hashes, runtime, local Git status,
and host observations. Aggregation uses the median of each process round's
medians, then an equal-document geometric mean of competitor time / Ferromark
time on all 57 documents. Values above 1 mean higher Ferromark throughput.
Schema-3 pairs retain the committed [scoring policy](scoring-policy.json), whose
hash and source revision prevent downgrading to historical matched-only scoring.
See the [decision](../../docs/decisions/2026-09-30-benchmark-performance-scope.md).
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
  plugin classes remain in verification and do not exclude documents from scoring.
- **remark 15.0.1:** frozen processor with remark-rehype 11.1.2 and
  rehype-stringify 10.0.1; `processSync` returns the HTML string. Three pinned
  micromark/mdast extensions enable tables, strikethrough, and task lists.
  Raw HTML passes through without an extra HTML parsing/sanitizing stage.
- **Showdown 2.1.0:** retained Converter and public `makeHtml`; heading IDs,
  ellipsis conversion, metadata, literal autolinking, and renderer extras off.
  Tables, strikethrough, and task lists follow the profile. Its Markdown dialect
  and styled task markup remain unchanged; every input contributes to scoring.
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

The [Linux workflow](../../.github/workflows/markdown-ecosystem.yml) runs all eighteen
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

## Expanded public APIs

The complete [campaign inventory](comparisons.json) retains 23 executable rows. The default
campaign and homepage show 20 (16 independent pairs plus the original native harness's four displayed engines). The extended scope adds markdown-exit, markdown-it-ts,
and MD4X WASM. Their adapters and pins remain available.
The [manual workflow](../manual-comparison/README.md) is the entry point for a new
complete campaign. Existing archives retain their original versions and scope.

| Adapter | Pin and complete public call | Effective contract |
| --- | --- | --- |
| Goldmark | Go module v2.1.6; `parser.Parse` then `html.Render` into a fresh bytes.Buffer | Native Go 1.27.1, CGO off, native generic target; raw HTML enabled; only table, strike, and task extensions, without Linkify. Configuration retained; output allocation and Go GC included, GOMAXPROCS=1, GOGC=100, no memory limit. |
| Remarkable | npm 2.0.1; retained Remarkable `render` | CommonMark preset on all 57 inputs in both engines. Its default bundle lacks task lists; no custom task plugin is invented. HTML on, typography/break conversion off. |
| markdown-exit | npm 1.3.0; retained parser `render` | CommonMark preset plus table/strike rules and pinned task-list plugin in the shared profile; `<s>` and plugin classes remain in verification. |
| markdown-it-ts | npm 1.1.2; ordinary full-document `render` | Same shared rules/plugin; streaming and chunked fallback explicitly disabled. Public repeated-input caches are not flushed inside timing; alternating-document guards and rotating controls remain mandatory. |
| Sätteri | npm 0.10.5; `markdownToHtml(...).html` | Native Node pipeline without MDX/plugins. Other extensions off. Public GFM also enables literal autolinks; there is no individual public switch, so the difference remains. `rawHtml: false` disables the extra AST conversion feature, while ordinary raw HTML remains verified. |
| MD4X NAPI | npm 0.0.30; explicit `md4x/napi` init then `renderToHtml` | Native Node addon with no WASM fallback. `headingIds`, `full`, `heal` off. Public API has no parser-feature flags; tables, strike, tasks, autolinks, footnotes, callouts, and frontmatter extraction remain enabled even for CommonMark-profile inputs. |
| MD4X WASM | npm 0.0.30; explicit `md4x/wasm` init with installed WASM bytes then `renderToHtml` | WASM initialization is untimed; identical public parser contract, separately timed JS/WASM boundary. No NAPI fallback. |
| TanStack Markdown | @tanstack/markdown 0.0.16; public `renderHtml(source, options)` from `/html` | Trusted raw HTML and unchanged URLs; heading IDs/anchors, frontmatter, highlighting, docs/AI extensions off. No public switches for tables, strike, tasks, footnotes, or code wrapper attributes. Deliberate syntax subset; differences are disclosed, and every input contributes to performance scoring. |
| OX-Content Node | @ox-content/napi 3.2.13; `parseAndRender(...).html` | Individual parser flags enable only shared extensions. Public renderer heading IDs, TOC, callouts, fence metadata handling, and literal URL transformations cannot be disabled. Complete public API cost is timed, including returned metadata. |

Every pair retains the manifest and contract code hashes. `contracts.py` asserts
these actual differences and keeps raw guard HTML; it never rewrites competitor
output. The v2 worker continues using the frozen input profile (CommonMark only
for cmark/commonmark.js/Remarkable). All 57 inputs contribute to every new factor.
A `*` beside a new homepage factor means some outputs differ. The report retains
actual agreement counts and API contracts; this compares public API performance
on the same inputs, without claiming identical feature sets or conformance.

New runs also retain `rotating-controls.json`: all 57 inputs grouped by effective
profile, both lifecycles, three independent process rounds, six alternating
40 ms windows after 60 ms warmup. The output is checked against per-document
verification before and after timing. Controls include disagreements for diagnosis
and do not enter per-document factors. Single-document timing may include a
library's repeated-input cache; review rotating controls before making claims
about parsing work. Full publication rejects missing/shortened controls.

The homepage exposes the new rows with GitHub links and backend labels, but
shows unmeasured cells until a complete validated run is imported. Dependency
installation and local smoke checks do not update official figures.
