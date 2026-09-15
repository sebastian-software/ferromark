# Real-document regression diagnosis

## Finding

The complete six-engine comparison exposes two material real-document regressions
against the preceding v2 core. Aggregate competitive position remains strong, but
does not justify clearing those individual regressions for release. No production
optimization was retained as part of this measurement commit.

## Ranked hypotheses and predictions

1. Structural reference discovery does excessive work: removing only definition
   markers should sharply reduce the old/new timing gap.
2. General parser or layout overhead: comparable no-definition controls should
   show a similarly large regression.
3. Between-run machine effects: directly alternating the existing old/new binaries
   on the same host should largely remove the gap.

## Paired reproduction

Both existing native binaries use the same worker, lock, compiler and allocator.
The before binary contains v2 `e93394e`; the after binary contains `c232d97`.
Each row uses 60 ms warmup per binary, then twelve alternating 40 ms windows per
binary. Each variant must produce identical HTML across versions, before and after
timing. Output-length checksums are checked in every window. The marker variant
replaces `]:` with `]=`; it deliberately changes the document, so its HTML is not
compared with the original document. The control has no such markers.

Positive change means the new v2 takes longer than the earlier v2. These are
direct paired time ratios, unlike competitive position ratios from separate runs.

| Document | Variant | Lifecycle | Paired time change |
| --- | --- | --- | ---: |
| comment-incident | original | fresh | +37.41% |
| comment-incident | original | reuse | +42.16% |
| comment-incident | no-definitions | fresh | +2.34% |
| comment-incident | no-definitions | reuse | +2.83% |
| rust-book-ch00-00-introduction | original | fresh | +22.15% |
| rust-book-ch00-00-introduction | original | reuse | +23.72% |
| rust-book-ch00-00-introduction | no-definitions | fresh | -0.19% |
| rust-book-ch00-00-introduction | no-definitions | reuse | +0.61% |
| comment-review | original | fresh | +2.07% |
| comment-review | original | reuse | +4.80% |
| comment-review | no-definitions | fresh | +2.56% |
| comment-review | no-definitions | reuse | +5.10% |

The first, independently repeated probe is preserved in `initial-probe.json`.
It retained paired ratios only; the archived rerun in `diagnosis.json` additionally
retains individual windows, checksums, full HTML, host details and binary hashes.
Both probes reproduce the large gaps and their reduction without definition markers.

## Interpretation and next optimization boundary

The strongest explanation is the structural definition prepass introduced across
the correctness/refactoring sequence, rather than the latest footnote patch alone.
Footnotes are off in these comparison lanes. With no definition candidates, the
parser avoids this prepass; removing the candidate markers removes most of each
outlier. Small remaining costs and the control regression are not explained by
excess structural discovery and should not be claimed fixed.

- `comment-incident` contains one apparent definition, entirely inside a fenced
  Markdown example. The candidate filter nevertheless triggers a structural pass
  across the document, which correctly rejects the fenced definition.
- `rust-book-ch00-00-introduction` contains three genuine root definitions in an
  approximately 10 KiB document. Sparse definitions trigger block parsing across
  otherwise ordinary prose before the document is parsed again.

Optimize unnecessary collection work while retaining the real block grammar as
the authority on containers and code boundaries. In particular, the old flat
scanner incorrectly let an unclosed quoted fence hide a later root definition.
Any candidate optimization must preserve that correction, list/quote precedence,
MDX and footnote scope, exact HTML/AST checks, and the full real-document corpus.
The paired probe is a diagnosis, not a confidence interval or an attribution of
every percent to one commit. No performance threshold is weakened.

## Reproduce

Rebuild each report using its frozen provenance instructions, then run:

```sh
python3 docs/reports/2026-09-15-release-native/diagnose.py \
  /path/to/previous/native-comparison-worker \
  /path/to/current/native-comparison-worker /tmp/native-diagnosis-new.json
```

Use a new output path. The script refuses to overwrite earlier measurements.
All real inputs come from the archived corpus and retain its attribution.
