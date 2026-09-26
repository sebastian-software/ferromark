# Optional locale-aware typography pass

- Status: Accepted
- Date: 2026-09-26
- Related: #403, #450, #452, [optional transform pipeline](../arch/ADR-0022-native-transform-pipeline.md)

## Context

Issue #403 asks for an implementation of locale-aware typography after parsing.
PR #450 added frozen output fixtures for upstream typography plugins; those
fixtures are reference oracles, not a typography implementation. PR #452 added
the optional Rust AST transform pipeline, not a built-in pass or a Node.js
configuration option. The behavior described by the issue therefore remained
unimplemented after those changes.

Typography can alter authored punctuation and needs a language choice. Doing
this during core parsing or rendering would add work to ordinary calls, guess
at locale, and make output depend on an implicit default. Ferromark also needs
to leave Markdown syntax and machine-readable text intact.

## Decision

Typography is an explicit post-parse operation. Rust applications add
`TypographyPass` from the optional `ferromark-transforms` crate. Node.js callers
set `typography` on `Options`; it is absent by default. The core Rust crate,
parser, renderer, and Node.js calls without the option do not run typography or
scan prose for URLs.

Each invocation requires exactly one reviewed language code: `en`, `es`, `fr`,
`pt`, `de`, `it`, `nl`, `pl`, `ru`, or `uk`. The pass does not detect language,
infer a region, or change rules within a document. `en` uses US English quote
conventions. `pt` uses the reviewed Portugal Portuguese quote convention.
Unsupported and regional-tag codes are errors rather than implicit fallbacks.

The initial rule set converts unescaped straight quotation marks and linguistic
apostrophes, three periods to an ellipsis, selected dashes, French punctuation
spacing, and a reviewed set of number-unit spaces. It preserves existing
Unicode punctuation and does not rewrite unrelated hyphens. Dash and ellipsis
conversions can be disabled independently. The supported quote systems and
spacing rules are enumerated in the [typography guide](../typography.md); this
is a reviewed subset, not a claim of complete compatibility with Remark,
Typograf, or another plugin.

Quote pairs can cross ordinary inline formatting. The pass protects code,
math, raw HTML, MDX expressions and module payloads, image metadata, link
destinations and titles, escaped/entity-authored quotes, and bare URLs recognized
by the renderer's configured autolink matcher. Quote context ends at protected
regions and block boundaries. Text replacement preserves each source span.

Run the pass after AST edits that inspect authored punctuation and before
heading IDs, outlines, or rendering are derived. Running it more than once is
idempotent.

## Consequences

Applications that want typography must select a language and opt in. Their
output changes only in the documented prose cases; the parser's syntax and
default renderer output remain stable. URL recognition runs only when the
optional pass is invoked. Rust users do not add the transforms crate unless
they choose that API; Node users use the built-in `typography` option.

The implementation owns its reviewed language data and must keep those rules,
the Rust pass, the Node binding, and the reference fixtures aligned. Adding a
locale or broadening compatibility requires its own fixtures and review. The
Node option is part of the public API and follows the existing option
validation and getter-order contract.

## Validation

Tests cover the ten language fixtures, nested and cross-format quotes,
protected Markdown and MDX nodes, links and autolinks, toggles, source spans,
idempotence, Node/Rust parity, and all Node rendering entry points. The frozen
oracle fixtures remain expected-output inputs; they do not substitute for the
native implementation tests.
