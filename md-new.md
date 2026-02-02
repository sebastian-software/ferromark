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

### 0.4 Synthesis: Starting Hypotheses (Not Dogma)

The following are **starting points for experimentation**, not fixed decisions. Each technique
must prove itself through benchmarks against our specific workloads:

| Hypothesis | Source | Status | Validate By |
|------------|--------|--------|-------------|
| Push model < iterator overhead | md4c | **To verify** | Benchmark both |
| Range-based < CowStr | Analysis | **To verify** | Memory profiling |
| Mark collection phase | md4c | **To verify** | Inline benchmarks |
| 256-byte lookup tables | Both | **Likely good** | Branch analysis |
| 4x loop unrolling | md4c | **To verify** | Perf counters |
| Modulo-3 emphasis stacks | md4c | **To verify** | Pathological tests |
| 1.5x buffer growth | Both | **Likely good** | Allocation tracking |

**Key principle**: If benchmarks show a simpler approach performs equally well, prefer simplicity.
We are not here to implement clever techniques for their own sake.

### 0.4.1 What We Will Definitely Use

Some patterns are proven and low-risk:
- `memchr` for byte scanning (already SIMD-optimized, battle-tested)
- Pre-sized output buffers (obvious win)
- Static size assertions (zero runtime cost)
- DoS limits (non-negotiable for safety)

### 0.4.2 What Needs Benchmarking

These require A/B comparison before committing:
- Push vs pull parsing model
- Three-phase vs two-phase inline parsing
- NEON intrinsics vs `memchr` alone
- Modulo-3 stacks vs simple linear scan

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

### 4.3 Ranges (Cache-Optimized)

Text is represented as ranges into the input buffer. Use `u32` for documents up to 4GB:

```rust
/// Compact range representation (8 bytes vs 16 for usize pair)
/// Fits 8 ranges per 64-byte L1 cache line
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
struct Range {
    start: u32,
    end: u32,
}

impl Range {
    #[inline]
    const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    #[inline]
    fn slice<'a>(&self, input: &'a [u8]) -> &'a [u8] {
        &input[self.start as usize..self.end as usize]
    }

    #[inline]
    const fn len(&self) -> u32 {
        self.end - self.start
    }

    #[inline]
    const fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

// Compile-time size check (pulldown-cmark pattern)
const _: () = assert!(std::mem::size_of::<Range>() == 8);
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

### 4.6 Cache-Aligned Hot Structures

For structures accessed in tight loops, ensure cache-friendly layout:

```rust
/// Mark entry for inline parsing (12 bytes)
/// Fits 5 marks per 64-byte L1 cache line
#[derive(Clone, Copy)]
#[repr(C)]
struct Mark {
    pos: u32,           // 4 bytes: start offset
    end: u32,           // 4 bytes: end offset
    prev: i16,          // 2 bytes: link to paired mark or opener chain (-1 = none)
    ch: u8,             // 1 byte: delimiter character
    flags: u8,          // 1 byte: OPENER | CLOSER | RESOLVED
}

const _: () = assert!(std::mem::size_of::<Mark>() == 12);

/// Parser hot state (fits in 64-byte L1 cache line)
#[repr(C, align(64))]
struct ParserHotState {
    pos: u32,           // Current position
    line_start: u32,    // Start of current line
    indent: u8,         // Current line indentation
    flags: u8,          // Parser state flags
    block_depth: u8,    // Current container nesting
    inline_depth: u8,   // Current inline nesting
    _pad: [u8; 24],     // Padding to 64 bytes
    // Frequently accessed together ↑
}

