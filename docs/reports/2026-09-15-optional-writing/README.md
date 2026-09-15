# Optional writing syntax: implementation experiment

## Question and decision

Can opt-in marked text (`==text==`) and inline footnotes (`^[note]`) be added
without a material cost when disabled? What does enabling them on unused syntax
cost, and can disabling reference links avoid work?

The change adds parser options `highlight` and `inline_footnotes`, both off in
all presets, and `allow_link_refs`, on in all presets. Node exposes `highlight`,
`inlineFootnotes`, and `allowLinkRefs`. See the [syntax contract](../../optional-writing.md)
and [design decision](../../decisions/2026-09-15-optional-writing.md).

The results and retention assessment below distinguish the unchanged default,
enabled-but-unused syntax, and inputs whose rendered meaning changes.

## Results

[Generated measurement tables](tables.md) include all row ranges and links to
raw evidence.

The disabled options meet the retention target on representative documents in
the full run and independent confirmation. No material ordinary-document
regression reproduced. Retain the implementation with both writing extensions
off by default.

Enabling an option does have a modest cost even when its syntax is absent. Both
options together cost several percent on ordinary documents, and literal marker
decoys cost more. Enabling both is therefore an explicit dialect choice, not a
free default. The malformed-marker corpus also remains separately visible; a
tiny parse-only baseline outlier is repeated with longer timing windows.

Active syntax has additional parsing, AST, and rendering work. Dense marked text
and dense footnotes compare against literal text with different HTML. Even one
note triggers document-wide identifier collection and lowering; sparse-note
ratios are included and must not be mistaken for an unused-feature measurement.

Disabling reference links was effectively neutral for prose and mixed Markdown
in the GFM run. The CommonMark profile showed modest savings when it could skip
the whole definition prepass; its reference footnotes are also disabled. This
profile dependence prevents a blanket speedup claim. Reference-heavy inputs
can be much faster because definition
collection and resolution are bypassed, but the output changes. Use the switch
for the desired syntax policy, not as a blanket performance setting.

## Reproduction and provenance

- Baseline: integration commit `6fded7699f5a6743d84fa1626614d83775fba1a2`.
- Candidate: frozen working-tree core, reconstructed by applying
  [candidate.patch](candidate.patch) to the baseline. Build metadata includes
  source, lockfile, worker, and binary SHA-256 identities.
- Rust 1.95.0, macOS ARM64, release optimization, fat LTO, one codegen unit,
  generic target CPU, and unchanged locked registry dependencies.
- The worker used for the baseline predates the new option-dispatch arms; its
  timed lifecycle code is identical. The new arms are conditional and are not
  compiled against the old API. Both worker digests are retained.
- Five alternating-order pairs per row, 10 ms warmup, 30 ms windows in the full
  GFM and selected CommonMark runs, 50 ms in the independent GFM confirmation.
  Job order is deterministically shuffled. Compilation and other task checks
  were paused during timing. This is a shared workstation, not an isolated host.
- [Harness instructions](../../../benchmarks/optional-writing/README.md) describe
  workloads, timing boundaries, equality checks, and repeat commands.
  [Harness digests](harness-sha256.json) identify the exact measurement scripts.

A separate longer-window run repeats the tiny malformed-input baseline outlier.

The full GFM run covers 29 inputs and three lifecycles: parse-only, reused
parse/render, and fresh parse/render. The confirmation repeats ordinary prose,
mixed Markdown, literal marker decoys, and selected project documents. The
CommonMark run repeats those shapes plus reference-heavy input with the default
parser profile. Input files, option JSON, per-row medians and paired ranges,
raw timings, verified HTML/AST, and host observations are archived in each run.

For every baseline comparison, both HTML and debug AST must be identical.
Enabled-but-unused comparisons require identical HTML; AST equality is recorded
because markers can segment text. Active syntax must change HTML. Timed checksums
and post-timing output checks guard all rows. These are measurements of this
corpus, not a claim of compatibility across every possible input.

The OS did not expose the CPU brand or thermal warning state to the measurement
process. Power and load observations are retained. Results on this ARM64 host do
not establish x86 or other-machine performance. Row medians and their unweighted
median summaries are descriptive; they are not confidence intervals or workload
frequency estimates. Do not interpret tiny negative ratios as established wins.

## Validation

The existing specification corpora remain unchanged. Additional tests cover
formatting boundaries, malformed syntax, optional policies, nested definitions,
identifier uniqueness, speculative links/images, source spans, visitors,
container traversal, hooks, and renderer reuse. Node integration tests exercise
all public rendering paths. No dependency or published historical benchmark
figures were changed.

Completed local gates:

- Workspace tests, including documentation tests: 807 passed.
- Instrumented core tests: 804 passed; line coverage 90.61% against the unchanged
  90% gate. The native Node crate is excluded by the existing coverage setup.
- Workspace Clippy with warnings denied, Rust formatting, and all workspace
  benchmark targets compiled with `cargo bench --workspace --no-run --locked`.
- Node build, 28 package tests and panic-unwind check, declarations/typecheck,
  lint, package packing, and clean-install smoke test.
- Repository contract tests, workflow pin checks, scripts formatting, and
  generated historical benchmark-section verification.
- Website typecheck and production build: six prerendered pages verified.
- Node and website audit gates at high severity passed. The website audit
  still reports existing low/moderate findings; dependencies were not changed.
