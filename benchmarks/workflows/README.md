# Practical Markdown workflow benchmarks

Measure the questions in [ARCH-COMP-003](../../docs/arch/ARCH-COMP-003-practical-workflows.md):
short secure previews, documentation with metadata, and rendering a complete
documentation collection with two output lifetimes. The native APIs include
HTML allocation and destruction. Filesystem I/O, templates, JavaScript bindings,
syntax highlighting, process startup, and concurrent requests are outside scope.

## Frozen corpus

`corpus.json` contains every input and its provenance. Twelve authored comments
exercise ordinary prose, quotes, code, checklists, a small table, Unicode, links,
images, raw HTML, and a disallowed URL. They are examples, not collected traffic.
Three guide pages and twelve documentation files are unmodified snapshots of
this repository. Their source revision, path, license, size, and hash are stored
alongside them. `make_corpus.py` declares the selection before any timing exists.

This is deliberately one project's document mix, not a claim about typical
Markdown sizes. No source is padded, repeated to reach an arbitrary size, or
filtered by whether Ferromark wins. Do not regenerate the corpus to reproduce
an archived run: use that archive's `corpus.json` and measured source revision.
The `.mdx` guide files selected here contain ordinary Markdown with front matter;
their component examples are fenced code, not executed MDX.

## Workloads and options

| Lane | Work and lifecycle | Compared variants |
| --- | --- | --- |
| Preview burst | All 12 comments, matched untrusted rendering with heading IDs and callouts, owned HTML released after each | Ferromark `to_html`; retained `Renderer::render`; pulldown-cmark + adapter; Comrak + adapter |
| Guide metadata | All 3 pages, HTML plus raw front matter and heading metadata consumed and released | Ferromark `parse`; pulldown-cmark + adapter; Comrak + adapter |
| Documentation HTML | All 12 files, fresh owned HTML per page, trusted CommonMark + tables + strikethrough + tasks | Ferromark; pulldown-cmark 0.13.4; Comrak 0.54.0 |

The HTML collection has separate immediate-release and retain-all variants for
each engine. Retained outputs and the vector that owns them are included in
the timer and heap scope, then freed before that workload returns. All engines
use normal buffer growth and the system allocator. No arenas or output buffers
are cached in the fresh engine comparison. Preview renderer scratch survives
iterations, but its output is still an owned string, just like the fresh API.

Raw HTML and arbitrary URL schemes are trusted only in the collection lane.
Bare autolinks, heading IDs, footnotes, and other extensions are disabled there.
Ferromark requires double tildes for strikethrough; the normal behavior of the
other engines remains unchanged. Output review detects differences on the
actual corpus rather than silently pretending the grammars are identical.

### Complete preview and metadata integrations

The comparison uses Ferromark 0.9.0 at the archived source revision,
pulldown-cmark 0.13.4, and Comrak 0.54.0. The four additional variants in
protocol 2 complete the engine comparison on the **same frozen corpus** as
the initial protocol 1 run. The original archive remains reproducible.

[`src/adapters.rs`](src/adapters.rs) provides the application work that the
other engines' HTML-only APIs do not provide in this configuration:

- Escape all user HTML and accept relative URLs or the same explicit scheme
  allowlist as Ferromark: HTTP(S), mailto, FTP, geo, IRC(S), matrix, SMS, tel,
  and XMPP. Reject other absolute schemes for links and images. Parsed URL
  destinations are checked after entity and ASCII whitespace normalization.
- Generate heading IDs and render recognized callouts. pulldown-cmark's event
  adapter buffers one heading at a time to generate its ID before HTML output.
  Comrak uses its public heading hook. Both use Comrak's `Anchorizer`; the
  generated IDs must match Ferromark on every input admitted to this corpus.
- For guides, borrow the raw delimited front matter and return owned heading
  level/text/ID records with the HTML. Comrak's AST is traversed to enforce the
  URL policy; pulldown-cmark checks destinations in its event stream. Neither
  adapter parses Markdown a second time or highlights fenced code.

These adapters use public APIs. Their allocations, checks, parsing, HTML
generation, and destruction are all included in measurement. The shared result
shape uses Ferromark's public `ParseResult`/`Heading` data containers; the other
engines do not call Ferromark's parser. Ferromark's resource-limit report must
be empty; the adapters do not invent equivalent limit diagnostics for engines
without that API. Metadata and reviewed HTML must agree across all engines.

