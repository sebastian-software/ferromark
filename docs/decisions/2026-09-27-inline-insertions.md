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
tokens; an odd run leaves one literal plus outside its pairs. On a close-only
odd run the literal plus follows the closing tags, so source spans do not
overlap: `++a+++` renders as `<ins>a</ins>+`. The rule of three for `*`/`_`
emphasis does not apply. A single plus, unmatched delimiters, and invalid pairs
stay literal. Pair tokens from the same source run cannot match one another,
so `a++++b` remains literal rather than creating an empty `<ins>`. For an odd
run on both sides, the remaining plus follows all closing tags:
`+++++a+++++` renders as `+<ins><ins>a</ins></ins>+`. Insertions accept
ordinary nested inline Markdown
and may include soft line breaks, but cannot cross paragraph or block
boundaries. Existing escapes and protected code, math, HTML, URL, and MDX
contexts keep their existing parsing rules.

With GFM bare-URL autolinks enabled, scheme URL candidates keep `++` inside the
link text. Raw-source protection stops before inline syntax that the autolink
post-pass cannot see across. A closing pair exactly at a scheme URL candidate's
end can still close an insertion that opened before it, but it cannot open a
new insertion. Fuzzy email and `www.` links are created after inline parsing,
so their plus delimiters remain active insertion syntax. Fixtures cover a
`++` pair in the middle of a scheme URL, a scheme URL inside an insertion,
markup adjacent to autolinks, and email local parts containing plus signs.

Reserve `++...++` for inserted text. Its meaning is independent of content, so
`++ctrl+c++` is an insertion even though other extensions use plus signs for
keyboard markup.

`Insertion` owns inline children and a delimiter-inclusive source span. The AST
visitor and walker expose it directly. Heading text, image alt text, autolinks,
span remapping, inline-note lowering, links, and render hooks traverse its
children. Heading IDs and transform metadata include inserted words with the
delimiters removed.

## Consequences

With the option off, `+` keeps its ordinary text meaning and output remains
unchanged. The disabled implementation path is not byte-for-byte identical to
`main`: it has option plumbing, an extra opener-bottom table, and insertion
pairing branches. Enabling insertions adds `+` to the fused inline marker
classifier and uses the existing delimiter stack and nesting bound. Benchmark
the enabled, disabled, unmatched-plus, and repeated-run cases separately; do
not infer a speedup from the optional default path.

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

A follow-up base-versus-PR run on 2026-09-28 included the review fixes. It used
the same 120 cases, Rust 1.95.0, fat LTO, and `-C target-cpu=generic`; HTML and
AST output stayed identical in every case. Geometric means were:

| Case group | Cases | Fresh | Parse-only |
| --- | ---: | ---: | ---: |
| All default-off inputs | 120 | 0.9959 | 0.9868 |
| Broad inputs | 57 | 0.9916 | 0.9851 |
| Bare-URL autolink inputs | 57 | 0.9992 | 0.9868 |
| Insertion-shaped diagnostics | 6 | 1.0056 | 1.0037 |

Ratios are baseline time divided by candidate time. The new comparison measured
0.4% slower fresh rendering and 1.3% slower parse-only. A same-binary A/A run
had geometric means of 1.0019 fresh and 0.9996 parse-only; across its rounds,
fresh ratios ranged from 0.9952 to 1.0039 and parse-only ratios from 0.9932 to
1.0218. The parse-only control itself therefore varied from 0.7% slower to
2.1% faster. Host load rose during that control, from a 7.6 one-minute average
to 20.6, so I treat the measured 1.3% parse-only difference as a small,
inconclusive cost rather than a stable regression. These figures are local to
this corpus and host.

The public Rust API keeps `Node` and `ParserOptions` exhaustive. Adding
`Node::Insertion` and `ParserOptions::insertions` therefore requires a major
release under the existing [API policy](2026-09-17-api-surface.md#exhaustiveness).

The [syntax guide](../optional-writing.md) documents defaults, edge cases,
semantic output, and API usage.
