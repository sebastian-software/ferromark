# Ferromark homepage

This private workspace contains the Ardo 6 site for Ferromark.
See [CONTRIBUTING.md](../CONTRIBUTING.md) for repository checks and the boundaries
between the native engine, Node.js bindings, and website.

## Monografie homepage and guides

The site consumes `ferramenta-family` from a pinned Git commit. Every page uses
the Monografie page system (`ferramenta-family/monografie`, Ferramenta
ADR-0012). The homepage takes `monografie.css`: the steel header, the
instrument as the opening plate, one reading column with margin notes, facing
sheets, cards, the comparison chart and the closing. The guides add the
documentation pages (`monografie-docs.css`) and the Ardo adapter
(`monografie-ardo.css`), which also carries the shell fixes: no padding for a
fixed header, a document that scrolls as a whole, anchors that clear the sticky
header and one menu button on a phone.

`app/root.tsx` passes `MonoHeader` through Ardo's header slot on every page and
renders `MonoFooter` outside Ardo's main landmark. On the guides the header also
carries Ardo's mobile documentation menu and search; `MonoReadingLayout`
combines the section navigation, the bounded reading surface and Ardo's live
outline. The build fails when a page renders a class of the Ferra theme or the
original kit, or when a source imports their stylesheets. The release tag in the header and the toolchain minimums come from the
build-time snapshot in `app/data/release.json`.

Ferromark's own specimens live in `app/components/`: the rendered sheets with
line linking (`rendered-sheets.tsx`), the engine drawing, the typography loupe,
the speed chart and agreement table (`homepage-evidence.tsx`) and the cards.
Their one stylesheet, `homepage-specimens.css`, uses the package's `--mono-*`
tokens, and the build fails on any selector there that is not scoped to an
`fm-` class. Shared patterns belong in the family package and its kit.

`app/components/document-heading.tsx` preserves published Ardo 4 heading URLs
as aliases of Ardo 6's slugs. This keeps compatibility markup out of the
generated benchmark guide, which remains owned by its publisher.

`MonoBenchmarkTable` supplies the captioned, keyboard-scrollable table and
shared measurement styles. `app/components/benchmark-comparison.tsx` retains
all platform grouping, source links, missing-value handling, conformance
qualifiers, and calculation rules. The seven benchmark fixture tests cover
those rules independently of presentation.

The build snapshots the canonical release version from
`../node/ferromark/package.json` (and the minimum Rust and Node.js versions from
`../Cargo.toml` and that manifest) into `app/data/release.json` before dev,
typecheck, and build. Browser code imports that snapshot rather than reaching
outside Ardo's project boundary. The build verifier checks the rendered
version against the canonical manifest.

## Homepage data

Every sample on the homepage is real output of the published packages.
`pnpm samples` (`scripts/render-homepage-samples.mjs`) installs the released
`ferromark` (the version in `node/ferromark/package.json`) and the pinned
`@ferriki/core` and `@ferrigura/core` into a temporary directory, renders each
sample and writes `app/data/homepage-samples.json` with the versions in
`renderedWith`. Never edit that file by hand; run the script again after a
release. The build fails when its Ferromark version differs from the release.

The chart, its table and the agreement table read the repository's own
benchmark and conformance data (`benchmark-platform-values.json`,
`benchmark-projects.json`, `comparison-conformance.json`) through
`app/components/homepage-data.ts`, which fails the build when a new campaign no
longer supports a claim the copy makes.

## Development and checks

Run these commands from `homepage/`:

```sh
pnpm install --frozen-lockfile
pnpm dev
```

Before shipping website changes, run:

```sh
pnpm format:check
pnpm lint
pnpm typecheck
pnpm run audit
pnpm build
```

Use `pnpm format` when formatting is needed. The build prerenders the site,
verifies routes, published section anchors, search records, navigation, and
benchmark content, and runs the benchmark platform tests. Preserve generated benchmark data and upstream attribution;
follow the repository's benchmark workflow when measured content needs updating.
