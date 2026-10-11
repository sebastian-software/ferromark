# Text extraction cost record

The focused Criterion workload measures parsing and extraction on a prepared
Rust tree in separate regions. These are absolute costs for one fixture on one
host, including owned extraction output and section copies.

- Measured: October 11, 2026, after other feature builds and tests stopped.
- Implementation: `208713a84507d0cc3d00b73e43bb75735992b95d`.
- Host: Apple M1 Ultra, arm64, macOS 27.0.1 (26A434).
- Toolchain: rustc 1.95.0 (59807616e 2026-04-14), LLVM 22.1.2.
- Profile: repository `bench` profile, optimized, fat LTO, one codegen unit.
- Criterion: 0.8.2, 10 samples, 0.5-second warmup and 1-second measurement per region.
- Fixture: `benches/fixtures/text-extraction.mdx`, 470 bytes repeated 64 times,
  totaling 30,080 UTF-8 input bytes.
- Fixture SHA-256:
  `66431105deb5cdbc5f6afe5e565a1fdebdb78eb4e0f2511d09efbc65b207cfb7`.

| Region | Estimate (µs) | 95% confidence interval (µs) |
| --- | ---: | ---: |
| `parse` | 403.89 | 401.71–407.04 |
| `prepared_with_code` | 111.33 | 110.88–111.75 |
| `prepared_without_code` | 110.76 | 110.25–111.45 |

The table uses Criterion's slope estimate. Each region's directory retains
`estimates.json`, `sample.json`, and `benchmark.json` from the final run; raw
durations in those files are nanoseconds.

`parse` creates an allocator and parses the complete input. The two extraction
regions reuse one document, include output allocation and section copies, and
vary only whether block code contributes text. Parsing, native passes, JSX
generation, highlighting, and Node conversion are outside those extraction
regions. Applications should measure those costs separately on their own
content; this short sample is not a cross-library or release performance claim.

```sh
CARGO_TARGET_DIR=/tmp/ferromark-extract-target cargo bench --bench text_extraction --features jsx --locked -- --sample-size 10 --warm-up-time 0.5 --measurement-time 1
```

The fixture verifies final heading ID agreement with native JSX before timing.
Criterion writes estimates under `criterion/text_extraction/` in the selected
Cargo target directory.
