# Source and build provenance

The [native comparison workflow](https://github.com/sebastian-software/ferromark/actions/runs/36011572163) ran on a GitHub-hosted ubuntu-latest runner for a workflow_dispatch event. The workflow checked out `0b94bade82d512ad06b62e685fd34267bcf9d9a5` for the harness and exported the measured revisions with `git archive`; working-tree sources are never built.

| Engine | Pin |
| --- | --- |
| Ferromark v2 | `09e5e866da50690d24ef7969780bff3d4983795e` |
| Ferromark v1 | `4e151415a15c67e9d3735f719b0bed3e25d8cff8` |
| OX-Content original | `a71a58939ffe7f154117cea026f6d6e71a139393` (archive SHA-256 `7df34e3e2db30678981f6df838eafe3e7950e966453221ae180acfa7938f1cfd`) |
| md4c | `65c6c9d72cebd9a731aaa5597414ce04d9ea5de3` |
| Bun | `76e9dcc6ad272a4fb1ee4a4dbbde4809201b71d1` |
| pulldown-cmark | 0.13.4 from the seeded lock |

The five comparison pins are the ones every Apple Silicon report since 2026-09-14 uses; only v2 moves. [restore.py](restore.py) restored them from their upstream URLs and checked the OX, mimalloc, and Highway archive checksums ([restore.json](restore.json)).

## Build conditions

- Host: 4 vCPUs, Linux 6.17.0-1022-azure, runner image ubuntu24 20260920.314.1; `x86_64-unknown-linux-gnu`.
- Rust: `rustc 1.99.0-nightly (9f36de775 2026-07-19)`, LLVM 22.1.8, the pinned `nightly-2026-07-20` required by the native Bun integration.
- C/C++: `Ubuntu clang version 18.1.3 (1ubuntu1)`. Linker: `/usr/bin/ld: GNU ld (GNU Binutils for Ubuntu) 2.42`.
- `RUSTFLAGS`: `-C target-cpu=generic`; optimization level 3, fat LTO, 1 codegen unit, panic abort. No PGO in the default build.
- C/C++ flags, mimalloc defines, and the md4c allocator redirection are the ones in `harness/prepare.py`, identical on both platforms.
- Executable SHA-256: `dba601f0b484e2f92f679cff5622592b6cae05a76b0a45c7b20d315782d1bfb9`.
- Final Cargo lock SHA-256: `b1fde1668d1cdf88920e29706880f37bae9b3c64ceb17a1fcebb1495d3367ca1`.

### Platform differences

Recorded per build in `build.json` under `platform`:

| Aspect | macOS (Apple Silicon) | Linux (x86-64) |
| --- | --- | --- |
| C/C++ compiler | clang, clang++ (Apple clang) | clang, clang++ (runner default) |
| C++ runtime | libc++, the only C++ runtime Apple clang links | libstdc++, clang's default C++ runtime on Linux |
| Rust link driver and linker | rustc default `cc` (Apple clang) with the system ld64; environment unchanged | clang through CARGO_TARGET_<host>_LINKER with the system GNU ld, not rustc's bundled rust-lld |
| Stack bound for Bun's recursion check | pthread_get_stackaddr_np and pthread_get_stacksize_np | pthread_getattr_np and pthread_attr_getstack |
| Bun Highway platform branch | OS(DARWIN): Highway memmem replaces libc memmem through an assembler alias | OS(LINUX): Highway memmem replaces libc memmem through a weak alias |
| `-C target-cpu=generic` | generic AArch64 (NEON) | x86-64 baseline (SSE2); engines with runtime detection still select SSSE3/AVX2 paths |
| Highway targets | runtime dispatch; SVE list disabled as in Bun | runtime dispatch over the x86 targets; the SVE list has no effect |
| PGO training-binary stubs | the shared list | the shared list plus the Bun support symbols GNU ld also resolves; the measured executable links none of them |

Bun's own Linux release build also turns on mimalloc's global `malloc` override and disables transparent huge pages for mimalloc arenas. The harness applies neither on either platform: every engine already allocates through the same mimalloc, Rust through Bun's global allocator and md4c through `md4c_alloc.h`, and one mimalloc configuration keeps the platforms comparable. The runner's transparent huge page mode is in [host.txt](host.txt).

### The lock was seeded, not replayed with `--locked`

The build seeded Cargo with `docs/reports/2026-09-21-native-round-4/Cargo.lock` through `--bun-lock` because the v2 path package's version line differs from that lock. `CARGO_NET_OFFLINE=true` and `--offline` kept the build itself offline, and `prepare.py` fails if Cargo resolves any registry package outside the union of the engine locks. All 67 resolved registry packages are identical to the seed lock in name, version, and checksum.

## Source audit

`harness/audit_sources.py` checked the exported sources against the pinned Git blobs and the checksummed archives ([source-audit.json.gz](source-audit.json.gz)):

- v1: 35 files, 0 mismatches
- v2: 182 files, 0 mismatches
- bun: 3495 files, 0 mismatches
- md4c: 10 files, 0 mismatches
- ox-content: 393 files, 0 mismatches
- mimalloc archive: 402 files, 0 mismatches
- highway archive: 278 files, 0 mismatches

## Measurement conditions

The job runs one step at a time, so no build or other step ran while run.py was timing. Load averages and hypervisor steal per process round:

| Round | Start (UTC) | End | Load before | Load after | Steal |
| ---: | --- | --- | ---: | ---: | ---: |
| 1 | 14:24:26 | 14:28:09 | 1.91 | 1.11 | 0.00% |
| 2 | 14:28:09 | 14:31:53 | 1.11 | 1.20 | 0.00% |
| 3 | 14:31:53 | 14:35:37 | 1.20 | 1.11 | 0.00% |

Thermal and clock readings, where the virtual machine exposes them, are in `run.json`.

## Reproduction

Run the workflow again with the same v2 revision:

```sh
gh workflow run native-comparison.yml -f revision=09e5e866da50690d24ef7969780bff3d4983795e -f pgo=true
```

or follow the harness README's reproduction commands on a Linux x86-64 host with clang, using this report's `restore.py` in an empty cache directory.
