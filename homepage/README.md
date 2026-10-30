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

## Transform showcase

`app/components/transform-showcase.tsx` presents four rocket stages: CommonMark,
GFM, FFM, and Afterburner, the optional processing stage before rendering. The
adjacent list explains and links to each capability. Keep the illustration
compact and the fins unmarked; it does not define a new family component. The
rocket labels read CommonMark, GitHub (GFM), Ferro (FFM), and Afterburner, with
bright metal letter faces for readability and a static 10-degree clockwise tilt.

`app/assets/rocket-stages.webp` is a photorealistic transparent cutout generated
with the built-in image tool, using the package's Ferromark icon as the material
and lighting reference. Its exact generation and edit prompts and provenance live in
`app/assets/rocket-stages.webp.json`. Preserve the native alpha and the family's
forged steel, worn edges, and orange-hot accent when replacing it. The shipping
asset is resized to 864 pixels high and encoded as WebP quality 88 with lossless
alpha; its displayed height is 288 pixels on desktop and 240 pixels on mobile.

The separate "Afterburner in action" demo groups typography, GitHub-reference,
and emoji controls within that fourth capability. These controls change the
example output; the four product stages stay assembled. Afterburner names the
processing stage, while public Rust and Node.js API names remain unchanged.

All three controls start enabled. A three-bit key selects one of eight frozen
outputs from `app/data/transform-samples.json`: typography first, GitHub
references second, emoji third. The sample records Ferromark 3.0.0 and source
commit `2b00ce0f`; its source and outputs were rendered with the real `ferromark`
and `ferromark-transforms` crates. Browser controls only select those outputs.
The preview accepts no visitor input or remote HTML.

When changing the sample, regenerate every combination with the native engine,
retain the version and source commit, and compare all eight outputs before
updating the data. Keep button pressed states and preview output in agreement.
The controls expose `aria-pressed` and a preview relationship; the
result announces changes politely. The rocket has descriptive alternative text
and no motion; its capability names and explanations also appear as live text.

The Rust and Node.js guides document opt-in passes in order, protected syntax,
an explicit GitHub repository, and the bundled emoji dictionary. Built-in passes
are available in both runtimes; arbitrary custom passes use Rust's API. The
Remark comparison describes a processing shape, without claiming plugin
compatibility. The FFM section presents publishing benefits and links to its
guide; benchmark and conformance sections retain their measured evidence.

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
