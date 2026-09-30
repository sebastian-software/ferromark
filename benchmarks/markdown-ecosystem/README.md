# markdown-rs and micromark comparison

Two independently measured Markdown-to-HTML pairs extend the feature inventory:

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
Micromark installs only the three required GFM syntax/HTML extensions. Both
competitors retain configuration, but expose no reusable parser/output API.

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
python3 -m unittest discover -s benchmarks/markdown-ecosystem -p 'test_*.py'
cargo fmt --manifest-path benchmarks/markdown-ecosystem/native/Cargo.toml --check
cargo clippy --manifest-path benchmarks/markdown-ecosystem/native/Cargo.toml \
  --locked -- -D warnings
```

Measurements of the recorded main revision are in
[the clean-core report](../../docs/reports/2026-09-30-markdown-ecosystem-main/README.md).
The publisher generates its website section and homepage JSON from the same
archived aggregates; `publish.py --check` rejects drift.
