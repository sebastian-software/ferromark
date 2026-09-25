# `remark-github` reference fixtures

This directory records the executed reference behavior for the narrow
`remark-github` qualification in [issue #400](https://github.com/sebastian-software/ferromark/issues/400)
and [issue #404](https://github.com/sebastian-software/ferromark/issues/404).
It does not implement native reference linking.

## Provenance

- Package: [`remark-github@12.0.0`](https://www.npmjs.com/package/remark-github/v/12.0.0), MIT.
- Source: [`9cd4e9d8fa6cd3520d11aa922462a0089e1204c5`](https://github.com/remarkjs/remark-github/tree/9cd4e9d8fa6cd3520d11aa922462a0089e1204c5), commit subject `12.0.0`.
- Registry tarball: `https://registry.npmjs.org/remark-github/-/remark-github-12.0.0.tgz`.
- Registry integrity: `sha512-ByefQKFN184LeiGRCabfl7zUJsdlMYWEhiLX1gpmQ11yFg6xSuOTW7LVCv0oc1x+YvUMJW23NU36sJX2RWGgvg==`.
- The registry `gitHead` matches the source revision above. The checked-in
  `scripts/pnpm-lock.yaml` pins this tarball and the complete harness dependency
  graph; `fixtures.json` repeats the identifying metadata for the corpus.
- Harness: Node.js 22.13 or newer, `remark@15.0.1`, `remark-gfm@4.0.0`, and
  `remark-github@12.0.0`, all pinned in `scripts/package.json` and its lockfile.

By default, the runner parses with `remark` plus `remark-gfm`, then applies
`remark-github@12.0.0` with the explicit repository `ferromark/fixtures`. A
case may override the parser plugins. The runner records normalized MDAST
before and after the transform and the serialized Markdown. Source positions
are omitted so the fixture compares logical structure; parser differences,
plugin changes, and serializer changes remain separately visible in test
failures.

The cases execute local `#` and `GH-` references, user/team mentions,
cross-repository issue and commit references, commit hashes and ranges, and
boundaries around existing links, code, bare URLs, email addresses, and
punctuation. One case disables `remark-gfm` so the raw parser text for a bare
URL and email is recorded before the reference transform; another keeps GFM
enabled and records its link nodes. This keeps parser behavior distinct from
reference behavior. Upstream repository inference and custom URL callbacks are
not enabled; they are outside the proposed Ferromark scope.

## Checks and regeneration

Ordinary tests use only the checked-in inputs and expected outputs; they do not
access the network or rewrite fixtures. CI runs them through
`node --test scripts/test-*.mjs` after installing the frozen `scripts/`
workspace.

To deliberately regenerate the reference outputs after reviewing a change to
the pinned oracle or fixture inputs, run from the repository root:

```sh
cd scripts
pnpm reference:remark-github:fixtures
```

The command requires an explicit `--write` in its implementation. Review the
diff in `fixtures.json`; a regenerated output is evidence from the pinned
reference, not an automatic approval of the resulting Ferromark behavior.
