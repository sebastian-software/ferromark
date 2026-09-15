# Apple Silicon iteration round — 2026-09-15

Ten commits between `a7f0a00` and `639c79b` came out of a round of short,
measured iterations aimed at aarch64 (Apple M1 Pro) hot paths. Every candidate
was screened, confirmed on the 57 broad documents in four stages, and either
committed or reverted; nine variants were rejected. This report records the
method, the numbers, the rejected variants, what was learned, and where the
next gains most likely are.

The original session evidence is preserved in [the evidence archive](evidence/README.md):
losslessly compressed result samples and verification, the frozen workers and
build metadata, the corpus and inputs, compact profiler exports, and source
patches reconstructed against each recorded baseline.

Follow-up: the [complete 207-case rerun](../2026-09-15-arm-full-suite/README.md)
confirms the finished branch after both fixes across all four stages. The
[new six-engine native comparison](../2026-09-15-native-arm/README.md) also
remeasures OX and v1 directly: v2 is effectively tied with OX fresh and reaches
1.018× OX throughput with reuse on the 14 agreeing inputs. The original
measurements and predictions below remain the historical account.

## Results

Cumulative, baseline `a7f0a00` against `9a5e071` (the two later fix commits are
neutral on these documents; see below). Ratios are baseline time over candidate
time, geometric means of per-document medians; higher is faster.

| Stage | 57 broad | Encyclopedia (12) | Technical docs (22) | Comments (12) | Plain prose (4) |
| --- | ---: | ---: | ---: | ---: | ---: |
| parse | **1.301×** | 2.008× | 1.203× | 1.097× | 1.036× |
| reuse | **1.257×** | 1.782× | 1.162× | 1.084× | 1.019× |
| fresh | **1.224×** | 1.697× | 1.152× | 1.048× | 1.020× |
| render | **1.188×** | 1.536× | 1.103× | 1.060× | 1.004× |

Every broad document is at or above 1.007× in parse, reuse, and fresh. By size
(fresh / reuse / parse / render): up to 512 B 1.060 / 1.106 / 1.093 / 1.088;
513–2,048 B 1.258 / 1.318 / 1.310 / 1.244; 2,049–16,384 B 1.110 / 1.119 /
1.137 / 1.087; 16,385–65,536 B 1.229 / 1.229 / 1.275 / 1.153; larger 1.399 /
1.403 / 1.517 / 1.351. The 57 autolink-profile replays gain 1.196 / 1.218 /
1.281 / 1.142.

The 14 documents of the [native OX comparison](../2026-09-14-native-matched/README.md)
(ten short comments, four plain-prose views) improve 1.044× fresh and 1.082×
reused. The previous comparison put the complete pipeline 4.0% fresh / 4.9%
reused behind original OX; this round more than covers that distance on paper,
but OX was not remeasured here and the native harness should confirm it.

After the two fix commits, the 45 link, table, and autolink diagnostics measure
1.128× fresh, 1.162× reused, 1.094× parse, and 1.198× render against `a7f0a00`
with no case below 0.988×.

Correctness: exact HTML and full AST `Debug` (including spans) agreed between
baseline and candidate for every case and stage before any timing; every timing
window's iteration checksum was verified; all 788 workspace tests, `rustfmt`,
Clippy with warnings denied, and the benchmark builds pass at every commit.
`ParseError` changed shape (`49e53a9`); no parsing or rendering semantics did.

## Method

- Harness: [`benchmarks/optimization-rounds`](../../../benchmarks/optimization-rounds/README.md)
  (`run.py`, `summarize.py`, frozen `worker.rs`, generic aarch64 codegen, fat
  LTO), 3 rounds × 3 pairs × 40 ms windows. Its `prepare.py` pins the baseline
  to `4de75d4`, so a session-local copy replaced that check with
  `git archive <revision>` and added a `--symbols` build for profiling. The
  baseline advanced to the last promoted commit after every accept.
