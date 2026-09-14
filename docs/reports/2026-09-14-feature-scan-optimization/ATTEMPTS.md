# Attempt ledger

All measured variants use the same worker, options, dependency versions, compiler,
and baseline `1728355`. Each screen verifies exact HTML/AST before measuring two
rounds of three 20 ms pairs, in parser-only and complete-reused modes. Definition
screens contain 37 cases; line-comment screens contain 36. The final combined
confirmation expands to 75 cases and all four modes, using nine 50 ms pairs.

Ratios below are baseline time / candidate time; larger is faster. Screens are
exploratory. Cross-screen differences are not paired incremental estimates.
Definition variants are cumulative D1 → D2 → D3 → D4. L1 and L2 are separate
alternatives, each built on the original core without definition-list changes.

| ID | Attempt and hypothesis | 4 KiB plain parser | 4 KiB active parser | Result |
| --- | --- | ---: | ---: | --- |
| D1 | Cache the next possible `:` body marker, including an exhausted suffix. Avoid term collection when no marker exists. | 2.882× | 1.089× | Retained. Plain prose benefits strongly. Late-definition prose is 0.994×, revealing the remaining speculative work. |
| D2 | Reject ordinary line prefixes before finding the line ending in `definition_body_at`. | 2.915× | 1.104× | Retained. Late-definition prose improves to 1.069×. |
| D3a | Scan term boundaries first; delay all term validation until a body is found. | Not timed | Not timed | Rejected during complexity review. A long indented body followed by another definition can make every continuation probe scan the remaining suffix. This introduces quadratic work where early rejection previously stopped immediately. The prototype patch is preserved. |
| D3 | Replace the temporary arena term vector with validated source coordinates/count, preserving early rejection while scanning. | 2.909× | 1.118× | Retained. Late-definition prose improves to 1.123×. Accepted terms receive a bounded reconstruction pass; unsuccessful probes no longer allocate the temporary vector. |
| D4 | For terms starting with an ASCII letter, skip block recognizers whose prefixes cannot match. Still check inline backticks; retain the full Unicode path. | 2.901× | 1.206× | Retained. Late-definition prose improves to 1.212×. Final long-body, multiline-term, Unicode, and span checks pass. |
| L1 | A process-wide `memmem` finder for `//` rejects slash-free paragraph slices before the existing discovery loop. | 1.030× | 0.975× | Rejected. The 64 KiB plain ratio is 0.998× and active ratio 0.969×; slash-heavy controls are about 1.00×. The extra search/setup does not reliably remove enough work. |
| L2 | Pass the first eligible comment already found by paragraph parsing into reconstruction. Use independent marker discovery only for reference definitions. | 1.179× | 1.085× | Retained. Slash-heavy 4/64 KiB prose reaches 1.428/1.473×. The final combined confirmation retains the gain. |

The initial L2 draft still treated `None` as an unknown position and performed
another search. Review corrected that before timing: for the paragraph helper,
`None` means already proven absent. The independent reference-definition entry
point performs its own discovery. Trailing comments at or after `content_end`
also return the original source slice. The [working notes](line-comment-attempt-notes.md)
retain this correction and its targeted validation.

## Evidence and patch replay

The [initial regression failure](checks/definition-probe-before.log) establishes
that a rejected ordinary definition-list probe allocated arena storage before
the changes. The final allocation test includes URL/time colons, inline colons,
and prose preceding a later definition. Renderer tests additionally cover
dedented list/quote definitions, three line endings, term spans across comments,
trailing comments, and slash-heavy ordinary text.

Each measured screen retains its build manifest, binary/worker hashes, build
log, full input/config corpus, exact HTML/AST verification, per-pair samples,
arena capacities, and per-round summaries under `screens/ID/`. Final combined
data is under `final/`. These workers use a presized arena, so unchanged reserved
capacity does not refute the removal of temporary term storage; reserved bytes
are not occupied bytes or allocation-call counts.

All patches apply to `1728355` individually; definition patches are cumulative:

- [D1 marker cache](patches/D1-marker-cache.patch)
- [D2 prefix-first](patches/D2-prefix-first.patch)
- [D3a deferred validation, rejected](patches/D3a-deferred-validation.patch)
- [D3 source ranges](patches/D3-source-ranges.patch)
- [D4 final definitions, including regression tests](patches/D4-final-definition-lists.patch)
- [L1 finder, rejected](patches/L1-memmem.patch)
- [L2 final comments, including regression tests](patches/L2-final-line-comments.patch)

To reproduce the combined retained source, apply D4 and L2 to a local checkout
of the baseline and run the [harness](../../../benchmarks/feature-scan-optimization/README.md).
Later test-only cleanup replaces a test fixture's `format!` with arena-string
construction to satisfy the parser's existing Clippy rules. It has no release
code effect; the initial Clippy failure and successful final checks are retained.