This measures these particular complete integrations, including adapter costs.
It does not assert that all possible integrations have the same cost, that the
engines' default security policies are identical, or that their dialects and
heading-slug conventions agree outside the archived corpus. The unit tests
exercise unsafe links/images, raw HTML, callouts, front matter, and repeated
heading IDs before the complete corpus verification.

`outputs.json.gz` stores original HTML, guide metadata, and effective options.
Every document must perform comparable work under the existing workload
review before timing. Instrumented and normal builds must agree exactly.
Resource-limit fallback, dropped metadata, or changed output lifetime/content
stops the run. Review mismatches; do not remove inconvenient documents.

## Prepare and measure

The initial host is macOS on Apple Silicon, with Python 3.11+ and Rust 1.97.1.
The runner's host/power probes currently require macOS. The workers themselves
are native Rust. The root's locked direct dependency versions are checked
against the isolated benchmark lock before building. `RUSTFLAGS` selects a
generic CPU target for every engine; normal release optimization, fat LTO,
one codegen unit, and panic abort apply to both binaries.

```sh
python3 -m unittest discover -s benchmarks/workflows -p 'test_*.py'
cargo fmt --manifest-path benchmarks/workflows/Cargo.toml --check
cargo test --manifest-path benchmarks/workflows/Cargo.toml --all-features --locked
cargo clippy --manifest-path benchmarks/workflows/Cargo.toml --all-targets --all-features --locked -- -D warnings
python3 benchmarks/workflows/prepare.py /private/tmp/workflow-build
python3 benchmarks/workflows/run.py /private/tmp/workflow-build /private/tmp/workflow-verify --verify-only
python3 benchmarks/workflows/run.py /private/tmp/workflow-build docs/reports/NEW-WORKFLOW-RUN
python3 benchmarks/workflows/publish.py docs/reports/NEW-WORKFLOW-RUN
mise run readme:write
python3 benchmarks/workflows/publish.py docs/reports/NEW-WORKFLOW-RUN --check
```

Use new build and result directories. Commit the measured source before
preparing a publication build; its hashes and clean revision are recorded.
Keep AC power connected and avoid concurrent builds, tests, and benchmarks
throughout measurement. The runner records process-CPU, load, thermal, and
power observations before and after each timing round for later inspection.

Three fresh process rounds use three seconds of warmup per variant, followed
by 80 alternating windows of at least 63 ms per variant. A time check occurs
after four complete workloads. Variant order rotates and reverses between
windows. All durations and completed-workload counts are archived; the public
time is the median of three round medians. The report exposes variation.

## Memory is a different measurement

Timing workers use the uninstrumented system allocator. The separate `heap`
build counts successful allocations, reallocations, and frees, with allocator
self-tests. Ten observations per variant in each of three fresh processes
must agree and return to baseline after session destruction.

- **Peak live heap:** maximum simultaneously live requested bytes above the
  infrastructure baseline, including parser scratch, metadata, and HTML.
- **Retained session heap:** live bytes after outputs have been dropped;
  reusable scratch remains counted, even though it was allocated during warmup.
- **Allocated per workload:** cumulative requested bytes, including realloc
  requests. This describes allocation traffic, not required memory capacity.
- **Cold peak:** peak while creating the session and executing its first
  workload after process-wide initialization. The headline uses the warmed
  session, consistent with the time measurement.

The preloaded input corpus and worker infrastructure are outside the scope.
Stack memory, allocator rounding/metadata/caches, committed pages, and transient
old-plus-new backing storage inside realloc are not visible. **This is not
RSS or peak process RAM.** There is no per-request p95/p99 claim, concurrency
scaling claim, or extrapolation from sequential heap peaks to retained builds.

## Evidence and publication

The archive keeps inputs, dependency lock, build provenance/logs, all original
outputs, admission decisions, 3,120 timing windows, 39 warmups, 390 memory
observations, host observations, completion metadata, and evidence checksums.
`publish.py --check` recomputes medians and verifies the complete protocol,
output byte counts, rotation order, corpus/source hashes, and memory balance.
It rejects screening, missing, duplicated, short, or internally inconsistent
measurements. Table values are generated; do not edit them by hand.

The generated workflow block in `README.md.src` precedes the historical
comparisons. The existing Bun/mimalloc and native pair reports retain their
original conditions and measurements. Regenerate the root README with its
pinned mdtheme workflow after updating workflow publication.