- A/A control first (identical sources in both roles, all 57 documents): fresh
  1.003, reuse 1.001, parse 0.999, render 1.006; single render cases moved up
  to 0.975–1.087 from code layout alone.
- Per idea: written prediction, patch, targeted differential tests, a screen on
  the 14 OX documents plus idea-specific cases (parse/reuse/fresh, or render for
  renderer changes), then a confirm on all 57 broad documents in four stages.
- Decision rule: promote when the target stage's geomean is at least 1.010 with
  every round above 1.000, fresh and reuse are at least 1.000, and no broad case
  falls below 0.970 without an explanation; otherwise revert. Render-only losses
  on documents whose render code did not change were re-checked with a
  `--lto thin` control pair before being attributed to layout.
- Profiling: `xctrace record --template 'Time Profiler' --all-processes` on a
  symbolized worker (fat LTO, `debug = 1`, no strip), exported with
  `xctrace export --xpath '…table[@schema="time-profile"]'` and symbolized with
  `atos -i`, which resolves inlined frames and line numbers. `samply` with
  `--unstable-presymbolicate` plus `atos` gave the same picture.

## Promoted changes

Broad geomeans of each confirm run against its own baseline (the previous
commit). Details and per-category tables are in the commit messages.

| Commit | Change | parse | reuse | fresh | render |
| --- | --- | ---: | ---: | ---: | ---: |
| `2113d22` | Link destinations: NEON nibble classifier finds the next stop byte instead of a byte loop | 1.160 | 1.088 | 1.079 | 0.995 |
| `600e994` | `ByteClass`: flag table and nibble tables derived from one definition at compile time; `scan_balanced` uses it | 1.034 | 1.022 | 1.023 | 1.002 |
| `30c34c4` | URL escaping: non-ASCII bytes flagged by the classifier, no scalar pre-scan for ASCII runs | 1.000 | 1.039 | 1.032 | 1.099 |
| `650a61a` | GFM autolink pre-flight: one `memchr2('@', ':')` pass plus the `www.` finder only without `@` | 1.029 | 1.021 | 1.015 | 1.004 |
| `a464110` | Closing fences: jump with the pre-pass's cached fence-run finder instead of visiting every body line | 1.020 | 1.013 | 1.011 | 0.996 |
| `49e53a9` | `ParseError` boxed: `ParseResult<()>` is one register, block results shrink by a third (breaking API shape) | 1.013 | 1.009 | 1.004 | 0.996 |
| `bcb42ff` | `push_run_long`: 17–64-byte runs copied with overlapping 16-byte moves out of line instead of `memmove` | 0.997 | 1.023 | 1.019 | 1.072 |
| `9a5e071` | Links: `scan_balanced` reports nested `[`, `parse_destination` reports `\`/`&`; both searches dropped | 1.027 | 1.017 | 1.014 | 0.999 |
| `e7067d1` | URL escaper: eight-byte table walk after each hit before re-entering the vector scan (fix, see below) | neutral | | | |
| `639c79b` | Pointy-bracket destinations keep the original walk and always unescape (fix, see below) | neutral | | | |

The render dips of the parser-only commits (`2113d22`, `a464110`) did not
reproduce under thin LTO (1.00–1.04 on the same documents) and are code-layout
effects.

## Rejected variants

Each was measured with the same screen and reverted. Patches were kept outside
the repository during the session and are now preserved in the
[evidence archive](evidence/README.md), together with the results and build
identities.

- Autolink pre-flight over `@`, `:`, and `.` with `memchr3`: 707 dots in one
  39.7 KB document made a compare per sentence; code-heavy docs lost 3–5%.
- Text escaper iterating whole 16/32-byte lane masks before loading the next
  block: plain prose rendered 9% slower; only escape-dense code blocks gained.
- List continuation scanning each line once: a tie (parse 1.003).
- Inline copies up to 64 bytes inside `push_run`: escape-dense code blocks lost
  4–8% in both fat- and thin-LTO builds because the inlined body of the escape
  loops grew; the out-of-line helper `bcb42ff` recovered the gain.
- One vector-to-scalar move per 32 bytes in the three parser scanners: parse
  0.988, encyclopedia 0.94–0.96. Marker-dense text has a hit in most blocks, so
  recomputing per-vector masks cost more than the saved `fmov`, whose latency
  the out-of-order core already hides.
- Slugify through a 64-byte stack buffer: render 0.977 on heading-heavy docs;
  `from_utf8` and `memcpy` calls cost more than the per-byte pushes for
  20–40-byte headings.
- `line_end` in 64-byte blocks: parse 0.986; the block bound was the document
  end, so short lines paid the per-vector recomputation on every call.
- `get_unchecked` at known-ASCII split points in the escaper: a tie
  (render 1.010 with rounds 1.006–1.012).
- The eight-byte scalar walk also on the *first* scan of a URL: link-dense pages
  lost 4–12% of render time; walking only after a hit (`e7067d1`) is neutral.
- Build control `-C target-cpu=apple-m1` against generic aarch64, identical
  sources: fresh 1.011, reuse 1.003, parse 1.008, render 0.997 — no benefit
  worth a build-flag change; NEON is already baseline for this target.

## Why two fix commits were needed

The confirm gate covered the 57 broad documents. The final 207-case run showed
that `30c34c4` had made the entity-dense link diagnostic render 1.7× slower
(each hit re-entered the vector scan with the whole remaining URL) and that
`9a5e071` had made entity-dense pointy-bracket destinations parse 10–18%
slower (an extra match arm and store per `&`). Bisecting with the retained
build pairs took minutes; `e7067d1` and `639c79b` restore the diagnostics
(render 2.25× and parse 1.21× against `9a5e071`) while staying neutral on the
broad set. Future confirm runs should include the `scan-`, `table-`, and
`autolink-` diagnostics.

## Findings

1. Once the inline text runs are scanned with vectors, the remaining
   byte-at-a-time loops dominate. Link destinations alone were 19% of the
   CommonMark parse stage on encyclopedia text; bracket bodies another 8–10%.
   Every such loop that only needs to find the next member of a small ASCII
   byte set is a candidate for `ByteClass`.
2. Repeated SIMD probing after dense hits is the recurring trap: the roadmap
   already recorded it for unescaping, and it reappeared in the URL escaper.
   A short table walk after a hit fixes it without cost to clean input.
3. Render timings of single documents move ±5–9% between builds of identical
   source under fat LTO. Judge by the 57-document geomean and per-round
   agreement, and re-check suspicious render losses with `--lto thin`.
4. Growing the body of an inlined hot loop (wider copies in `push_run`) can
   cost more than the operation it saves; out-of-line helpers keep the loop
   compact.
5. Profiles attribute a large self-time share to the vector-to-scalar move
   (`vreinterpret_u64_u8`, `vgetq_lane_u64`, 11–17%), but that is where
   samples land while the branch waits, not removable work: fewer transfers
   per block did not help on marker-dense text.
6. `ParseResult<()>` at 64 bytes was returned through memory by every inline
   construct; a boxed error made it a register. The gain is small (parse
   1.013) but consistent, and the change is API-visible.
7. Ideas that looked promising in the profile but measured as ties or losses
   (single line scan for lists, unchecked slices, buffered slugify) show that
   the cutoff of 1.010 on the geomean is needed to keep the history honest.

## Remaining hot spots and recommendations

From the profile at `bcb42ff` (self / inclusive share of the stage), in order of
expected value.

1. **Renderer output growth checks (render, ~8–10%).** `RawVecInner::capacity`
   6.1% self and `Vec::reserve` 8.8% inclusive on documentation: every
   `push_run` re-checks capacity against `String` fields in memory. An escaper
   that reserves once per node and writes through a raw cursor kept in
   registers (unsafe, `set_len` at the end) could remove most of it. Expect
   render +4–6%; the unsafe surface needs the existing byte-oracle tests plus
   capacity edge cases.
2. **Paragraph text is scanned twice (parse, ~10% on long lines).**
   `line_end_neon` is 9.6% inclusive of the CommonMark parse stage on
   encyclopedia articles because block parsing finds the line end and inline
   parsing rescans the same bytes for markers (which include `\n`). Letting
   the paragraph collector hand line boundaries to the inline scan, or letting
   the inline scan report the line end back, would remove one pass. This is a
   structural change to `parse_paragraph_impl` and `parse_inline`; expect
   parse +3–5% on long-line documents.
3. **Reference-definition pre-pass bail (parse, ~3% with definitions).**
   `has_definition_candidate` walks every `[` with a `memrchr` back to the line
   start once a `]:` exists anywhere. Iterating the rarer `]:` occurrences and
   checking the preceding block lines (labels span at most 1,003 bytes) would
   cut that on link-dense technical documents. Needs a differential test
   against the current predicate.
4. **URL sanitization (render, ~3% on link-dense pages).** `is_safe_url` scans
   for control bytes with `bytes().any`, then `find(':')` and
   `find(&['/', '?', '#'])`; the last is a scalar multi-char search. `memchr3`
   and a SWAR control-byte test are direct swaps. `ipv6_authority_brackets` also
   runs a `memchr` for `[` on every URL.
5. **Heading slugs (render, ~6% on heading-heavy docs).** The buffered variant
   failed on call overhead; writing directly into the reserved output via
   `as_mut_vec` with a fixed-size overlapping copy, or a small
   `[u8; 16]`-chunked append without `from_utf8`, is the remaining option.
6. **Renderer preparation scan (render, ~5%).** `scan_node_for_render` and
   `is_toc_marker_paragraph` walk the tree before rendering to count headings
   and find `[[toc]]`. Skipping the walk when `inline_toc` is off, and
   reserving the heading map from a size heuristic instead of a count, avoids
   it in the strict profiles; the default profile would need the parser to flag
   the marker paragraph.
7. **Short-haystack `memchr` calls (parse, ~4%).** `memchr::fwd_byte_by_byte`
   still shows 3.7% on encyclopedia parsing from `memchr2` in
   `unescape_link_component` (titles, pointy destinations) and `memrchr` in
   `has_closer_from` on short slices. A scalar table walk for haystacks under
   16 bytes avoids the crate's dispatch.
8. **Emphasis bookkeeping (parse, ~3%).** `children.retain` after pairing
   (`DrainFilter::drop` 1.8%) and the `chars().next_back()` flanking decode
   (1.3%) are small, safe targets: an ASCII fast path for the neighbour bytes
   and a swap-remove or in-place compaction for empty text slots.
9. **Still untested from the original plan.** Fusing attribute escaping with
   the `\r`/`\n` handling; a two-shuffle classifier for the renderer's URL
   autolink terminators; first-byte dispatch in the tag filter; footnote
   reference counting without a `String` key; an integer formatter instead of
   `write!` for line numbers and spans; the inline children capacity ceiling
   (`(len / 20).clamp(4, 12)`) measured together with arena occupancy; the
   16 KB arena floor for tiny inputs.
10. **Portability.** The new `ByteClass` scans have a NEON path and a table
    fallback; x86-64 gets the fallback. Adding `pshufb` paths mirrors the
    existing SSSE3 code in `inline/scan.rs` and `escape/nibble.rs`, and the
    per-call `is_x86_feature_detected!` there could be hoisted at the same
    time. None of this is measured on x86-64 yet.
11. **Process.** Add `--baseline-revision` to
    `benchmarks/optimization-rounds/prepare.py` so rounds can compare against
    the last promoted commit without a local copy; include the diagnostics in
    the confirm gate; keep an A/A control per session; try PGO
    (`-Cprofile-generate`, `llvm-profdata`) as a labeled build experiment — the
    codegen-only `target-cpu` flag showed nothing, but profile-guided layout
    may address the render variance directly.
