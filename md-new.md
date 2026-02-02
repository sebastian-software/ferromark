# Ultra-High-Performance Markdown/GFM Parser — Architecture Specification
*(Performance-first, implementation-oriented blueprint for Rust)*

> Goal: a **Markdown → HTML compiler** that is **structurally capable** of beating existing Rust parsers by **20–30%** on real-world corpora, by choosing an architecture that is fundamentally cache-friendly, allocation-minimal, and SIMD-ready.  
> Trade-off: **performance > full GFM fidelity**. This spec explicitly lists what is supported, what is omitted, and why.

---

## Table of Contents

0. [Lessons from Reference Implementations](#0-lessons-from-reference-implementations)
1. [Non-Goals and Ground Rules](#1-non-goals-and-ground-rules)
2. [System Overview](#2-system-overview)
3. [Scope: What We Implement vs. What We Refuse](#3-scope-what-we-implement-vs-what-we-refuse)
4. [Core Data Model: Cursors, Ranges, Events](#4-core-data-model-cursors-ranges-events)
5. [Block Parser](#5-block-parser)
6. [Inline Parser](#6-inline-parser)
7. [HTML Renderer](#7-html-renderer)
8. [Escaping, URLs, and Safety Boundaries](#8-escaping-urls-and-safety-boundaries)
9. [SIMD and Hot-Path Engineering](#9-simd-and-hot-path-engineering)
10. [Complexity Guarantees and DoS Resistance](#10-complexity-guarantees-and-dos-resistance)
11. [Public API Design](#11-public-api-design)
12. [Benchmarking and Performance Engineering](#12-benchmarking-and-performance-engineering)
13. [Testing Strategy](#13-testing-strategy)
14. [Build/Release Profile and Tooling](#14-buildrelease-profile-and-tooling)
15. [Implementation Plan](#15-implementation-plan)
16. [Appendix A: Detailed Feature Semantics](#appendix-a-detailed-feature-semantics)
17. [Appendix B: Suggested Internal Modules](#appendix-b-suggested-internal-modules)
18. [Appendix C: Micro-Optimizations Checklist](#appendix-c-micro-optimizations-checklist)
19. [Appendix D: Apple Silicon Optimization Guide](#appendix-d-apple-silicon-optimization-guide)

---

## 0. Lessons from Reference Implementations

This section synthesizes insights from analyzing two high-performance Markdown parsers:
- **md4c** (C): Push-based callback parser, full CommonMark compliance
- **pulldown-cmark** (Rust): Pull-based iterator parser, zero-copy design

### 0.1 Architecture Comparison

| Aspect | md4c | pulldown-cmark | Our Approach |
|--------|------|----------------|--------------|
| Parsing Model | Push (callbacks) | Pull (iterator) | **Push** (lower overhead) |
| Memory Model | Reusable mark arrays | CowStr + arena | **Ranges + reusable buffers** |
| Inline Strategy | 3-phase mark collection | Stack-based resolution | **2-phase scan-then-resolve** |
| SIMD Usage | Loop unrolling (4x) | memchr only | **NEON intrinsics + memchr** |
| Allocation | Growth factor 1.5x | CowStr inline optimization | **Pre-sized + pooling** |

### 0.2 Key Techniques from md4c

**Three-Phase Inline Processing** (critical for performance):
1. **Mark Collection**: Single pass collecting delimiter positions into compact array
2. **Mark Analysis**: Process by precedence (entities → code → HTML → links → emphasis)
3. **Rendering**: Walk resolved marks and emit output

**DoS Prevention via Limits**:
- `CODESPAN_MARK_MAXLEN = 32`: Prevents quadratic code span resolution
- `TABLE_MAXCOLCOUNT = 128`: Prevents output explosion
- Early HTML recognition during mark collection

**Loop Unrolling** (measured 4x improvement):
```c
while(off + 3 < end && !IS_MARK(off+0) && !IS_MARK(off+1)
                    && !IS_MARK(off+2) && !IS_MARK(off+3))
    off += 4;
```

**Mark Character Map** (256-entry lookup table):
- O(1) character classification
- Eliminates complex conditional chains

**Emphasis Modulo-3 Optimization**:
- 6 separate stacks (asterisk/underscore × 3 modulo classes)
- Reduces complexity of finding matching openers

### 0.3 Key Techniques from pulldown-cmark

**Zero-Copy String Handling**:
```rust
enum CowStr<'a> {
    Borrowed(&'a str),      // Most common: zero allocation
    Boxed(Box<str>),        // When modification needed
    Inlined(InlineStr),     // 22 bytes inline for short strings
}
```

**Tree Structure with Vec-Based Arena**:
- Nodes stored contiguously in `Vec`
- `NonZeroUsize` indices save memory
- Better cache locality than pointer-based trees

**Link Reference Expansion Limit**:
- Tracks expansion count to prevent recursive DoS
- Limit: `min(text_len, 100KB)`

**Static Size Assertions** (enforce compile-time constraints):
```rust
const _: [(); 2] = [(); mem::size_of::<TagEnd>()]; // Must be 2 bytes
```

### 0.4 Synthesis: Our Optimal Approach

Based on analysis, we adopt:

1. **Push model with reusable event buffers** (md4c-style, lower overhead than iterators)
2. **Range-based text representation** (no CowStr complexity, just `(start, end)`)
3. **Mark collection phase** before inline parsing (md4c's biggest win)
4. **256-byte lookup tables** for character classification
5. **Loop unrolling in scanning** (4x unroll, proven effective)
6. **Separate delimiter stacks by type and modulo-3** (md4c emphasis optimization)
7. **Pre-allocated buffers with 1.5x growth** (both parsers use this)
8. **Static size assertions** (pulldown-cmark's compile-time guarantees)

### 0.5 What We Improve Upon

| Limitation in Existing Parsers | Our Improvement |
|-------------------------------|-----------------|
| md4c: No SIMD, only loop unrolling | ARM NEON intrinsics for scanning |
| pulldown-cmark: Iterator overhead | Direct buffer writes |
| Both: Generic x86/ARM code | Apple Silicon-specific optimizations |
| md4c: C memory safety concerns | Rust with minimal unsafe |
| pulldown-cmark: CowStr allocation overhead | Pure range-based model |

---

## 1. Non-Goals and Ground Rules

### 1.1 Non-Goals
These are **explicitly out of scope** because they either:
- require heavy lookahead/backtracking,
- push us toward building an AST,
- add non-linear edge cases,
- or drag performance down in hot paths.

**Out of scope (hard “no”)**
- Full CommonMark compliance (strict conformance behavior in all edge cases)
- Full GitHub Flavored Markdown fidelity (including GitHub’s exact HTML heuristics)
- Tables (pipe tables)
- Footnotes
- Math / LaTeX blocks
- Heading ID generation (GitHub-style slugger)
- Reference-style links with global definitions (optional; see scope section)
- HTML block parsing heuristics that attempt GitHub parity
- Plugin architecture / user-defined syntax extensions
- Source maps / positional mapping for rendered HTML
- Sanitization of HTML (belongs to a separate security layer)
- Markdown “round-tripping” (HTML → Markdown or stable formatting)

### 1.2 Ground Rules (Performance Contract)
- **No AST** (no tree of nodes). We use streaming events.
- **No regex** in core parsing.
- **No backtracking** in hot paths.
- **No recursion** in parsing.
- **O(n)** time on all inputs, with fixed upper bounds on nested constructs.
- Parsing is byte-oriented (`&[u8]`), using ASCII semantics for syntax.
- All intermediate text is represented as **ranges** into the input buffer.
- Output is written to a single `Vec<u8>` (or `String`), with aggressive reservation and optional reuse.

---

## 2. System Overview

### 2.1 Pipeline
We compile Markdown to HTML through a deterministic streaming pipeline:

```
input: &[u8]
   │
   ▼
Block parser  (line-oriented, indentation-aware)
   │  produces: BlockEvent stream + inline ranges
   ▼
Inline parser (range-oriented, token scanning)
   │  produces: InlineEvent stream
   ▼
HTML renderer (single writer, escape-aware)
   │
   ▼
output: Vec<u8>
```

### 2.2 Why This Wins
Most parsers lose time due to:
- AST creation and traversal (allocations, pointer chasing, cache misses),
- too much Unicode/`char` iteration,
- regex and backtracking,
- generic rendering layers (formatting APIs, virtual DOM-like components).

This design avoids those costs:
- one-pass-ish streaming,
- byte-level scanning,
- fixed state machines,
- pre-sized output buffers,
- optimized escaping.

---

## 3. Scope: What We Implement vs. What We Refuse

### 3.1 Supported Features (Tier 0: always-on, hot)
#### Block-level
- Paragraphs
- ATX headings (`#`..`######`)
- Thematic breaks (`---`, `***`, `___` with CommonMark-ish rules simplified)
- Fenced code blocks (``` and ~~~) with optional info string
- Indented code blocks (optional; can be disabled if it harms perf)
- Blockquotes (`>`)
- Lists:
  - unordered (`-`, `*`, `+`)
  - ordered (`1.` etc., simplified rules)
  - nested lists with capped depth

#### Inline-level
- Text
- Emphasis (`*em*` / `_em_`) with simplified delimiter rules
- Strong (`**strong**` / `__strong__`)
- Inline code spans (`` `code` `` with exact backtick matching)
- Links `[text](url "title")` (titles optional)
- Autolinks:
  - `<https://...>` style
  - bare URLs (optional Tier 1; expensive if done fully)
- Strikethrough `~~strike~~` (GFM-lite)

### 3.2 Supported Features (Tier 1: optional, warm)
- Task list items `- [ ]` / `- [x]` (only at list item start)
- Hard line breaks (two spaces + newline) (optional)
- Reference-style links (global definitions) **only if** implemented with a low-cost map (see Appendix)

### 3.3 Refused Features (Tier 2: never)
- Tables
- Footnotes
- Math
- GitHub-perfected HTML block parsing
- Automatic heading IDs
- Full delimiter-run emphasis parsing (CommonMark exactness)
- Arbitrary HTML sanitization

### 3.4 “Compatibility” Philosophy
We aim for:
- “Looks right” for common Markdown,
- stable output across versions,
- no catastrophic edge-case slowdowns.

We do **not** aim for:
- matching GitHub output byte-for-byte,
- supporting every obscure corner case.

---

## 4. Core Data Model: Cursors, Ranges, Events

### 4.1 Input Representation
All parsing uses:
- `&[u8]` (bytes)
- positions are `usize` offsets into that slice

### 4.2 Cursor
Use a pointer-based cursor for hot scanning (safe wrapper around raw pointers):

```rust
#[derive(Clone, Copy)]
struct Cursor {
    ptr: *const u8,
    end: *const u8,
    base: *const u8, // for offset computations
}

impl Cursor {
    #[inline] fn offset(&self) -> usize { unsafe { self.ptr.offset_from(self.base) as usize } }
    #[inline] fn remaining(&self) -> usize { unsafe { self.end.offset_from(self.ptr) as usize } }
    #[inline] fn peek(&self) -> u8 { unsafe { *self.ptr } }
    #[inline] fn advance(&mut self, n: usize) { unsafe { self.ptr = self.ptr.add(n) } }
}
```

Notes:
- Keep `#[inline]` on tiny accessors.
- Avoid bounds checks in inner loops (guard at loop entry).

### 4.3 Ranges
Text is represented as:

```rust
#[derive(Clone, Copy)]
struct Range { start: usize, end: usize }
```

No `String` allocation. Only ranges.

### 4.4 Events

#### 4.4.1 Block Events
Block parser emits high-level structure. Inline text is emitted as ranges:

```rust
enum BlockEvent<'a> {
    ParagraphStart,
    ParagraphEnd,

    HeadingStart { level: u8 },
    HeadingEnd,

    CodeFenceStart { info: Range },
    CodeFenceEnd,

    BlockQuoteStart,
    BlockQuoteEnd,

    ListStart { kind: ListKind, indent: u8 },
    ListEnd,
    ListItemStart { task: TaskState },
    ListItemEnd,

    ThematicBreak,

    InlineRange(Range),
}
```

Where `TaskState` is `None | Unchecked | Checked`.

#### 4.4.2 Inline Events
Inline parser emits events that renderer turns into HTML:

```rust
enum InlineEvent {
    Text(Range),
    Code(Range),

    EmphStart, EmphEnd,
    StrongStart, StrongEnd,
    StrikeStart, StrikeEnd,

    LinkStart { url: Range, title: Option<Range> },
    LinkEnd,

    SoftBreak,
    HardBreak,
}
```

### 4.5 No Trait Objects / Dynamic Dispatch
Events should be plain enums, processed via `match`. Avoid virtual calls.

---

## 5. Block Parser

### 5.1 Responsibilities
- Identify block boundaries and nesting.
- Emit block events.
- Provide inline content ranges to inline parser.

Block parser is line-oriented and handles:
- indentation,
- list prefixes,
- blockquote markers,
- fenced code fences.

### 5.2 Line Scanning
We scan for newline with `memchr(b'
', ...)` (or an internal fast scanner).  
Avoid iterating byte-by-byte unless absolutely needed.

**Key primitive: `next_line()`**
- Returns `(line_range, line_end_including_newline)` or similar.
- Recognize `
` and normalize to `
` behavior without copying.

### 5.3 Indentation Model
- Count leading spaces; treat tabs optionally as 4 spaces (or reject for simplicity).
- Keep indentation as `u8` (cap at 255).
- Do not support “complex” CommonMark indentation rules; implement a stable subset.

### 5.4 Block Types (Detailed)

#### 5.4.1 Blank Lines
- Detect blank line (only spaces/tabs then newline).
- Ends paragraphs.

#### 5.4.2 Headings (ATX)
- Match leading `#` run up to 6.
- Require a space or end-of-line after marker (simplified).
- Emit `HeadingStart{level}` → inline range → `HeadingEnd`.

#### 5.4.3 Thematic Break
- Recognize lines made of `-` or `*` or `_` repeated with optional spaces.
- Simplify: require at least 3 markers and nothing else but spaces.

#### 5.4.4 Fenced Code Blocks
- Detect fence start at line beginning (after indentation): ``` or ~~~ (length >= 3).
- Record fence char and length.
- Capture info string range (trim spaces).
- Consume subsequent lines until a matching fence line found (same char, len >= opening len).
- Emit `CodeFenceStart{info}` then `InlineRange` of raw code content?  
  **No**: code block content is not inline-parsed; renderer escapes as text and wraps in `<pre><code>`.
- Emit `CodeFenceEnd`.

Performance notes:
- Code blocks often large; scanning must be linear and avoid per-byte overhead.
- Use `memchr` to find newline; fence check only at line starts.

#### 5.4.5 Indented Code Blocks (Optional)
- Trigger when line has indent ≥ 4 and not inside list item contexts requiring different rules.
- Because indentation interaction is costly, consider disabling by default to avoid complexity.

#### 5.4.6 Blockquotes
- Recognize `>` optionally preceded by up to 3 spaces.
- Consume one optional space after `>`.
- Maintain a stack-like state: blockquote open if consecutive quote lines.
- Emit `BlockQuoteStart/End`.

#### 5.4.7 Lists
- Unordered markers: `-`, `*`, `+` followed by space.
- Ordered marker: digits + `.` + space (digits count capped, e.g. <= 9 chars).
- List nesting: track indentation and container stack.

**Capping for DoS resistance**
- Max list nesting depth (e.g. 32).
- Max digits in ordered marker (e.g. 9) to avoid big-integer parsing.

Emit:
- `ListStart` at container enter,
- `ListItemStart` at each item,
- inline ranges for item content lines,
- handle continuation lines with indent rules (simplified).

#### 5.4.8 Task List Items (Optional Tier 1)
At list item start, after marker and a single space:
- Detect `[ ]` or `[x]` / `[X]` then space.
- Set `task` on `ListItemStart`.

Do not support nested task marker patterns; only at start.

### 5.5 Block Parser State
State is a small struct:
```rust
struct BlockState {
    container_stack: SmallVec<[Container; 8]>,
    in_code_fence: Option<FenceState>,
    in_paragraph: bool,
}
```

Prefer `SmallVec` to avoid heap allocation for typical docs.

### 5.6 Output from Block Parser
Block parser should not allocate; it pushes `BlockEvent`s into a preallocated `Vec<BlockEvent>`, or yields them via an iterator-like interface.

**Preferred:** push into a caller-provided `Vec<BlockEvent>` to reuse allocations.

---

## 6. Inline Parser

### 6.1 Responsibilities
Given a text range, emit inline events:
- text spans,
- emphasis/strong,
- inline code,
- strike,
- links,
- line breaks.

Inline parsing is where most CPU is spent on typical docs.

### 6.2 General Strategy: Scan-to-Special
Inline parser should:
1. Emit long runs of plain text via `memchr` / multi-char scan to find next special marker.
2. On encountering a marker, attempt a deterministic parse.
3. If parse fails, treat marker as text and continue.

### 6.3 Special Character Set
Common specials:
- `*`, `_` for emphasis/strong
- `` ` `` for code span
- `[` `]` `(` `)` for links
- `~` for strike
- `
` for breaks
- `& < > " '` for escaping (renderer concern)

Use a byte lookup table `is_special[256]` for fast checks.

### 6.4 Emphasis/Strong (Simplified)
CommonMark’s delimiter rules are expensive. We implement a pragmatic subset:

- `**` opens/closes strong.
- `*` opens/closes emphasis.
- Same for `_` and `__`.
- No complex left/right-flanking rules; instead:
  - require non-space inside delimiters (simple heuristic),
  - limit nesting depth (e.g. 32),
  - do not treat underscores inside words as emphasis (optional heuristic).

Maintain a small delimiter stack with entries:
```rust
struct Delim { kind: DelimKind, pos: usize }
```

### 6.5 Inline Code Spans
- On encountering backticks, count run length `n`.
- Search for matching run of `n` backticks.
- If found: emit `Code(range)`.
- If not found: treat opening backticks as text.

Optimization:
- Search for matching backtick run using `memchr(b'`', ...)` then verify run length.
- Avoid scanning byte-by-byte.

### 6.6 Strikethrough (GFM-lite)
- Detect `~~`.
- Similar to emphasis, but only for `~~`.
- No nested complexity beyond a capped stack.

### 6.7 Links (Inline Only)
Support:
- `[text](url "title")`
- `[text](url)`  
Simplify:
- no nested brackets beyond a fixed max depth (e.g. 8),
- URL parsing stops at `)` with escape support for `\)` optionally.

Algorithm:
1. On `[`, push position to bracket stack.
2. On `](` pattern, parse destination:
   - skip spaces,
   - parse URL until space or `)` (or quoted title begins),
   - optional title in quotes (single/double) until matching quote,
   - require closing `)`.

Emit:
- `LinkStart{url,title}` then parse link text range recursively?  
  **No recursion**. Instead, inline-parse link text as a nested inline parse with capped depth; implement an explicit stack of “contexts”.

### 6.8 Autolinks
Two variants:

#### 6.8.1 Angle autolinks `<https://...>`
Cheap:
- detect `<` then scan until `>` without whitespace.
- if prefix is `http://` or `https://` or `mailto:`, treat as link.

#### 6.8.2 Bare URLs (Optional; expensive)
Bare URL recognition can dominate CPU. If implemented:
- only detect `http://` and `https://` tokens at word boundaries,
- stop at whitespace or common trailing punctuation.
- keep it strictly heuristic.

If performance is the top goal, **default off**.

### 6.9 Line Breaks
- Soft break: newline becomes `SoftBreak` (renderer turns into `
` or space depending on mode)
- Hard break: two trailing spaces before newline → `HardBreak` (optional)

### 6.10 Inline Parser State & Limits
- nesting depth cap (e.g. 32)
- bracket depth cap (e.g. 8)
- delimiter stack cap (e.g. 64)
All caps prevent pathological inputs from exploding time.

---

## 7. HTML Renderer

### 7.1 Responsibilities
Consume events and write HTML into a single buffer.

### 7.2 Output Buffer
Use `Vec<u8>` for maximum control. Provide helper:

```rust
struct HtmlWriter {
    out: Vec<u8>,
}
```

Reserve heuristic:
- `out.reserve(input_len + input_len / 4)` for typical docs.

If rendering lots of small docs:
- implement buffer pooling:
  - `thread_local!` with `RefCell<Vec<u8>>`
  - or a user-provided buffer reuse API.

### 7.3 Rendering Rules (Block)
- Paragraph: `<p>` + inline + `</p>
`
- Heading: `<hN>` + inline + `</hN>
`
- Blockquote: `<blockquote>
` ... `</blockquote>
`
- Lists:
  - `<ul>
` or `<ol>
`
  - list items: `<li>` + content + `</li>
`
  - task items: inject `<input type="checkbox" disabled ...>` (optional; can be expensive)
- Code fence:
  - `<pre><code class="language-...">` optional language class
  - escape content
  - `</code></pre>
`
- Thematic break: `<hr />
`

### 7.4 Rendering Rules (Inline)
- Text: escaped
- Emph: `<em>` / `</em>`
- Strong: `<strong>` / `</strong>`
- Strike: `<del>` / `</del>`
- Code: `<code>` + escape + `</code>`
- Link: `<a href="...">` + text + `</a>`

### 7.5 Escaping
Renderer owns escaping. Provide two routines:

- `escape_text_into(out, bytes)`
- `escape_attr_into(out, bytes)` (stricter; also escapes quotes)

Fast path:
- scan for first escapable char; if none, bulk copy.
- use `memchr` or SIMD to find `&` `<` `>` `"` `'`.

Slow path:
- write segments between escapes using `extend_from_slice`.

### 7.6 Attribute Handling (URLs)
- Do not normalize URLs (expensive).
- Do not percent-encode (expensive).
- Minimal escaping for HTML attribute safety.

Optionally:
- enforce a allowlist of URL schemes (`http`, `https`, `mailto`) for security contexts.  
This is a policy layer and can be enabled/disabled.

---

## 8. Escaping, URLs, and Safety Boundaries

### 8.1 Security Model (Explicit)
This engine is **not a sanitizer**. It compiles Markdown into HTML.

- If raw HTML passthrough is enabled, the consumer must sanitize.
- If disabled, `<` in input becomes `&lt;` in output.

### 8.2 Minimal HTML Passthrough (Optional)
Full HTML block parsing is expensive. If passthrough exists:
- only allow inline HTML tags with a conservative grammar:
  - `<tag ...>` and `</tag>` detection without full validation.
- never attempt GitHub’s block HTML rules.

Default should be: **escape all HTML**.

---

## 9. SIMD and Hot-Path Engineering

### 9.1 Where SIMD Helps
SIMD is valuable for:
- scanning for newline,
- scanning for special inline markers,
- scanning for escapable HTML chars.

### 9.2 Where SIMD Does Not Help
- deeply branched parsing logic,
- nested container tracking,
- bracket matching.

### 9.3 Strategy
- Start with `memchr` / `memchr2/3`.
- Measure.
- If needed: add optional `std::simd` (nightly features vary) or a hand-rolled byte-scanner behind a cfg.

### 9.4 Microarchitectural Goals
- Minimize unpredictable branches.
- Prefer “scan then handle”.
- Keep critical structs small (fit in cache lines).
- Keep hot data contiguous.

---

## 10. Complexity Guarantees and DoS Resistance

### 10.1 Linear-Time Parsing
Every stage must be O(n).
- no backtracking
- no regex
- no unbounded nested parsing

### 10.2 Caps / Limits
Enforce:
- max list nesting: 32
- max inline nesting: 32
- max bracket depth: 8
- max delimiter stack: 64
- max line length to consider for certain features (optional)

### 10.3 Fallback Behavior
If caps are exceeded:
- stop interpreting further markers
- treat remaining content as text

This avoids time blowups while still producing output.

---

## 11. Public API Design

### 11.1 Minimal API
```rust
pub fn to_html(input: &str) -> String;
```

### 11.2 High-Performance API
```rust
pub fn to_html_with_buffer(input: &str, out: &mut Vec<u8>);
```

### 11.3 Streaming API (Optional)
Expose an iterator of events for advanced renderers:

```rust
pub fn parse_block_events(input: &[u8], out: &mut Vec<BlockEvent>);
pub fn parse_inline_events(input: &[u8], range: Range, out: &mut Vec<InlineEvent>);
```

No trait objects; no dynamic dispatch.

### 11.4 Feature Flags
Provide feature flags (Cargo):
- `task_list`
- `hard_breaks`
- `autolink_bare` (off by default)
- `allow_html` (off by default)
- `indented_code` (off by default)

Also runtime options struct, but compile-time flags should control code size and branch count.

---

## 12. Benchmarking and Performance Engineering

### 12.1 Benchmark Corpus
Use:
- Open-source README corpus
- docs from real projects
- synthetic worst-cases:
  - deep nesting
  - long delimiter runs
  - huge code blocks
  - repeated bracket patterns

### 12.2 Metrics
- Throughput (MB/s)
- ns/byte
- allocations/op (should be near zero in parsing)
- branch misses, cache misses (`perf stat`)
- flamegraphs (`perf record`, `cargo flamegraph`)

### 12.3 Competitor Baselines
Benchmark against:
- `pulldown-cmark`
- `comrak`
- `cmark-gfm`
- `markdown-rs`

### 12.4 Build Profiles
For benchmark builds:
- `lto = "fat"`
- `codegen-units = 1`
- `panic = "abort"`
- `opt-level = 3`

Consider:
- PGO with representative corpus.

### 12.5 Regression Gates
CI should reject:
- throughput regressions > 2–3%
- allocation regressions
- extreme-case slowdowns

---

## 13. Testing Strategy

### 13.1 Correctness vs Performance Tests
Separate suites:
- **Correctness**: targeted examples; stable output snapshots.
- **Performance**: benchmarks; not run on every PR unless stable environment exists.

### 13.2 Fuzzing
Fuzz block parser and inline parser:
- ensure no panics,
- ensure termination under caps.

### 13.3 Golden Tests
For supported features:
- maintain minimal “golden” HTML outputs.
- avoid importing entire CommonMark spec suite unless you can accept failures.

---

## 14. Build/Release Profile and Tooling

### 14.1 Crate Layout
- `no_std` is possible but likely not necessary.
- Avoid heavy deps. Favor `memchr`, `smallvec`.

### 14.2 Safety and `unsafe`
`unsafe` is allowed in hot scanning paths if:
- encapsulated,
- verified by fuzzing,
- minimal surface area.

### 14.3 Observability
Provide optional `trace` feature:
- emits counters (lines scanned, events produced)
- does not allocate strings in hot path (use numeric counters)

---

## 15. Implementation Plan

### Phase 0: Skeleton
- `Range`, `Cursor`, `HtmlWriter`
- basic escaping functions

### Phase 1: Block Parser MVP
- paragraphs, headings, fences
- lists (flat)
- blockquotes

### Phase 2: Inline Parser MVP
- text, emphasis/strong (simplified)
- code spans
- links

### Phase 3: Integrate Renderer
- block + inline event consumption
- stable output formatting

### Phase 4: Add Tier-1 Features
- strike
- task list items
- hard breaks

### Phase 5: Performance Pass
- profile hot loops
- reserve tuning
- specialized scanners
- optional SIMD
- PGO

---

## Appendix A: Detailed Feature Semantics

### A.1 Paragraphs
- consecutive non-blank lines form a paragraph unless interrupted by block construct.
- paragraph output always wrapped in `<p>...</p>`.

### A.2 Headings
- `#` run defines level 1–6.
- strip leading/trailing spaces from heading content.

### A.3 Lists
- simplified list continuation rules:
  - continuation lines must be indented >= marker indent + 2
  - nested lists require greater indent than parent
- no blank-line “loose/tight” distinctions (optional)

### A.4 Links
- allow spaces around URL and title minimally.
- titles supported in single or double quotes.
- no reference link shortcuts unless Tier 1 is enabled.

### A.5 Code Blocks
- fenced code blocks preserve inner newlines.
- renderer escapes content and does not inline-parse.

---

## Appendix B: Suggested Internal Modules

- `cursor.rs` — Cursor and scanning primitives
- `range.rs` — Range type, slicing helpers
- `scan.rs` — `memchr` wrappers, special-byte scanners
- `block.rs` — block parser + container stack
- `inline.rs` — inline parser + delimiter/bracket stacks
- `escape.rs` — text/attr escaping
- `render.rs` — HtmlWriter + rendering logic
- `bench/` — criterion benches and corpus harness
- `fuzz/` — fuzz targets

---

## Appendix C: Micro-Optimizations Checklist

- Avoid `String` in parsing.
- Avoid `format!` in rendering (write bytes).
- Use `SmallVec` for stacks.
- Reserve output buffer early.
- Avoid `Vec::push` in tight loops without reservation.
- Use lookup tables for classification (`is_space`, `is_special`).
- Keep event enums small (use `u8`/`u16` where possible).
- Reduce branch depth: scan → handle.
- Profile before SIMD.
- Use `#[cold]` for slow-path error handling.

---

**End of document.**
