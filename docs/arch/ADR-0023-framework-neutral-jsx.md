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
Oxc transform for this step.

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
