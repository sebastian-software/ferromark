# ADR-0023: Compile authored Markdown and MDX to framework-neutral JSX

- Status: Accepted
- Date: 2026-10-01

## Context

Ardo needs one native Markdown and MDX engine for its documentation pages.
Ferromark already owns the arena-backed syntax tree and ordered native passes.
Its HTML APIs retain an intentionally permissive MDX recognition mode, while
JSX compilation needs reliable JavaScript boundaries and syntax diagnostics.

## Decision

Add an optional `jsx` Cargo feature and a framework-neutral JSX renderer.
The Node binding enables the feature and exposes `compileJsx` and `JsxCompiler`. The result
contains a JSX body, preserved ESM with original UTF-8 source ranges, authored
component roots, generated Markdown elements, fenced code metadata, front
matter, headings, and zero-based source positions with UTF-16 columns.

The JSX API opts MDX into strict parsing with Oxc. Existing HTML APIs and their
MDX defaults keep their current behavior. Native passes run before JSX and
metadata are derived. The JSX renderer plans heading IDs and its outline
together, including visible Markdown text nested inside authored JSX.

Ferromark emits syntax; it does not import React, evaluate JavaScript, or build
executable modules. Consumers own provider bindings, layouts, framework page
context, ESM analysis, and downstream JSX compilation. Ardo uses Vite's normal
Oxc transform for this step. The amendment below moves module assembly and ESM
analysis into Ferromark.

A synchronous, trusted code-block callback can emit JSX. Callback exceptions
propagate, and returning no replacement selects the native renderer. The
document and its arena remain local to one call; no self-referential AST handle
or asynchronous native callback is exposed.

The reusable Node `JsxCompiler` owns a native Ferriki highlighter and loads
verified standard theme and grammar assets on first use. Rust consumers can
enable `jsx` and `ferriki` and supply `FerrikiJsxHooks`. A light/dark pair emits
token color and font CSS variables. Fence annotations and a generic codeblock
component retain the original code independently of the highlighted children.
Whole-fence callbacks and language-specific component mappings take precedence
over highlighting. Plain `compileJsx` remains available without highlighting.

## Consequences

Core-only HTML consumers acquire no JavaScript parser dependency unless they
enable `jsx`. JSX compilation consumes trusted authored programs and does not
inherit the HTML renderer's untrusted-content policy. Framework-specific HTML
options are excluded from the JSX API rather than silently ignored.

The existing HTML snapshots and conformance baseline remain release gates.
The new parser and renderer need semantic regressions for JavaScript grammar,
whitespace, component ownership, heading IDs, and source maps. This decision
makes no performance claim.

The all-features dependency graph introduces two narrowly versioned license
exceptions without changing the family-wide allow-list. For Oxc's
`dragonbox_ecma` 0.1.12, select its [Boost Software License
1.0](https://spdx.org/licenses/BSL-1.0.html) option. Source redistribution must
retain the copyright notices and license; the license exempts copies solely
in machine-executable object code from that notice requirement.

`webpki-root-certs` 1.0.9 contains Mozilla certificate data under
[CDLA-Permissive-2.0](https://cdla.dev/permissive-2-0/). It appears only in the
wasm32 branch of `rustls-platform-verifier`, which cargo-deny includes when
checking all targets. The native Node sidecars use platform trust stores and
do not ship this data. Any future distribution that includes the certificate
data must include the agreement text with it. Both exceptions require review
when the dependency version changes.

## Amendment (2026-10-01): MDX module output

The first consumer showed what the body-only boundary costs. Ardo rebuilt the
MDX module contract in about 940 lines: scope analysis, layout rewriting,
component resolution, and source-map merging. None of it was specific to Ardo
or React. It parsed each module block a second time, with a different Oxc
version and a different grammar than Ferromark, and it compiled each document
once more only to find free identifiers.

The module contract is MDX semantics, not framework semantics. Ferromark
therefore offers module output next to the body
([#504](https://github.com/sebastian-software/ferromark/issues/504)). The
module holds the authored ESM in document order, a content function, and
`MDXContent`. An authored default export becomes the layout. A component
reference uses a module binding when one exists and the components object
otherwise, with an error that names an undefined component. Generated names
are reserved, which removes the naming pass.

The original limits stay. The module still contains JSX, imports no framework,
and is never evaluated; only the provider import source is supplied by the
caller. Consumers keep their page context, their own exports, and downstream
JSX compilation. Authored JavaScript stays source text: Ferromark does not
analyze scopes inside expressions or module blocks, so a component used only
there must be imported. Module blocks stay JavaScript, and TypeScript syntax is
reported as such.

Module assembly is a layer over the JSX renderer behind the `jsx` feature. The
parser and the AST are unchanged. The assembler parses each module block again
with the same Oxc version and grammar as the parser, and it reads the syntax
tree only inside one module that returns names and text edits. Ferromark now
depends on the shape of Oxc's export declarations, so an Oxc upgrade can need
changes there. `oxc_ast` and `oxc_ecmascript` become direct dependencies; both
were already in the dependency graph.

This amendment makes no performance claim.
