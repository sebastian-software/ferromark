# Initial baseline validation

Validated on 2026-09-13 with Rust 1.95.0 / Cargo 1.95.0 on
`aarch64-apple-darwin`.

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` | Passed |
| `cargo test --workspace --all-features --locked` | 626 passed, 0 failed, 0 ignored across 41 test targets |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | Passed |
| `cargo bench --workspace --no-run --locked` | Passed; all seven Criterion suites build |
| Retained snapshot outputs compared with pinned upstream | All 168 bodies byte-identical |
| Specification text and known-failure lists | Byte-identical to upstream |
| MIT license and prepass input fixture | Byte-identical to upstream |
| Retained registry package versions/checksums | All identical to upstream |

The test count includes unit, integration, and doc tests. The conformance harness
internally checks the CommonMark and GFM examples against the inherited baseline;
these examples are not each counted as a separate Rust test. The deterministic
mutation suite also passed.

The first test attempt detected snapshot filenames still using the old crate
prefix. The existing snapshots were renamed without changing expected output,
then the complete suite passed. No new snapshots or failure baselines were accepted.

## Size comparison

The downloaded upstream archive contains 3,572 files and 26.17 MB of uncompressed
file contents. The cleaned source is about 1.45 MB, a reduction of roughly 94.5%.
These figures exclude `.git` and Cargo's ignored `target/` build artifacts; source,
tests, fixtures, and documentation are included. The inherited full changelog is
counted only once as a frozen benchmark fixture.

- Workspace crates: 27 upstream to 4 retained core crates plus the facade.
- Lockfile packages, including workspace and development dependencies: 380 to 98.
- Packages have local `publish = false` metadata.

This is structural and correctness validation. No performance improvement is
claimed, no timed benchmark result was generated, and x86/other targets were not
executed in this local run. Re-run the commands above and collect architecture-
specific differential and performance evidence for each subsequent optimization.
