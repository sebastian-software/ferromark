# Apple Silicon iteration round 2 — 2026-09-15

Twelve commits between `ae409578` and `e2a3df70` came out of a second round
of short, measured iterations on the single-crate v2 core (`fea50462`,
v2.0.0-rc.1). Five problem areas were implemented in parallel by separate
agents in isolated worktrees; every candidate was screened on 20 documents,
and the combined promoted set was confirmed on the 57 broad documents and the
45 authored diagnostics in all four stages. Two large candidates and three
small experiments were rejected; their patches and numbers are archived here.

Ratios are baseline time over candidate time, geometric means of per-document
medians; higher is faster.

## Results

Combined promoted set (`e2a3df70`) against `fea50462`, 3 rounds × 5 pairs ×
40 ms, measured while the host was still indexing after an OS update (see
[Method](#method) for the A/A control taken under the same conditions).

| Input set | N | Fresh | Reuse | Parse | Render |
| --- | ---: | ---: | ---: | ---: | ---: |
| Original broad documents | 57 | **1.050×** | **1.041×** | **1.012×** | **1.102×** |
| Authored diagnostics | 45 | 1.060× | 1.070× | 1.005× | 1.129× |

Round geometric means agree to within 0.016 on every stage. On the broad set
56 of 57 documents improve fresh, 57 of 57 reuse, 40 of 57 parse, and 56 of
57 render.

| Category (broad) | N | Fresh | Reuse | Parse | Render |
| --- | ---: | ---: | ---: | ---: | ---: |
| Comments | 12 | 1.107× | 1.046× | 1.006× | 1.160× |
| Technical docs | 22 | 1.041× | 1.048× | 1.005× | 1.125× |
| Reference | 4 | 1.016× | 1.016× | 1.010× | 1.065× |
| Readme | 2 | 1.027× | 1.029× | 0.999× | 1.100× |
| Encyclopedia | 12 | 1.025× | 1.032× | 1.028× | 1.032× |
| Plain prose | 4 | 1.020× | 1.010× | 0.999× | 1.032× |

By size (fresh / reuse / parse / render): up to 512 B 1.115 / 1.058 / 1.015 /
1.178; 513–2,048 B 1.042 / 1.047 / 1.015 / 1.106; 2,049–16,384 B 1.036 /
1.040 / 1.008 / 1.102; 16,385–65,536 B 1.024 / 1.026 / 1.010 / 1.053; larger
1.022 / 1.028 / 1.020 / 1.019. The gains are concentrated in per-render and
per-renderer fixed costs, so they scale inversely with document size.

Measured losses, all retained: `typescript-handbook-typescript-5-0` fresh
0.989×, `vite-docs-features` parse 0.980× (a document whose A/A control
moved 1.041× fresh in the same session), `typescript-handbook-compiler-options`
render 0.992×. Among the diagnostics the 7-byte `scan-short-link` parses at
0.971× and the 33–98-byte `autolink-*-64` cases at 0.980–0.987×: a few
nanoseconds of parser setup per document on inputs that take 0.1–0.2 µs.

Correctness: exact HTML and full AST `Debug` (including spans) agreed between
baseline and candidate for every case and stage before any timing; every
timing window's iteration checksum was verified; the branch passes
`cargo fmt`, 867 workspace tests, Clippy with warnings denied, and the
benchmark builds.

## Method

- Harness: [`benchmarks/optimization-rounds`](../../../benchmarks/optimization-rounds/README.md)
  with the new `--baseline-revision` option (`ae409578`), so the baseline is
  the branch tip rather than the frozen reference core. Generic AArch64, fat
  LTO, Rust 1.95. The exact scripts used to drive it are in [`harness/`](harness/).
- Profiles: `xctrace record --template 'Time Profiler'` attached to a
  symbolized worker (`debug = 1`, no strip); the XML export already resolves
  inlined frames, so no `atos` pass was needed. [`profiles/`](profiles/) holds
  the self/inclusive summaries for CommonMark parsing of the 113 KB chess
  article, GFM rendering of the 40 KB Vite features page, and the fresh
  pipeline on a 957-byte GFM comment.
- Implementation: five agents worked in parallel worktrees, each on one
  problem area, with a written prediction per variant, differential tests
  proving byte-identical output, and the full workspace gate before each
  commit. Agents did not run timings.
- Measurement stayed centralized and sequential. Screens taken while the five
  agents compiled (load average 25–40) produced single-case medians between
  0.3× and 3.0× and were discarded. All numbers here come from runs without
  concurrent compilation: an A/A control on 20 cases (render 1.001, fresh
  1.002, parse 0.998, reuse 1.004), then one 20-case screen per candidate
  (3 × 3 × 40 ms), then the confirmation.
- The confirmation ran while Spotlight, mail and photo indexers were active
  after an OS update. Because the harness alternates baseline and candidate
  inside the same 40 ms windows, steady background load cancels out of the
  ratio; to check, an A/A control on 10 cases ran under the same conditions
  first (fresh 1.007, reuse 0.999, parse 1.001, render 1.004; outliers
  `vite-docs-features` fresh 1.041, `wiki-chess-article-body` parse 0.977 /
  render 1.035) and the confirmation used five pairs per round instead of
  three. The runner now waits for the absence of our own compiler and profiler
  processes instead of a load-average threshold.
- Decision rule as in the first round: promote when the target stage's
  geometric mean is at least 1.010 with every round above 1.000, fresh and
  reuse at least 1.000, and no broad case below 0.970 without an explanation.

## Profiles at HEAD

CommonMark parse, `wiki-chess-article-body` (self time): the NEON scanners
hold ~30% (`vreinterpret_u64_u8` 13.3%, `trailing_zeros` 7.5%,
`next_special_neon_with_tables` 4.5%, `ByteClass::first_in_neon` 3.2%,
`line_end_neon` 1.5%); the inline loop and its marker memo ~11%
(`parse_inline` 4.1%, `InlineMarkerScan::next` 4.1%, `ForwardScan::hit` 2.9%);
link parsing ~7% (`parse_link` 4.0%, `parse_destination` 1.4%,
`scan_balanced` 1.1%); `memmove`/`copy_nonoverlapping` 3.8%; the emphasis
`retain` (`DrainFilter::drop`) 1.7%; `str::is_char_boundary` 2.5%.

GFM render, `vite-docs-features`: escaping and copying ~45%, of which the
`String` growth bookkeeping (`RawVecInner::capacity` 7.1%, `non_null` 2.3%,
`set_len` 2.1%, `NonNull::eq` 2.0%) is ~14%; `render_node` 8.1% self;
headings 19.6% inclusive (`slugify_heading_into` 4.6% self, `write_heading_id`
15.7% inclusive); `prepare_render` 5.5% inclusive (`scan_node_for_render`
2.3%, `is_toc_marker_paragraph` 1.5%); `write_source_span_attr` 1.7% self for
an out-of-line boolean test.

Fresh pipeline, `comment-review-long` (957 B): `malloc`/`free` ~15%
(the worker's per-iteration `HtmlRendererOptions::clone` 8.8% inclusive,
`RendererOptions` drop 5.1%, the renderer's three pre-sized scratch strings);
`Parser::with_options` 7.8% inclusive, almost all in `definition_candidates`
whose one-shot `memmem::find` built a searcher per call; `may_contain_autolink`
8.1%; `parse_list` 18.4% inclusive.

## Screens

20 cases, 3 rounds × 3 pairs × 40 ms, quiet machine. Render-side candidates
were screened in the render and fresh stages, parser-side candidates in
parse, reuse and fresh.

| Candidate | Render | Fresh | Parse | Reuse | Outcome |
| --- | ---: | ---: | ---: | ---: | --- |
| A1 raw output cursor, text escaper | 0.973 | 0.987 | | | rejected |
| A2 + URL escaper on the cursor | 0.940 | 0.974 | | | rejected (`scan-dense-escapes` 0.771, `scan-unicode-links` 0.867) |
| A3 escape runs cut from byte slices (no `is_char_boundary`) | 1.003 | 1.002 | | | tie: rounds 1.004 / 0.983 / 0.994; 14 cases 1.00–1.04, `wiki-chess-article-body` 0.925 |
| C1 no escape pass for generated slugs | 0.995 | 0.996 | | | neutral alone; kept as the base of C2/C3 |
| C1+C2 slugify single-text headings from the source | 1.011 | 1.001 | | | heading-bearing docs 1.02–1.04 |
| C1+C2+C3 ASCII slug bytes through a cursor | 1.012 | 1.007 | | | rust-book 1.059, vite 1.061, comment-incident 1.102 |
| Source-span attribute gate inline | 1.004 | 1.000 | | | comments and block-heavy docs 1.02–1.03 |
| D1 lazy heading scratch buffers | 0.995 | 1.033 | | | comments fresh 1.09–1.24 |
| D1+D2 autolink index once per renderer | 1.042 | 1.023 | | | comment renders 1.36, autolink profile 1.37 |
| D1–D3 skip setup scan when unread | 1.043 | 1.027 | | | neutral in these profiles (heading ids on) |
| D1–D4a 64-byte output floor | 1.044 | 1.029 | | | |
| D1–D4b minimal footnote reset | 1.050 | 1.026 | | | promoted |
| E1 record inline markers during line scan | | 0.967 | 0.949 | 0.962 | rejected (only `rust-book-ch17` gained, 1.054) |
| B1 definition pre-pass from `]:` | | 0.998 | 0.999 | 0.997 | neutral |
| B1+B2 short-slice SWAR scans | | 1.003 | 1.003 | 1.003 | encyclopedia parse 1.02 |
| B1+B2+B3 emphasis bookkeeping | | 1.003 | 1.008 | 1.002 | `table-formatted-4096` 1.114, `wiki-tea-article-body` 1.033 |
| Inline marker set derived once per parser (against `e2a3df70`, 5 pairs) | | 0.998 | 1.000 | 1.001 | tie |

Own quick experiments, screened during the agents' compilation and therefore
indicative only: arena floor 16 KB → 4 KB (fresh 1.010 with rounds 1.002 /
1.035 / 1.057, reuse 1.002), inline children capacity ceiling 12 → 32 (parse
1.003) and 12 → 64 (parse 1.009, fresh 0.990). None showed a consistent
signal; all three are archived in [`rejected/`](rejected/).

## Promoted commits

| Commit | Change |
| --- | --- |
| `09589339` | Heading scratch buffers start empty and reserve on first use; `to_html`/`to_html_into` build through `HtmlRenderer::new()` instead of materializing default option strings |
| `ee662c5e` | The autolink `FirstByteIndex` is built once per renderer, not on every render; incremental fragments stop rebuilding it |
| `3819e0cd` | The setup walk skips the `[[toc]]` predicate when no option reads it, and is skipped entirely when heading ids are off |
| `bcc8ab13` | Output reservation has a 64-byte floor so comment-sized documents stop growing the buffer two or three times |
| `33ef51bb` | Per-render footnote reset touches only the state the enabled options can write |
| `3ce3e432` | Generated heading slugs are written without the attribute-escape pass; explicit `{#id}` values keep it |
| `fd69a215` | Single-`Text` headings are slugified straight from the source text; the text scratch is filled only when a reader exists |
| `6fef693f` | The ASCII run of `slugify_heading_into` writes bytes through a cursor with one `set_len`; Unicode runs keep the safe path |
| `f1e6dcb4` | `write_source_span_attr` keeps its boolean gate inline and moves the formatting body out of line |
| `c8a118f2` | The definition pre-pass iterates `]:` occurrences with a static searcher and inspects only the bounded window of `[` before each |
| `6e2442fa` | Link labels, destinations, titles and image alts probe short slices with an eight-byte SWAR test before falling back to `memchr` |
| `e2a3df70` | Emphasis pairing skips the final `retain` when nothing was emptied and classifies ASCII flanking from bytes |

## Rejected variants

Patches are in [`rejected/`](rejected/), with their screen results under
[`results/`](results/).

- **Raw output cursor in the escapers (A1, A2).** Hoisting `reserve`/`set_len`
  out of the scan/copy loop into a write pointer kept in locals, with
  replacements packed into single words. Every document rendered 2–6% slower
  and entity- or Unicode-dense URLs up to 23% slower. The capacity bookkeeping
  the profile attributes ~14% of render self time to is where samples land,
  not removable work — the same lesson the first round drew for the
  vector-to-scalar move in the parser.
- **Recording inline markers during block-level line scanning (E1).** A fused
  NEON scan that finds the line end, the table pipe and every inline marker
  in one pass, replaying the positions in the paragraph's inline pass. Parse
  0.949×, worse on nearly every document. The follow-up profiles show the
  design worked as intended and still lost: on 47 KB of link-free prose the
  two vector scans fell from 17.5% + 20.1% inclusive to 2.5% + 4.9%, but the
  recording put more back — classification is per 16 bytes and vectorized,
  recording is per marker and scalar (`trailing_zeros` 5.0% → 10.0% self,
  the `SmallVec` push 2.1%, the drain chain 9.8% inclusive), and prose lands
  a marker every 20–40 bytes. On link-dense text the recording is also
  exhaustive where the demand-driven walk skips whole link targets, the
  wider `InlineMarkerScan` taxed every nested inline parse it never served
  (+24% self time in the classifier), and comment-sized documents paid the
  per-sub-parser setup. Capturing the double scan would need the inline walk
  to consume the classifier's output as it is produced — a parser
  restructure, not a port.
- **Table probe gated on the next line (E2).** Finding the line end with
  NEON `line_end` and searching for `|` only when the following line's
  first non-space byte is `|`, `-` or `:`. The profile on a fence-heavy
  documentation page shows the gate admitting constantly (real Markdown
  starts lines with `-` all the time) and then paying a second pass, so the
  probe cost rises from 3.65% to 4.96% inclusive; `memchr3` had answered
  "line end" and "pipe present" in one pass. Predicted parse ≈ 0.99×; not
  timed, archived.
- **Byte-slice cuts in the escapers (A3).** Slicing `&bytes[start..i]`
  instead of `&s[start..i]` drops the `is_char_boundary` checks. Render
  1.003× with rounds 1.004 / 0.983 / 0.994: fourteen documents gained
  0.5–4.4%, while the two documents whose render time moves most between
  builds of identical source lost 5–8%. A tie under the decision rule.
- **Inline marker set derived once per parser.** Caching the option-derived
  marker bits on the parser instead of recomputing them for every nested
  inline parse: parse 1.000×, reuse 1.001×, fresh 0.998× against `e2a3df70`
  with five pairs per round. A tie; archived.
- Arena floor and inline children capacity ceiling, see above.

## Findings

1. Fixed per-render and per-renderer costs were the largest unclaimed item:
   two 256-byte tables rebuilt per render, three pre-sized scratch strings per
   renderer, a tree walk whose result no option read, and an undersized
   output reservation together cost a comment-sized document 15–45% of its
   render time.
2. Heading ids were 20% of the render stage on documentation pages. Reading
   the slug source directly and skipping the escape pass a slug can never
   need paid more than the byte-cursor rewrite of the slugifier itself.
3. Both attempts to remove bookkeeping the profile blames — output growth
   checks in the escaper, the second paragraph scan in the parser — lost.
   The samples attributed there are stalls behind the stores and loads the
   work needs anyway.
4. `str` slicing at scanner-chosen cut points re-proves char boundaries the
   scanner already established; `is_char_boundary` showed 2.2% (prose) to
   14% (URL-dense) of render self time. Cutting the runs from the byte slice
   instead (A3) removed the symbol from the profile but measured as a tie
   with inconsistent rounds, so it was not promoted; the patch is archived.
5. Measuring under concurrent compilation is not salvageable by pairing:
   P-core saturation moves the worker onto efficiency cores mid-window. Steady
   background load (indexers) cancels out; an A/A control under the same
   conditions is the cheap way to tell the two apart.

## Remaining hot spots

From the HEAD profiles, after this round: the NEON scanning core (~30% of
parse) and `render_node` dispatch (8% of render) are structural floors; the
emphasis and link-parsing loops (~7% of parse) and code fence metadata
parsing (~3% of render) are the next measurable targets. `HtmlRendererOptions`
owning five `String`s and a `Vec<String>` makes a renderer per document cost
clone and drop of all of them (8.8% + 5.1% of the fresh pipeline on a 1 KB
comment in the worker); a cheaper representation is an API decision for v2.
GFM paragraphs are still read by up to four scans (block line scan, inline
markers, `@`/`:` pre-flight, `www.` search). The second-round profiles also
point at two smaller, separable items (a third, caching the marker option
bits per parser, measured as a tie): `line_end_neon` reaching 20% inclusive on prose because `line_at`,
`consume_line`, `skip_blank_lines`, `next_line_start` and list parsing
re-scan the same line; and the fence-run `memmem` finders at 11% inclusive
on a page with 114 fence lines.
