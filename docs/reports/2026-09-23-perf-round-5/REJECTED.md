# Rejected: the autolink pre-flight inside the inline marker scan (#415)

Branch `perf/autolink-preflight-marker-scan` (`50f3f182`), pull request #415,
closed without merging after the paired measurement. The patch is archived as
[`patches/autolink.patch`](patches/autolink.patch); the branch stays on the
remote for reference.

## The idea

On the `gfm` documents, `gfm_autolink::may_contain_autolink` takes about 7–9%
of parse time in the [first profile](PROFILES.md#what-the-first-profile-pointed-at).
It runs once per block, after `parse_inline` has walked the same content with
the inline marker scan: a `memchr2` for `@` and `:`, and, when the block holds
no `@`, a `memmem` for `www.`. A throwaway prototype that answered the same
question in one standalone NEON pass (`@`, `:` and a `w.` bigram) kept the
output identical and gained only about 1% in parsing (quick A/B, about 1.009,
not archived), so the remaining cost looked like the second pass itself.

#415 removed that pass by collecting the pre-flight's facts during the marker
walk that already reads the text:

- With autolinks on, the block-level `InlineMarkerScan` uses tracking tables
  in which `@` joins the backtick bit and `:` the `<` bit, so the classifier
  stays one shuffle pair; a `w.` pair test with `vextq_u8` and a carried
  previous vector finds the end of `www.`, five extra vector operations per
  16 bytes.
- A watermark records how far every trigger byte has been visited. Bytes the
  marker scan never classifies (code spans, link text and destinations,
  autolinks, raw HTML, math, entities) are filled by a side scan before the
  next marker scan runs, and the rest of the block at its end.
- The gate value itself was shown to be unchanged: an in-parser assertion in
  test builds, a differential parse and render over the spec examples under
  18 option profiles, two frozen corpora and 2,500 generated documents, and
  four deliberate mutations that the suites caught.
- Other targets keep the separate `memchr` pre-flight.

## The measurement

Against `cb352020`, same harness and settings as the other candidates, output
verified identical for every case and stage:

| Run | fresh | reuse | parse | render |
| --- | ---: | ---: | ---: | ---: |
| Screen, 36 cases, 3 × 3 | 0.991 | 0.993 | 0.988 | 1.001 |
| Broad, 57 documents, 3 × 5 | 0.980 | 0.979 | **0.968** | 1.002 |

- **Every broad round lost**: parse 0.967 / 0.966 / 0.968, fresh 0.978 /
  0.982 / 0.983.
- **30 of 57 documents parse below 0.970**, down to
  `legacy-docs-migration-0-4` 0.905, `legacy-docs-migration-0-8` 0.908,
  `legacy-docs-readme-theme` 0.914 and `vue-docs-suspense` 0.920. By
  category, the technical documentation parses at 0.941 and the READMEs at
  0.940 (geomeans).
- **The `commonmark` documents lose too** (parse 0.975 over 17 documents,
  `wiki-tea-article-body` 0.944, `wiki-chess-article-body` 0.945), although
  they run without autolinks and take the untracked path. The extra code in
  `InlineMarkerScan::next` (a `match` on the tracking state per memo miss)
  costs every profile, a risk the pull request named in advance.
- **Only the small comments gain**: the comments category parses at 1.012,
  `comment-reproduction` 1.113, `comment-table` 1.065.
- In the screen, the plain and sparse table diagnostics parse at 1.04–1.10
  and the formatted ones, whose cells are dense with inline markers, at
  0.87–0.90.

Per-case numbers: [TABLES.md](TABLES.md#autolink-415-pre-flight-inside-the-marker-scan-screen)
and `results/autolink-screen`, `results/autolink-broad`.

## Why it loses

The five extra vector operations per 16 bytes in the hottest loop of the
parser, the byte check at every marker stop, and the side scans over
construct bytes cost more than the separate pass they replace. The separate
pass is cheap per byte (`memchr2` and `memmem` run at full vector width with
nothing else in the loop), so moving its work into a loop that already does
more per byte adds instructions where they are most expensive.

That repeats what the earlier rounds found for fused scans:

- [Round 2](../2026-09-15-arm-round-2/README.md) recorded every inline marker
  during the block-level line scan (E1) and measured parse 0.949: the two
  vector scans shrank as designed, but per-marker scalar recording put more
  back.
- [Round 3](../2026-09-16-arm-round-3/README.md) gated the per-block
  pre-flight with one document-level scan for `@`, `://` and `www.` (J); it
  gained only on short needle-free comments and lost 2–3% on documents whose
  first needle sits late.
- [Round 4](../2026-09-21-perf-round-4/REJECTED.md) declined to derive
  delimiter runs from the marker mask for the same reason, before writing a
  patch.

The pre-flight stays the largest single parse item after the round (about
10% of parse samples at `060b02d2`, [PROFILES.md](PROFILES.md#after-the-round-at-060b02d2)).
What is left to try is making the separate pass cheaper per block rather than
removing it; the one variant of that route that was tried, the standalone
NEON pass, gained about 1%. The item stays open in the
[roadmap](../../optimization-roadmap.md) with the failed fusion noted.
