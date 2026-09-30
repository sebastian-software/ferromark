# Markdown ecosystem comparison — 2026-09-30

## markdown-rs and micromark

Two separate pairs extend the frozen 57-document corpus comparison. These are
new measurements of the recorded main revision, with no core changes. Their runtime and
allocator settings differ from the archived six-engine comparison; the results are not pooled. **Values below are Ferromark throughput
relative to the competitor: higher is faster.**

| Track | Competitor | Agreeing documents | Fresh | Reuse |
| --- | --- | ---: | ---: | ---: |
| Native | markdown-rs 1.0.0 | 57/57 | 68.03× | 75.39× |
| Node | micromark 4.0.2 | 56/57 | 81.19× | 123.83× |

The native pair uses rustc 1.95.0 (59807616e 2026-04-14), a shared system allocator,
optimization level 3, fat LTO, one codegen unit, and a generic CPU target.
The Node pair uses Node v24.21.0, Ferromark's local addon built
with the release-node profile without PGO, and both public JavaScript string APIs.
It includes wrapper/N-API conversion costs and JavaScript GC. The Node result
describes application calls, not isolated language overhead.

CommonMark disables optional syntax. The extension lane enables only tables,
strikethrough and task lists, with raw HTML passthrough. It is not full GFM;
MDX, math, frontmatter, bare URL autolinking, footnotes, heading IDs and other
renderer extras are off. The competitors expose no retained parser/output API:
their reuse column retains only configuration and makes fresh HTML calls.

Every input is checked in both lifecycles before timing and again around timed
windows. The score uses only exact or serialization-equivalent HTML pairs.
Micromark differs on one Vite document's code-block line breaks; that document
remains a measured diagnostic and is excluded from the score. Code whitespace
is not normalized away. Three process rounds with six rotating engine-order
windows per round use equal-document geometric means. These are measurements
on a shared macOS arm64 workstation, not a universal ranking or a significance
claim. MDX and AST capabilities belong in the separate feature inventory.

[Raw results, source hashes, and output differences](https://github.com/sebastian-software/ferromark/blob/main/docs/reports/2026-09-30-markdown-ecosystem-main/README.md),
[reproduction commands](https://github.com/sebastian-software/ferromark/blob/main/benchmarks/markdown-ecosystem/README.md).

## Evidence

- Native: [run metadata](native/run.json), [aggregate](native/summary.json),
  [outputs](native/verification.json.gz), [windows](native/samples.json.gz),
  [option guards](native/behavior.json.gz), [corpus](native/corpus.json.gz),
  [registry lock](native/Cargo.lock).
- Node: [run metadata](node/run.json), [aggregate](node/summary.json),
  [outputs](node/verification.json.gz), [windows](node/samples.json.gz),
  [option guards](node/behavior.json.gz), [corpus](node/corpus.json.gz),
  [npm lock](node/package-lock.json).

The retained corpus includes every original input, byte length and SHA-256.
The run metadata records the binary/addon hashes, the measured core commit,
local Git status and source hashes. The local native and Node builds are separate from published PGO release artifacts.
Input attribution remains in the [broad corpus](../../../benchmarks/broad-comparison/README.md).

Regenerate this report and website section with
`python3 benchmarks/markdown-ecosystem/publish.py`; use `--check` to verify them.
The older native report and homepage headline numbers remain historical evidence.
