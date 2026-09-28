# Opt-in inline insertions

- Status: Accepted
- Date: 2026-09-27
- Related: [Issue #465](https://github.com/sebastian-software/ferromark/issues/465)

## Context

Authors can express deleted text with `~~...~~`, but there is no compact native
notation for inserted text. Writing `<ins>...</ins>` interrupts Markdown prose.
The new notation must be opt-in so existing Markdown profiles and default HTML
remain unchanged.

## Decision

Add inline `++...++` as an explicit insertion syntax. Rust exposes
`ParserOptions::insertions`; Node exposes the equivalent `insertions` option on
all rendering and transformation entry points. The flag defaults off in Rust,
CommonMark/GFM/MDX presets, and Node. When enabled, a matched span becomes a
native `Insertion` AST node and renders as semantic `<ins>...</ins>` through the
normal renderer, including its untrusted mode and hook path. This is parser
syntax and does not require the transforms package.

Delimiter flanking follows CommonMark emphasis' whitespace and punctuation
conditions, as does [markdown-it-ins](https://github.com/markdown-it/markdown-it-ins).
Each plus run of two or more signs is split into independent `++` delimiter
tokens; an odd run leaves one literal plus before its pairs. The rule of three
for `*`/`_` emphasis does not apply. A single plus, unmatched delimiters, and
invalid pairs stay literal. Pair tokens from the same source run cannot match
one another, so `a++++b` remains literal rather than creating an empty
`<ins>`. For an odd closing run, the remaining plus follows all closing tags:
`+++++a+++++` renders as `+<ins><ins>a</ins></ins>+`. Insertions accept
ordinary nested inline Markdown
and may include soft line breaks, but cannot cross paragraph or block
boundaries. Existing escapes and protected code, math, HTML, URL, and MDX
contexts keep their existing parsing rules.

With GFM bare-URL autolinks enabled, plus pairs within the URL candidate stay
part of the link. A closing pair exactly at a candidate's end can still close
an insertion that opened before it. Fixtures cover both a `++` pair in the
middle of a URL and a URL inside an insertion.

Reserve `++...++` for inserted text. Its meaning is independent of content, so
`++ctrl+c++` is an insertion even though other extensions use plus signs for
keyboard markup.

`Insertion` owns inline children and a delimiter-inclusive source span. The AST
visitor and walker expose it directly. Heading text, image alt text, autolinks,
span remapping, inline-note lowering, links, and render hooks traverse its
children. Heading IDs and transform metadata include inserted words with the
delimiters removed.

## Consequences

With the option off, `+` stays on the normal text scan path and output remains
unchanged. Enabling insertions adds `+` to the fused inline marker classifier
and uses the existing delimiter stack and nesting bound. Benchmark the enabled,
disabled, unmatched-plus, and repeated-run cases separately; do not infer a
speedup from the optional default path.

A local Criterion run on Rust 1.95.0 (`aarch64-apple-darwin`, 10 samples per
case, one-second warm-up and measurement) observed these parser times:

| Input shape | Bytes | Insertions off | Insertions on |
| --- | ---: | ---: | ---: |
| 256 `C++ is still plain text.` repetitions | 6,400 | 669 ns | 6.10 µs |
| 16,384 unmatched pluses and `tail` | 16,388 | 1.59 µs | 62.9 µs |
| 512 `++inserted text++` repetitions | 9,216 | 0.926 µs | 31.1 µs |

These are local sample measurements for comparing input shapes, not a before
and after speedup claim or a cross-machine performance guarantee.

A paired base-versus-PR comparison checked the default-disabled path on
`0bd64f49` and `91a02048` with Rust 1.95.0, fat LTO, and
`-C target-cpu=generic` on macOS ARM64. The 120 cases comprised 57 broad
inputs, the same 57 with bare-URL autolinks, and six authored insertion-shaped
diagnostics. Three rounds each used three paired 40 ms windows. HTML and AST
output matched exactly for every case. Ratios below are baseline time divided
by candidate time, so values below one mean the candidate was slower:

| Case group | Cases | Fresh | Parse-only |
| --- | ---: | ---: | ---: |
| All default-off inputs | 120 | 0.9979 | 0.9879 |
| Broad inputs | 57 | 0.9956 | 0.9864 |
| Bare-URL autolink inputs | 57 | 0.9993 | 0.9882 |
| Insertion-shaped diagnostics | 6 | 1.0064 | 1.0000 |
| Same-commit A/A control | 120 | 1.0010 | 0.9979 |

The default-off change measured 0.2% slower in fresh rendering and 1.2% slower
in parse-only mode. The A/A fresh rounds ranged from 0.999 to 1.001, so fresh
rendering stayed within that A/A spread. The A/A parse-only rounds ranged from
0.996 to 0.999, so the 1.2% parse-only difference was larger than this run's
measured variation. These local paired samples describe this host and corpus;
they are not a cross-machine guarantee.

The public Rust API keeps `Node` and `ParserOptions` exhaustive. Adding
`Node::Insertion` and `ParserOptions::insertions` therefore requires a major
release under the existing [API policy](2026-09-17-api-surface.md#exhaustiveness).

The [syntax guide](../optional-writing.md) documents defaults, edge cases,
semantic output, and API usage.
