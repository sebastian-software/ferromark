# Ferromark homepage

This private workspace contains the React Router and Ardo site for Ferromark.
See [CONTRIBUTING.md](../CONTRIBUTING.md) for repository checks and the boundaries
between the native engine, Node.js bindings, and website.

## Inherited family system

The visual authority is the existing
[Ferramenta design system](https://github.com/sebastian-software/ferramenta/blob/main/DESIGN.md),
implemented by `ferramenta-family@2.0.0`. The package owns its tokens, Barlow
Condensed fonts, textures, member icons, landing components, and site chrome.
Consult the package README and its kit before adding a shared pattern. Keep
system changes in the family package rather than maintaining a second local
token specification.

`app/root.tsx` loads Ardo styles, then the family tokens, fonts, theme, landing
styles, and docs styles. Site styles follow; family chrome loads last so it wins
selector ties. Keep docs styles after the theme and landing styles before local
styles. The root also preloads the package's Barlow Condensed 700 font.

The family header and footer sit outside `ArdoRoot`, with `handle.chrome = false`
disabling Ardo's own chrome. Documentation uses `fam-docs-shell`, `SiteMenu`, and
the `site-search` slot while retaining Ardo's sidebar, reading layout, and theme
toggle. Only the landing content uses `fam-page`, which pins its authored scheme.
The home route uses `ProjectHero`, `Relations`, and the `WorkWithUs` close.
Member icons and their provenance sidecars come from the package.

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
verifies routes, navigation, and benchmark content, and runs the benchmark
platform tests. Preserve generated benchmark data and upstream attribution;
follow the repository's benchmark workflow when measured content needs updating.
