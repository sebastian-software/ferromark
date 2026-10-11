# Text extraction cost record

The focused Criterion workload isolates parsing from extraction on a prepared
tree. Final timing results are pending the combined feature build on an idle
host. Concurrent worker builds would make the initial timings unsuitable for a
durable comparison; no speed claim is made.

- Host: Apple M1 Ultra, arm64, macOS 27.0.1 (26A434).
- Toolchain: rustc 1.95.0 (59807616e 2026-04-14).
- Profile: repository `bench` profile, optimized, fat LTO, one codegen unit.
- Fixture: `benches/fixtures/text-extraction.mdx`, 470 bytes repeated 64 times,
  totaling 30,080 UTF-8 input bytes.
- Fixture SHA-256:
  `66431105deb5cdbc5f6afe5e565a1fdebdb78eb4e0f2511d09efbc65b207cfb7`.
- Regions: `parse`, `prepared_with_code`, `prepared_without_code`.
- Extraction includes owned output and section copies; parsing, native passes,
  JSX generation, highlighting, and Node conversion are outside that region.

```sh
CARGO_TARGET_DIR=/tmp/ferromark-extract-target cargo bench --bench text_extraction --features jsx --locked -- --sample-size 10 --warm-up-time 0.5 --measurement-time 1
```

The fixture verifies final heading ID agreement with native JSX before timing.
Criterion writes estimates under `target/criterion/text_extraction/` in the
selected target directory. Keep this report's regions separate when recording
application-specific highlighting and Node boundary costs.
