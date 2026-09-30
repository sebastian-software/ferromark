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

## Node.js ecosystem coverage

Five further public Markdown-to-HTML APIs use the same frozen 57 inputs.
Each is independently paired with Ferromark's local Node addon without PGO.
The core and addon source hashes match the earlier micromark run; the adapter
commit is recorded separately. Ratios use only each pair's agreeing documents,
so these rows do not establish a shared-set ranking.

| Project | Version | Agreeing documents | Fresh | Reuse |
| --- | --- | ---: | ---: | ---: |
| marked | 18.0.14 | 56/57 | 8.66× | 13.26× |
| markdown-it | 15.0.2 | 56/57 | 9.39× | 14.21× |
| remark / unified | 15.0.1 | 56/57 | 92.18× | 139.01× |
| Showdown | 2.1.0 | 31/57 | 37.57× | 66.44× |
| commonmark.js | 0.31.2 | 57/57 | 4.08× | 6.28× |

commonmark.js runs **CommonMark only in both engines on all inputs**, because
its public parser has no GFM extension lane. All other pairs use the frozen
per-document CommonMark or tables/strikethrough/task-list profile. marked uses
its public URL-tokenizer override to disable literal autolinking. markdown-it
keeps its native `<s>` spelling and task-list plugin classes; remark includes
remark-rehype and rehype-stringify, with only the three matched syntax extensions.
Showdown keeps its own Markdown dialect, extra fenced-code classes, and task-list
styles. Its smaller agreement set also includes differences in custom HTML and
Markdown parsing; the retained outputs show each excluded document. Heading IDs,
metadata, ellipsis conversion, and optional renderer extras are disabled.

All pairs retain configured public parser/processor objects outside timing;
each call parses and returns a new JavaScript HTML string. Both Ferromark
lifecycles are verified before timing and around every process's windows.
Three process rounds, six alternating windows, 40 ms timing windows, and 60 ms
warmup use the same median/geometric-mean aggregation as the micromark pair.
The strict output classifier is unchanged; differing text, tags, task classes,
styles, code whitespace, and other attributes are not discarded to improve
agreement. Disagreements remain timed diagnostics and are excluded from ratios.

These macOS arm64 workstation results describe these inputs and build profiles;
Linux cells stay unmeasured. Configuration, agreement counts, raw windows,
locks, source hashes, and adapter snapshots are retained in the report below.
[Reproduction and adapter contracts](https://github.com/sebastian-software/ferromark/blob/main/benchmarks/markdown-ecosystem/README.md),
[project coverage and framework integrations](https://ferromark.dev/guide/feature-comparison#nodejs-projects).

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

- marked: [metadata](node-marked/run.json), [aggregate](node-marked/summary.json), [outputs](node-marked/verification.json.gz), [windows](node-marked/samples.json.gz), [option guards](node-marked/behavior.json.gz), [corpus](node-marked/corpus.json.gz), [npm lock](node-marked/package-lock.json), [adapter sources](node-adapters/4019c6a8eaf2916de1038a3a83787782ecb1d721/).

- markdown-it: [metadata](node-markdown-it/run.json), [aggregate](node-markdown-it/summary.json), [outputs](node-markdown-it/verification.json.gz), [windows](node-markdown-it/samples.json.gz), [option guards](node-markdown-it/behavior.json.gz), [corpus](node-markdown-it/corpus.json.gz), [npm lock](node-markdown-it/package-lock.json), [adapter sources](node-adapters/4019c6a8eaf2916de1038a3a83787782ecb1d721/).

- remark / unified: [metadata](node-remark/run.json), [aggregate](node-remark/summary.json), [outputs](node-remark/verification.json.gz), [windows](node-remark/samples.json.gz), [option guards](node-remark/behavior.json.gz), [corpus](node-remark/corpus.json.gz), [npm lock](node-remark/package-lock.json), [adapter sources](node-adapters/4019c6a8eaf2916de1038a3a83787782ecb1d721/).

- Showdown: [metadata](node-showdown/run.json), [aggregate](node-showdown/summary.json), [outputs](node-showdown/verification.json.gz), [windows](node-showdown/samples.json.gz), [option guards](node-showdown/behavior.json.gz), [corpus](node-showdown/corpus.json.gz), [npm lock](node-showdown/package-lock.json), [adapter sources](node-adapters/4019c6a8eaf2916de1038a3a83787782ecb1d721/).

- commonmark.js: [metadata](node-commonmark/run.json), [aggregate](node-commonmark/summary.json), [outputs](node-commonmark/verification.json.gz), [windows](node-commonmark/samples.json.gz), [option guards](node-commonmark/behavior.json.gz), [corpus](node-commonmark/corpus.json.gz), [npm lock](node-commonmark/package-lock.json), [adapter sources](node-adapters/4019c6a8eaf2916de1038a3a83787782ecb1d721/).