const _: () = assert!(std::mem::size_of::<ParserHotState>() == 64);
```

**Alignment guidelines** (see [Appendix D](#appendix-d-apple-silicon-optimization-guide)):
- L1 cache line: 64 bytes (all Apple Silicon)
- L2 cache line: 128 bytes (M1-M4)
- Align hot structs to 64 bytes to avoid false sharing
- Keep related fields adjacent for spatial locality

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

### 6.2 Three-Phase Inline Processing (Critical Optimization)

Inspired by md4c's architecture, inline parsing uses three phases to avoid backtracking:

**Phase 1: Mark Collection** (single pass)
- Scan text once, collecting all potential delimiter positions into a mark array
- Use loop unrolling (4x) with lookup table for speed
- Store: `(position, character, run_length, flags)`

```rust
#[derive(Clone, Copy)]
struct Mark {
    pos: u32,       // Offset in text
    end: u32,       // End of mark (pos + run_length)
    ch: u8,         // The delimiter character
    flags: u8,      // POTENTIAL_OPENER | POTENTIAL_CLOSER | RESOLVED
}

// Pre-allocated, reusable across blocks
struct MarkBuffer {
    marks: Vec<Mark>,   // Reuse capacity
}
```

**Phase 2: Mark Resolution** (by precedence)
Process marks in order of precedence to avoid ambiguity:
1. **Entities** (`&amp;`, `&#123;`) — resolve first, never interact with others
2. **Code spans** (`` ` ``) — highest delimiter precedence
3. **Raw HTML** (`<tag>`) — recognized early to avoid false positives
4. **Links** (`[text](url)`) — bracket matching
5. **Emphasis/Strong** (`*`, `_`) — lowest precedence, uses modulo-3 stacks

```rust
fn resolve_marks(marks: &mut [Mark], text: &[u8]) {
    resolve_code_spans(marks, text);    // Skip resolved regions in later passes
    resolve_html_spans(marks, text);
    resolve_links(marks, text);
    resolve_emphasis(marks, text);      // Uses 6 stacks (2 chars × 3 mod classes)
}
```

**Phase 3: Event Emission** (linear walk)
- Walk marks array, emit events between resolved pairs
- Text between marks becomes `Text(range)` events

This approach is **O(n)** with low constant factors because:
- Single collection pass (Phase 1)
- Each resolution pass skips already-resolved regions
- No backtracking or re-scanning

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

### 6.4 Emphasis/Strong (Modulo-3 Stack Optimization)

CommonMark's "rule of three" for emphasis matching is expensive if implemented naively.
Optimization from md4c: use **6 separate stacks** (2 characters × 3 modulo classes):

```rust
struct EmphasisResolver {
    // Stacks indexed by: (char == '_') * 3 + (run_length % 3)
    stacks: [Vec<usize>; 6],  // Each holds mark indices
}

impl EmphasisResolver {
    fn stack_index(ch: u8, run_len: usize) -> usize {
        let char_offset = if ch == b'_' { 3 } else { 0 };
        char_offset + (run_len % 3)
    }
}
```

**Why this works**: CommonMark requires that opener + closer lengths sum to a multiple of 3
(or both are multiples of 3). By separating stacks by `run_length % 3`, we only search
the correct stack for potential matches.

**Simplified rules** (pragmatic subset):
- `**` opens/closes strong, `*` opens/closes emphasis
- Same for `__` and `_`
- Require non-space adjacent to delimiter (simple flanking heuristic)
- Underscore inside words does not trigger emphasis (configurable)
- Limit nesting depth (32 levels)

```rust
struct Delim {
    mark_idx: u32,      // Index into marks array
    run_length: u8,     // Original delimiter run length
    can_open: bool,
    can_close: bool,
}
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

### 7.2 Output Buffer (Optimized for Reuse)

Use `Vec<u8>` for maximum control. md4c's growth strategy (1.5x + 128-byte alignment):

```rust
/// HTML output writer with optimized buffer management
struct HtmlWriter {
    out: Vec<u8>,
}

impl HtmlWriter {
    /// Create with pre-allocated capacity based on input size
    fn with_capacity_for(input_len: usize) -> Self {
        // Typical HTML is 1.25x input size; reserve extra for safety
        let capacity = input_len + input_len / 4;
        Self { out: Vec::with_capacity(capacity) }
    }

    /// Grow buffer using md4c's strategy: 1.5x + 128-byte alignment
    #[cold]
    fn grow(&mut self, needed: usize) {
        let new_cap = ((self.out.len() + needed) * 3 / 2 + 128) & !127;
        self.out.reserve(new_cap - self.out.capacity());
    }

    /// Write bytes without formatting overhead
    #[inline]
    fn write_bytes(&mut self, bytes: &[u8]) {
        self.out.extend_from_slice(bytes);
    }

    /// Write static string (compile-time known)
    #[inline]
    fn write_str(&mut self, s: &'static str) {
        self.out.extend_from_slice(s.as_bytes());
    }

    /// Clear for reuse (keeps capacity)
    fn clear(&mut self) {
        self.out.clear();
    }

    /// Take ownership of output
    fn into_vec(self) -> Vec<u8> {
        self.out
    }
}
```

**Buffer pooling for high-throughput scenarios**:

```rust
use std::cell::RefCell;

thread_local! {
    static HTML_BUFFER_POOL: RefCell<Vec<u8>> = RefCell::new(Vec::with_capacity(64 * 1024));
}

/// Render with pooled buffer (for many small documents)
pub fn to_html_pooled(input: &str) -> String {
    HTML_BUFFER_POOL.with(|pool| {
        let mut buf = pool.borrow_mut();
        buf.clear();

        // ... render into buf ...

        // Return owned String, buffer stays in pool
        String::from_utf8(buf.clone()).unwrap_or_default()
    })
}
```

**Alternative: User-provided buffer API** (zero allocation for caller):

```rust
pub fn to_html_into(input: &str, out: &mut Vec<u8>) {
    out.clear();
    out.reserve(input.len() + input.len() / 4);
    // ... render into out ...
}
```

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
- scanning for special inline markers (`*`, `_`, `` ` ``, `[`, `~`),
- scanning for escapable HTML chars (`&`, `<`, `>`, `"`, `'`),
- bulk copying non-special byte ranges to output.

**Measured speedups** (ARM NEON on Apple Silicon):
- Byte search: up to 9x faster
- Multi-char search: 4-7x faster
- Bulk escape scanning: 3-5x faster

### 9.2 Where SIMD Does Not Help
- deeply branched parsing logic,
- nested container tracking,
- bracket matching,
- emphasis delimiter resolution (stack-based),
- data with frequent type conversions (avoid!).

**Warning**: `std::simd` can be *slower* than scalar for unsuitable workloads (measured up to 7.7x slower for interleaved data patterns).

### 9.3 Strategy

**Phase 1: memchr baseline**
```rust
use memchr::{memchr, memchr2, memchr3};

// Fast path: find next special char
let specials = memchr3(b'*', b'_', b'`', &input[pos..]);
```

The `memchr` crate already uses SIMD on x86_64, wasm32, and aarch64.

**Phase 2: Custom multi-char scanner (if profiling shows need)**
```rust
// Mark character lookup table (256 entries)
const MARK_CHARS: [bool; 256] = make_mark_table();

#[inline]
fn scan_to_special(input: &[u8], start: usize) -> usize {
    let mut i = start;
    // 4x loop unrolling (proven 2-4x faster in md4c)
    while i + 4 <= input.len() {
        if MARK_CHARS[input[i] as usize] { return i; }
        if MARK_CHARS[input[i + 1] as usize] { return i + 1; }
        if MARK_CHARS[input[i + 2] as usize] { return i + 2; }
        if MARK_CHARS[input[i + 3] as usize] { return i + 3; }
        i += 4;
    }
    while i < input.len() {
        if MARK_CHARS[input[i] as usize] { return i; }
        i += 1;
    }
    input.len()
}
```

**Phase 3: Platform-specific NEON (Apple Silicon)**

See [Appendix D](#appendix-d-apple-silicon-optimization-guide) for detailed NEON intrinsics.

### 9.4 Scanning Primitives

Implement these as the core scanning API:

| Primitive | Description | Implementation |
|-----------|-------------|----------------|
| `scan_to_newline(buf, pos)` | Find next `\n` | `memchr` |
| `scan_to_special(buf, pos)` | Find next inline marker | lookup table + unroll |
| `scan_to_fence_end(buf, pos, fence_char, fence_len)` | Find code fence close | `memchr` + verify |
| `scan_to_escapable(buf, pos)` | Find `&<>"'` for escaping | `memchr` or SIMD |

### 9.5 Microarchitectural Goals
- Minimize unpredictable branches.
- Prefer "scan then handle" pattern.
- Keep critical structs small (≤64 bytes for L1, ≤128 bytes for L2).
- Keep hot data contiguous.
- Use `#[inline]` on tiny scanning functions.
- Use `#[cold]` on error/rare paths.

### 9.6 SIMD Crate Recommendations (Rust 1.93+)

| Crate | Use Case | Notes |
|-------|----------|-------|
| `memchr` | Byte/multi-byte search | Production-ready, NEON support |
| `std::arch::aarch64` | NEON intrinsics | Stable in Rust 1.93+ |
| `pulp` / `macerator` | Generic SIMD abstractions | Good for cross-platform |
| `std::simd` | Portable SIMD | Still nightly, variable performance |

**Recommendation**: Use `memchr` + direct NEON intrinsics for hot paths. Avoid `std::simd` until it stabilizes and proves competitive.

---

## 10. Complexity Guarantees and DoS Resistance

### 10.1 Linear-Time Parsing
Every stage must be O(n).
- no backtracking
- no regex
- no unbounded nested parsing

### 10.2 Caps / Limits (Learned from md4c and pulldown-cmark)

```rust
/// Compile-time constants for DoS prevention
pub mod limits {
    /// Maximum nesting depth for block containers (lists, blockquotes)
    pub const MAX_BLOCK_NESTING: usize = 32;

    /// Maximum nesting depth for inline elements (emphasis, links)
    pub const MAX_INLINE_NESTING: usize = 32;

    /// Maximum bracket depth in link parsing [[[...]]]
    pub const MAX_BRACKET_DEPTH: usize = 8;

    /// Maximum delimiter stack size per type
    pub const MAX_DELIMITER_STACK: usize = 64;

    /// Maximum backtick run length for code spans (prevents O(n²) matching)
    /// md4c uses 32; longer runs are treated as literal text
    pub const MAX_CODE_SPAN_BACKTICKS: usize = 32;

    /// Maximum parentheses nesting in link destinations (CommonMark spec: 32)
    pub const MAX_LINK_PAREN_DEPTH: usize = 32;

    /// Maximum digits in ordered list marker (prevents big-integer parsing)
    pub const MAX_LIST_MARKER_DIGITS: usize = 9;

    /// Link reference expansion limit (prevents recursive expansion DoS)
    /// pulldown-cmark: min(text_len, 100KB) total expansions
    pub const MAX_LINK_REF_EXPANSIONS: usize = 100 * 1024;

    /// Maximum table columns (if tables are implemented)
    /// md4c uses 128 to prevent output explosion
    pub const MAX_TABLE_COLUMNS: usize = 128;

    /// Maximum math brace nesting (if math is implemented)
    pub const MAX_MATH_BRACE_DEPTH: usize = 25;
}
```

### 10.3 Quadratic Complexity Prevention

**Problem patterns** (from md4c analysis):

| Pattern | Naive Complexity | Solution |
|---------|-----------------|----------|
| Many different-length backtick openers | O(n²) | Limit `MAX_CODE_SPAN_BACKTICKS` |
| Deeply nested link brackets | O(n²) | Limit `MAX_BRACKET_DEPTH` |
| Recursive link reference expansion | O(n²) | Track expansion count |
| Many potential emphasis openers | O(n²) | Modulo-3 stacks + limits |
| Huge tables with many columns | O(n × cols) | Limit `MAX_TABLE_COLUMNS` |

**Implementation strategy**:

```rust
struct InlineState {
    bracket_depth: u8,
    emphasis_depth: u8,
    code_span_backtick_counts: [u8; MAX_CODE_SPAN_BACKTICKS + 1],
    link_ref_expansion_count: usize,
}

impl InlineState {
    fn can_open_bracket(&self) -> bool {
        self.bracket_depth < limits::MAX_BRACKET_DEPTH as u8
    }

    fn can_expand_link_ref(&self, expansion_size: usize) -> bool {
        self.link_ref_expansion_count + expansion_size <= limits::MAX_LINK_REF_EXPANSIONS
    }
}
```

### 10.4 Fallback Behavior
If caps are exceeded:
- stop interpreting further markers of that type
- treat remaining content as literal text
- continue parsing other element types normally

This avoids time blowups while still producing valid output.

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

### 14.1 Crate Layout and Dependencies

```toml
# Cargo.toml
[package]
name = "md-fast"
version = "0.1.0"
edition = "2024"       # Rust 1.93+ required
rust-version = "1.93"

[dependencies]
memchr = "2.7"         # SIMD-accelerated byte search
smallvec = "1.13"      # Stack-allocated small vectors

[dev-dependencies]
criterion = "0.5"      # Benchmarking
proptest = "1.4"       # Property-based testing

[features]
default = ["std"]
std = []
neon = []              # Enable NEON intrinsics (auto-detected on aarch64)
trace = []             # Performance counters (dev only)

# Conditional deps
[target.'cfg(target_arch = "aarch64")'.dependencies]
# NEON intrinsics available in std::arch
```

### 14.2 Build Profiles

```toml
# Cargo.toml continued

[profile.release]
lto = "fat"
codegen-units = 1
panic = "abort"
opt-level = 3
strip = true

[profile.release-debug]
inherits = "release"
debug = true
strip = false

[profile.bench]
inherits = "release"
debug = true           # For flamegraph symbols
```

### 14.3 Platform-Specific Configuration

```toml
# .cargo/config.toml

[target.aarch64-apple-darwin]
rustflags = [
    "-C", "target-cpu=apple-m1",    # Or apple-m2, apple-m4
    "-C", "target-feature=+neon",
    "-C", "link-arg=-Wl,-ld_classic",  # Faster linking on macOS 14+
]

[target.x86_64-unknown-linux-gnu]
rustflags = [
    "-C", "target-cpu=native",
    "-C", "target-feature=+sse4.2,+avx2",
]

# For development: faster incremental builds
[profile.dev]
opt-level = 1
[profile.dev.package."*"]
opt-level = 2
```

### 14.4 Profile-Guided Optimization (PGO)

For maximum performance on Apple Silicon:

```bash
# Step 1: Build instrumented binary
RUSTFLAGS="-Cprofile-generate=/tmp/pgo-data" \
    cargo build --release

# Step 2: Run with representative workload
./target/release/md-fast bench/corpus/*.md

# Step 3: Merge profile data
llvm-profdata merge -o /tmp/pgo-data/merged.profdata /tmp/pgo-data

# Step 4: Build optimized binary
RUSTFLAGS="-Cprofile-use=/tmp/pgo-data/merged.profdata" \
    cargo build --release
```

### 14.5 Safety and `unsafe`

`unsafe` is allowed in hot scanning paths if:
- encapsulated in a safe public API,
- verified by fuzzing (10M+ iterations),
- minimal surface area (< 50 lines per unsafe block),
- documented with `// SAFETY:` comments.

**Unsafe allowlist**:
- Cursor pointer arithmetic (bounds checked at block entry)
- NEON intrinsics (behind `#[target_feature]`)
- Unchecked UTF-8 conversion (input validated at entry)

### 14.6 Observability

Provide optional `trace` feature:

```rust
#[cfg(feature = "trace")]
pub struct ParseStats {
    pub lines_scanned: u64,
    pub blocks_emitted: u64,
    pub inline_marks_collected: u64,
    pub escapes_written: u64,
}

#[cfg(feature = "trace")]
thread_local! {
    pub static STATS: std::cell::Cell<ParseStats> = const { std::cell::Cell::new(ParseStats::ZERO) };
}
```

- Does not allocate strings in hot path
- Uses numeric counters only
- Zero overhead when feature disabled

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

```
src/
├── lib.rs              — Public API, feature flags
├── cursor.rs           — Cursor type, pointer-based scanning
├── range.rs            — Range type (u32-based), slicing helpers
├── tables.rs           — 256-entry lookup tables (is_special, is_space, etc.)
├── limits.rs           — DoS prevention constants
│
├── scan/
│   ├── mod.rs          — Scanner trait and common functions
│   ├── scalar.rs       — Loop-unrolled scalar scanner (portable)
│   └── neon.rs         — ARM NEON SIMD scanner (cfg(target_arch = "aarch64"))
│
├── block/
│   ├── mod.rs          — Block parser entry point
│   ├── container.rs    — Container stack (lists, blockquotes)
│   ├── fence.rs        — Fenced code block handling
│   └── list.rs         — List parsing logic
│
├── inline/
│   ├── mod.rs          — Inline parser entry point
│   ├── marks.rs        — Mark collection and buffer
│   ├── emphasis.rs     — Modulo-3 emphasis resolution
│   ├── code_span.rs    — Code span matching
│   └── links.rs        — Link/autolink parsing
│
├── escape.rs           — HTML text/attr escaping (SIMD-ready)
├── render.rs           — HtmlWriter + event consumption
│
├── bench/              — Criterion benchmarks
│   ├── corpus/         — Real-world markdown samples
│   └── pathological/   — DoS test cases
│
└── fuzz/               — Fuzz targets (cargo-fuzz)
    ├── block_fuzz.rs
    └── inline_fuzz.rs
```

**Module dependencies** (acyclic):
```
lib → render → inline → block → scan → cursor/range/tables
                ↓
              escape
```

---

## Appendix C: Micro-Optimizations Checklist

### Memory & Allocation
- [ ] Avoid `String` in parsing — use `Range` only
- [ ] Avoid `format!` in rendering — write bytes directly
- [ ] Use `SmallVec<[T; 8]>` for stacks (typical nesting < 8)
- [ ] Reserve output buffer early (`input_len * 1.25`)
- [ ] Reuse buffers across parses (mark buffer, event buffer)
- [ ] Use buffer pooling for high-throughput scenarios

### Data Structures
- [ ] Keep `Range` as `(u32, u32)` — 8 bytes, not 16
- [ ] Keep `Mark` ≤ 12 bytes (5 per L1 cache line)
- [ ] Keep event enums small (use `u8`/`u16` discriminants)
- [ ] Align hot structs to 64 bytes (L1 cache line)
- [ ] Use `#[repr(C)]` for predictable layout

### Scanning & Branching
- [ ] Use 256-entry lookup tables for character classification
- [ ] Apply 4x loop unrolling for scanning loops
- [ ] Reduce branch depth: scan → handle pattern
- [ ] Use `#[inline]` on tiny hot functions (< 10 lines)
- [ ] Use `#[cold]` on error/rare paths
- [ ] Use `#[inline(never)]` on large cold functions

### SIMD (after profiling)
- [ ] Start with `memchr` crate (already SIMD-optimized)
- [ ] Profile before adding custom SIMD
- [ ] Use NEON intrinsics for aarch64, not `std::simd`
- [ ] Benchmark SIMD vs scalar — SIMD can be slower for some patterns

### Compiler & Build
- [ ] Enable LTO (`lto = "fat"`) in release
- [ ] Use `codegen-units = 1` for better optimization
- [ ] Set `panic = "abort"` if unwinding not needed
- [ ] Use `-C target-cpu=native` or `apple-m1` for builds
- [ ] Consider PGO with representative corpus

### Static Verification
- [ ] Add compile-time size assertions for critical structs
- [ ] Use `debug_assert!` for invariants (zero cost in release)
- [ ] Run with `RUSTFLAGS="-C overflow-checks=on"` in CI

---

## Appendix D: Apple Silicon Optimization Guide

> **Target**: Apple M1/M2/M3/M4 processors (ARMv8.4+/ARMv9)
> **Rust Version**: 1.93.0+ (stable NEON intrinsics)

### D.1 Apple Silicon Memory Hierarchy

| Level | M1/M2/M3 | M4 | Latency | Line Size |
|-------|----------|-----|---------|-----------|
| L1D | 128 KB/core | 128 KB/core | 3 cycles | 64 bytes |
| L1I | 192 KB/core | 192 KB/core | — | — |
| L2 | 12-16 MB shared | 16-24 MB shared | ~15 cycles | 128 bytes |
| SLC | 8-16 MB | 16+ MB | ~30 cycles | 128 bytes |
| DRAM | Unified | Unified | ~100+ cycles | — |

**Key Insight**: L2/SLC uses 128-byte cache lines. Align hot data structures to 128 bytes to prevent false sharing and maximize prefetch efficiency.

### D.2 SIMD Strategy for ARM NEON

Apple Silicon supports:
- **NEON**: 128-bit SIMD, 4 instructions/cycle on P-cores (mandatory on all AArch64)
- **SVE**: Only on M4+ (ARMv9), variable-length vectors up to 2048 bits
- **AMX**: Undocumented matrix coprocessor (not usable from Rust)

**Recommendation**: Target NEON for M1-M4 compatibility; SVE as optional future path.

#### D.2.1 NEON Intrinsics for Parsing (Rust 1.93+)

```rust
#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

/// Find first occurrence of any byte in `needles` (up to 16 bytes)
/// Returns index or None if not found
#[inline]
#[target_feature(enable = "neon")]
unsafe fn find_special_neon(haystack: &[u8], needles: &[u8; 16]) -> Option<usize> {
    let needle_vec = vld1q_u8(needles.as_ptr());
    let mut i = 0;

    while i + 16 <= haystack.len() {
        let chunk = vld1q_u8(haystack.as_ptr().add(i));
        // Compare against all needles
        let mut mask = vdupq_n_u8(0);
        for j in 0..needles.len() {
            let cmp = vceqq_u8(chunk, vdupq_n_u8(needles[j]));
            mask = vorrq_u8(mask, cmp);
        }
        // Check if any match
        let reduced = vmaxvq_u8(mask);
        if reduced != 0 {
            // Find exact position
            let mask_bytes: [u8; 16] = std::mem::transmute(mask);
            for k in 0..16 {
                if mask_bytes[k] != 0 {
                    return Some(i + k);
                }
            }
        }
        i += 16;
    }
    // Scalar fallback for remainder
    for j in i..haystack.len() {
        if needles.contains(&haystack[j]) {
            return Some(j);
        }
    }
    None
}
```

#### D.2.2 Optimal NEON Intrinsics for Markdown Parsing

| Operation | NEON Intrinsic | Use Case |
|-----------|---------------|----------|
| Load 16 bytes | `vld1q_u8` | Chunk scanning |
| Compare equal | `vceqq_u8` | Find delimiters |
| OR combine | `vorrq_u8` | Multi-character search |
| Horizontal max | `vmaxvq_u8` | Quick "any match" check |
| Count leading zeros | `vclzq_u8` | Find first match position |
| Bitwise select | `vbslq_u8` | Conditional operations |

#### D.2.3 When NOT to Use SIMD

SIMD overhead outweighs benefits for:
- Short strings (< 32 bytes) — use scalar with loop unrolling
- Complex branching logic — SIMD cannot help
- Non-contiguous memory — gather operations are slow

**Benchmark first**: `std::simd` can be 7.7x *slower* than scalar for unsuitable workloads.

### D.3 Memory Access Patterns

#### D.3.1 Prefetch Hints

```rust
#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::_prefetch;

// Prefetch for read, high temporal locality, L1 cache
unsafe { _prefetch::<{ _PREFETCH_READ }, { _PREFETCH_LOCALITY3 }>(ptr) };
```

Use prefetching when:
- Processing large code blocks (> 4KB)
- Predictable sequential access patterns
- About to scan a known buffer region

#### D.3.2 Structure Alignment

```rust
#[repr(C, align(128))]  // Align to L2 cache line
struct HotParserState {
    cursor_pos: usize,
    cursor_end: usize,
    // ... keep under 128 bytes total
}

#[repr(C, align(64))]   // Align to L1 cache line
struct MarkEntry {
    start: u32,
    end: u32,
    kind: u8,
    flags: u8,
    _pad: [u8; 2],      // Explicit padding
}
// Size: 12 bytes, fits 5 per L1 line
```

### D.4 Branch Prediction Optimization

Apple Silicon has excellent branch predictors, but we can help:

```rust
#[inline]
fn is_special_char(b: u8) -> bool {
    // Use lookup table instead of match/if-chain
    const TABLE: [bool; 256] = {
        let mut t = [false; 256];
        t[b'*' as usize] = true;
        t[b'_' as usize] = true;
        t[b'`' as usize] = true;
        t[b'[' as usize] = true;
        t[b']' as usize] = true;
        t[b'~' as usize] = true;
        t[b'\n' as usize] = true;
        t[b'<' as usize] = true;
        t[b'&' as usize] = true;
        t
    };
    TABLE[b as usize]
}
```

### D.5 Compiler Flags for Apple Silicon

```toml
# .cargo/config.toml
[target.aarch64-apple-darwin]
rustflags = [
    "-C", "target-cpu=apple-m1",      # Or apple-m2, apple-m4
    "-C", "target-feature=+neon",
    "-C", "link-arg=-Wl,-ld_classic", # Faster linking on macOS 14+
]

[profile.release]
lto = "fat"
codegen-units = 1
panic = "abort"
opt-level = 3

[profile.release-with-pgo]
inherits = "release"
# Use with: RUSTFLAGS="-Cprofile-generate=/tmp/pgo" cargo build ...
# Then:     RUSTFLAGS="-Cprofile-use=/tmp/pgo" cargo build ...
```

### D.6 M4 Specific: ARMv9 and SVE

M4 introduces ARMv9 with Scalable Vector Extension (SVE). For future-proofing:

```rust
#[cfg(all(target_arch = "aarch64", target_feature = "sve"))]
fn scan_sve(data: &[u8]) {
    // SVE code path (M4+)
}

#[cfg(all(target_arch = "aarch64", not(target_feature = "sve")))]
fn scan_neon(data: &[u8]) {
    // NEON fallback (M1-M3)
}
```

**Note**: SVE in Rust is still experimental as of Rust 1.93. NEON remains the production target.

### D.7 Benchmark Results (Reference)

From published benchmarks on Apple Silicon:

| Operation | Scalar | NEON | Speedup |
|-----------|--------|------|---------|
| memchr (find byte) | 1x | 9.2x | ✓ Strong |
| Dot product (f32) | 1x | 4.7x | ✓ Strong |
| CSV parsing | 1x | 7-14x | ✓ Strong |
| RGB interleave | 1x | 0.3x | ✗ Avoid |

**Recommendation**: Use NEON for byte scanning, escaping, and delimiter finding. Avoid for complex data transformations.

---

**End of document.**
