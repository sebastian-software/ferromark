# Line comment differential oracle

This small oracle compares the pinned Ferromark v1 renderer with the current
v2 facade on 40 source-only `//` comment cases. It uses explicit profiles:

The recorded v1 source is 0.9.0 at
`143ec2ce151d87d2a3d804a048014afc97733ae0`, the same archive as the
[native comparison](../../docs/reports/2026-09-14-native-engines/PROVENANCE.md).
Supply that snapshot via `--v1-source`; the runner accepts a source directory
and does not verify its Git revision. The recorded worker checksum identifies
the test inputs and profiles.

- v1: `Options::commonmark()` plus trusted HTML, tables, task lists,
  strikethrough, autolink literals, heading IDs off, and `line_comments=true`.
- v2: `ParserOptions::gfm_spec()` plus `line_comments=true`; the GFM HTML
  renderer has heading IDs, the tag filter, and its extra URL scanner disabled
  to match v1's trusted HTML profile.

The archived `results.json` records the original list-item definition limitation
(39/40 exact matches, one semantic mismatch). It remains unchanged as historical
evidence. The corrected parser resolves the list definition globally. The
current runner requires all 40 cases and both baseline probes to agree under the
existing conservative HTML comparator; it preserves exact-output differences.
Only insignificant serialization differences are admitted, not heading IDs or
changed link targets. See the [correction report](../../docs/reports/2026-09-15-container-references/README.md).

Run from the repository root:

```sh
python3 benchmarks/line-comments-oracle/run.py \
  --v1-source /private/tmp/ferromark-v2-native-comparison/build-01/sources/ferromark_v1 \
  --output /tmp/ferromark-line-comments-current.json
```

The runner creates an ephemeral Cargo project with path dependencies, uses the
repository's current v2 facade, and leaves no lockfile or source copy in the
repository. Add `--no-offline` when the local Cargo cache is incomplete. The
results JSON contains exact mismatching source names and rendered HTML.
