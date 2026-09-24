# Rejected: four candidates of round 6

Four pull requests were closed without merging after their paired
measurements:

- two second iterations of round 4's revisions (#424, #425);
- a third design for the GFM autolink pre-flight, in two versions (#427);
- the x86-64 port of round 5's fused root scan, in two versions (#428).

Their branches stay on the remote for reference. The patches are archived in
[`patches/`](patches/) as `git format-patch` series, one file per pull request.
Ratios are baseline time over candidate time, so above 1.000 is faster, and
output was verified identical in every run.

## #424: the NEON table pipe cursor, second iteration

Branch `perf/table-pipe-masks-v2`, commits `b609ffe7` (round 4's `be641a8f`,
ported onto `060b02d2`) and `d5f0dd31` (the revision). Patch:
[`patches/tables-424.patch`](patches/tables-424.patch).

### What round 4 left open

Round 4 measured the first iteration, [`tables.patch`](../2026-09-21-perf-round-4/patches/tables.patch).
Dense pipe rows parsed 1.31–1.43× faster, while the formatted rows, with an
escaped pipe every 26 bytes, parsed at 0.90–0.93 and `table-sparse-256` at
0.961. The report asked for a revision of the escaped-pipe path and of short
rows.

### The revision

- **Windows start at the pipe `memchr` found**, not at an aligned block, and
  nothing is classified before the first pipe is requested.
- **Pipes closer than 16 bytes need no call.** A spent window is followed
  directly by the next one, and only a window without a pipe hands the rest of
  the gap back to `memchr`.
- **Short rows and row tails take no padded copy.** Rows under 16 bytes use a
  byte loop, and a row's last window reloads its final 16 bytes.
- **The cell decoder runs only up to the cell's last recorded escape.**
- **No window carries escape parity to the next.** Recording an escape is a
  `min` and a store.

The previous splitter and decoder stay as test oracles. A differential run of
6,828 table-heavy documents under six option sets was identical in AST, spans,
HTML and arena bytes.

### The measurement

Baseline `060b02d2`:

| Run | fresh | reuse | parse | render |
| --- | ---: | ---: | ---: | ---: |
| Screen, 36 cases, 3 × 3 | 1.085 | 1.089 | 1.089 | 1.001 |
| Broad, 57 documents, 3 × 5 | 1.002 | 1.001 | 1.001 | 1.001 |

The screen gain comes entirely from the dense diagnostics. The broad set does
not move: no document is below 0.984 or above 1.039 in parse. A recheck ran
seven cases at 3 × 9, next to an A/A control on the same cases that held
0.995–1.008:

| Case | A/A parse | parse | A/A fresh | fresh |
| --- | ---: | ---: | ---: | ---: |
| `table-dense-256` | 1.001 | **2.289** | 1.003 | **2.123** |
| `table-formatted-256` | 1.007 | 0.935 | 1.002 | 0.946 |
| `table-sparse-256` | 1.000 | 0.949 | 1.001 | 0.953 |
| `table-plain-256` | 1.003 | 0.959 | 0.998 | 0.970 |
| `table-formatted-4096` | 1.003 | 0.977 | 1.001 | 0.984 |
| `rust-book-appendix-02-operators` | 1.003 | 0.979 | 1.002 | 0.986 |
| `comment-table` | 0.996 | 0.994 | 0.995 | 0.989 |

In the screen, `table-dense-4096` and `table-dense-16000` parsed at 3.035 and
3.112. The plain and sparse rows of 4 KB and more gained 3–6%, and their
256-byte versions lost 4–5%. The formatted rows lost at every size
(0.936–0.976).

### Why it loses

The vector cursor pays off only where pipes are a few bytes apart, as in the
synthetic dense rows. Realistic cells of 5–30 bytes, with or without escapes,
stay faster with one `memchr` per pipe. That includes the one table-heavy real
document, `rust-book-appendix-02-operators`.

Round 4 reached the same verdict for the first iteration. If the cursor is
picked up again, it needs a gate on pipe density: switch to the window cursor
only after a row has shown pipes closer than about 8 bytes, and keep today's
`memchr` path otherwise.

## #425: container line facts, second iteration

Branch `perf/list-line-facts-v2`, commit `fdfd9822` on `9e741a37`. Patch:
[`patches/lines-425.patch`](patches/lines-425.patch).

### What round 4 left open

The list item collector and the block quote walk answered each question about
a line with its own scan: its end, the next line's start, its indentation,
whether it is blank, and what follows the indentation. Round 4's
[`lines.patch`](../2026-09-21-perf-round-4/patches/lines.patch) answered all
of them from one walk of the leading space/tab run plus one terminator search.
List-heavy pages gained 5–8%, but `comment-quote` lost (parse 0.949): the
blank line that closes a quote paid for a terminator search the old walk never
made.

### The revision

- **The block quote breaks on a blank line before searching for the
  terminator.** It measures the space/tab run first and looks at the byte
  after it.
- **Trimming follows #414:** `whitespace::trim_start` and `is_blank`, not the
  Unicode `str::trim_start`.
- **The trimmed line is computed on demand.**

A thread-local switch sends each walk down either path, and a differential
test compares both parses under six option profiles.

### The measurement

Baseline `9e741a37`:

| Run | fresh | reuse | parse | render |
| --- | ---: | ---: | ---: | ---: |
| Screen, 36 cases, 3 × 3 | 0.999 | 1.000 | 0.998 | 1.001 |
| Broad, 57 documents, 3 × 5 | 0.999 | 1.001 | 0.999 | 1.001 |

- **What gains remain:** list-heavy pages and long prose, by 2–4%.
  `vite-docs-api-plugin` parses at 1.024 and reuses at 1.032, and
  `wiki-rainbow-plain-prose` parses at 1.042. Round 4 had
  `vite-docs-api-plugin` at parse 1.105.
- **The block-quote fix helped only partly:** `comment-quote` parse moved
  from 0.949 to 0.979.
- **A new loss:** `comment-checklist` measures parse 0.962, reuse 0.959 and
  fresh 0.967 in the broad run, and parse 0.956 in the screen.

### Why it no longer pays

#414 replaced the Unicode `str::trim` calls in the same list and quote code
with byte-level ASCII trimming. That removed much of what the shared line facts
used to save. What is left is a tie with small-document losses. The two
attempts at trimming per-line work inside the copy-and-reparse design are
behind #432, which proposes parsing containers on the original source instead.

## #427: the autolink pre-flight from recorded trigger offsets

Branch `perf/autolink-root-index`, commits `4fbf1b8e` (v1) and `cb73e496`
(v2), both on `9e741a37`. Patch:
[`patches/autolink-index-427.patch`](patches/autolink-index-427.patch), both
commits.

### The idea

`gfm_autolink::may_contain_autolink` runs once per block after `parse_inline`
has read the same content, and it is about 10% of parse on the M1. #427 records
the offsets of every possible trigger once and answers each block's pre-flight
from the offsets inside its range. A trigger is every `@`, and every `:` or `w`
that a `.` or `/` follows. That covers the colon of every `://` and the last
`w` of every `www.`, and leaves out the colons of prose and code.

A block holding an `@` still takes the full pass, which also settles
`mailto:` and `xmpp:`. Output is identical by construction, test builds assert
the gate value for every block, and a differential test compares AST and HTML.

- **v1** collected the offsets inside the aarch64 root scan from #413.
- **v2** put the root scan back to `main`'s. It records lazily, only the
  bytes blocks ask for, in one forward run with a 512-byte read-ahead, and
  stops for good past a density bound of 32 offsets plus 16 per KB.

### The measurement

Baseline `9e741a37`:

| Version | Run | fresh | reuse | parse | render |
| --- | --- | ---: | ---: | ---: | ---: |
| v1, offsets from the root scan | Screen, 36 cases | 0.996 | 0.994 | 0.996 | 1.004 |
| v1 | Broad, 57 documents | 1.008 | 1.010 | 1.010 | 1.005 |
| v2, lazy per-block record | Screen, 36 cases | 0.997 | 0.996 | 0.998 | 0.999 |
| v2 | Broad, 57 documents | 1.000 | 0.998 | 1.001 | 0.993 |

**v1** gained 1% across the broad set:

- 41 of 57 documents reused faster.
- The best parse gains were `comment-table` 1.085 and
  `legacy-docs-adr-readme-theme-composition` 1.082.
- `typescript-handbook-compiler-options` lost 13% in parse (0.870, fresh
  0.959, reuse 0.955). It holds 54 KB of raw HTML tables and only two trigger
  bytes, and the root scan did extra vector work over every byte of a body
  whose inline blocks never needed it.
- In the screen, the sparse table diagnostics lost too:
  `table-sparse-4096` parse 0.842 and `table-sparse-16000` 0.799.

**v2** removed those losses, and the broad gain went with them. The per-block
recording cost now falls on the smallest documents:

- In the broad run, `comment-ack` parses at 0.941 and reuses at 0.949,
  `comment-question` parses at 0.959, `comment-quote` at 0.963 and
  `comment-links` at 0.970.
- `comment-table` still parses at 1.050.

### Why it is a floor

This is the third design measured for the autolink pre-flight:

1. A standalone single-pass NEON pre-flight: about 1.009 in a prototype.
2. Folding the triggers into the inline marker loop (#415): parse 0.968, see
   [round 5](../2026-09-23-perf-round-5/REJECTED.md).
3. This index: v1 at 1.010 with a 13% loss, v2 a tie.

Every attempt either leaves the second pass's cost in place or moves it to
where it costs more. The per-block pre-flight is treated as the floor, unless
a fundamentally different approach comes up, such as autolinking during inline
parsing itself.

## #428: the fused root scan on x86-64

Branch `perf/x86-root-scan`, commits `bdde4db0` (v1) and `2bf6ba56` (v2) on
`05a55a01`. Patch:
[`patches/x86-root-scan-428.patch`](patches/x86-root-scan-428.patch), both
commits.

### The idea

On x86-64 the root parse still ran separate searches: the NUL `memchr` in
normalization, then the pre-pass's `[` probe and `]:` `memmem`. #413's
follow-up `0610f606` keeps them off aarch64, and the first x86 profile put
`Parser::with_phase` at 5.9% of parse, inclusive. #428 ports #413's fused NEON
loop:

- **SSE2** handles 16-byte blocks. Four blocks share one branch, and their NUL
  test folds into an unsigned lane-wise minimum.
- **AVX2** handles 32-byte blocks, behind `is_x86_feature_detected!`.
- **Both** end on one overlapping vector, with the `]:` lookahead built from
  `movemask` bits.

v1 took AVX2 from 32 bytes on. v2 took it only for bodies of 512 bytes or
more, because v1 had put comments of 37–298 bytes below parity. The aarch64
code is unchanged.

### The measurement

`x86-64 paired benchmark` workflow, baseline `05a55a01`, 57 broad documents,
3 × 5:

| Commit | Host CPU | fresh | reuse | parse | render |
| --- | --- | ---: | ---: | ---: | ---: |
| `bdde4db0` (AVX2 at every size) | AMD EPYC 7763 | 1.016 | 1.018 | 1.025 | 1.004 |
| `bdde4db0` | AMD EPYC 7763 | 1.017 | 1.020 | 1.027 | 1.006 |
| `bdde4db0` | AMD EPYC 9V74 | 1.019 | 1.020 | 1.027 | 0.995 |
| `2bf6ba56` (SSE2 below 512 bytes), run 1 | Intel Xeon Platinum 8573C | 1.017 | 1.017 | 1.020 | 1.024 |
| `2bf6ba56`, run 1 | AMD EPYC 7763 | 1.010 | 1.005 | 0.998 | 1.020 |
| `2bf6ba56`, run 1 | Intel Xeon 6973P-C | 1.017 | 1.018 | 1.019 | 1.028 |
| `2bf6ba56`, run 2 | AMD EPYC 9V74 | 1.011 | 1.013 | 1.012 | 1.009 |
| `2bf6ba56`, run 2 | AMD EPYC 7763 | 1.015 | 1.006 | 0.996 | 1.020 |
| `2bf6ba56`, run 2 | AMD EPYC 7763 | 1.013 | 1.006 | 1.001 | 1.018 |

- **Small documents lose on the AMD hosts.** v2's second run measured
  `comment-reproduction` (298 bytes) at parse 0.900–0.936 and reuse
  0.912–0.941, and `comment-review` at fresh 0.943–0.953. v1 had shown the
  same losses on `comment-ack` (fresh 0.940–0.961) and `comment-reproduction`
  (reuse 0.913–0.943).
- **The hosts disagree on v2.** The two Intel hosts of run 1 parse at
  1.019–1.020, with no case below 0.970 apart from `comment-inline-code`
  (0.948) on one of them. The AMD hosts parse at 0.996–1.012, with five to
  eight cases below 0.970.
- **Render moves without the renderer changing,** by up to +2.8%. The ~1–2%
  fresh gains sit at that level, the placement floor of these builds
  ([README](README.md#code-placement)).

### Why it loses on x86

On aarch64 the fused scan paid (fresh 1.014, parse 1.021 in #413), because
`memchr` there is 16-byte NEON and saving a pass saves real work. On x86-64,
`memchr` runs AVX2: 32 bytes per step, unrolled. Two separate passes, NUL and
then `[`/`]:`, are therefore cheap, and a fused SSE2 loop cannot beat them on
small bodies. An AVX2 loop behind a feature check adds a call that cannot
inline across its `target_feature` boundary, plus the check and the
upper-register clearing on return, and on small bodies that costs more than
it saves. x86-64 keeps the path from `0610f606`, and the roadmap records the
root scan as deliberately `memchr`/`memmem` there.
