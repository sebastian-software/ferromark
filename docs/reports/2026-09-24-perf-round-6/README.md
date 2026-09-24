# Optimization round 6: render hot spots, round 4's revisions and the first x86-64 measurements — 2026-09-24

Round 5 ended with a list of what its profile still showed: the GFM autolink
pre-flight, tables and lists in parsing, heading ids and the tag filter in
rendering. It also noted that none of this had ever been measured on x86-64.
This round worked on both. On Apple Silicon, five candidates took on that list:
two new render changes, second iterations of round 4's table and list
revisions, and a third design for the autolink pre-flight in two versions. Two
were merged. For x86-64, two CI workflows made the architecture measurable: the
paired harness on GitHub's runners (#419) and a `perf` profile (#423). The
first x86 profile pointed at four scanners that ran scalar or out-of-line code
there. Four x86 SIMD changes were measured on the new workflow, and three were
merged. Together they make the code of release 2.1.2 11–16% faster than 2.1.1
on x86-64 in every stage.

**Apple Silicon.** Same host (Apple M1 Pro), toolchain (Rust 1.95.0) and
corpus as round 5 (SHA-256 `31583750…` in every `run.json`). The harness is
also the same: every run of both architectures records the runner and worker
hashes of round 5. Workers are fat-LTO builds with `-C target-cpu=generic`.
Each branch was measured against the `main` state it started from:

- `060b02d2` (release 2.1.0) for #418, #422 and #424;
- `9e741a37` (`060b02d2` plus #418 and the CI-only #419) for #425 and #427;
- `af65cb43` for the aarch64 check of #426.

Run shapes:

- Screens: 3 rounds × 3 pairs × 40 ms windows per case and stage, on the 20
  screen documents plus 16 table and scan diagnostics.
- Broad confirmations: 3 rounds × 5 pairs on the 57 broad documents.
- Rechecks: 3 rounds × 7 pairs (#418, #422) or × 9 pairs (#424) on the
  documents that moved, each right after an A/A control on the same documents.

**x86-64.** The `x86-64 paired benchmark` workflow runs the same harness on
three `ubuntu-latest` hosts in parallel. Each host builds the fat-LTO workers
with `-C target-cpu=generic` (Rust 1.95.0) and measures the same
baseline/candidate pair on its own: the 57 broad documents, 3 rounds × 5 pairs.
GitHub assigns the CPU, so the round's hosts were Intel Xeon Platinum 8370C and
8573C, Intel Xeon 6973P-C, and AMD EPYC 7763 and 9V74. Results are read per
host.

Every case and stage was verified for exact HTML and AST equality between the
two workers before timing, on both architectures. Ratios are baseline time over
candidate time, medians of the paired windows; **higher is faster, below 1.000
the candidate is slower**. The other files of this report:

- [NUMBERS.md](NUMBERS.md): every stage table, per host for x86.
- [TABLES.md](TABLES.md): every Apple Silicon case.
- [X86-TABLES.md](X86-TABLES.md): every x86 case, one column per host.
- [REJECTED.md](REJECTED.md): the four closed candidates.
- [PROFILES.md](PROFILES.md): the first x86-64 profile.

Decision rule, unchanged from the earlier rounds: a candidate is kept when its
broad geomean is at least 1.010 with every round above 1.000, fresh and reuse
are at or above 1.000, no broad case is below 0.970, and outputs are
identical. A targeted win, such as round 5's tag filter gate, is judged on the
documents it targets and must reproduce in a recheck. On x86-64, a stage effect
has to show on all three hosts.

## Where the round started

Round 5's [remaining hot spots](../2026-09-23-perf-round-5/README.md#after-the-round-remaining-hot-spots),
from its profile of `060b02d2` on the M1 Pro:

| Item | Share | Round 6 candidate |
| --- | ---: | --- |
| GFM autolink pre-flight | about 10% of parse | #427, two designs |
| Tables | 10.1% of parse | #424, the second iteration of round 4's NEON pipe cursor |
| Lists | 11.8% of parse | #425, the second iteration of round 4's container line facts |
| Heading ids | 11.5% of render | #422 |
| Tag filter | 4.3% of render | #418 |
| Escaper | 36% of render | a measured floor on aarch64 |

x86-64 had never been timed. Round 5's roadmap named the paths that were
NEON-only: the `ByteClass` classifiers for link destinations and bracket
bodies, the line scan, round 5's fused root scan, and the renderer's ASCII
URL-span scan. Their fallbacks elsewhere are scalar code or `memchr`.

## The candidates

| Name | Pull request | Target | Idea | Outcome |
| --- | --- | --- | --- | --- |
| `tagdispatch` | #418 | all | The tag filter compares a raw HTML tag only with the filtered names that share its initial, from a compile-time table per byte | merged |
| `headingid` | #422 | all | Heading ids planned in one reused string with a hash-to-claim map, instead of one `CompactString` key per id | merged |
| `tables2` | #424 | aarch64 | Round 4's NEON pipe cursor, revised: windows start at the pipe, the decoder stops at the last escape | closed |
| `lists2` | #425 | all | Round 4's container line facts, revised: a block quote's closing blank line stops before the terminator search | closed |
| `autoidx`, `autoidx2` | #427 | aarch64 | The autolink pre-flight answered from recorded trigger offsets: from the root scan (v1), then recorded lazily per block (v2) | closed |
| x86 `ByteClass` | #426 | x86-64 | SSSE3 and AVX2 scans for the stop-byte classes, with runtime dispatch | merged |
| x86 root scan | #428 | x86-64 | Round 5's fused NUL + `]:` root scan with SSE2 and AVX2 | closed |
| x86 escaper | #430 | x86-64 | The escape classifiers as SSE2 compares inlined into the escape loops, replacing an out-of-line SSSE3 function | merged |
| x86 line scan | #431 | x86-64 | `line_end` with SSE2: an inline 16-byte probe and an out-of-line continuation | merged |

Two infrastructure changes made the x86 half possible: #419 (the paired
workflow, `.github/workflows/bench-x86.yml`) and #423 (the profile workflow,
`.github/workflows/profile-x86.yml`, and the profiling driver in
`benchmarks/optimization-rounds/profile`).

