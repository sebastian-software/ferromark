# The first x86-64 profile

Until #423, every profile of the optimization rounds had come from Apple
Silicon, where most scanners have a NEON path. The x86-64 profile workflow
(`.github/workflows/profile-x86.yml`) took its first profile in #423's own
check run, of `060b02d2` (release 2.1.0), on two GitHub-hosted runners:

- host 1: AMD EPYC 9V74;
- host 2: Intel Xeon Platinum 8573C.

**The driver** is the profiling driver in
`benchmarks/optimization-rounds/profile`, the cleaned-up version of round 5's
[`harness/profile`](../2026-09-23-perf-round-5/harness/profile/). It is built
with fat LTO, one codegen unit and line tables, and with `-C target-cpu=generic
-C force-frame-pointers=yes`, the baseline the published addons target. It
loops one stage over the 57 broad documents for 20 s, each document with the
options of its profile in the paired worker, and every document contributes
about the same byte volume per sweep.

**Sampling:** `perf record -F 999 -g`, about 19,000 samples per stage and host.

The reports are in [`profiles/x86-060b02d2/`](profiles/x86-060b02d2/): self
time (`self-*.txt`), inclusive time (`inclusive-*.txt`) and caller graphs
(`callers-*.txt`) for parse and render, and each host's `host.txt`. **These
are profile shares, not measured speedups**. The measurements are in
[NUMBERS.md](NUMBERS.md).

## Render

| Symbol | Self, host 1 | Self, host 2 | Inclusive, host 1 | Inclusive, host 2 | Apple M1, inclusive |
| --- | ---: | ---: | ---: | ---: | ---: |
| `escape::escape_into` | 20.3% | 22.8% | 36.1% | 41.8% | 36.0% |
| `escape::nibble::first_flagged_ssse3` | 19.2% | 23.4% | 19.2% | 23.4% | — |
| `HtmlRenderer::render_node` | 17.1% | 14.9% | 92.6% | 94.0% | 94.4% |
| `write::HtmlRenderer::write_heading_id` | 0.5% | 0.6% | 11.5% | 11.3% | 11.5% |
| `heading::slugify_heading_into` | 5.2% | 5.2% | 5.2% | 5.2% | 5.7% |
| `heading::HeadingIdPlanner::plan_into` | 0.5% | 0.5% | 5.4% | 5.0% | 4.4% |
| `hashbrown` `HashMap::insert` | 3.4% | 2.9% | 3.4% | 2.9% | 1.1% |
| `escape::push_run_long` | 4.4% | 4.1% | 4.4% | 4.1% | 8.5% |
| `escape::url::write_url_segment` | 2.3% | 2.3% | 5.4% | 5.2% | 4.9% |
| `write::HtmlRenderer::write_html_value` | 3.0% | 2.8% | 3.9% | 3.5% | 6.5% |
| `blocks::HtmlRenderer::visit_table_row_with_header` | 2.5% | 2.5% | 5.8% | 6.0% | 6.1% |

