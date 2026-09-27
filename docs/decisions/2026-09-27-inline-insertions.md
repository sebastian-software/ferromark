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
invalid pairs stay literal. Insertions accept ordinary nested inline Markdown
and may include soft line breaks, but cannot cross paragraph or block
boundaries. Existing escapes and protected code, math, HTML, URL, and MDX
contexts keep their existing parsing rules.

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

The [syntax guide](../optional-writing.md) documents defaults, edge cases,
semantic output, and API usage.