## Apple Silicon

### A/A control

`060b02d2` against itself, 36 screen cases:

| Stage | Geomean | Rounds |
| --- | ---: | --- |
| fresh | 0.9956 | 0.966 / 0.985 / 1.055 |
| reuse | 1.0044 | 0.995 / 0.998 / 0.972 |
| parse | 0.9973 | 0.962 / 0.977 / 1.000 |
| render | 1.0002 | 0.977 / 0.998 / 0.938 |

This control is weaker than round 5's. The one-minute load rose from 6.3 to
45.8 during the run while implementation agents ran their test suites (see
[the load gate](#the-load-gate)). The stage geomeans stay within ±0.5%, but
single rounds swing by up to 6% and single cases by up to 17%.
`table-plain-256` parse measured 0.826, with round medians
1.018 / 0.658 / 0.826. For that reason, every document a candidate moved was
rechecked afterwards at low load, next to a fresh A/A control on the same
documents. Those three controls held every document and stage within
0.984–1.012.

### #418: filtered names by initial

Round 5's first-byte gate (#411) settles tags that no filtered name starts
like, such as `<div`, `<a` and `<br`. Common tags that share an initial with
a filtered name still walked all nine names, though: `<td`, `<tr`, `<th`,
`<span`, `<strong`, `<p` and `<img`. #418 replaces the boolean gate with a
compile-time table from every byte to the set of filtered names starting with
it, in either case. `matching_tag` compares a tag with those one or two names
only, on bytes. The previous matcher stays the test oracle.

Broad, 57 documents, 3 × 5: fresh 1.007, reuse 1.004, parse 1.002, render
0.994. The change targets `typescript-handbook-compiler-options`, 54 KB of
raw HTML tables with 3,848 `<` (1,086 `code`, 1,070 `td`, 754 `p`, 564 `tr`).
It rendered at **1.288**, fresh 1.195. Three documents fell below 0.970 in
render. The run had started at a one-minute load of 45.8, right after the A/A
control's spike. The recheck ran five documents at 3 × 7, next to an A/A
control on the same five:

| Document | A/A render | #418 render | A/A fresh | #418 fresh |
| --- | ---: | ---: | ---: | ---: |
| `typescript-handbook-compiler-options` | 1.000 | **1.291** | 0.996 | **1.167** |
| `rust-book-ch00-00-introduction` (broad 0.842) | 1.002 | 1.002 | 1.001 | 1.000 |
| `comment-table` (broad 0.888) | 1.002 | 1.000 | 0.997 | 1.021 |
| `vue-docs-slots` | 0.996 | 0.995 | 1.004 | 1.018 |
| `rust-book-ch17-00-async-await` (broad 0.810) | 1.000 | 0.937 | 0.994 | 0.991 |

- **The render gain reproduces within 0.3%** (1.288 in the broad run).
- **Two of the three broad losses did not reproduce.** The broad run had
  started at a load of 45.8, and `comment-table` holds no `<` at all.
- **`rust-book-ch17-00-async-await` is code placement.** It holds 15 `<`
  (`figure`, `figcaption`, `img`). `f` starts no filtered name, and `img` is
  compared with one name, so the filter does almost no work there. Round 5
  measured #411 at render 1.056 on the same document, a swing in the other
  direction.
- **The recheck's own load rose too**, from 6.8 to 34.5 during the run. It
  shows in its reuse column (`vue-docs-slots` 0.903) and not in the render or
  fresh numbers above.

### #422: heading ids in reusable claim storage

Heading ids are on by default, so every renderer user pays for them. After
#408–#410, the planner stored one `CompactString` key per claimed id. Each
heading therefore hashed its slug twice, keys longer than 24 bytes went to the
heap, and the renderer copied the slug into a second scratch buffer. #422
writes each candidate id at the end of one reused `String` and claims it in
place. A `FxHashMap<u64, usize>` maps an id's hash to a claim record, and
records that share a full hash are chained, so equality is still decided on
the bytes. `clear` keeps every capacity. The previous planner stays the
oracle, and a render hook with the previous routine is compared byte for byte
with the built-in renderer.

| Run | fresh | reuse | parse | render |
| --- | ---: | ---: | ---: | ---: |
| Screen, 36 cases, 3 × 3 | 1.007 | 1.004 | 1.005 | 1.024 |
| Broad, 57 documents, 3 × 5 | 1.003 | **1.012** | 1.000 | **1.035** |

Broad render rounds were 1.033 / 1.037 / 1.037, and 51 of 57 documents render
faster. Documents with a few long headings in little text gain most:

- `legacy-docs-migration-0-2` (1,985 bytes, 5 headings, 31 characters on
  average): render 1.199, reuse 1.079.
- `vue-docs-ways-of-using-vue`: render 1.133.
- `legacy-docs-migration-0-3`: render 1.132.

Three Wikipedia article bodies fell below 0.970 in render. The screen ran
while the load rose from 8.1 to 61.5, and the broad run started right after at
that load, so the six documents were rechecked at a load of 5–6:

| Document | A/A render | #422 render | A/A reuse | #422 reuse |
| --- | ---: | ---: | ---: | ---: |
| `legacy-docs-migration-0-2` | 0.999 | **1.195** | 0.998 | **1.071** |
| `vue-docs-ways-of-using-vue` | 0.999 | **1.136** | 1.001 | **1.041** |
| `wiki-rainbow-article-body` | 0.995 | 1.027 | 0.990 | 1.003 |
| `wiki-volcano-article-body` (broad 0.969) | 1.010 | 0.975 | 1.007 | 0.992 |
| `wiki-tea-article-body` (broad 0.952) | 0.993 | 0.967 | 1.002 | 0.988 |
| `wiki-chess-article-body` (broad 0.935) | 0.984 | 0.924 | 0.988 | 1.001 |

The render loss on `wiki-chess-article-body` reproduces, but heading ids do
not explain it:

- **Headings are a small part of the document.** It is 113 KB with 47
  headings, so a 7% render change is far more than all its heading work.
- **Reuse is flat.** Reuse renders the same document after parsing it, and it
  stays at 1.001. A real render cost would show there as well.
- **These documents swing anyway.** The Wikipedia article bodies are the
  documents known to move 5–9% in render-only numbers between fat-LTO builds
  of equivalent code (round 5: `wiki-volcano-article-body` render 0.939 under
  a parser change).

### Round 4's revisions: #424 and #425, closed

| Candidate | Run | fresh | reuse | parse | render |
| --- | --- | ---: | ---: | ---: | ---: |
| #424 tables | Screen, base `060b02d2` | 1.085 | 1.089 | 1.089 | 1.001 |
| #424 tables | Broad | 1.002 | 1.001 | 1.001 | 1.001 |
| #425 container lines | Screen, base `9e741a37` | 0.999 | 1.000 | 0.998 | 1.001 |
| #425 container lines | Broad | 0.999 | 1.001 | 0.999 | 1.001 |

- **#424** keeps round 4's gain on dense pipe rows: the dense table
  diagnostics parse 2.3–3.1× faster. It still loses on realistic cells of 5–30
  bytes, from `table-formatted-256` parse 0.935 to
  `rust-book-appendix-02-operators` 0.979 in the recheck, and no real document
  gains.
- **#425** fixed round 4's block-quote loss only partly (`comment-quote` parse
  0.949 → 0.979) and added a new one: `comment-checklist` parse 0.962. Most of
  round 4's list gain was gone because #414 had already removed the Unicode
  trims from that code.

Details: [REJECTED.md](REJECTED.md#424-the-neon-table-pipe-cursor-second-iteration)
and [REJECTED.md](REJECTED.md#425-container-line-facts-second-iteration).

### The autolink pre-flight, a third design: #427, closed

| Version | Run | fresh | reuse | parse | render |
| --- | --- | ---: | ---: | ---: | ---: |
| v1 `4fbf1b8e`, offsets from the root scan | Screen, base `9e741a37` | 0.996 | 0.994 | 0.996 | 1.004 |
| v1 | Broad | 1.008 | 1.010 | 1.010 | 1.005 |
| v2 `cb73e496`, lazy per-block record | Screen | 0.997 | 0.996 | 0.998 | 0.999 |
| v2 | Broad | 1.000 | 0.998 | 1.001 | 0.993 |

- **v1** gained 1% across the broad set but parsed
  `typescript-handbook-compiler-options` at 0.870: the root scan did extra
  vector work over 54 KB of raw HTML that no inline block needed.
- **v2** removed that loss, and the gain went with it. Its per-block recording
  cost the smallest comments 3–6% (`comment-ack` parse 0.941).

After a standalone NEON pass, the fusion into the marker scan (#415) and this
index, the per-block pre-flight is treated as a floor; see
[REJECTED.md](REJECTED.md#427-the-autolink-pre-flight-from-recorded-trigger-offsets).

### x86 changes on aarch64

- **#426** changes the `first_in` threshold check that aarch64 shares, so it
  was measured there: broad fresh 1.001, reuse 1.003, parse 1.001, render
  1.001, no case below 0.970 (`results/arm426-broad`, base `af65cb43`).
- **#430 and #431** leave the aarch64 release assembly identical to `main`
  apart from panic-location line numbers, as their pull requests show, so
  there was nothing to measure.

## x86-64

### The workflows: #419 and #423

`bench-x86.yml` (#419) runs `benchmarks/optimization-rounds` unchanged. Its
inputs are a baseline revision, a candidate revision, the case set and the
number of pairs. Three hosts run in parallel, and each builds both workers,
verifies output equality and measures the pair alone. Each host uploads
`summary.json`, `grouped.json`, `run.json`, `build.json` and a `host.txt` with
its CPU model and SIMD flags. A pull request that touches the harness measures
itself against itself. `profile-x86.yml` (#423) builds the profiling driver
for any revision with `-C target-cpu=generic -C force-frame-pointers=yes` and
samples parse and render with `perf record -g` on two hosts.

### A/A control on three hosts

#419's own head `a4d2fd0a` against itself, 57 broad documents, 3 × 5:

| Host | CPU | fresh | reuse | parse | render | Cases outside ±3% (fresh / reuse / parse / render) |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| 1 | Intel Xeon Platinum 8370C (AVX-512) | 1.0019 | 1.0005 | 1.0015 | 1.0025 | 0 / 1 / 0 / 0 |
| 2 | AMD EPYC 7763 (AVX2) | 0.9992 | 0.9985 | 0.9988 | 1.0022 | 1 / 2 / 1 / 0 |
| 3 | AMD EPYC 7763 (AVX2) | 0.9973 | 0.9953 | 0.9982 | 0.9996 | 0 / 1 / 2 / 1 |

- **Geomeans:** every stage geomean is within ±0.5%, and every per-round
  geomean within 0.992–1.012.
- **Per case:** the 5th to 95th percentile of per-case ratios spans about
  0.975–1.025, with extremes of 0.955 and 1.058.
- **What it resolves:** a stage effect of about 1% that shows on all three
  hosts, and per-case effects above about 3%.

This control measures VM noise between two builds of the same source. It does
not measure code placement, which differs only when the code differs; see
[methodology](#methodology-what-the-round-learned-about-measuring).

### The first x86-64 profile

#423's own check run profiled `060b02d2` on an AMD EPYC 9V74 and an Intel
Xeon Platinum 8573C. The ranking differs from Apple Silicon's where x86 ran
scalar or out-of-line code ([PROFILES.md](PROFILES.md)):

- **Escaping took 40–46% of render self time**, against 36% inclusive on the
  M1. `nibble::first_flagged_ssse3` alone took 19–23%. The SSSE3 classifier
  lived in a `#[target_feature]` function behind `is_x86_feature_detected!`.
  A baseline build cannot inline it, so the escape loop called it once per
  replaced byte, even for strings under 16 bytes. This became #430.
- **`parse_destination` took 4.4–4.7% of parse**, all self time, with the
  scalar `ByteClass` walk inlined into it. `scan_balanced_matched` took about
  2%. This became #426.
- **`Parser::with_phase` took 5.9% inclusive**: the separate NUL `memchr`, `[`
  probe and `]:` `memmem` of the root. This became #428.
- **Line walking:** `cursor::line_at` 1.3% and `probe_line_inner` about 1%
  self, plus the SWAR loop inlined into `parse_paragraph` and `parse_list`.
  This became #431.

### #414 on x86-64

The workflow's first dispatched run measured round 5's trimming fix on x86:
`aceafd32` against #414's `7eda5a4b`.

| Host | CPU | fresh | reuse | parse | render |
| --- | --- | ---: | ---: | ---: | ---: |
| 1 | AMD EPYC 9V74 | 1.010 | 1.019 | 1.027 | 0.998 |
| 2 | Intel Xeon Platinum 8573C | 1.020 | 1.027 | 1.046 | 1.000 |
| 3 | AMD EPYC 9V74 | 1.015 | 1.022 | 1.029 | 1.004 |

The fix pays on x86 as well: parse 1.027–1.046, against 1.051 on the M1. The
median over hosts puts four tiny inputs at or below 0.970 in fresh:
`guard-angle-link` 0.943, `comment-question` 0.952, `comment-ack` 0.958 and
`comment-inline-code` 0.970. Later runs showed the same pattern (see below).

### #426: `ByteClass` with SSSE3 and AVX2

`ByteClass` classifies stop bytes for link destinations and bracket bodies.
It had a NEON scan on aarch64 and the flag-table walk everywhere else. #426
adds two x86 scans:

- **SSSE3:** a 16-byte `pshufb` nibble classifier over the existing tables.
- **AVX2:** the first 16 bytes with 128-bit operations, then 32-byte steps.

The published addons target the x86-64 baseline, so the scan level is
detected at run time. Remainders under 16 bytes keep the inlined table walk.

| Commit | Host CPU | fresh | reuse | parse | render |
| --- | --- | ---: | ---: | ---: | ---: |
| `09538a76`, dispatch inlined | AMD EPYC 7763 | 1.029 | 1.027 | 1.046 | 0.991 |
| `09538a76` | AMD EPYC 7763 | 1.031 | 1.029 | 1.046 | 0.996 |
| `09538a76` | AMD EPYC 9V74 | 1.027 | 1.024 | 1.044 | 0.992 |
| **`b64cc1e6`, dispatch out of line** | AMD EPYC 7763 | **1.047** | **1.055** | **1.069** | 1.022 |
| **`b64cc1e6`** | AMD EPYC 9V74 | **1.041** | **1.048** | **1.060** | 1.007 |
| **`b64cc1e6`** | AMD EPYC 7763 | **1.046** | **1.054** | **1.068** | 1.020 |

The first commit lost 4–7% on seven documents. A throwaway probe that counted
every `first_in` call showed that **none of the seven calls `ByteClass`
at all**, so the losses could not come from the scan. What changed for them
was code size: the inlined dispatch, two feature checks with their cold
detector calls and a second copy of the table walk, grew the four call sites
by 40–61 instructions each. The revision moved the dispatcher out of line. It
caches the level in one byte and tail-calls the scan, so a call site now
carries the length check and one call (+0 to +24 instructions). Median over
hosts:

| Document | v1 parse | v2 parse | v1 reuse | v2 reuse |
| --- | ---: | ---: | ---: | ---: |
| `comment-unicode` | 0.937 | 0.992 | 0.974 | 1.000 |
| `comment-reproduction` | 0.937 | 1.013 | 0.926 | 0.974 |
| `rust-book-appendix-02-operators` | 0.952 | 1.022 | 0.957 | 1.024 |
| `legacy-docs-migration-0-8` | 0.960 | 0.994 | 0.976 | 0.993 |
| `comment-review` | 0.985 | 0.998 | 0.951 | 1.012 |
| `comment-question` | 0.999 | 0.997 | 0.956 | 0.991 |
| `comment-ack` | 1.014 | 0.985 | 0.962 | 0.966 |

- **Largest gains:** link-dense prose. The Wikipedia leads, first paragraphs
  and article bodies parse at 1.19–1.43 (median over hosts), and the
  encyclopedia category at 1.293.
- **Still below 0.970 on the median:**
  - fresh: `comment-ack` 0.966 (hosts 0.940–0.971) and `comment-reproduction`
    0.967;
  - parse: `guard-angle-link` 0.933 and `comment-checklist` 0.949, each
    0.99–1.00 on one of the three hosts.
- **Render moved +0.7–2.2%**, a stage the change does not touch.
- **aarch64:** unaffected (see above).

### #428: the fused root scan on x86-64, closed

Round 5's fused NUL + `]:` root scan, ported to SSE2 and AVX2, gained about
1–2% in fresh at best and lost 5–10% on small documents, mostly on the AMD
hosts. On x86, `memchr` itself runs AVX2 with 32-byte unrolled steps, so the
two separate passes stay cheap. x86 keeps the separate searches of #413's
follow-up `0610f606`. Details and per-host numbers:
[REJECTED.md](REJECTED.md#428-the-fused-root-scan-on-x86-64).

### #430: escape classifiers in inlined SSE2

SSE2 is part of the x86-64 baseline. #430 classifies both needle sets with
SSE2 compares in plain `#[inline]` code, the way the NEON path already works:

- **HTML** (`& < > " '`): the fold of the SWAR `escape_mask`, three compares
  and one `movemask`.
- **URL:** the same folds, plus a wrapping add that maps `[ \ ]` onto
  `0x80..=0x82`, and `v` itself ORed in for the non-ASCII lanes.

No SSSE3 path and no runtime detection remain, and the escape loops are
monomorphized with their classifier. Baseline `f8fff4c9` (#426 merged):

| Host | CPU | fresh | reuse | parse | render |
| --- | --- | ---: | ---: | ---: | ---: |
| 1 | Intel Xeon Platinum 8573C | 1.031 | 1.031 | 1.006 | **1.092** |
| 2 | AMD EPYC 7763 | 1.018 | 1.016 | 0.996 | **1.091** |
| 3 | AMD EPYC 7763 | 1.022 | 1.021 | 1.002 | **1.093** |

- **Render:** every round on every host was between 1.089 and 1.094, and 56
  or 57 of 57 documents render faster. The comments gain most:
  `comment-inline-code` 1.138 and `comment-links` 1.137 (median over hosts).
  The smallest gain is on `typescript-handbook-compiler-options` (1.002),
  which is raw HTML.
- **One outlier:** `rust-book-appendix-02-operators` measures fresh 0.961 and
  parse 0.944–0.951 on all hosts while its render gains 1.058. The parse is
  code #430 does not touch. The same document moved with placement alone in
  #426's first commit and later in #431's placebo.

### #431: `line_end` with SSE2

`line_end` finds line ends for the pre-pass and most block-parser line walks.
aarch64 scans with NEON, and every other target used the eight-byte SWAR word
test. Four variants were measured against `dfcfe983`; the ranges are over the
three hosts:

| Candidate | Hosts | fresh | reuse | parse | render |
| --- | --- | ---: | ---: | ---: | ---: |
| `273b6bf6`: whole scan out of line | 3 × AMD EPYC 7763 | 1.039–1.041 | 1.037–1.048 | 1.055–1.065 | 1.012–1.014 |
| **`d6e0bad3`: inline 16-byte probe, out-of-line rest (merged)** | 3 × AMD EPYC 7763 | **1.037–1.039** | **1.038–1.039** | **1.062–1.065** | 1.000–1.005 |
| `78429a94`: fully inline, comparison only | 3 × AMD EPYC 7763 | 1.031–1.033 | 1.034–1.035 | 1.046–1.048 | 1.004–1.011 |
| `41f4e99e`: placebo, `d6e0bad3`'s structure without SSE2 | 2 × Intel Xeon Platinum 8573C, 1 × AMD EPYC 7763 | 0.998–1.002 | 0.997–0.999 | 0.991–0.993 | 1.006–1.027 |

The variants:

- **`273b6bf6`:** the `>= 16` threshold tests the bytes left in the document,
  not the line, so nearly every line end became a call.
- **`d6e0bad3`:** probes the 16 bytes at `from` inline and calls the 32-byte
  SSE2 scan only for longer lines. Main callers keep their register saves,
  and the parser shrinks by 589 instructions against `main`.
- **`78429a94`:** grows the parser by 2,649 instructions, according to its
  commit message, and gains least.
- **`41f4e99e`:** keeps `d6e0bad3`'s structure (the inline probe and the
  out-of-line continuation) but runs main's word tests in both. It was
  built only to measure what the structure and the new code layout cost
  without the SSE2.

Plain prose with long lines gains most under `d6e0bad3`: `wiki-*-plain-prose`
parse 1.38–1.48 and fresh 1.26–1.29, median over hosts. Small documents, also
median over hosts:

| Document | Variant | fresh | reuse | parse | render |
| --- | --- | ---: | ---: | ---: | ---: |
| `comment-ack` (37 bytes) | `273b6bf6` | 0.942 | 1.027 | 1.000 | 1.009 |
| | `d6e0bad3` | 0.929 | 0.992 | 1.003 | 1.002 |
| | `78429a94` | 0.913 | 0.980 | 0.942 | 1.004 |
| | placebo | 1.011 | 1.006 | 0.999 | 1.003 |
| `comment-question` (160 bytes) | `273b6bf6` | 0.940 | 1.020 | 0.969 | 1.008 |
| | `d6e0bad3` | 0.940 | 0.999 | 1.000 | 0.988 |
| | `78429a94` | 0.908 | 0.967 | 0.948 | 1.014 |
| | placebo | 0.980 | 1.013 | 1.003 | 1.008 |
| `comment-quote` | `273b6bf6` | 1.000 | 0.994 | 1.027 | 0.999 |
| | `d6e0bad3` | 0.995 | 0.909 | 0.914 | 0.997 |
| | `78429a94` | 0.999 | 0.970 | 0.906 | 1.008 |
| | placebo | 1.019 | 1.010 | 0.981 | 1.043 |

- **`comment-ack` and `comment-question` lose in fresh only.** Their parse,
  reuse and render stay at 0.99–1.00. Neither calls `line_end` in a release
  build: the pull request logged every call. For inputs this small, fresh is
  mostly creating and freeing the arena and the renderer, code the line scan
  never runs. The SSE2 variants shifted that code, and the placebo, with its
  different layout, happened not to.
- **`comment-quote` makes a single `line_end` call.** Its parse ranges from
  0.906 to 1.027 across the four variants, the placebo included. That spread
  comes from code placement, not from the scan.
- **The placebo moves other documents instead.**
  `rust-book-appendix-02-operators` parses at 0.92–0.95 under it, and its
  untouched render moves by up to +2.7%.

### Cumulative: 2.1.1 against the code of 2.1.2

`05a55a01` (release 2.1.1) against `c6a29e34`, which is `main` after #426,
#430 and #431 and the code of release 2.1.2 (`09e5e866` changes only the
version). The harness needs the same `Cargo.lock` on both sides, so the release
commit itself cannot be a candidate. The candidate worker is byte-identical to
the one measured as #431's revision, and the baseline worker to #428's: the
`build.json` files record the same binary hashes. 57 documents, 3 × 5:

| Host | CPU | fresh | reuse | parse | render |
| --- | --- | ---: | ---: | ---: | ---: |
| 1 | Intel Xeon 6973P-C | **1.148** | **1.152** | **1.160** | **1.126** |
| 2 | AMD EPYC 7763 | **1.116** | **1.125** | **1.135** | **1.113** |
| 3 | AMD EPYC 7763 | **1.117** | **1.129** | **1.143** | **1.112** |

Every document renders faster on every host, and every round of every stage is
within 0.5% of its host's geomean. Fresh is faster on 54–57 of 57 documents,
reuse on 54–56.

- **Parse by category** (median over hosts): the encyclopedia pages parse at
  1.454 and plain prose at 1.428, where long lines and dense links meet the
  new line scan and `ByteClass`. The `commonmark` profile parses at 1.417, the
  `gfm` profile at 1.043.
- **Small-document parse losses:**
  - `comment-quote` 0.930 (hosts 0.916–1.005);
  - `comment-unicode` 0.955 (0.951–1.021);
  - `comment-reproduction` 0.915 on the Intel host only.
- **The gains roughly multiply.** The solo parse gains, about 1.065 for #426
  and 1.063 for #431, multiply to about 1.13. The render gains, 1.092 for #430
  times #426's placement shift, come to about 1.11. The cumulative run measured
  parse 1.135–1.160 and render 1.112–1.126, on a different mix of hosts.

#418 and #422, the round's Apple Silicon merges, were already in 2.1.1, so
this run does not include them. Neither was measured on x86-64, and there was
no cumulative Apple Silicon run in this round.

## Methodology: what the round learned about measuring

### Code placement

- **A change can move documents that never run it.** #426's first commit lost
  4–7% on seven documents that make no `ByteClass` call. The inlined
  `is_x86_feature_detected!` dispatch grew four callers by 40–61 instructions,
  and that changed how the code around them was laid out. Moving the dispatch
  out of line kept the scan and removed the losses.
- **Three hosts do not rule placement out.** Each CI host builds its workers
  from the same source with the same toolchain and flags. In every run of the
  round, the three hosts produced byte-identical workers: their `build.json`
  files record the same binary hashes. A layout effect therefore repeats on
  every host and looks like a consistent result. Agreement across hosts rules
  out VM noise, not placement.
- **The x86 placement floor is ±2–3% on a stage the change does not touch.**
  Render moved +0.7–2.2% under #426's revision, up to +2.8% under #428, and
  +0.6–2.7% under #431's placebo. Parse moved 0.94–0.95 on
  `rust-book-appendix-02-operators` under #430. The A/A control cannot show
  this, because both of its workers are built from the same source.
- **A placebo build measures placement directly.** #431's placebo kept the
  candidate's code structure and ran the old algorithm. Next to it, reading
  every stage per document, not only fresh, showed that the small-comment
  losses were fresh-only. They sat in setup and teardown code the scan never
  runs, and they moved with the layout, not with the scan.
- **On x86, prefer baseline code that inlines.** Hot loops should use SSE2
  code that can inline over `#[target_feature]` functions, and runtime
  dispatch should stay out of line:
  - #430 replaced an out-of-line SSSE3 call per escaped byte with inlined SSE2
    compares, and render gained 9%.
  - #426 gained once its dispatch left the call sites.
  - #431's inline probe plus out-of-line continuation beat the fully inline
    variant.
  - #428's AVX2 path behind a feature check lost on small bodies.

### Measuring next to agents

#### The load gate

The Apple Silicon runs shared the machine with implementation agents that ran
test suites, and their test binaries load the CPU without showing up as
`rustc` or `cargo`. The A/A screen ran while the one-minute load rose from 6.3
to 45.8, and the heading-id screen from 8.1 to 61.5.

After the heading-id runs, `measure.sh` gained a load gate. It waits until no
`rustc`, `cargo` or `clang` runs and the one-minute load is below 8, gives up
after 20 minutes and logs that. Every later run started at a load of 3.8–7.8.
The gate only checks the start, though: `autoidx2-broad` ended at 16.2. So an
outlier still needs a recheck next to an A/A control on the same documents,
which is how this round confirmed #418 and #422.

#### Avoiding a deadlock between waiting agents

Agents that compile while measurements may run wait before every `cargo`
command:

```sh
while pgrep -f '[Pp]ython.* [^ ]*(optimization-rounds|native-comparison)/run[.]py' >/dev/null; do sleep 30; done
```

Round 5 used `pgrep -f optimization-rounds/run.py`. That pattern also matches
the command line of another agent's wait loop, which contains the same text.
Two waiting agents then waited for each other indefinitely. The bracket
expressions keep the pattern from matching its own text, and the interpreter
prefix requires a Python process that actually runs a harness.

## Decisions

- **Merged on Apple Silicon evidence:**
  - #418 (`4b7b2591`): a targeted render win, like #411. Render 1.29 on the
    raw-HTML document it targets, reproduced in the recheck. The broad
    outliers did not reproduce, apart from a placement swing on a document
    the filter barely touches.
  - #422 (`77b2fcd1`): broad render 1.035 and reuse 1.012, output identical,
    public API unchanged. The Wikipedia article-body render readings are
    read as placement, since reuse is flat on the same documents.
- **Merged on x86-64 evidence:**
  - #426 (`f8fff4c9`): parse 1.060–1.069 on three hosts after the dispatch
    moved out of line; neutral on aarch64.
  - #430 (`dfcfe983`): render 1.091–1.093 on three hosts; aarch64 assembly
    unchanged.
  - #431 (`c6a29e34`): parse 1.062–1.065, fresh and reuse 1.037–1.039;
    aarch64 disassembly unchanged.
- **Merged infrastructure:** #419 (`9e741a37`) and #423 (`af65cb43`).
- **Closed:** #424, #425, #427 and #428 ([REJECTED.md](REJECTED.md)). Their
  branches stay on the remote, and the patches are archived in
  [`patches/`](patches/).
- **Opened #432:** parsing block quotes and list items on the original source
  instead of copying and reparsing them. It follows from the two line-facts
  attempts and is tracked outside the rounds.

Releases: #418 and #422 shipped in 2.1.1 (`05a55a01`), #426, #430 and #431 in
2.1.2 (`09e5e866`).

## After the round

- **No profile after the round.** The x86 profile predates every x86 change
  of the round.
- **Still open on x86:**
  - The renderer's ASCII URL-span scan for autolinks is still NEON-only.
  - The root scan stays on `memchr`/`memmem` on purpose (#428).
- **Parse:**
  - The autolink pre-flight is treated as a floor after three designs.
  - Tables have one idea left: a pipe-density gate in front of #424's
    cursor.
  - Containers move to #432.
- **Render:** the slugifier (`slugify_heading_into`, about 5% of render on
  both architectures) is the part of heading ids that #422 left alone.
- **Small documents:** fresh on inputs of a few hundred bytes is dominated by
  arena and renderer setup and teardown. It is the most placement-sensitive
  stage on x86 and should be read together with the other stages.

## How the round was run

**Agents implemented, and the coordinator measured.** Each implementation
agent worked in its own worktree and waited with the `pgrep` loop above before
every `cargo` command. The Apple Silicon candidates and baselines were built
from `git archive` of the pushed branches and revisions, whose headers carry
the commit ids:

| Candidate | Commit | Baseline |
| --- | --- | --- |
| #418 | `1fd328df` | `060b02d2` |
| #422 | `a620dad6` | `060b02d2` |
| #424 | `d5f0dd31` | `060b02d2` |
| #425 | `fdfd9822` | `9e741a37` |
| #427 v1 | `4fbf1b8e` | `9e741a37` |
| #427 v2 | `cb73e496` | `9e741a37` |
| #426 | `b64cc1e6` | `af65cb43` |

**Apple Silicon run times.** The runs took place between 15:55 and 19:47 UTC on
2026-09-23. [measure-log.txt](measure-log.txt) records the load before and
after each one. The two lines of round 5's follow-up run `fix414-screen`, which
ran in between, are in round 5's log.

**x86 runs.** They were dispatched one after another, because the workflow's
concurrency group cancels a second dispatch on the same ref. Each run's
baseline is the verified revision in its `build.json`, and each candidate is
the dispatch input recorded in the workflow log:

| Name | Workflow run | Started (UTC) | Baseline | Candidate | Hosts |
| --- | --- | --- | --- | --- | --- |
| `aa` | 35884585467 | 2026-09-23 15:51 | `a4d2fd0a` | `a4d2fd0a` | Intel 8370C, AMD 7763 ×2 |
| `414` | 35887708555 | 2026-09-23 16:17 | `aceafd32` | `7eda5a4b` | AMD 9V74, Intel 8573C, AMD 9V74 |
| `426` | 35905106741 | 2026-09-23 18:49 | `af65cb43` | `09538a76` | AMD 7763 ×2, AMD 9V74 |
| `428` | 35906097857 | 2026-09-23 18:58 | `05a55a01` | `bdde4db0` | AMD 7763 ×2, AMD 9V74 |
| `426b` | 35910061074 | 2026-09-23 19:32 | `af65cb43` | `b64cc1e6` | AMD 7763, AMD 9V74, AMD 7763 |
| `428b` | 35910866752 | 2026-09-23 19:40 | `05a55a01` | `2bf6ba56` | Intel 8573C, AMD 7763, Intel 6973P-C |
| `428c` | 35914952264 | 2026-09-23 20:17 | `05a55a01` | `2bf6ba56` | AMD 9V74, AMD 7763 ×2 |
| `430` | 35924115738 | 2026-09-23 21:42 | `f8fff4c9` | `5782b91c` | Intel 8573C, AMD 7763 ×2 |
| `431` | 35970657056 | 2026-09-24 07:37 | `dfcfe983` | `273b6bf6` | AMD 7763 ×3 |
| `431b` | 35972718389 | 2026-09-24 08:00 | `dfcfe983` | `d6e0bad3` | AMD 7763 ×3 |
| `431inl` | 35974061535 | 2026-09-24 08:14 | `dfcfe983` | `78429a94` | AMD 7763 ×3 |
| `431pl` | 35975486243 | 2026-09-24 08:28 | `dfcfe983` | `41f4e99e` | Intel 8573C, AMD 7763, Intel 8573C |
| `total` | 35990842455 | 2026-09-24 11:03 | `05a55a01` | `c6a29e34` | Intel 6973P-C, AMD 7763 ×2 |

The runners' one-minute load stayed between 0.7 and 1.9. Round 5's #413
follow-up `0610f606` was not measured on its own; #428 measured the
alternative to it and lost.

## Reproduction

**Apple Silicon.** The scripts in [`harness/`](harness/) read two
environment variables:

- `ROUND6_WORK`: the round's scratch directory, a session path during the
  round.
- `FERROMARK_REPO`: a checkout of this repository.

To rebuild and remeasure:

- `harness/extract.py <name>=<archive.tar>...` unpacks `git archive`
  tarballs next to itself; the round ran it from `ROUND6_WORK`.
- `harness/build.sh <name> <candidate-dir>` builds a candidate against
  `main210` (`git archive 060b02d2`), reusing the A/A baseline build.
- `harness/build-stack.sh <name> <baseline-dir> <revision> <candidate-dir>`
  builds against another baseline (`main9e7` and `baseaf6` in this round).
- `harness/measure.sh <name> <filter> [pairs]` runs one gated measurement
  with `filter-<filter>.txt`: `screen`, `broad`, `recheck6`, `recheck422` or
  `recheck424`.

The corpus comes from `make_corpus.py --include-scanner-diagnostics`
(SHA-256 `31583750…`) and is not archived. `harness/tags.py` and
`harness/headings.py` count the raw-HTML tags and headings the text cites.

**x86-64.** To reproduce a run:

1. Dispatch it:
   `gh workflow run bench-x86.yml -f baseline=<rev> -f candidate=<rev> -f cases=broad -f pairs=5`.
2. Download its artifacts: `gh run download <run-id> --dir <dir>`.
3. Archive them: `harness/x86/collect.py <dir> <name>`.

`harness/x86/spread.py`, `cases.py` and `compare.py` read the archived runs.

**Generated files and results.**

- `harness/numbers.py` regenerates [NUMBERS.md](NUMBERS.md), and
  `harness/tables.py` regenerates [TABLES.md](TABLES.md) and
  [X86-TABLES.md](X86-TABLES.md).
- `harness/rule.py <run>...` prints the decision-rule view of an Apple
  Silicon run, and `harness/percase.py` compares runs side by side.
- [`results/`](results/) holds every Apple Silicon run in round 5's layout:
  `summary.json`, `grouped.json`, `run.json`, `build.json`, gzipped raw
  windows and the `.summary` line, plus `cases.json` with the case metadata.
- `results/x86/<name>/host-N/` holds each x86 host's `host.txt`, `run.json`
  and gzipped `summary.json`, `grouped.json` and `build.json`, gzipped to keep
  39 host directories small.
- [`profiles/`](profiles/) holds the x86 `perf` reports.
- [`patches/`](patches/) archives the closed candidates and #431's two
  comparison variants as `git format-patch` series.
- [SHA256SUMS](SHA256SUMS) covers every file of the report.
