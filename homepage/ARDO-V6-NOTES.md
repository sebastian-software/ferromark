# Ferra integration with Ardo 6

This draft keeps the final Ferra homepage and documentation implementation
reviewable while its framework and shared materials are still in development.

## Dependency gates

- Ardo: [PR #355](https://github.com/sebastian-software/ardo/pull/355), commit
  `e4c0c439970517fff9165d0110bddfb6a37df51b`. The package manifest still says
  `5.0.0`; the source includes the upcoming managed-runtime and styling APIs.
- Ferramenta: the merged Ferra components and assets at
  `1cb7c16`, plus the pending shared refinements for neutral paper, scrolling
  grain, a metal hero ground, vertical reading rules, material-transition edges,
  and subtle sheet shadows. These shared refinements need their own reviewed
  release before this website can be reproduced from published packages.
- Replace both relative `file:` dependencies with appropriate reviewed releases,
  regenerate the real lockfile, and pass standalone frozen-install CI before
  merging this migration. No native engine or benchmark data changes are part
  of this website PR.

## Framework integration

Ardo continues to own Markdown/MDX rendering, highlighting, search, metadata,
breadcrumbs, pagination, and the page outline. Ferra owns the visual system.
The adapter uses public theme variables and the additive CSS hooks documented
in Ardo's adoption guide; it does not select generated class names.

`ArdoRoot.className` provides document scrolling. The outline therefore needs
the viewport as its observer root and a height-based detection band, including
after resizing. Theme variables live on HTML through
`ArdoRootLayout.className`, so search portals inherit the same oxide color.
The site provides direct sidebar elements to preserve the route outlet.

Ardo #355 also fixes trailing-slash breadcrumb/pagination matching, copying
native highlighted code, custom favicon MIME types, and narrow pagination
wrapping. The adapter targets the current sidebar section tabs and keeps a
separate documentation menu available on desktop and mobile.

All browser runtime imports use Ardo's managed React and router exports. Release
metadata crosses the package boundary only in Vite's build configuration and is
injected into the local version module. The existing production build check
still verifies the facade's actual release version.

## Shared materials

Only metal-to-paper transitions have horizontal edges. Consecutive paper
sections have no horizontal separators. Code and result sheets use Ferra's
shared shadow token. Paper grain scrolls with the document and stays at a natural
scale rather than stretching across a long page. These rules belong in the
shared package rather than being copied into the website adapter.

## Content preservation

Current main's guides, prepared document-analysis content, benchmark platform
records, frozen samples, and evidence links remain intact. The current complete
comparison stays on the homepage. The historical Fresh/Reuse chart in the guide
supports both recorded platforms and states its narrower agreement-subset scope.
The build retains current main's benchmark and search-record checks.
