# Interpretation of the broad Markdown comparison

The initial v2 core is faster on most of this corpus. With full renderer and
output reuse, v2 leads on 35 of the 36 equivalent-output inputs; main leads on
the 310-byte table comment. Every classified lead exceeds 5% in all nine paired
windows. Fresh calls give a different split: 30 v2 leads and six main leads.
This is a local corpus result, not a claim about all Markdown or all platforms.

## What the broader inputs change

The previous synthetic diagnostic mix gave heavy influence to repeated nested
lists, reference links, and tables. This corpus instead contains 12 authored
comment shapes, 28 unchanged repository documents, 16 views of four Wikipedia
articles, and one minimized syntax diagnostic. Input sizes range from 37 to
113,609 bytes. Article excerpts overlap; there are four encyclopedia sources,
not 16 independent articles. The prose-only views retain real article wording
but deliberately remove link destinations and emphasis and flatten headings.

Equal-document geometric means, restricted to equivalent HTML:

| Content | Comparable / selected | Fresh main/v2 | Full reuse main/v2 |
| --- | ---: | ---: | ---: |
| Comment shapes | 12 / 12 | 1.006× | 1.259× |
| Encyclopedia with Markdown links | 9 / 12 | 1.773× | 1.618× |
| Full prose adaptations | 4 / 4 | 1.349× | 1.188× |
| Technical documentation | 9 / 22 | 1.453× | 1.341× |

Values above 1 favor v2. Readme and reference categories have only one admitted
case each and should not be generalized; the complete report shows their
coverage and individual measurements. Size bins are also uneven: there is no
claim about megabyte inputs, and only two equivalent-output cases exceed 50 KiB.

## The important small-input exception

The 282-byte plain review comment takes **0.408 µs on main versus 0.492 µs on
v2** with a fresh call. With full reuse the same input takes **0.238 versus
0.164 µs**, reversing the ranking. Very short, simple prose can favor main's
fresh API even when v2 wins after state reuse. These are warm-process rendering
times; process startup, I/O, and bindings are excluded.

The 310-byte table comment favors main in all three modes. With full reuse it
takes **0.855 versus 1.079 µs**, approximately a **1.26× main speed advantage**.
That is a concrete remaining counterexample to an across-the-board v2 lead.

## Reconnecting to the earlier Ferromark repository result

The exact older collection's 12 documents are retained, totaling 53,656 bytes.
For the entire rotating collection, full reuse takes **142.59 µs on main versus
96.65 µs on v2**; the paired median is **1.475× in v2's favor**. Fresh calls give
a paired median of **1.647×**. This broadly supports the earlier observation
that OX-derived rendering was faster on real documentation.

These collection numbers are **diagnostic**, because eight documents have
different heading IDs. Both engines still compute IDs; the rendered document
content otherwise matches under the stated serialization rules. The earlier
workflow report used a different compiler, OX revision, extension profile, and
lifecycle contract, so its absolute timings are not a regression baseline for
this run. Here both engines use the same frozen binaries as the first v2
comparison and GFM for the documentation collection.

## Compatibility findings retained alongside timings

There are 17 byte-identical outputs, 19 serialization-equivalent outputs, 14
heading-ID-only differences, and seven other differences. No heading IDs, text,
code classes, or changed link targets are erased to obtain admission.

- Fourteen cases differ only in heading anchors. They remain outside the
  equivalent-output aggregate even though they are useful performance screens.
- Two Vue documents differ in fenced-code language classes: main retains suffixes
  such as `{2}`, whereas v2 produces the base language class.
- The Vite features document also differs in whitespace and link handling.
- Three linked encyclopedia bodies expose extra text/emphasis in main around
  link destinations containing underscores (for example `H.J.R._Murray`). Chess
  additionally has heading-ID differences. All original HTML and token diffs
  remain archived; these are observations, not parser fixes in this change.
- The 41-byte angle-link diagnostic demonstrates main's duplicate-link output
  for `[The guide](<https://example.org/guide>)`. It was found while validating
  the converter before timing. Article conversion uses ordinary parenthesized
  destinations, and all selected topics/excerpts remain present.

Literal Unicode versus UTF-8 percent encoding of the same HTTP(S) URL path,
query, or fragment is treated as serialization spelling. Existing escapes,
ASCII reserved characters, and hosts are not rewritten. This narrowly scoped
addition to the previous comparator is documented and tested.

## What to investigate next

The clearest optimization screens are fresh-call overhead on short simple
documents and table rendering. Profile them before selecting a donor algorithm
or SIMD port. Keep linked prose, plain prose, and ordinary documentation as
regression screens so gains on one synthetic case do not hide losses elsewhere.
Resolve or explicitly specify heading-anchor and link semantics separately from
performance work.

## Measurement limits and validation

Three process rounds × three alternating pairs × 189 document/lifecycle groups
produce 1,701 paired windows, each at least 75 ms after warmup. Repeated rotating
outputs, per-engine lifecycle equality, timed output-length checksums, and
before/after HTML checks passed. Ratios are medians of paired ratios, so a ratio
can differ slightly from the quotient of the two displayed median times.

An active browser and backup processes were observed before timing. The desktop
was not isolated or CPU-pinned. Most paired results are tightly grouped, but
the legacy owned-output batch has an outlier up to 2.64× and the fresh Volcano
prose round medians range from about 1.36× to 1.58×. The complete ranges and raw
samples remain available; these do not support portable single-digit claims.

The unchanged core passes 626 Rust tests, formatting, and clippy. Sixteen Python
harness tests pass. Frozen source exports were independently byte-checked against
their original archives. Both measured binary hashes and original lockfile
checksums were verified. No library source, upstream repository, remote, or
publication was changed.

[Complete results and raw data](README.md) ·
[Methodology and corpus regeneration](../../../benchmarks/broad-comparison/README.md) ·
[Background-process observation](preflight.json) ·
[Source verification](source-validation.json)
