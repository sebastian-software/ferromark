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

The list-item reference-definition mismatch is deliberately pinned as an
existing limitation. With the current sources the comment-enabled run is
`39/40` matching, with `list-definition-title` as the one known difference.
The same list-reference case also differs when comments are disabled and the
comment line is physically removed, proving it is outside the line-comment
feature.

Run from the repository root:

```sh
python3 benchmarks/line-comments-oracle/run.py \
  --v1-source /private/tmp/ferromark-v2-native-comparison/build-01/sources/ferromark_v1 \
  --output benchmarks/line-comments-oracle/results.json
```

The runner creates an ephemeral Cargo project with path dependencies, uses the
repository's current v2 facade, and leaves no lockfile or source copy in the
repository. Add `--no-offline` when the local Cargo cache is incomplete. The
results JSON contains exact mismatching source names and rendered HTML.
