# Candidate adapter readiness — macOS arm64

The complete **22-row matrix** built and passed local option/output verification
on Apple M1 Ultra, 64 GB RAM, macOS 27.0, from clean source commit `5f9c74d9aa9ec2096f68505d5c04368ae0d1810f`.
This report establishes execution on this host. Linux/x86-64 and Blacksmith still
need their own resource, installation, and output verification.

The follow-up adds seven projects as eight execution variants. Every displayed
candidate checks all 57 immutable corpus inputs in both lifecycles. New pairs also
check rotating batches against the single-document output before timing.

| New lane | Equivalent HTML | Excluded documents |
| --- | ---: | ---: |
| goldmark | 56/57 | 1 |
| markdown-exit | 55/57 | 2 |
| markdown-it-ts | 55/57 | 2 |
| md4x-napi | 41/57 | 16 |
| md4x-wasm | 41/57 | 16 |
| ox-content-napi | 13/57 | 44 |
| remarkable | 44/57 | 13 |
| satteri | 55/57 | 2 |

These are output agreement counts, not speed factors. MD4X's fixed extensions,
Sätteri's GFM autolinks, and OX-Content Node's renderer builtins remain intact;
reference isolation, raw HTML, Unicode, and literal code whitespace are checked.
The 19 real Node adapter tests include both explicit MD4X backends, large Unicode
strings, and their equality. Goldmark includes its allocator/GC and uses the
pinned Go 1.27.1 compiler. The new Node packages were rechecked against npm's
stable versions on 2026-09-30; pins match. The only npm audit finding remains the
known moderate Showdown advisory; there are no high/critical findings.

## Commands and evidence

The normal CLI was executed from a clean committed worktree with Python 3.12.14,
Node 24.21.0, the pinned Rust toolchains, and Go 1.27.1. Ambient NODE_OPTIONS and
RUSTUP_TOOLCHAIN were unset, as required by preflight. Preparation used a fresh
isolated clone and rebuilt all native workers and the local plain Node addon.

```sh
./scripts/benchmark-comparison doctor
./scripts/benchmark-comparison prepare /private/tmp/ferromark-complete-candidates-20260930
./scripts/benchmark-comparison verify /private/tmp/ferromark-complete-candidates-20260930
```

[Prepared provenance](prepared.json), [required inventory](comparisons.json),
[verification summary](verification-summary.json), and compressed raw HTML/option
checks under `verification/` retain all 22 displayed lanes. The original native
harness's build metadata is retained instead of a nonexistent verify-only timing
config. Logs retain builds, harness tests, real addon-loader tests, and the full
verify command sequence. Lockfiles and [source audit](source-audit.json) identify
what was installed and built. The two MD4X lanes explicitly import `md4x/napi`
and `md4x/wasm`; WASM init and all imports are outside timing.

The original six-engine worker and eight new pair adapters then passed a separate
**diagnostic timing smoke**: one process round, one 1 ms sample after 1 ms warmup,
all 57 inputs, both lifecycles, and rotating controls. Raw short windows are under
`diagnostics/`; [diagnostic summary](diagnostic-summary.json) labels every run
unpublishable. All eight pair smokes were passed to the full archive validator
and rejected for the shortened contract. The six-engine smoke has 118 jobs/windows;
each new pair has 114 per-document windows and separate rotating windows.

No complete measurement campaign was run, no new factors were imported, and no
Blacksmith job was dispatched. Short timing summaries are diagnostic only;
repeated-input caches can affect them. Do not use them as public speed evidence.
Future full pairs require three rounds, six 40 ms windows after 60 ms warmup,
complete output coverage, committed manifest/source proofs, and rotating controls.
Historical archives, homepage factors, and platform selections remain unchanged.
The final follow-up also adds publication-scope regression tests, keeps historical
14-row report descriptions accurate, and clarifies the homepage's unmeasured-cell
legend. A final build also disables ambient Go workspaces so they cannot replace
the pinned module. The [Go workspace guard](go-workspace-guard/build.json) was
rebuilt with an intentionally nonexistent GOWORK path, followed by the real native
option/lifecycle tests; both passed. Its exact build script and logs are retained.
The Goldmark source/module hashes and Rust/C worker hashes match the clean
full-corpus source commit above. This supplemental build checks workspace isolation;
the 22-row corpus evidence remains bound to its original recorded revision.

[SHA256SUMS](SHA256SUMS) covers all retained files. Review syntax differences and
rotating controls before freezing or publishing an official campaign.
