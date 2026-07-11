# Comprehensive performance profiling implementation plan

Status: approved for implementation
Date: 2026-07-11
Design: [`2026-07-11-comprehensive-performance-profiling-design.md`](./2026-07-11-comprehensive-performance-profiling-design.md)

## Task 1 — Reproducible run model and corpus registry

- [x] Add typed parser, configuration, and corpus selectors to
      `benchmarks/pulldown-comparison`.
- [x] Keep parity options centralized in the existing comparison library.
- [x] Add focused synthetic corpora plus the existing 5/20/50 KB fixtures and a
      250 KB scaling fixture.
- [x] Add serializable run metadata covering commit, dirty state, compiler,
      target, flags, corpus, options, trust policy, and byte sizes.
- [x] Test selector parsing, corpus identity, and configuration compatibility.

## Task 2 — Diagnostic runner and throughput matrix

- [x] Add a release-mode diagnostic binary that repeatedly runs one exact lane.
- [x] Support iteration- and duration-based execution without allocations in the
      measured render loop beyond parser work.
- [x] Emit one machine-readable JSON summary per run.
- [x] Add a focused Criterion profiling matrix without replacing the small parity
      benchmark.
- [x] Verify every parser/configuration/corpus selector with smoke tests.

## Task 3 — Allocation and memory evidence

- [x] Add a counting global allocator only to the diagnostic binary.
- [x] Record allocation, reallocation, deallocation, allocated-byte, and
      deallocated-byte deltas inside an explicit measurement window.
- [x] Warm output buffers before the window and report retained capacity.
- [x] Add tests proving reset/window semantics and disabled-window isolation.
- [x] Document that allocator counts include all parser-internal allocations but
      exclude setup and summary serialization.

## Task 4 — Feature-gated pipeline work counters

- [x] Add a non-default root `profiling` feature with a hidden diagnostics module.
- [x] Ensure the feature-off build contains no counter fields, branches, or API.
- [x] Count block events, inline events, marks, emit points, paragraph bytes,
      relevant capacities, and selected fast-path outcomes.
- [x] Reset and snapshot counters per diagnostic run.
- [x] Test counter correctness and verify normal all-feature/all-target gates.

## Task 5 — Profiler and compiler-matrix orchestration

- [x] Add scripts that build the exact diagnostic binary with debug symbols.
- [x] Add Time Profiler/`xctrace`, `sample`, and Samply commands without requiring
      all tools on every platform.
- [x] Add explicit portable, `apple-m1`, `native`, PGO, and non-PGO modes.
- [x] Store compact metadata and exports under an ignored results directory.
- [x] Document stable machine conditions and the three-run protocol.

## Task 6 — First comprehensive evidence pass

- [x] Run the primary 5/20/50 KB and 250 KB lanes.
- [x] Run focused simple, code, URL, refs, tables, containers, delimiters, HTML,
      and Unicode/entity lanes.
- [x] Capture CPU profiles for the highest-value representative lanes.
- [x] Capture allocation and pipeline-counter summaries.
- [x] Compare portable/current/native codegen and PGO where available.
- [x] Write a ranked opportunity map including extensibility impact and stop
      conditions.
- [x] Update the existing pulldown-gap checklist with completed and rejected
      findings.

## Validation gate

- [x] `cargo fmt --all -- --check`
- [x] `cargo test --all-targets --all-features --locked`
- [x] `cargo clippy --all-targets --all-features --locked -- -D warnings`
- [x] Focused comparison tests, Clippy, and benchmark build with `--locked`.
- [x] Diagnostic smoke matrix succeeds in feature-off and profiling builds.
- [x] Root package contents exclude generated traces and comparison artifacts.
