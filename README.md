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

These are the four parsers included in the main benchmark. Ratings are per technical feature (1-5), focused on **end-to-end Markdown-to-HTML throughput** in typical workloads. A higher score means the feature is better aligned with high throughput, not necessarily broader functionality.

Legend:
- 5 = strong fit for fast end-to-end HTML
- 3 = balanced or neutral
- 1 = poor fit for fast end-to-end HTML

Each feature row is followed by a short plain-language explanation.

<table>
  <thead>
    <tr>
      <th>Feature</th>
      <th>md-fast</th>
      <th>md4c</th>
      <th>pulldown-cmark</th>
      <th>comrak</th>
    </tr>
  </thead>
  <tbody>
    <tr><td colspan="5"><strong>Core Parsing Architecture</strong></td></tr>
    <tr>
      <td>Parser model (streaming, no AST)</td>
      <td align="center">5 *****</td>
      <td align="center">5 *****</td>
      <td align="center">4 ****</td>
      <td align="center">2 **</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> Streaming parsers can emit output as they scan input, which avoids building an intermediate tree and keeps memory and cache pressure low. <em>Mapping:</em> md-fast and md4c stream; pulldown-cmark uses a pull iterator; comrak builds an AST.</td></tr>

    <tr>
      <td>API style (push callbacks / pull iterator / AST)</td>
      <td align="center">4 ****</td>
      <td align="center">5 *****</td>
      <td align="center">4 ****</td>
      <td align="center">2 **</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> Push callbacks and pull iterators are good for streaming output; AST APIs are better for transformations but add overhead for straight HTML rendering. <em>Mapping:</em> md4c is push callbacks; pulldown-cmark is pull iterator; comrak is AST; md-fast is streaming events.</td></tr>

    <tr>
      <td>Parse/render separation</td>
      <td align="center">4 ****</td>
      <td align="center">5 *****</td>
      <td align="center">5 *****</td>
      <td align="center">3 ***</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> Clear separation lets parsers stay simple and fast, while renderers can be swapped or tuned. <em>Mapping:</em> md4c and pulldown-cmark separate parse and render clearly; md-fast is mostly separated; comrak leans on AST-based renderers.</td></tr>

    <tr>
      <td>Inline parsing pipeline (multi-phase, delimiter stacks)</td>
      <td align="center">5 *****</td>
      <td align="center">4 ****</td>
      <td align="center">4 ****</td>
      <td align="center">2 **</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> Multi-phase inline parsing (collect -> resolve -> emit) keeps the hot path linear and avoids backtracking. <em>Mapping:</em> md-fast uses multi-phase inline parsing; md4c and pulldown-cmark are optimized byte scanners; comrak does more AST bookkeeping.</td></tr>

    <tr>
      <td>Emphasis matching efficiency</td>
      <td align="center">5 *****</td>
      <td align="center">4 ****</td>
      <td align="center">4 ****</td>
      <td align="center">2 **</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> Efficient emphasis handling reduces rescans and backtracking. Stack-based algorithms tend to win on long text-heavy documents. <em>Mapping:</em> md-fast uses modulo-3 stacks; md4c and pulldown-cmark are optimized; comrak pays AST overhead.</td></tr>

    <tr>
      <td>Link reference processing cost</td>
      <td align="center">4 ****</td>
      <td align="center">4 ****</td>
      <td align="center">4 ****</td>
      <td align="center">3 ***</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> Link labels need normalization (case folding and entity handling). Optimized implementations reduce allocations and Unicode overhead. <em>Mapping:</em> All four normalize labels; md-fast, md4c, and pulldown-cmark focus on minimizing allocations; comrak handles more feature paths.</td></tr>

    <tr>
      <td>Unicode handling configurability</td>
      <td align="center">3 ***</td>
      <td align="center">5 *****</td>
      <td align="center">3 ***</td>
      <td align="center">3 ***</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> Configurable Unicode handling can simplify hot paths or support special environments. <em>Mapping:</em> md4c can be built for UTF-8, UTF-16, or ASCII-only; the Rust parsers generally assume UTF-8.</td></tr>

    <tr>
      <td>Spec compliance focus (CommonMark)</td>
      <td align="center">5 *****</td>
      <td align="center">5 *****</td>
      <td align="center">4 ****</td>
      <td align="center">5 *****</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> Full compliance adds edge-case handling. All four are strong here, but more features usually means more code on the hot path. <em>Mapping:</em> All four target CommonMark; comrak and md4c emphasize full compliance; pulldown-cmark adds extensions; md-fast is focused.</td></tr>

    <tr>
      <td>Extension configuration surface</td>
      <td align="center">3 ***</td>
      <td align="center">5 *****</td>
      <td align="center">4 ****</td>
      <td align="center">4 ****</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> Fine-grained flags let you disable features to reduce work. <em>Mapping:</em> md4c has many flags; pulldown-cmark and comrak use options; md-fast keeps configuration minimal.</td></tr>

    <tr>
      <td>Raw HTML control (allow/deny)</td>
      <td align="center">3 ***</td>
      <td align="center">5 *****</td>
      <td align="center">3 ***</td>
      <td align="center">4 ****</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> Disabling raw HTML can simplify parsing and output. <em>Mapping:</em> md4c and comrak expose explicit switches; md-fast and pulldown-cmark are more fixed in defaults.</td></tr>

    <tr><td colspan="5"><strong>Performance-Critical Mechanics</strong></td></tr>
    <tr>
      <td>Zero-copy text handling</td>
      <td align="center">5 *****</td>
      <td align="center">4 ****</td>
      <td align="center">4 ****</td>
      <td align="center">2 **</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> Zero-copy means most text slices point directly into input, which reduces allocations and copy costs. <em>Mapping:</em> md-fast uses ranges; md4c and pulldown-cmark borrow slices; comrak allocates AST nodes.</td></tr>

    <tr>
      <td>Allocation pressure (hot path)</td>
      <td align="center">5 *****</td>
      <td align="center">5 *****</td>
      <td align="center">4 ****</td>
      <td align="center">2 **</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> Fewer allocations in tight loops improves CPU utilization and reduces allocator overhead. <em>Mapping:</em> Streaming parsers allocate less during parse/render; AST parsers allocate many nodes.</td></tr>

    <tr>
      <td>Output buffer reuse</td>
      <td align="center">5 *****</td>
      <td align="center">5 *****</td>
      <td align="center">4 ****</td>
      <td align="center">2 **</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> Reusing output buffers avoids repeated allocations across runs and stabilizes performance. <em>Mapping:</em> md-fast, md4c, and pulldown-cmark allow reuse; comrak allocates internally.</td></tr>

    <tr>
      <td>Memory locality (working set size)</td>
      <td align="center">5 *****</td>
      <td align="center">5 *****</td>
      <td align="center">4 ****</td>
      <td align="center">2 **</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> A small working set fits in cache and reduces memory traffic. <em>Mapping:</em> Streaming parsers keep the working set small; AST-based parsing expands it.</td></tr>

    <tr>
      <td>Cache friendliness</td>
      <td align="center">5 *****</td>
      <td align="center">4 ****</td>
      <td align="center">4 ****</td>
      <td align="center">2 **</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> Linear scans and contiguous buffers are usually best for CPU caches. <em>Mapping:</em> md-fast and md4c favor linear scans; pulldown-cmark is close; comrak traverses AST allocations.</td></tr>

    <tr>
      <td>SIMD availability (optional)</td>
      <td align="center">4 ****</td>
      <td align="center">3 ***</td>
      <td align="center">4 ****</td>
      <td align="center">2 **</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> SIMD can accelerate scanning for special characters if the SIMD path is hot enough. <em>Mapping:</em> md-fast and pulldown-cmark have SIMD paths; md4c relies on C optimizations; comrak is not SIMD-focused.</td></tr>

    <tr>
      <td>Unsafe usage / low-level scanning</td>
      <td align="center">3 ***</td>
      <td align="center">2 **</td>
      <td align="center">4 ****</td>
      <td align="center">4 ****</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> Targeted unsafe can remove bounds checks and speed up hot loops, but increases maintenance cost. <em>Mapping:</em> md4c is C; md-fast uses targeted unsafe; pulldown-cmark and comrak are mostly safe Rust.</td></tr>

    <tr>
      <td>Dependency footprint</td>
      <td align="center">5 *****</td>
      <td align="center">5 *****</td>
      <td align="center">4 ****</td>
      <td align="center">2 **</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> Fewer dependencies simplify builds and reduce binary bloat. <em>Mapping:</em> md4c and md-fast are minimal; pulldown-cmark is moderate; comrak is heavier.</td></tr>

    <tr>
      <td>Throughput ceiling (architectural)</td>
      <td align="center">5 *****</td>
      <td align="center">5 *****</td>
      <td align="center">4 ****</td>
      <td align="center">2 **</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> With fewer allocations and tighter hot loops, streaming architectures generally allow higher throughput ceilings. <em>Mapping:</em> md-fast and md4c lead here; pulldown-cmark is close; comrak trades throughput for flexibility.</td></tr>

    <tr>
      <td>Core compactness (moving parts)</td>
      <td align="center">4 ****</td>
      <td align="center">5 *****</td>
      <td align="center">4 ****</td>
      <td align="center">3 ***</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> A compact core is easier to tune and reason about. <em>Mapping:</em> md4c is very compact; md-fast is lean; pulldown-cmark is moderate; comrak is larger by design.</td></tr>

    <tr>
      <td>Portability</td>
      <td align="center">4 ****</td>
      <td align="center">5 *****</td>
      <td align="center">4 ****</td>
      <td align="center">4 ****</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> Portability matters for embedding and wide deployment. <em>Mapping:</em> md4c compiles almost anywhere with a C toolchain; the Rust crates are broadly portable too.</td></tr>

    <tr><td colspan="5"><strong>Rendering and Output</strong></td></tr>
    <tr>
      <td>Output streaming (incremental)</td>
      <td align="center">5 *****</td>
      <td align="center">5 *****</td>
      <td align="center">4 ****</td>
      <td align="center">2 **</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> Output streaming lets you write HTML incrementally, which lowers peak memory and removes extra passes. <em>Mapping:</em> md-fast and md4c stream to buffers or callbacks; pulldown-cmark streams events; comrak often renders after AST work.</td></tr>

    <tr>
      <td>Output customization hooks</td>
      <td align="center">3 ***</td>
      <td align="center">5 *****</td>
      <td align="center">4 ****</td>
      <td align="center">5 *****</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> Callbacks and ASTs are great for custom rendering but add indirection compared to a single tight rendering loop. <em>Mapping:</em> md4c callbacks and comrak AST are very flexible; pulldown-cmark iterators are easy to transform; md-fast is lower level.</td></tr>

    <tr>
      <td>Output formats</td>
      <td align="center">2 **</td>
      <td align="center">3 ***</td>
      <td align="center">4 ****</td>
      <td align="center">5 *****</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> More output formats increase flexibility but add complexity. <em>Mapping:</em> comrak can emit HTML, XML, and CommonMark; pulldown-cmark provides HTML plus event streams; md4c has HTML renderer and callbacks; md-fast targets HTML.</td></tr>

    <tr>
      <td>Source position support</td>
      <td align="center">2 **</td>
      <td align="center">2 **</td>
      <td align="center">5 *****</td>
      <td align="center">4 ****</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> Tracking source positions is useful for diagnostics and tooling, but adds overhead. <em>Mapping:</em> pulldown-cmark has strong source map support; comrak can emit source positions; md-fast and md4c are lighter.</td></tr>

    <tr>
      <td>Source map tooling (API or CLI)</td>
      <td align="center">2 **</td>
      <td align="center">2 **</td>
      <td align="center">5 *****</td>
      <td align="center">4 ****</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> Source maps improve debuggability and tooling integration. <em>Mapping:</em> pulldown-cmark exposes event ranges; comrak can emit source position attributes; md-fast and md4c keep this minimal.</td></tr>

    <tr>
      <td>IO friendliness (small writes)</td>
      <td align="center">4 ****</td>
      <td align="center">5 *****</td>
      <td align="center">3 ***</td>
      <td align="center">2 **</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> Many small writes can be expensive without buffering. <em>Mapping:</em> md4c and md-fast stream into buffers or callbacks; pulldown-cmark recommends buffered output; comrak often builds strings after AST work.</td></tr>

    <tr><td colspan="5"><strong>Feature Surface and Extensibility</strong></td></tr>
    <tr>
      <td>Extension breadth (GFM and extras)</td>
      <td align="center">2 **</td>
      <td align="center">3 ***</td>
      <td align="center">4 ****</td>
      <td align="center">5 *****</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> More extensions increase compatibility but add parsing work. <em>Mapping:</em> comrak offers the broadest extension catalog; pulldown-cmark and md4c support common GFM features; md-fast focuses on CommonMark.</td></tr>

    <tr>
      <td>Tables, task lists, strikethrough</td>
      <td align="center">2 **</td>
      <td align="center">4 ****</td>
      <td align="center">4 ****</td>
      <td align="center">5 *****</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> These GFM features are common in real-world Markdown. <em>Mapping:</em> md4c, pulldown-cmark, and comrak support them; md-fast keeps the core smaller for speed.</td></tr>

    <tr>
      <td>Footnotes</td>
      <td align="center">1 *</td>
      <td align="center">1 *</td>
      <td align="center">4 ****</td>
      <td align="center">5 *****</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> Footnotes add extra parsing and rendering complexity. <em>Mapping:</em> pulldown-cmark and comrak support footnotes; md-fast and md4c do not focus on them.</td></tr>

    <tr>
      <td>Math support</td>
      <td align="center">1 *</td>
      <td align="center">4 ****</td>
      <td align="center">1 *</td>
      <td align="center">4 ****</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> Math support often requires custom extensions. <em>Mapping:</em> md4c includes LaTeX math flags; comrak supports math extensions; md-fast and pulldown-cmark do not target math in the core.</td></tr>

    <tr>
      <td>Permissive autolinks</td>
      <td align="center">2 **</td>
      <td align="center">5 *****</td>
      <td align="center">3 ***</td>
      <td align="center">4 ****</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> Permissive autolinks trade strictness for convenience. <em>Mapping:</em> md4c exposes explicit flags for permissive URL/email/WWW autolinks; comrak has relaxed autolinks; pulldown-cmark focuses on spec defaults.</td></tr>

    <tr>
      <td>Wiki links</td>
      <td align="center">1 *</td>
      <td align="center">4 ****</td>
      <td align="center">1 *</td>
      <td align="center">4 ****</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> Wiki links are a non-CommonMark extension used in some ecosystems. <em>Mapping:</em> md4c and comrak support wiki links via flags/extensions; pulldown-cmark and md-fast do not.</td></tr>

    <tr>
      <td>Underline extension</td>
      <td align="center">1 *</td>
      <td align="center">4 ****</td>
      <td align="center">1 *</td>
      <td align="center">4 ****</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> Underline is an extension that changes emphasis semantics. <em>Mapping:</em> md4c and comrak include underline extensions; pulldown-cmark and md-fast stick closer to CommonMark emphasis rules.</td></tr>

    <tr>
      <td>Task list flexibility</td>
      <td align="center">1 *</td>
      <td align="center">3 ***</td>
      <td align="center">3 ***</td>
      <td align="center">5 *****</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> Relaxed task list parsing can improve compatibility with messy inputs. <em>Mapping:</em> comrak offers relaxed task list options; md4c and pulldown-cmark support task lists with fewer knobs.</td></tr>

    <tr>
      <td>Output safety toggles</td>
      <td align="center">2 **</td>
      <td align="center">5 *****</td>
      <td align="center">3 ***</td>
      <td align="center">5 *****</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> Safety toggles control whether raw HTML is emitted or escaped. <em>Mapping:</em> md4c and comrak provide explicit unsafe/escape switches; md-fast and pulldown-cmark are more fixed in defaults.</td></tr>

    <tr>
      <td>no_std viability</td>
      <td align="center">2 **</td>
      <td align="center">4 ****</td>
      <td align="center">5 *****</td>
      <td align="center">1 *</td>
    </tr>
    <tr><td colspan="5"><em>Detail:</em> no_std support matters for embedded or constrained environments. <em>Mapping:</em> pulldown-cmark supports no_std builds with features; md4c can be embedded in C environments; md-fast and comrak assume std.</td></tr>

  </tbody>
</table>

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
