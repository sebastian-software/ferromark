# ferromark

[![CI](https://github.com/sebastian-software/ferromark/actions/workflows/ci.yml/badge.svg)](https://github.com/sebastian-software/ferromark/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/ferromark.svg)](https://crates.io/crates/ferromark)
[![docs.rs](https://docs.rs/ferromark/badge.svg)](https://docs.rs/ferromark)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust 1.85+](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)

Markdown to HTML at 309 MiB/s. Faster than pulldown-cmark, md4c (C), and comrak. Passes all 652 CommonMark spec tests. Every GFM extension included.

## Quick start

```rust
let html = ferromark::to_html("# Hello\n\n**World**");
```

One function call, no setup. When allocation pressure matters:

```rust
let mut buffer = Vec::new();
ferromark::to_html_into("# Reuse me", &mut buffer);
// buffer survives across calls — zero repeated allocation
```

## Benchmarks

Numbers, not adjectives. Apple Silicon (M-series), February 2026. All parsers run with GFM tables, strikethrough, and task lists enabled. Output buffers reused where APIs allow. Non-PGO binaries for a fair comparison.

**CommonMark 5 KB** (wiki-style, mixed content with tables)
| Parser | Throughput | vs ferromark |
|--------|----------:|------------:|
| **ferromark** | **289.9 MiB/s** | **baseline** |
| pulldown-cmark | 247.7 MiB/s | 0.85x |
| md4c (C) | 242.3 MiB/s | 0.84x |
| comrak | 73.7 MiB/s | 0.25x |

**CommonMark 50 KB** (same style, scaled)
| Parser | Throughput | vs ferromark |
|--------|----------:|------------:|
| **ferromark** | **309.3 MiB/s** | **baseline** |
| pulldown-cmark | 271.7 MiB/s | 0.88x |
| md4c (C) | 247.4 MiB/s | 0.80x |
| comrak | 76.0 MiB/s | 0.25x |

17% faster than pulldown-cmark. 25% faster than md4c. 4x faster than comrak.

The fixtures are synthetic wiki-style documents with paragraphs, lists, code blocks, and tables. Nothing cherry-picked. Run them yourself: `cargo bench --bench comparison`

## What you get

**Full CommonMark**: 652/652 spec tests pass. No filtering, no exceptions.

**All five GFM extensions**: Tables, strikethrough, task lists, autolink literals, disallowed raw HTML.

**Beyond GFM**: Footnotes, front matter extraction (`---`/`+++`), heading IDs (GitHub-compatible slugs), math spans (`$`/`$$`), and callouts (`> [!NOTE]`, `> [!WARNING]`, ...).

12 feature flags to turn on exactly what you need:

```
allow_html · allow_link_refs · tables · strikethrough · task_lists
autolink_literals · disallowed_raw_html · footnotes · front_matter
heading_ids · math · callouts
```

## Trade-offs

ferromark is built for one job: turning Markdown into HTML as fast as possible. That focus means some things it deliberately skips:

- **No AST access.** You can't walk a syntax tree or write custom renderers against parsed nodes. If you need that, pulldown-cmark's iterator model or comrak's AST are better fits.
- **No source maps.** No byte-offset tracking for mapping HTML back to Markdown positions.
- **HTML only.** No XML, no CommonMark round-tripping, no alternative output formats.

These aren't planned. They'd compromise the streaming architecture that makes ferromark fast.

## How it works

No AST. Block events stream from the scanner to the HTML writer with nothing in between.

```
Input bytes (&[u8])
       │
       ▼
   Block parser (line-oriented, memchr-driven)
       │ emits BlockEvent stream
       ▼
   Inline parser (mark collection → resolution → emit)
       │ emits InlineEvent stream
       ▼
   HTML writer (direct buffer writes)
       │
       ▼
   Output (Vec<u8>)
```

What makes this fast in practice:

- **Block scanning** runs on `memchr` for line boundaries. Container state is a compact stack, not a tree.
- **Inline parsing** has three phases: collect delimiter marks, resolve precedence (code spans, math, links, emphasis, strikethrough), emit. No backtracking.
- **Emphasis resolution** uses the CommonMark modulo-3 rule with a delimiter stack instead of expensive rescans.
- **SIMD scanning** (NEON on ARM) detects special characters in inline content.
- **Zero-copy references**: events carry `Range` pointers into the input, not copied strings.
- **Compact events**: 24 bytes each, cache-line friendly.
- **Hot/cold annotation**: `#[inline]` on tight loops, `#[cold]` on error paths, table-driven byte classification.

### Design principles

- **Linear time.** No regex, no backtracking, no quadratic blowup on adversarial input.
- **Low allocation pressure.** Compact events, range references, reusable output buffers.
- **Operational safety.** Depth and size limits guard against pathological nesting.
- **Small dependency surface.** Minimal crates, straightforward integration.

<details>
<summary><strong>Detailed parser comparison</strong></summary>

<br>

How ferromark compares to the other three top-tier parsers across architecture, features, and output. Ratings use a 4-level heatmap focused on end-to-end Markdown-to-HTML throughput. Scoring is relative per row, so each row has at least one top mark.

Legend: 🟩 strongest &nbsp; 🟨 close behind &nbsp; 🟧 notable tradeoffs &nbsp; 🟥 weakest

Ferromark optimization backlog: [docs/arch/ARCH-PLAN-001-performance-opportunities.md](docs/arch/ARCH-PLAN-001-performance-opportunities.md)

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
    <tr><td colspan="5"><b>Performance-critical architecture and memory</b></td></tr>
    <tr>
      <td><b>Parser model (streaming, no AST)</b></td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
      <td align="center">🟥</td>
    </tr>
    <tr><td colspan="5"><small>Streaming parsers emit output as they scan, avoiding intermediate trees. ferromark and md4c stream directly; pulldown-cmark uses a pull iterator; comrak builds an AST.</small></td></tr>
    <tr>
      <td><b>API overhead profile</b></td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
      <td align="center">🟥</td>
    </tr>
    <tr><td colspan="5"><small>Measures overhead on straight Markdown-to-HTML throughput. md4c callbacks and ferromark streaming events are lean; pulldown-cmark pull iterators are close; comrak's AST model adds more overhead for this workload.</small></td></tr>
    <tr>
      <td><b>Parse/render separation</b></td>
      <td align="center">🟨</td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟧</td>
    </tr>
    <tr><td colspan="5"><small>Clear separation lets renderers be swapped or tuned. md4c and pulldown-cmark separate parse and render clearly; ferromark is mostly separated; comrak leans on AST-based renderers.</small></td></tr>
    <tr>
      <td><b>Inline parsing pipeline</b></td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
      <td align="center">🟨</td>
      <td align="center">🟥</td>
    </tr>
    <tr><td colspan="5"><small>Multi-phase inline parsing (collect, resolve, emit) keeps the hot path linear. ferromark uses this approach; md4c and pulldown-cmark are optimized byte scanners; comrak does more AST bookkeeping.</small></td></tr>
    <tr>
      <td><b>Emphasis matching efficiency</b></td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
      <td align="center">🟨</td>
      <td align="center">🟥</td>
    </tr>
    <tr><td colspan="5"><small>Stack-based algorithms reduce rescans on text-heavy documents. ferromark uses modulo-3 stacks; md4c and pulldown-cmark are optimized; comrak pays AST overhead.</small></td></tr>
    <tr>
      <td><b>Link reference processing cost</b></td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
    </tr>
    <tr><td colspan="5"><small>Link labels need normalization. ferromark, md4c, and pulldown-cmark minimize allocations; comrak handles more feature paths.</small></td></tr>
    <tr>
      <td><b>Zero-copy text handling</b></td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
      <td align="center">🟨</td>
      <td align="center">🟥</td>
    </tr>
    <tr><td colspan="5"><small>Text slices that point directly into input reduce allocation and copy costs. ferromark uses ranges; md4c and pulldown-cmark borrow slices; comrak allocates AST nodes.</small></td></tr>
    <tr>
      <td><b>Allocation pressure (hot path)</b></td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
      <td align="center">🟥</td>
    </tr>
    <tr><td colspan="5"><small>Fewer allocations in tight loops means better CPU utilization. Streaming parsers allocate less during parse/render; AST parsers allocate many nodes.</small></td></tr>
    <tr>
      <td><b>Output buffer reuse</b></td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
      <td align="center">🟥</td>
    </tr>
    <tr><td colspan="5"><small>Reusing buffers avoids repeated allocations across runs. ferromark, md4c, and pulldown-cmark allow reuse; comrak allocates internally.</small></td></tr>
    <tr>
      <td><b>Memory locality</b></td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
      <td align="center">🟥</td>
    </tr>
    <tr><td colspan="5"><small>A small working set fits in cache. Streaming parsers keep it small; AST-based parsing expands it.</small></td></tr>
    <tr>
      <td><b>Cache friendliness</b></td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
      <td align="center">🟥</td>
    </tr>
    <tr><td colspan="5"><small>Linear scans and contiguous buffers work well for CPU caches. ferromark and md4c favor linear scans; pulldown-cmark is close; comrak traverses AST allocations.</small></td></tr>
    <tr>
      <td><b>SIMD availability</b></td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
      <td align="center">🟩</td>
      <td align="center">🟥</td>
    </tr>
    <tr><td colspan="5"><small>SIMD accelerates scanning for special characters. ferromark and pulldown-cmark have SIMD paths; md4c relies on C compiler optimizations; comrak is not SIMD-focused.</small></td></tr>
    <tr>
      <td><b>Hot-path control</b></td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟧</td>
      <td align="center">🟥</td>
    </tr>
    <tr><td colspan="5"><small>Performance headroom from low-level control in inner loops. md4c (C) and ferromark use tighter tuning; pulldown-cmark is mostly safe-Rust hot loops; comrak prioritizes flexibility.</small></td></tr>
    <tr>
      <td><b>Dependency footprint</b></td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
      <td align="center">🟥</td>
    </tr>
    <tr><td colspan="5"><small>Fewer dependencies simplify builds. md4c and ferromark are minimal; pulldown-cmark is moderate; comrak is heavier.</small></td></tr>
    <tr>
      <td><b>Throughput ceiling (architectural)</b></td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
      <td align="center">🟥</td>
    </tr>
    <tr><td colspan="5"><small>Streaming architectures with fewer allocations generally allow higher throughput ceilings. ferromark and md4c lead; pulldown-cmark is close; comrak trades throughput for flexibility.</small></td></tr>
    <tr><td colspan="5">&nbsp;</td></tr>
    <tr><td colspan="5"><b>Feature coverage and extensibility</b></td></tr>
    <tr>
      <td><b>Extension breadth</b></td>
      <td align="center">🟩</td>
      <td align="center">🟧</td>
      <td align="center">🟨</td>
      <td align="center">🟩</td>
    </tr>
    <tr><td colspan="5"><small>comrak has the broadest catalog; ferromark implements all 5 GFM extensions plus footnotes, front matter, heading IDs, math, and callouts; pulldown-cmark supports common GFM features; md4c supports common GFM features.</small></td></tr>
    <tr>
      <td><b>Spec compliance (CommonMark)</b></td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
      <td align="center">🟩</td>
    </tr>
    <tr><td colspan="5"><small>All four target CommonMark. Beyond CommonMark and GFM, ferromark, pulldown-cmark, and comrak also support footnotes, heading IDs, math spans, and callouts.</small></td></tr>
    <tr>
      <td><b>Extension configuration surface</b></td>
      <td align="center">🟨</td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
      <td align="center">🟨</td>
    </tr>
    <tr><td colspan="5"><small>Fine-grained flags let you disable features to reduce work. md4c has many flags; ferromark has 12 options; pulldown-cmark and comrak use option structs.</small></td></tr>
    <tr>
      <td><b>Raw HTML control</b></td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟧</td>
      <td align="center">🟩</td>
    </tr>
    <tr><td colspan="5"><small>md4c and comrak expose explicit switches; ferromark provides <code>allow_html</code> and <code>disallowed_raw_html</code>; pulldown-cmark is more fixed.</small></td></tr>
    <tr>
      <td><b>GFM tables</b></td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
    </tr>
    <tr><td colspan="5"><small>All four support GFM tables.</small></td></tr>
    <tr>
      <td><b>Task lists, strikethrough</b></td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
      <td align="center">🟨</td>
      <td align="center">🟩</td>
    </tr>
    <tr><td colspan="5"><small>All four support both.</small></td></tr>
    <tr>
      <td><b>Footnotes</b></td>
      <td align="center">🟩</td>
      <td align="center">🟥</td>
      <td align="center">🟨</td>
      <td align="center">🟩</td>
    </tr>
    <tr><td colspan="5"><small>ferromark, pulldown-cmark, and comrak support footnotes; md4c does not.</small></td></tr>
    <tr>
      <td><b>Permissive autolinks</b></td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟧</td>
      <td align="center">🟨</td>
    </tr>
    <tr><td colspan="5"><small>ferromark and md4c support GFM autolink literals (URL, www, email); comrak has relaxed autolinks; pulldown-cmark focuses on spec defaults.</small></td></tr>
    <tr>
      <td><b>Output safety toggles</b></td>
      <td align="center">🟨</td>
      <td align="center">🟩</td>
      <td align="center">🟧</td>
      <td align="center">🟩</td>
    </tr>
    <tr><td colspan="5"><small>md4c and comrak provide explicit unsafe/escape switches; ferromark provides <code>allow_html</code> and <code>disallowed_raw_html</code>; pulldown-cmark is more fixed.</small></td></tr>
    <tr><td colspan="5">&nbsp;</td></tr>
    <tr><td colspan="5"><b>Rendering and output</b></td></tr>
    <tr>
      <td><b>Output streaming</b></td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
      <td align="center">🟥</td>
    </tr>
    <tr><td colspan="5"><small>Incremental output lowers peak memory and removes extra passes. ferromark and md4c stream to buffers; pulldown-cmark streams events; comrak renders after AST work.</small></td></tr>
    <tr>
      <td><b>Output customization hooks</b></td>
      <td align="center">🟧</td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
      <td align="center">🟩</td>
    </tr>
    <tr><td colspan="5"><small>Callbacks and ASTs are great for custom rendering but add indirection. md4c callbacks and comrak AST are very flexible; pulldown-cmark iterators are easy to transform; ferromark is lower level.</small></td></tr>
    <tr>
      <td><b>Output formats</b></td>
      <td align="center">🟥</td>
      <td align="center">🟧</td>
      <td align="center">🟨</td>
      <td align="center">🟩</td>
    </tr>
    <tr><td colspan="5"><small>comrak emits HTML, XML, and CommonMark; pulldown-cmark provides HTML plus event streams; md4c has HTML and callbacks; ferromark targets HTML only.</small></td></tr>
    <tr>
      <td><b>Source position support</b></td>
      <td align="center">🟥</td>
      <td align="center">🟥</td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
    </tr>
    <tr><td colspan="5"><small>pulldown-cmark has strong source map support; comrak can emit source positions; ferromark and md4c skip this for speed.</small></td></tr>
    <tr>
      <td><b>Source map tooling</b></td>
      <td align="center">🟥</td>
      <td align="center">🟥</td>
      <td align="center">🟩</td>
      <td align="center">🟨</td>
    </tr>
    <tr><td colspan="5"><small>pulldown-cmark exposes event ranges; comrak can emit source position attributes; ferromark and md4c keep this minimal.</small></td></tr>
    <tr>
      <td><b>IO friendliness</b></td>
      <td align="center">🟩</td>
      <td align="center">🟩</td>
      <td align="center">🟧</td>
      <td align="center">🟥</td>
    </tr>
    <tr><td colspan="5"><small>md4c and ferromark stream into buffers; pulldown-cmark recommends buffered output; comrak often builds strings after AST work.</small></td></tr>
  </tbody>
</table>

</details>

## Building

```bash
cargo build            # development
cargo build --release  # optimized (recommended for benchmarks)
cargo test             # run tests
cargo test --test commonmark_spec -- --nocapture  # CommonMark spec
cargo bench            # benchmarks
```

## Project structure

```
src/
├── lib.rs          # Public API (to_html, to_html_into, parse, Options)
├── main.rs         # CLI binary
├── block/          # Block-level parser
│   ├── parser.rs   # Line-oriented block parsing
│   └── event.rs    # BlockEvent types
├── inline/         # Inline-level parser
│   ├── mod.rs      # Three-phase inline parsing
│   ├── marks.rs    # Mark collection + SIMD integration
│   ├── simd.rs     # NEON SIMD character scanning
│   ├── event.rs    # InlineEvent types
│   ├── code_span.rs
│   ├── emphasis.rs      # Modulo-3 stack optimization
│   ├── strikethrough.rs # GFM strikethrough resolution
│   ├── math.rs          # Math span resolution ($/$$ delimiters)
│   └── links.rs         # Link/image/autolink parsing
├── footnote.rs     # Footnote store and rendering
├── link_ref.rs     # Link reference definitions
├── cursor.rs       # Pointer-based byte cursor
├── range.rs        # Compact u32 range type
├── render.rs       # HTML writer
├── escape.rs       # HTML escaping (memchr-optimized)
└── limits.rs       # DoS prevention constants
```

## License

MIT -- Copyright 2026 Sebastian Software GmbH, Mainz, Germany
