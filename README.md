# ferromark

Fast Markdown-to-HTML for Rust workloads where throughput and predictable latency matter.

## Why ferromark

- Built for production paths, not toy inputs: docs pipelines, API rendering, and CLIs.
- Streaming parser design avoids AST overhead on the hot path.
- CommonMark-compliant while still tuned for raw speed.
- Small dependency surface and straightforward integration.

## Design goals

- **Linear time behavior**: no regex backtracking, no parser surprises on large inputs.
- **Low allocation pressure**: compact `Range` references into the input instead of copying text.
- **Cache-friendly execution**: tight scanning loops, lookup tables, and reusable buffers.
- **Operational safety**: explicit depth/limit guards against pathological nesting.

## Architecture at a glance

```
Input bytes (&[u8])
       │
       ▼
   Block parser (line-oriented)
       │ emits BlockEvent stream
       ▼
   Inline parser (per text range)
       │ emits InlineEvent stream
       ▼
   HTML writer (direct buffer writes)
       │
       ▼
   Output (Vec<u8>)
```

### Why this is fast

- **Block pass stays simple**: cheap line scanning via `memchr`, container stack for quotes/lists.
- **Inline pass is staged**: collect marks -> resolve precedence (code, links, emphasis) -> emit.
- **Hot-path tuning**: `#[inline]` where it matters, `#[cold]` for rare paths, table-driven classification.
- **CommonMark emphasis done right**: modulo-3 delimiter handling without expensive rescans.

## Performance

Benchmarked on Apple Silicon (M-series), latest run: February 6, 2026.
Workload: synthetic wiki-style documents with text-heavy paragraphs, lists, code blocks, and representative CommonMark features (`benches/fixtures/commonmark-5k.md`, `benches/fixtures/commonmark-50k.md`).
Method: output buffers are reused for ferromark, md4c, and pulldown-cmark where APIs allow; comrak allocates output internally. Main table uses non-PGO binaries for apples-to-apples defaults.

**CommonMark 5KB**
| Parser | Throughput | Relative (vs ferromark) |
|--------|-----------:|----------------------:|
| **ferromark** | **301.1 MiB/s** | **1.00x** |
| md4c | 264.8 MiB/s | 0.88x |
| pulldown-cmark | 268.5 MiB/s | 0.89x |
| comrak | 79.2 MiB/s | 0.26x |

**CommonMark 50KB**
| Parser | Throughput | Relative (vs ferromark) |
|--------|-----------:|----------------------:|
| **ferromark** | **317.8 MiB/s** | **1.00x** |
| md4c | 260.6 MiB/s | 0.82x |
| pulldown-cmark | 274.3 MiB/s | 0.86x |
| comrak | 77.3 MiB/s | 0.24x |

Other candidates like markdown-rs are far slower in this workload and are omitted from the main tables to keep the comparison focused. Happy to run them on request.

