# Ferromark homepage

This private workspace contains the Ardo 6 site for Ferromark.
See [CONTRIBUTING.md](../CONTRIBUTING.md) for repository checks and the boundaries
between the native engine, Node.js bindings, and website.

## Shared Ferra system

The site consumes `ferramenta-family/ferra` and `ferramenta-family/ferra.css`
from a pinned Git commit. The family package owns the warm light scheme,
IBM Plex Sans display type, Blender illustrations, soft cards, site chrome,
documentation layout, and benchmark table presentation. Shared patterns belong
in that package and its kit; project-specific content stays here. The site has
no local CSS. It imports `ferra.css` and the shared optional `ferra-ardo.css`
adapter, including the homepage, docs, tables, and interactive examples.

`app/root.tsx` passes `FerraHeader` through Ardo's header slot and renders
`FerraFooter` outside Ardo's main landmark. `FerraReadingLayout` combines the
section navigation, bounded reading surface, and Ardo's live outline. A controlled mobile menu
keeps every guide reachable on narrow screens. Ardo owns routing, search,
code blocks, and prerendering; the homepage opts into its bare layout.

`app/components/document-heading.tsx` preserves published Ardo 4 heading URLs
as aliases of Ardo 6's slugs. This keeps compatibility markup out of the
generated benchmark guide, which remains owned by its publisher.

`FerraBenchmarkTable` supplies the captioned, keyboard-scrollable table and
shared measurement styles. `app/components/benchmark-comparison.tsx` retains
all platform grouping, source links, missing-value handling, conformance
qualifiers, and calculation rules. The seven benchmark fixture tests cover
those rules independently of presentation.

The build snapshots the canonical release version from
`../node/ferromark/package.json` into `app/data/release.json` before dev,
typecheck, and build. Browser code imports that snapshot rather than reaching
outside Ardo's project boundary. The build verifier checks the rendered
version against the canonical manifest.

## FFM example

`app/components/ffm-showcase.tsx` shows one fixed Markdown source beside its
rendered result. The source and HTML live in `app/data/ffm-sample.json`, which
records the Ferromark version, source commit and options used
(`ParserOptions::ffm()` with `HtmlRendererOptions::new()`). The preview is that
frozen output; it accepts no visitor input or remote HTML.

When changing the sample, render it again with the native engine, update the
recorded version and commit, and keep the syntax notes below the example in
agreement with what the sample demonstrates.

`app/assets/rocket-stages.webp` is a photorealistic transparent cutout generated
with the built-in image tool, using the package's Ferromark icon as the material
and lighting reference. Its exact generation and edit prompts and provenance live in
`app/assets/rocket-stages.webp.json`. Preserve the native alpha and the family's
forged steel, worn edges, and orange-hot accent when replacing it.

Afterburner, the optional processing stage before rendering, is linked from the
FFM section; its Rust and Node.js guides document opt-in passes in order,
protected syntax, an explicit GitHub repository, and the bundled emoji
dictionary. Benchmark and conformance sections retain their measured evidence.

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
