# md-fast

Ultra-high-performance Markdown to HTML compiler in Rust.

## Design Philosophy

**Speed through simplicity.** Every architectural decision prioritizes throughput:

### Zero-Copy Parsing
- All text references use `Range` (8-byte `u32` pair) instead of `String`
- No allocations during parsing - ranges point into original input
- Streaming events, no intermediate AST

### O(n) Guaranteed
- No regex, no backtracking
- Single-pass block parsing
- Three-phase inline parsing (collect → resolve → emit)
- DoS-resistant via hard limits on nesting depth

### Minimal Dependencies
- `memchr` - SIMD-accelerated byte searching
- `smallvec` - Stack-allocated vectors for typical nesting depths

## Architecture

```
Input bytes (&[u8])
       │
       ▼
   Block Parser (line-oriented)
       │ emits: BlockEvent stream
       ▼
   Inline Parser (per text range)
       │ emits: InlineEvent stream
       ▼
   HTML Writer (direct buffer writes)
       │
       ▼
   Output (Vec<u8>)
```

### Block Parser
- Line-by-line scanning with `memchr` for newlines
- Container stack for blockquotes/lists
- Emits ranges for inline content

### Inline Parser
Three-phase approach inspired by md4c:

1. **Mark Collection**: Single pass collecting delimiter positions (`*`, `` ` ``, `[`, etc.)
2. **Mark Resolution**: Process by precedence (code spans → links → emphasis)
3. **Event Emission**: Walk resolved marks, emit events

### Key Optimizations
- 256-byte lookup tables for character classification
- Modulo-3 stacks for emphasis matching (CommonMark "rule of three")
- `#[inline]` on hot paths, `#[cold]` on error paths
- Buffer reuse across parse calls

## Performance

Benchmarked on Apple Silicon (M-series) against other Rust Markdown parsers (latest run: Feb 5, 2026).
Input: synthetic wiki-style articles with text-heavy paragraphs, lists, and code blocks, plus CommonMark features used at least once (`benches/fixtures/commonmark-5k.md`, `benches/fixtures/commonmark-50k.md`).
Output buffers are reused for md-fast, md4c, and pulldown-cmark where their APIs allow; comrak allocates output internally.

**CommonMark 5KB**
| Parser | Throughput | Relative (vs md-fast) |
|--------|-----------:|----------------------:|
| **md-fast** | **265.4 MiB/s** | **1.00x** |
| md4c | 264.6 MiB/s | 1.00x |
| pulldown-cmark | 242.7 MiB/s | 0.92x |
| comrak | 78.0 MiB/s | 0.29x |

**CommonMark 50KB**
| Parser | Throughput | Relative (vs md-fast) |
|--------|-----------:|----------------------:|
| **md-fast** | **276.3 MiB/s** | **1.00x** |
| md4c | 261.0 MiB/s | 0.94x |
| pulldown-cmark | 270.9 MiB/s | 0.98x |
| comrak | 77.0 MiB/s | 0.28x |

Other candidates like markdown-rs are far slower in this workload and are omitted from the main tables to keep the comparison focused. Happy to run them on request.

**Key results:**
- md-fast is **~2% faster** than pulldown-cmark at 50KB and **~9% faster** at 5KB.
- md-fast is **~3.4-3.6x faster** than comrak across 5-50KB.
- md-fast is **~5-6% faster** than md4c at 50KB; essentially tied at 5KB.

Run benchmarks: `cargo bench --bench comparison`

## Technical Notes (Top-Tier Approaches)

These are the four parsers included in the main benchmark. The ratings are about **performance-oriented architecture for end-to-end Markdown-to-HTML** (not feature depth or ergonomics).

**md-fast** (Rating: 5/5)
- Streaming, zero-copy design: `Range`-based slices into input, no AST.
- Three-phase inline parsing (collect → resolve → emit), lookup tables, and aggressive buffer reuse.
- HTML writer emits directly into a reusable `Vec<u8>`.

**md4c** (Rating: 5/5)
- C parser with a single entrypoint (`md_parse`) and a callback-based push model.
- Very compact core (single C file + header), no external dependencies, fast by design.
- HTML renderer (`md_html`) streams chunks to a callback, leaving policy decisions to the caller.

**pulldown-cmark** (Rating: 4/5)
- Pull-parser design (iterator of events) with minimal allocations and CoW text.
- Pure Rust with optional SIMD scanning; supports source maps and clean separation of parse/render.
- Fast and flexible, but the event model can add overhead for pure HTML throughput.

**comrak** (Rating: 3/5)
- Full AST built in an arena; optimized for transformations and rich extensions.
- CommonMark + GFM, plus many extensions and multiple output formats.
- The AST/arena approach trades throughput for flexibility and rich document manipulation.

## CommonMark Compliance

**Full compliance: 100% (652/652 tests)**

All CommonMark spec tests pass (no filtering).

## Usage

```rust
use md_fast::to_html;

let html = md_fast::to_html("# Hello\n\n**World**");
assert!(html.contains("<h1>Hello</h1>"));
assert!(html.contains("<strong>World</strong>"));
```

### Zero-allocation API

```rust
let mut buffer = Vec::new();
md_fast::to_html_into("# Reuse me", &mut buffer);
// buffer can be reused for next call
```

## Building

```bash
# Development
cargo build

# Optimized release (recommended for benchmarks)
cargo build --release

# Run tests
cargo test

# Run CommonMark spec tests
cargo test --test commonmark_spec -- --nocapture

# Run benchmarks
cargo bench
```

## Project Structure

```
src/
├── lib.rs          # Public API (to_html, to_html_into)
├── block/          # Block-level parser
│   ├── parser.rs   # Line-oriented block parsing
│   └── event.rs    # BlockEvent types
├── inline/         # Inline-level parser
│   ├── mod.rs      # Three-phase inline parsing
│   ├── marks.rs    # Mark collection
│   ├── code_span.rs
│   ├── emphasis.rs # Modulo-3 stack optimization
│   └── links.rs    # Link/image/autolink parsing
├── cursor.rs       # Pointer-based byte cursor
├── range.rs        # Compact u32 range type
├── render.rs       # HTML writer
├── escape.rs       # HTML escaping (memchr-optimized)
└── limits.rs       # DoS prevention constants
```

## Future Optimizations

Planned for Phase 7:
- `simdutf8` for SIMD UTF-8 input validation
- NEON intrinsics for ARM marker scanning
- Profile-guided optimization (PGO)
- Loop unrolling in hot scanning paths

## License

MIT OR Apache-2.0