**Key results:**
- ferromark is **~12.1% faster** than pulldown-cmark at 5KB and **~15.9% faster** at 50KB.
- ferromark is **~3.8-4.1x faster** than comrak across 5-50KB.
- ferromark is **~13.7% faster** than md4c at 5KB and **~21.9% faster** at 50KB.

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
      <th>ferromark</th>
      <th>md4c</th>
      <th>pulldown-cmark</th>
      <th>comrak</th>
    </tr>
  </thead>
  <tbody>
    <tr><td colspan="5"><strong>Performance and Memory</strong></td></tr>
    <tr>
      <td>Zero-copy text handling</td>
      <td align="center">*****</td>
      <td align="center">****</td>
      <td align="center">****</td>
      <td align="center">**</td>
    </tr>
    <tr><td colspan="5"><small>Zero-copy means most text slices point directly into input, which reduces allocations and copy costs. <em>Mapping:</em> ferromark uses ranges; md4c and pulldown-cmark borrow slices; comrak allocates AST nodes.</small></td></tr>
    <tr>
      <td>Allocation pressure (hot path)</td>
      <td align="center">*****</td>
      <td align="center">*****</td>
      <td align="center">****</td>
      <td align="center">**</td>
    </tr>
    <tr><td colspan="5"><small>Fewer allocations in tight loops improves CPU utilization and reduces allocator overhead. <em>Mapping:</em> Streaming parsers allocate less during parse/render; AST parsers allocate many nodes.</small></td></tr>
    <tr>
      <td>Output buffer reuse</td>
      <td align="center">*****</td>
      <td align="center">*****</td>
      <td align="center">****</td>
      <td align="center">**</td>
    </tr>
    <tr><td colspan="5"><small>Reusing output buffers avoids repeated allocations across runs and stabilizes performance. <em>Mapping:</em> ferromark, md4c, and pulldown-cmark allow reuse; comrak allocates internally.</small></td></tr>
    <tr>
      <td>Memory locality (working set size)</td>
      <td align="center">*****</td>
      <td align="center">*****</td>
      <td align="center">****</td>
      <td align="center">**</td>
    </tr>
    <tr><td colspan="5"><small>A small working set fits in cache and reduces memory traffic. <em>Mapping:</em> Streaming parsers keep the working set small; AST-based parsing expands it.</small></td></tr>
    <tr>
      <td>Cache friendliness</td>
      <td align="center">*****</td>
      <td align="center">****</td>
      <td align="center">****</td>
      <td align="center">**</td>
    </tr>
    <tr><td colspan="5"><small>Linear scans and contiguous buffers are usually best for CPU caches. <em>Mapping:</em> ferromark and md4c favor linear scans; pulldown-cmark is close; comrak traverses AST allocations.</small></td></tr>
    <tr>
      <td>SIMD availability (optional)</td>
      <td align="center">****</td>
      <td align="center">***</td>
      <td align="center">****</td>
      <td align="center">**</td>
    </tr>
    <tr><td colspan="5"><small>SIMD can accelerate scanning for special characters if the SIMD path is hot enough. <em>Mapping:</em> ferromark and pulldown-cmark have SIMD paths; md4c relies on C optimizations; comrak is not SIMD-focused.</small></td></tr>
    <tr>
      <td>Unsafe usage / low-level scanning</td>
      <td align="center">***</td>
      <td align="center">**</td>
      <td align="center">****</td>
      <td align="center">****</td>
    </tr>
    <tr><td colspan="5"><small>Targeted unsafe can remove bounds checks and speed up hot loops, but increases maintenance cost. <em>Mapping:</em> md4c is C; ferromark uses targeted unsafe; pulldown-cmark and comrak are mostly safe Rust.</small></td></tr>
    <tr>
      <td>Dependency footprint</td>
      <td align="center">*****</td>
      <td align="center">*****</td>
      <td align="center">****</td>
      <td align="center">**</td>
    </tr>
    <tr><td colspan="5"><small>Fewer dependencies simplify builds and reduce binary bloat. <em>Mapping:</em> md4c and ferromark are minimal; pulldown-cmark is moderate; comrak is heavier.</small></td></tr>
    <tr>
      <td>Throughput ceiling (architectural)</td>
      <td align="center">*****</td>
      <td align="center">*****</td>
      <td align="center">****</td>
      <td align="center">**</td>
    </tr>
    <tr><td colspan="5"><small>With fewer allocations and tighter hot loops, streaming architectures generally allow higher throughput ceilings. <em>Mapping:</em> ferromark and md4c lead here; pulldown-cmark is close; comrak trades throughput for flexibility.</small></td></tr>
    <tr>
      <td>Core compactness (moving parts)</td>
      <td align="center">****</td>
      <td align="center">*****</td>
      <td align="center">****</td>
      <td align="center">***</td>
    </tr>
    <tr><td colspan="5"><small>A compact core is easier to tune and reason about. <em>Mapping:</em> md4c is very compact; ferromark is lean; pulldown-cmark is moderate; comrak is larger by design.</small></td></tr>
    <tr>
      <td>Portability</td>
      <td align="center">****</td>
      <td align="center">*****</td>
      <td align="center">****</td>
      <td align="center">****</td>
    </tr>
    <tr><td colspan="5"><small>Portability matters for embedding and wide deployment. <em>Mapping:</em> md4c compiles almost anywhere with a C toolchain; the Rust crates are broadly portable too.</small></td></tr>
    <tr><td colspan="5"><strong>Feature Coverage and Extensibility</strong></td></tr>
    <tr>
      <td>Extension breadth (GFM and extras)</td>
      <td align="center">**</td>
      <td align="center">***</td>
      <td align="center">****</td>
      <td align="center">*****</td>
    </tr>
    <tr><td colspan="5"><small>More extensions increase compatibility but add parsing work. <em>Mapping:</em> comrak offers the broadest extension catalog; pulldown-cmark and md4c support common GFM features; ferromark focuses on CommonMark.</small></td></tr>
    <tr>
      <td>Spec compliance focus (CommonMark)</td>
      <td align="center">*****</td>
      <td align="center">*****</td>
      <td align="center">****</td>
      <td align="center">*****</td>
    </tr>
    <tr><td colspan="5"><small>Full compliance adds edge-case handling. All four are strong here, but more features usually means more code on the hot path. <em>Mapping:</em> All four target CommonMark; comrak and md4c emphasize full compliance; pulldown-cmark adds extensions; ferromark is focused.</small></td></tr>
    <tr>
      <td>Extension configuration surface</td>
      <td align="center">***</td>
      <td align="center">*****</td>
      <td align="center">****</td>
      <td align="center">****</td>
    </tr>
    <tr><td colspan="5"><small>Fine-grained flags let you disable features to reduce work. <em>Mapping:</em> md4c has many flags; pulldown-cmark and comrak use options; ferromark keeps configuration minimal.</small></td></tr>
    <tr>
      <td>Raw HTML control (allow/deny)</td>
      <td align="center">****</td>
      <td align="center">*****</td>
      <td align="center">***</td>
      <td align="center">****</td>
    </tr>
    <tr><td colspan="5"><small>Disabling raw HTML can simplify parsing and output. <em>Mapping:</em> md4c and comrak expose explicit switches; ferromark also exposes an explicit <code>allow_html</code> option; pulldown-cmark is more fixed in defaults.</small></td></tr>
    <tr>
      <td>Tables, task lists, strikethrough</td>
      <td align="center">**</td>
      <td align="center">****</td>
      <td align="center">****</td>
      <td align="center">*****</td>
    </tr>
    <tr><td colspan="5"><small>These GFM features are common in real-world Markdown. <em>Mapping:</em> md4c, pulldown-cmark, and comrak support them; ferromark keeps the core smaller for speed.</small></td></tr>
    <tr>
      <td>Footnotes</td>
      <td align="center">*</td>
      <td align="center">*</td>
      <td align="center">****</td>
      <td align="center">*****</td>
    </tr>
    <tr><td colspan="5"><small>Footnotes add extra parsing and rendering complexity. <em>Mapping:</em> pulldown-cmark and comrak support footnotes; ferromark and md4c do not focus on them.</small></td></tr>
    <tr>
      <td>Math support</td>
      <td align="center">*</td>
      <td align="center">****</td>
      <td align="center">*</td>
      <td align="center">****</td>
    </tr>
    <tr><td colspan="5"><small>Math support often requires custom extensions. <em>Mapping:</em> md4c includes LaTeX math flags; comrak supports math extensions; ferromark and pulldown-cmark do not target math in the core.</small></td></tr>
    <tr>
      <td>Permissive autolinks</td>
      <td align="center">**</td>
      <td align="center">*****</td>
      <td align="center">***</td>
      <td align="center">****</td>
    </tr>
    <tr><td colspan="5"><small>Permissive autolinks trade strictness for convenience. <em>Mapping:</em> md4c exposes explicit flags for permissive URL/email/WWW autolinks; comrak has relaxed autolinks; pulldown-cmark focuses on spec defaults.</small></td></tr>
    <tr>
      <td>Wiki links</td>
      <td align="center">*</td>
      <td align="center">****</td>
      <td align="center">*</td>
      <td align="center">****</td>
    </tr>
    <tr><td colspan="5"><small>Wiki links are a non-CommonMark extension used in some ecosystems. <em>Mapping:</em> md4c and comrak support wiki links via flags/extensions; pulldown-cmark and ferromark do not.</small></td></tr>
    <tr>
      <td>Underline extension</td>
      <td align="center">*</td>
      <td align="center">****</td>
      <td align="center">*</td>
      <td align="center">****</td>
    </tr>
    <tr><td colspan="5"><small>Underline is an extension that changes emphasis semantics. <em>Mapping:</em> md4c and comrak include underline extensions; pulldown-cmark and ferromark stick closer to CommonMark emphasis rules.</small></td></tr>
    <tr>
      <td>Task list flexibility</td>
      <td align="center">*</td>
      <td align="center">***</td>
      <td align="center">***</td>
      <td align="center">*****</td>
    </tr>
    <tr><td colspan="5"><small>Relaxed task list parsing can improve compatibility with messy inputs. <em>Mapping:</em> comrak offers relaxed task list options; md4c and pulldown-cmark support task lists with fewer knobs.</small></td></tr>
    <tr>
      <td>Output safety toggles</td>
      <td align="center">***</td>
      <td align="center">*****</td>
      <td align="center">***</td>
      <td align="center">*****</td>
    </tr>
    <tr><td colspan="5"><small>Safety toggles control whether raw HTML is emitted or escaped. <em>Mapping:</em> md4c and comrak provide explicit unsafe/escape switches; ferromark provides a simple <code>allow_html</code> toggle (without sanitizer mode); pulldown-cmark is more fixed in defaults.</small></td></tr>
    <tr>
      <td>no_std viability</td>
      <td align="center">**</td>
      <td align="center">****</td>
      <td align="center">*****</td>
      <td align="center">*</td>
    </tr>
    <tr><td colspan="5"><small>no_std support matters for embedded or constrained environments. <em>Mapping:</em> pulldown-cmark supports no_std builds with features; md4c can be embedded in C environments; ferromark and comrak assume std.</small></td></tr>
    <tr><td colspan="5"><strong>Architecture and Parsing Model</strong></td></tr>
    <tr>
      <td>Parser model (streaming, no AST)</td>
      <td align="center">*****</td>
      <td align="center">*****</td>
      <td align="center">****</td>
      <td align="center">**</td>
    </tr>
    <tr><td colspan="5"><small>Streaming parsers can emit output as they scan input, which avoids building an intermediate tree and keeps memory and cache pressure low. <em>Mapping:</em> ferromark and md4c stream; pulldown-cmark uses a pull iterator; comrak builds an AST.</small></td></tr>
    <tr>
      <td>API style (push callbacks / pull iterator / AST)</td>
      <td align="center">****</td>
      <td align="center">*****</td>
      <td align="center">****</td>
      <td align="center">**</td>
    </tr>
    <tr><td colspan="5"><small>Push callbacks and pull iterators are good for streaming output; AST APIs are better for transformations but add overhead for straight HTML rendering. <em>Mapping:</em> md4c is push callbacks; pulldown-cmark is pull iterator; comrak is AST; ferromark is streaming events.</small></td></tr>
    <tr>
      <td>Parse/render separation</td>
      <td align="center">****</td>
      <td align="center">*****</td>
      <td align="center">*****</td>
      <td align="center">***</td>
    </tr>
    <tr><td colspan="5"><small>Clear separation lets parsers stay simple and fast, while renderers can be swapped or tuned. <em>Mapping:</em> md4c and pulldown-cmark separate parse and render clearly; ferromark is mostly separated; comrak leans on AST-based renderers.</small></td></tr>
    <tr>
      <td>Inline parsing pipeline (multi-phase, delimiter stacks)</td>
      <td align="center">*****</td>
      <td align="center">****</td>
      <td align="center">****</td>
      <td align="center">**</td>
    </tr>
    <tr><td colspan="5"><small>Multi-phase inline parsing (collect -> resolve -> emit) keeps the hot path linear and avoids backtracking. <em>Mapping:</em> ferromark uses multi-phase inline parsing; md4c and pulldown-cmark are optimized byte scanners; comrak does more AST bookkeeping.</small></td></tr>
    <tr>
      <td>Emphasis matching efficiency</td>
      <td align="center">*****</td>
      <td align="center">****</td>
      <td align="center">****</td>
      <td align="center">**</td>
    </tr>
    <tr><td colspan="5"><small>Efficient emphasis handling reduces rescans and backtracking. Stack-based algorithms tend to win on long text-heavy documents. <em>Mapping:</em> ferromark uses modulo-3 stacks; md4c and pulldown-cmark are optimized; comrak pays AST overhead.</small></td></tr>
    <tr>
      <td>Link reference processing cost</td>
      <td align="center">****</td>
      <td align="center">****</td>
      <td align="center">****</td>
      <td align="center">***</td>
    </tr>
    <tr><td colspan="5"><small>Link labels need normalization (case folding and entity handling). Optimized implementations reduce allocations and Unicode overhead. <em>Mapping:</em> All four normalize labels; ferromark, md4c, and pulldown-cmark focus on minimizing allocations; comrak handles more feature paths.</small></td></tr>
    <tr>
      <td>Unicode handling configurability</td>
      <td align="center">***</td>
      <td align="center">*****</td>
      <td align="center">***</td>
      <td align="center">***</td>
    </tr>
    <tr><td colspan="5"><small>Configurable Unicode handling can simplify hot paths or support special environments. <em>Mapping:</em> md4c can be built for UTF-8, UTF-16, or ASCII-only; the Rust parsers generally assume UTF-8.</small></td></tr>
    <tr><td colspan="5"><strong>Rendering and Output</strong></td></tr>
    <tr>
      <td>Output streaming (incremental)</td>
      <td align="center">*****</td>
      <td align="center">*****</td>
      <td align="center">****</td>
      <td align="center">**</td>
    </tr>
    <tr><td colspan="5"><small>Output streaming lets you write HTML incrementally, which lowers peak memory and removes extra passes. <em>Mapping:</em> ferromark and md4c stream to buffers or callbacks; pulldown-cmark streams events; comrak often renders after AST work.</small></td></tr>
    <tr>
      <td>Output customization hooks</td>
      <td align="center">***</td>
      <td align="center">*****</td>
      <td align="center">****</td>
      <td align="center">*****</td>
    </tr>
    <tr><td colspan="5"><small>Callbacks and ASTs are great for custom rendering but add indirection compared to a single tight rendering loop. <em>Mapping:</em> md4c callbacks and comrak AST are very flexible; pulldown-cmark iterators are easy to transform; ferromark is lower level.</small></td></tr>
    <tr>
      <td>Output formats</td>
      <td align="center">**</td>
      <td align="center">***</td>
      <td align="center">****</td>
      <td align="center">*****</td>
    </tr>
    <tr><td colspan="5"><small>More output formats increase flexibility but add complexity. <em>Mapping:</em> comrak can emit HTML, XML, and CommonMark; pulldown-cmark provides HTML plus event streams; md4c has HTML renderer and callbacks; ferromark targets HTML.</small></td></tr>
    <tr>
      <td>Source position support</td>
      <td align="center">**</td>
      <td align="center">**</td>
      <td align="center">*****</td>
      <td align="center">****</td>
    </tr>
    <tr><td colspan="5"><small>Tracking source positions is useful for diagnostics and tooling, but adds overhead. <em>Mapping:</em> pulldown-cmark has strong source map support; comrak can emit source positions; ferromark and md4c are lighter.</small></td></tr>
    <tr>
      <td>Source map tooling (API or CLI)</td>
      <td align="center">**</td>
      <td align="center">**</td>
      <td align="center">*****</td>
      <td align="center">****</td>
    </tr>
    <tr><td colspan="5"><small>Source maps improve debuggability and tooling integration. <em>Mapping:</em> pulldown-cmark exposes event ranges; comrak can emit source position attributes; ferromark and md4c keep this minimal.</small></td></tr>
    <tr>
      <td>IO friendliness (small writes)</td>
      <td align="center">****</td>
      <td align="center">*****</td>
      <td align="center">***</td>
      <td align="center">**</td>
    </tr>
    <tr><td colspan="5"><small>Many small writes can be expensive without buffering. <em>Mapping:</em> md4c and ferromark stream into buffers or callbacks; pulldown-cmark recommends buffered output; comrak often builds strings after AST work.</small></td></tr>
  </tbody>
</table>

## CommonMark Compliance


**Full compliance: 100% (652/652 tests)**

All CommonMark spec tests pass (no filtering).

## Usage

```rust
use ferromark::to_html;

let html = ferromark::to_html("# Hello\n\n**World**");
assert!(html.contains("<h1>Hello</h1>"));
assert!(html.contains("<strong>World</strong>"));
```

### Zero-allocation API

```rust
let mut buffer = Vec::new();
ferromark::to_html_into("# Reuse me", &mut buffer);
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
