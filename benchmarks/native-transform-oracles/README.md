# Typography and emoji reference fixtures

This directory records executed behavior for selected reference packages in [#403](https://github.com/sebastian-software/ferromark/issues/403)
and [#405](https://github.com/sebastian-software/ferromark/issues/405) as part
of the shared qualification work in [#400](https://github.com/sebastian-software/ferromark/issues/400).
Each case stores the input, parser MDAST, transformed MDAST, and serialized
Markdown separately. These outputs describe the named reference packages; they
do not define which behavior Ferromark should adopt.

## Provenance and artifact boundaries

Registry metadata was checked on 2026-09-25.

- [remark-smartypants 3.0.3](https://www.npmjs.com/package/remark-smartypants/v/3.0.3)
  is MIT licensed. Its npm gitHead matches the [#403 source revision](https://github.com/silvenon/remark-smartypants/tree/a7a30398af761cafce6b1b2372f7eaf428db3908).
  The lockfile also pins its retext-smartypants 6.2.0 dependency.
- The executed [@mavrin/remark-typograf 2.1.6 registry tarball](https://www.npmjs.com/package/@mavrin/remark-typograf/v/2.1.6)
  is MIT licensed and integrity-pinned. Its npm gitHead is
  685611ff7c305ed14ccc34a694084c35a66f08cb, whose package manifest reports
  version 2.1.5. The [#403 source revision](https://github.com/mavrin/remark-typograf/tree/4a42cbd51ec0b1346b8dbe8bfd16537ae43e581d)
  reports version 2.1.6. The fixture records these as different provenance
  identities. The source lockfile pins typograf 6.11.3, so
  scripts/pnpm-workspace.yaml overrides the registry package's broad range to
  that exact, integrity-pinned version.
- At qualification time, npm had no published typograf 7.9.0 package. The cited
  [source revision](https://github.com/typograf/typograf/tree/a2ae7dc0d9abfc46376fef5175d6739b7bd7ed9b)
  has no built distribution, and its locale table lacks Portuguese. It is not
  executed here. [remark-textr 6.1.0](https://github.com/remarkjs/remark-textr/tree/d80c469b83c9d0d664be1f7ca69d3761becb9def)
  is a wrapper; without a pinned texturizer and configuration it does not
  provide one behavior baseline.
- The executed [remark-gemoji 8.0.0 registry artifact](https://www.npmjs.com/package/remark-gemoji/v/8.0.0)
  and [gemoji 8.1.0 dataset](https://www.npmjs.com/package/gemoji/v/8.1.0)
  are separately pinned. The plugin's [#405 source revision](https://github.com/remarkjs/remark-gemoji/tree/3c2d6b66eda69a22e57234067de71c67fd73e47f)
  differs from its npm gitHead in the README; the transform code matches.
  The dataset's npm gitHead is
  95b8d4669ce0b67ad6cfe09632c0515b9b0e01e2. It differs from the [#405 dataset candidate](https://github.com/wooorm/gemoji/tree/2952469469d6e9215ce0ed391ac4193b840be89b):
  the generated data and tooling differ, and that source generator reads
  mutable upstream API data. These fixtures therefore represent the published
  npm dataset only.
- The executed [remark-emoji 5.0.2 registry artifact](https://www.npmjs.com/package/remark-emoji/v/5.0.2)
  is MIT licensed and pinned separately from its [#405 source revision](https://github.com/rhysd/remark-emoji/tree/16fb3927cb7241ab9fecc7758a80be9bce8a7edd).
  Its published transform implementation is unchanged from that source
  revision. Its node-emoji, emojilib, skin-tone, and emoticon data dependencies
  are all locked to exact MIT releases with recorded integrity values.

The complete package versions, source revisions, integrity hashes, and resolved
data dependencies are repeated in fixtures.json and checked against the
scripts lockfile. The local Typograf override is required for a reproducible
comparison with the dependency version cited in #403.

## Cases

Typography observations execute SmartyPants and Typograf across their configured
English, Spanish, French, German, Italian, Dutch, Polish, Russian, and Ukrainian
locales. A missing Portuguese Typograf oracle is recorded above rather than
represented as a Typograf result.

Emoji observations compare the two released plugin/data chains with known
shortcodes, aliases, unknown and differently cased names, overlap, code and
link content, and bare URL text with and without GFM parser autolinks. Cases
whose result depends on unresolved protected-content or provenance rules are
observations only. The native implementation policy selected from these
observations is recorded in
[`docs/decisions/2026-09-26-native-github-and-emoji-passes.md`](../../docs/decisions/2026-09-26-native-github-and-emoji-passes.md).
Issue #400 remains the shared qualification and acceptance tracker; this data
does not by itself close it.

## Checks and regeneration

Ordinary tests read the checked-in fixtures only; they use the locked npm
artifacts installed by CI and do not access the network or rewrite output.
The scripts CI job runs these through node --test scripts/test-*.mjs.

To deliberately regenerate outputs from the pinned references, run from the
repository root:

    cd scripts
    pnpm reference:transforms:fixtures

The command requires an explicit --write flag. Review the fixture diff after
regeneration; a reference result does not approve a native Ferromark rule.