The Apple M1 column is round 5's profile of the same revision
([its PROFILES.md](../2026-09-23-perf-round-5/PROFILES.md#render-7771-samples-at-cb352020-7807-at-060b02d2)).

- **Escaping is 40–46% of render self time on x86**, and
  `first_flagged_ssse3` alone is 19–23%. On x86 the `pshufb` nibble classifier
  needs SSSE3, which the x86-64 baseline lacks. So it lived in a
  `#[target_feature(enable = "ssse3")]` function behind
  `is_x86_feature_detected!`, and a baseline build cannot inline it. The
  caller graphs put 15–18% of render under the call from `escape_into` and
  about 3% under the one from `write_url_segment`. Every call paid for the detection
  check, the call, the table setup and the spilled registers, and strings
  under 16 bytes paid it only to be told to use the word scan. On aarch64 the
  NEON classifier compiles into the escape loop. **#430** replaced the SSSE3
  function with SSE2 compares that inline, and x86 render gained 9%.
- **Heading ids are about 11% of render on both architectures.** On x86, the
  id map's `HashMap::insert` (2.9–3.4%, all under `plan_into` in the caller
  graph) weighs more than on the M1. **#422** replaced that map with reusable
  claim storage. It was measured on Apple Silicon, not on x86.

## Parse

| Symbol | Self, host 1 | Self, host 2 | Inclusive, host 1 | Inclusive, host 2 |
| --- | ---: | ---: | ---: | ---: |
| `inline::Parser::parse_inline` | 12.2% | 12.8% | 41.6% | 41.6% |
| `Parser::parse_document` | 7.6% | 8.4% | 89.8% | 90.1% |
| `inline::scan::x86::next_special_avx2_with_tables` | 6.4% | 7.6% | 6.4% | 7.6% |
| `block::Parser::parse_paragraph` | 6.1% | 4.9% | 51.3% | 50.6% |
| `inline::Parser::parse_inline_block` | 4.8% | 4.6% | 52.1% | 52.8% |
| `inline::link_target::parse_destination` | 4.7% | 4.4% | 4.7% | 4.4% |
| `memchr` `packedpair::Finder::find_impl` (`memmem`) | 4.2% | 4.7% | 4.3% | 4.7% |
| `inline_link::Parser::parse_link` | 4.0% | 3.4% | 18.8% | 18.3% |
| `inline::marker_scan::InlineMarkerScan::next` | 3.3% | 3.2% | 9.4% | 10.0% |
| `list::Parser::parse_list` | 2.9% | 2.5% | 10.5% | 10.7% |
| `table::Parser::parse_table` | 0.6% | 0.4% | 10.5% | 10.3% |
| `delimiters::Parser::scan_balanced_matched` | 2.0% | 2.1% | 2.2% | 2.2% |
| `Parser::with_phase` | 1.9% | 1.6% | 5.9% | 5.9% |
| `cursor::Parser::line_at` | 1.3% | 1.3% | 1.3% | 1.3% |
| `cursor::Parser::probe_line_inner` | 0.9% | 0.8% | 2.2% | 2.1% |

- **`parse_destination`, 4.4–4.7%, all self time.** The `ByteClass` scan of a
  link destination is inlined into it. On x86 that scan was the scalar
  flag-table walk; on aarch64 it is NEON and much smaller in the profile.
  Bracket bodies (`scan_balanced_matched`, about 2%) use the same classes.
  **#426** added SSSE3 and AVX2 scans, and x86 parse gained 6–7%.
- **`Parser::with_phase`, 5.9% inclusive.** On x86 the root still ran the NUL
  `memchr`, the `[` probe and the `]:` `memmem`; the fused scan of #413 is
  aarch64-only. **#428** ported the fused scan and was closed: `memchr` on x86
  is AVX2, so the separate passes were already cheap
  ([REJECTED.md](REJECTED.md#428-the-fused-root-scan-on-x86-64)).
- **Line walking.** `line_at` takes 1.3% and `probe_line_inner` about 1% self
  time. The SWAR line-end loop is also inlined into the self time of
  `parse_paragraph` and `parse_list`, so the profile understates it. **#431**
  gave `line_end` an SSE2 probe and scan, and x86 parse gained 6%, most on
  long-line prose.
- **The autolink pre-flight shows here too:** `parse_inline_block` self time
  (4.6–4.8%) and part of the `memmem` finder, as on the M1. #427's third
  design for it was closed.
- **The inline marker scan** already had an x86 path
  (`next_special_avx2_with_tables`, 6.4–7.6%), like the escaper. It is the
  only parse scanner that did before this round.

## Not repeated after the round

No x86 profile was taken after #426, #430 and #431, so the post-round ranking
on x86 is not known. The cumulative measurement, 2.1.1 against the code of
2.1.2, is in the [README](README.md#cumulative-211-against-the-code-of-212).
