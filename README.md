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

## CommonMark Compliance

Current: **292/652 tests passing (44.8%)**

| Section | Coverage |
|---------|----------|
| Blank lines | 100% |
| Textual content | 100% |
| Inlines | 100% |
| ATX headings | 89% |
| Emphasis | 87% |
| Autolinks | 79% |
| Paragraphs | 75% |
| Thematic breaks | 68% |
| Code spans | 68% |
| Fenced code blocks | 62% |
| Block quotes | 52% |
| Links | 34% |

**Intentionally out of scope:**
- Full HTML block parsing
- Reference link definitions
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
