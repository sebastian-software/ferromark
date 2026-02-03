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

Benchmarked on Apple Silicon (M-series) against other Rust Markdown parsers:

| Parser | Throughput (Medium) | Throughput (Large) | Relative |
|--------|--------------------:|-------------------:|----------|
| **md-fast** | **189 MiB/s** | **191 MiB/s** | **1.0x** |
| pulldown-cmark | 154 MiB/s | 217 MiB/s | 0.81x |
| comrak | 49 MiB/s | 63 MiB/s | 0.26x |
| markdown-rs | 7.2 MiB/s | 7.3 MiB/s | 0.04x |

**Key results:**
- **23% faster** than pulldown-cmark on typical documents
- **3.9x faster** than comrak (full CommonMark/GFM)
- **26x faster** than markdown-rs

Run benchmarks: `cargo bench --bench comparison`

## CommonMark Compliance

**In-scope compliance: 87.1% (317/364 tests)**

We track compliance against a filtered subset of CommonMark tests, excluding features intentionally not supported (HTML blocks, setext headings, indented code blocks, reference links, tabs).

| Section | Coverage |
|---------|----------|
| ATX headings | 100% |
| Autolinks | 100% |
| Backslash escapes | 100% |
| Blank lines | 100% |
| Code spans | 100% |
| Emphasis | 100% |
| Fenced code blocks | 100% |
| Hard line breaks | 100% |
| Images | 100% |
| Inlines | 100% |
| Paragraphs | 100% |
| Precedence | 100% |
| Soft line breaks | 100% |
| Textual content | 100% |
| Thematic breaks | 100% |
| Links | 76% |
| Block quotes | 65% |
| List items | 61% |
| Lists | 53% |
| Entity references | 31% |

**Intentionally out of scope:**
- HTML blocks (security by design)
- Reference link definitions
- Setext headings
- Indented code blocks
- Tabs
- Tables (GFM extension)

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
