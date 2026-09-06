# Bun Markdown parser provenance

Investigated Bun revision: [`76e9dcc6ad272a4fb1ee4a4dbbde4809201b71d1`](https://github.com/oven-sh/bun/tree/76e9dcc6ad272a4fb1ee4a4dbbde4809201b71d1/src/md).

The documented lineage is **md4c (C) → Bun's Zig port → Bun's Rust port**.

- **C to Zig — January 29, 2026.** The [introducing commit](https://github.com/oven-sh/bun/commit/1bfe5c6b37e65995ac58e761ccbff7bb7b8dc954) explicitly describes porting md4c from C to Zig under `src/md/`. The [official Bun 1.3.8 announcement](https://bun.com/blog/bun-v1.3.8) independently states the same origin.
- **Zig to Rust — May 14, 2026.** The Rust Markdown sources were introduced by [“Rewrite Bun in Rust”](https://github.com/oven-sh/bun/commit/23427dbc12fdcff30c23a96a3d6a66d62fdc091d). The [parser at that revision](https://github.com/oven-sh/bun/blob/23427dbc12fdcff30c23a96a3d6a66d62fdc091d/src/md/parser.rs#L5) documents direct translations of Zig types, nested declarations, and allocator handling.
- **Rust became the sole compiled implementation.** The [June 25 removal of Zig sources](https://github.com/oven-sh/bun/commit/d4514457e837fc3c884496c70a90df555bb95221) explains that the remaining Zig files had only served as behavioral references alongside their Rust replacements.

This makes `bun_md` an **md4c-derived Rust implementation**, not evidence of an independently designed parser algorithm. Benchmarking it alongside C md4c remains useful: it compares implementations with shared ancestry and different language, allocation, rendering, and maintenance choices. It does not isolate the effect of the implementation language.

The current code is also more than a frozen translation. Subsequent changes address quadratic behavior in [unclosed link brackets](https://github.com/oven-sh/bun/commit/30c007120807d4576417b897f86e22f96e45d939), [unterminated inline HTML](https://github.com/oven-sh/bun/commit/aff1bb1a060fe257a5777bb08afd2e7da193d2f7), and [nested link/image labels](https://github.com/oven-sh/bun/commit/27d5181ad62d2aa76a9d2adebada09a16bdabddd). Results must therefore identify the tested Bun revision rather than treating Bun as interchangeable with upstream md4c.

## License observation

Bun's [license document at the investigated revision](https://github.com/oven-sh/bun/blob/76e9dcc6ad272a4fb1ee4a4dbbde4809201b71d1/LICENSE.md) identifies Bun itself as MIT-licensed. No md4c-specific credit was found in that document or the inspected Markdown module headers. This is a limited source observation, not a legal conclusion; the commit history above establishes provenance independently of those headers.
