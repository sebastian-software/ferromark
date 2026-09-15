# Rust convenience functions

Add a small owned-output layer to the existing facade: `to_html`,
`to_html_with_options`, `to_html_into`, and `to_html_into_with_options`.
The project owner requested useful conveniences as a separate step after
specification corrections, with a coherent v2 API rather than v1 emulation.

Keep syntax and rendering options separate, matching the low-level API. Default
helpers use the same Rust defaults, including raw HTML passthrough; examples
show explicit sanitization for untrusted input. Node's safe default is unchanged.
All helpers return the existing parser `Result`. Append helpers accept a `String`
and preserve its contents on parse failure. Each call owns its temporary arena
and renderer; the caller can use the underlying types for explicit reuse.

Do not duplicate the public `Renderer` trait with a same-named session struct or
silently recreate old streaming/event APIs. CLI and reusable high-level session
design can be considered separately. Tests cover defaults, syntax/output policy,
append behavior, document isolation, empty input, and transactional error handling.

Validation: 818 all-feature workspace and documentation tests passed. Clippy
with warnings denied, formatting, benchmark compilation, and Rustdoc with
warnings denied passed. Line coverage is 90.80% against the unchanged 90% gate.
Repository contracts and generated README checks passed, and the website built
all six pages with the updated examples. Parser and Node behavior are unchanged
by the convenience layer; the preceding container-reference commit contains the
core and native-package validation.
