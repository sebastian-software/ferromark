# README theme composition

## Status

Active. Update this document when the contract changes.

## Decision

The project README is composed by native mdtheme from `README.md.src`. The
outer frame is Sebastian Software; Ferramenta is the inner frame. Footers close
in reverse order. The project content remains the main focus; shared branding
is compact and maintained upstream. The Ferramenta footer excludes this project
and includes sibling descriptions and the family icon.

Pin the CLI with mise and both Git theme revisions in mdtheme.yaml. CI checks
the generated output. Source and output are committed together. Standards
repositories explicitly delegate README ownership to mdtheme, so standards
cannot append a second company footer. Published subpackage READMEs keep their
compact registry format and existing generator.

## Consequences

Contributors edit the source, then regenerate. No JavaScript configuration or
Node installation is needed for the root README. Shared theme updates are
reviewable Git diffs. Rendering requires network access to the Git sources.

See [the contributor guide](../readme-theme.md) for commands.

## Theme badge placement

The project pins mdtheme 0.4.0 and uses one `mdtheme:badges:start` /
`mdtheme:badges:end` comment pair in `README.md.src`. Sebastian's
`badges-prepend.md` places its badge before the authored project badges.
Ferramenta's header and footer stay unchanged. Upgrade the CLI and lockfile
before adopting this theme revision. Keep badge markup outside raw HTML blocks.

## Content ownership

The README introduces the product and offers a quick start. Detailed integration
guidance, architecture, and benchmarks live on website guide pages. Archived v2 measurements retain their source revisions and are reproduced on
the website; mdtheme only composes the concise README.
The shared frames continue to be maintained in their theme repositories.

## Documentation audiences

The website has independent Rust and Node.js tutorial sections. Installation,
API configuration, metadata, highlighting, and runtime errors belong to their
package's guide. Shared pages explain Markdown syntax and rendering behavior
without runtime-specific code. Navigation follows the active section and keeps
shared concepts available. Rust is the primary entry point on the landing page;
Node.js is presented as the binding to the Rust engine. Matching tutorial pages
offer a Rust-first runtime switch that preserves the topic and links to each
package's own URL. Runtime-only topics do not imply an equivalent in the other API.

The website uses the shared Ferramenta tokens and bundled display font; local
styles own page composition, not a second palette or remote font family.
