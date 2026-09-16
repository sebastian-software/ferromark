# ADR-0020: Preserve consumer contracts before sharing native and workflow tooling

- Status: Proposed
- Date: 2026-09-16
- Issue: [#269](https://github.com/sebastian-software/ferromark/issues/269)

## Context

The v1 extraction candidates no longer describe the v2 architecture. This
assessment compares Ferromark at `59968af225460f14ab5e973acbc47ba8e792a72c`
with an actual second consumer, Ferriki at
`c8c9e61a6fe01ac9253b5bdbee259ad7f2cbda2f`, and the available standards action at
`09db5e91d5c813d4e5800366fad791b56486cb7f`. These are inspected revisions,
not promises about future consumer APIs.

## Native packages

Ferriki is a concrete second native Node package, but its current loader is
not a behavior-preserving replacement for Ferromark's loader.

| Contract | Ferromark | Ferriki |
| --- | --- | --- |
| Targets | Eight, including Linux musl and Windows ARM64 | Five; Linux GNU and Windows x64 only |
| Resolution order | Local target binary, then optional package entry point | Optional package binary, then three local file candidates |
| Load errors | Immediately reports an incompatible binary or other non-missing error | Collects errors and continues to later candidates |
| Caching | Caches the loaded binding in the wrapper | Resolves candidates on each loader call; Node caches loaded modules |
| Registry verification | Checks expected package name/version and publish result | Also requires registry provenance metadata |

Sources: Ferromark's [target resolver](../../node/ferromark/native-target.mjs),
[loader](../../node/ferromark/index.mjs), and
[publication verification](../../node/scripts/verify-npm-publish.mjs);
Ferriki's [target matrix](https://github.com/sebastian-software/ferriki/blob/c8c9e61a6fe01ac9253b5bdbee259ad7f2cbda2f/node/ferriki/platforms.mjs),
[loader](https://github.com/sebastian-software/ferriki/blob/c8c9e61a6fe01ac9253b5bdbee259ad7f2cbda2f/node/ferriki/native.mjs), and
[publication verification](https://github.com/sebastian-software/ferriki/blob/c8c9e61a6fe01ac9253b5bdbee259ad7f2cbda2f/node/scripts/verify-npm-publish.mjs).

The shared candidate is a small set of pure target-selection and registry-polling
helpers with explicit consumer policy, not either whole loader. Before extracting,
both consumers must supply target, candidate-order, missing/incompatible-binary,
retry, and publication-policy fixtures to the same proposed API. Keep package
names, supported targets, asset requirements, public API smoke tests, and release
orchestration with each consumer. This assessment does not change provenance
requirements or either release pipeline.

## Workflow pin checking

The [standards composite checker](https://github.com/sebastian-software/standards/blob/09db5e91d5c813d4e5800366fad791b56486cb7f/.github/actions/check-action-pins/check-action-pins.mjs)
uses a line scanner. Ferromark's [checker](../../scripts/check-workflow-pins.mjs)
parses YAML and visits workflow jobs and steps. Running both against all 17
existing [YAML fixtures](../../scripts/test-fixtures/workflow-pins) gives:

| Fixture | Ferromark exit | Standards exit |
| --- | ---: | ---: |
| `aliased-key-mutable-ref.yml` | 2 | 0 |
| `aliased-pinned-ref.yml` | 2 | 0 |
| `cyclic-alias.yml` | 2 | 1 |
| `flow-style-mutable-ref.yml` | 1 | 1 |
| `full-sha.yml` | 0 | 1 |
| `invalid-self-repository-actions.yml` | 1 | 1 |
| `local-and-docker.yml` | 0 | 1 |
| `malformed.yml` | 2 | 0 |
| `merged-mutable-ref.yml` | 1 | 1 |
| `missing-ref.yml` | 1 | 1 |
| `multiple-flow-mappings.yml` | 1 | 1 |
| `mutable-ref.yml` | 1 | 1 |
| `no-matches.yml` | 0 | 0 |
| `non-action-text.yml` | 0 | 0 |
| `quoted-full-sha.yml` | 0 | 1 |
| `self-repository-actions.yml` | 0 | 1 |
| `unknown-alias.yml` | 2 | 1 |

Exit 0 means accepted, 1 a policy violation, and 2 a parse/scan failure.
The standards checker accepts the malformed and aliased-key fixtures with zero
recognized references. It additionally requires version comments and immutable
Docker digests, and rejects the current `$/` self-repository syntax. Those
stricter policies are separate behavior decisions, not compatibility fixes.
The standards checker rejects the cyclic-alias fixture for a missing version
comment, not for its cycle. Equal exit codes alone do not establish equal
validation.

Obtain the inspected standards revision in a fresh directory:

```sh
git clone https://github.com/sebastian-software/standards.git /tmp/standards-issue-269
git -C /tmp/standards-issue-269 checkout --detach 09db5e91d5c813d4e5800366fad791b56486cb7f
```

Reproduce with that checkout and the pinned Ferromark checkout:
install Ferromark's `scripts/` dependencies with `pnpm install --frozen-lockfile`,
then copy each fixture into its own temporary directory as `workflow.yml`.
Run `node scripts/check-workflow-pins.mjs <directory>` and
`node <standards-checkout>/.github/actions/check-action-pins/check-action-pins.mjs <directory>`
and record each exit status. Use the resolved absolute standards path: its CLI
entry guard compares `import.meta.url` with the supplied script path.

Retain the local checker and its shell wrapper. A shared successor must preserve
the YAML rejection boundaries and explicit local-reference policy before adoption.
Run the existing fixture suite unchanged against that successor. Ferriki's
[local checker](https://github.com/sebastian-software/ferriki/blob/c8c9e61a6fe01ac9253b5bdbee259ad7f2cbda2f/scripts/check-workflow-pins.mjs)
also scans lines; it is not evidence that the structural contracts can be removed.

## Rust highlighting adapter

Ferriki contains public Rust highlighter methods inside its private native
crate, so the old assertion that no Rust API exists is too broad. However,
[`ferriki-core`](https://github.com/sebastian-software/ferriki/blob/c8c9e61a6fe01ac9253b5bdbee259ad7f2cbda2f/crates/ferriki-core/Cargo.toml)
is a private N-API crate with path dependencies and an N-API build step. Its
[`HighlighterCore`](https://github.com/sebastian-software/ferriki/blob/c8c9e61a6fe01ac9253b5bdbee259ad7f2cbda2f/crates/ferriki-core/src/highlighter.rs)
uses `napi::Result` and needs grammar/theme assets. It is not yet a standalone,
published Rust dependency for Ferromark consumers.

Defer a maintained adapter until Ferriki offers a supported Rust library and
asset-loading contract. At that point, implement it in a consumer/example through
[`HtmlRenderHooks`](../../crates/ferromark/src/renderer/html/renderer/hooks.rs),
with code-block fallback and HTML-output tests. Do not reintroduce the removed
Rust `FencedCodeRenderer` interface or split the published Ferromark crate.

## Decision and consequences

Keep the current implementations while the explicit compatibility gaps above
remain. This completes the candidate assessment without introducing a speculative
shared package or changing runtime behavior. Reopen extraction when both consumers
can validate one bounded API; reopen the adapter when the Rust library boundary is
available. No performance claim follows from this source and fixture comparison.
