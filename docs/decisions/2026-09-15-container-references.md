# Document-wide reference definitions in containers

The project owner prioritizes a coherent v2 over v1 compatibility and archival
v1 documentation. Work proceeds in separate reviewable steps: spec-correct bugs,
then useful convenience APIs, then release readiness. No legacy documentation
site or compatibility layer is required merely because v1 had one.

CommonMark 0.31.2 section 4.7 requires definitions inside lists and block quotes
to affect the whole document, with the first definition winning. Fixing this is
a correctness requirement, independent of v1 output.

Use the existing block grammar for candidate-bearing containers. A temporary
arena and block-only mode avoid retaining a duplicate tree or parsing inline
content twice. Only definition values survive into the real parser's arena.
Keep the existing fast path for documents without candidates. Definitions must
also bypass the one-line list inline shortcut.

This is deliberately a bounded correction rather than a parser architecture
rewrite. It adds measurable work on reference-heavy and container-bearing inputs.
The [report](../reports/2026-09-15-container-references/README.md) records the
costs, tests, source patch, and raw measurements. A future optimization must
preserve this behavior, including code exclusion and document-order precedence.
