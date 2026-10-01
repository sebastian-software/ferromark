# TanStack 1.0.0 and balanced campaign readiness

Source: `9af95fc3175bc7c2a199682ba3e6a1c276bfdad9`. Host: Apple M1 Ultra,
64 GB RAM, macOS 27.0 arm64; Node 24.21.0, Go 1.27.1 and the committed Rust
toolchain pins. The [sampling decision](../../decisions/2026-10-01-bounded-comparison-sampling.md)
retains three process rounds and all 57 documents, with three 10 ms samples,
30 ms warmup and 5-second lane pauses.

## Dependency and execution checks

The [official-registry/tag recheck](version-review.json) found TanStack 1.0.0
as the only direct candidate/extension pin update. npm regenerated its manifest
and lock without changing other dependencies. The existing direct HTML options,
empty extension list and subset guards pass on the released API. The private
benchmark audit retains the existing unfixed Showdown moderate advisory; there
are no high or critical findings.

```sh
./scripts/benchmark-comparison doctor
./scripts/benchmark-comparison prepare /private/tmp/ferromark-balanced-preflight-20261001 --scope main --timing-profile balanced
./scripts/benchmark-comparison verify /private/tmp/ferromark-balanced-preflight-20261001
```

The normal CLI built a fresh isolated committed clone, both native harnesses,
the Go adapter and the default Node addon, replayed pinned locks, and audited
native sources. All 20 main execution variants passed option/state checks and
all 57 inputs in both lifecycles. [Prepared provenance](prepared.json),
[source audit](source-audit.json), [build commands](prepare.log.gz),
[verification commands](verify.log.gz), and the raw compressed outputs in
`verification/` retain the evidence. These are output checks, not conformance
admission gates; disagreements remain visible.

Required workspace Rust checks, 76 Node repository contracts, 50 Python
repository tests, 52 native benchmark contracts, 22 ecosystem contracts
(including the actual build-dependent integration check during preparation),
25 manual contracts, and the actual-addon Node adapter/loader tests passed.
Scripts formatting passed. No parser, renderer, default feature or application
lock was changed.

## Real balanced timing and archive pilot

The prepared clone ran the actual published TanStack adapter with all 57 inputs:

```sh
python3.14 benchmarks/markdown-ecosystem/run.py node /private/tmp/ferromark-balanced-pilot-tanstack-20261001 --competitor tanstack-markdown --rounds 3 --samples 3 --window-ms 10 --warmup-ms 30
python3.14 benchmarks/markdown-ecosystem/archive-results.py /private/tmp/ferromark-balanced-pilot-tanstack-20261001 /private/tmp/ferromark-balanced-pilot-tanstack-evidence-20261001 --revision 9af95fc3175bc7c2a199682ba3e6a1c276bfdad9
```

The committed-policy validator accepted all **1,026 document samples and 36
rotating controls**. [Raw run, inputs, outputs, samples and lock](pilot-tanstack/)
and the [log](pilot.log.gz) retain this actual measurement. Separate copies with
a missing document sample, missing rotating control, or 9,999,999 ns window
were all rejected. Unit gates also reject altered profile parameters and
attempts to relabel historical measurements.

This single-pair pilot validates the execution and archival contract. It is not
a complete homepage campaign and does not replace any published factor. Review
all candidate round spreads and controls before interpreting a complete run.
The calculated main-profile window/warmup/pause budget is 14.83 minutes, versus
84.08 minutes for standard; process overhead, slow calls, verification, downloads
and builds are additional. It is not a measured whole-suite runtime promise.

## Managed comparison

The [initial resource/tool probes](https://github.com/sebastian-software/ferromark/actions/runs/36817117530)
passed for the configured M4 Pro macOS arm64 and AMD EPYC Linux x86-64 VMs.
The [balanced comparison](https://github.com/sebastian-software/ferromark/actions/runs/36822651743)
was dispatched from the exact tested source above, one allocation per platform
with three internal process rounds. Both VMs build and verify before timing.
At this record's creation it is still running; these local checks do not claim
completed managed performance evidence. Historical reports, platform selections
and homepage values remain unchanged.
