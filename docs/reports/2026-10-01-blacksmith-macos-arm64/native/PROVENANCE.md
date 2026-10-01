# Source and build provenance

The [Blacksmith benchmark run](https://github.com/sebastian-software/ferromark/actions/runs/36822651743) used an isolated committed checkout. The workflow checked out `9af95fc3175bc7c2a199682ba3e6a1c276bfdad9` for the harness and exported the measured revisions with `git archive`; working-tree sources are never built.

| Engine | Pin |
| --- | --- |
| Ferromark v2 | `9af95fc3175bc7c2a199682ba3e6a1c276bfdad9` |
| Ferromark v1 | `4e151415a15c67e9d3735f719b0bed3e25d8cff8` |
| OX-Content | `616cc793d4d1b2256098d9775d62a3f3baef524f` (archive SHA-256 `acf5720ed792c10d1f94b47719f26d14d567fc457a0384bad445d720cfe6fee1`) |
| md4c | `7fc1815a5eeba2af7d6120a76202bf59f3b6e6e4` |
| Bun | `744846f844374847c902b5e7fd59b4342a51ef99` |
| pulldown-cmark | 0.13.4 from the seeded lock |

This report records the comparison pins above; older reports retain their own pins. [restore.py](restore.py) restored the sources from their upstream URLs and checked the OX, mimalloc, and Highway archive checksums ([restore.json](restore.json)).

## Build conditions

- Host: Apple M4 Pro (Virtual), 24 GB RAM, macOS 26.3; `aarch64-apple-darwin`.
- Rust: `rustc 1.99.0-nightly (9f36de775 2026-07-19)`, LLVM 22.1.8, the pinned `nightly-2026-07-20` required by the native Bun integration.
- C/C++: `Apple clang version 21.0.0 (clang-2100.1.1.101)`. Linker: `/Applications/Xcode_26.6.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/ld: @(#)PROGRAM:ld PROJECT:ld-1267`.
- `RUSTFLAGS`: `-C target-cpu=generic`; optimization level 3, fat LTO, 1 codegen unit, panic abort. No PGO in the default build.
- C/C++ flags, mimalloc defines, and the md4c allocator redirection are the ones in `harness/prepare.py`, identical on both platforms.
- Executable SHA-256: `fb3d672755379c6cca0ea3e8d98ea9137577caf8569c181870ab3f692c794ccf`.
- Final Cargo lock SHA-256: `898791409d1b2093f44e4a024ce14cde43143f721876e3116d5378f2faa5fb39`.

### Platform differences

Recorded per build in `build.json` under `platform`:

| Aspect | macOS (Apple Silicon) | Linux (x86-64) |
| --- | --- | --- |
| C/C++ compiler | clang, clang++ (Apple clang) | clang, clang++ (the system's default) |
| C++ runtime | libc++, the only C++ runtime Apple clang links | libstdc++, clang's default C++ runtime on Linux |
| Rust link driver and linker | rustc default `cc` (Apple clang) with the system ld64; environment unchanged | clang through CARGO_TARGET_<host>_LINKER with the system GNU ld, not rustc's bundled rust-lld |
| Stack bound for Bun's recursion check | pthread_get_stackaddr_np and pthread_get_stacksize_np | pthread_getattr_np and pthread_attr_getstack |
| Bun Highway platform branch | OS(DARWIN): Highway memmem replaces libc memmem through an assembler alias | OS(LINUX): Highway memmem replaces libc memmem through a weak alias |
| `-C target-cpu=generic` | generic AArch64 (NEON) | x86-64 baseline (SSE2); engines with runtime detection still select SSSE3/AVX2 paths |
| Highway targets | runtime dispatch; SVE list disabled as in Bun | runtime dispatch over the x86 targets; the SVE list has no effect |
| PGO training-binary stubs | the shared list | the shared list plus the Bun support symbols GNU ld also resolves; the measured executable links none of them |

Bun's own Linux release build also turns on mimalloc's global `malloc` override and disables transparent huge pages for mimalloc arenas. The harness applies neither on either platform: every engine already allocates through the same mimalloc, Rust through Bun's global allocator and md4c through `md4c_alloc.h`, and one mimalloc configuration keeps the platforms comparable.

### The lock was seeded, not replayed with `--locked`

The build seeded Cargo with `benchmarks/native-comparison/Cargo.lock` through `--lockfile`/`--bun-lock` because the v2 path package's version line differs from that lock. `CARGO_NET_OFFLINE=true` and `--offline` kept the build itself offline, and `prepare.py` fails if Cargo resolves any registry package outside the union of the engine locks. All 71 resolved registry packages are identical to the seed lock in name, version, and checksum.

## Source audit

`harness/audit_sources.py` checked the exported sources against the pinned Git blobs and the checksummed archives ([source-audit.json.gz](source-audit.json.gz)):

- v1: 35 files, 0 mismatches
- v2: 190 files, 0 mismatches
- bun: 3495 files, 0 mismatches
- md4c: 10 files, 0 mismatches
- ox-content: 395 files, 0 mismatches
- mimalloc archive: 402 files, 0 mismatches
- highway archive: 278 files, 0 mismatches

## Measurement conditions

No other benchmark process was started while `run.py` was timing. Load averages per process round:

| Round | Start (UTC) | End | Load before | Load after |
| ---: | --- | --- | ---: | ---: |
| 1 | 06:08:11 | 06:08:59 | 3.11 | 2.25 |
| 2 | 06:08:59 | 06:09:47 | 2.25 | 1.72 |
| 3 | 06:09:47 | 06:10:34 | 1.72 | 1.50 |

Thermal and clock readings, where the virtual machine exposes them, are in `run.json`.

## Reproduction

Follow the harness README's reproduction commands on a macOS arm64 host with clang and `--ferromark-v2-revision 9af95fc3175bc7c2a199682ba3e6a1c276bfdad9`, using this report's `restore.py` in an empty cache directory.
