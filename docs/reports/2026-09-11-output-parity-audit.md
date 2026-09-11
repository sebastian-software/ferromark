# Output parity audit: ten exclusions, four causes

**Historical audit:** the subsequent [workload contract](../arch/ARCH-COMP-002-workload-comparability.md)
supersedes this report's strict timing-exclusion policy. Bun alignment and task
renderer differences are now admitted. [ADR-0014](../arch/ADR-0014-reference-resolution-budget.md)
raises Ferromark's reference budget; the captured outputs below retain the old
limit and are not rewritten.

The original 32/42 gate was too strict about ordinary HTML whitespace and also
concealed an adapter configuration error. After correcting the md4c task-list
flag and normalizing HTML flow whitespace, **34/42 cases agree across all five
parsers**. The remaining eight cases form repeatable groups, rather than eight
unrelated discrepancies. Agreement is evidence of equivalence, not a correctness
vote; Ferromark is not the oracle.

This audit uses the original frozen inputs and HTML, a fresh verification of all
42 cases, and small native reproductions against the same pinned parser sources.
No production parser or safety limit was changed.

## Groups and causes

F = Ferromark, P = pulldown-cmark, C = Comrak, B = Bun, M = C-md4c.
Groups below use the corrected adapter and flow-whitespace comparison.

| Original excluded cases | Agreeing groups | Finding |
| --- | --- | --- |
| `commonmark/commonmark-5k`, `commonmark/commonmark-50k` | F/P/C/B/M | Only serialization whitespace before nested lists. Both now pass. |
| `gfm_overlap/commonmark-5k`, `gfm_overlap/commonmark-50k`, `gfm_overlap/tables-5k` | F/P/C/M; B | Bun applies the last table's alignment to earlier tables. |
| `commonmark/commonmark-1m`, `gfm_overlap/commonmark-1m` | P/C/B/M; F | Ferromark exhausts its document-wide reference-resolution work budget. |
| `gfm_overlap/tasks`, `task_lists/tasks` | F/P; B/M; C | Same tasks and checkbox states; different checkbox placement and CSS classes. |
| `gfm_overlap/gfm-features` | F/P/C; B/M | Tight tasks: class attributes and spacing beside checkboxes differ. |

The [generated groups](2026-09-11-benchmark-refresh/parity-groups.json) retain both
the original outputs under the revised comparison and the corrected outputs.
The [native probes](2026-09-11-benchmark-refresh/parity-probes.json) retain minimal
inputs and actual HTML, plus the large-document resource-limit observation.

### Whitespace: a comparator issue

For a nested list, four parsers emitted `parent\n<ul>` where md4c emitted
`parent<ul>`. The newline is formatting before a block element. The old check
removed standalone newline-only text nodes but missed a newline attached to text.
It also incorrectly removed a standalone newline between inline elements, which
can erase a visible word separator.

The revised comparator collapses ASCII HTML flow whitespace and trims it at
known block boundaries. Thus `a\n b` and `a b` compare equally, but `a b` and `ab`
do not. Spaces inside `pre`, `code`, `textarea`, `script`, and `style`, non-breaking
spaces, URLs, alignment, classes, and checkbox states remain significant.
Regression tests cover both newly accepted differences and preserved distinctions.

This assumes default HTML flow. It is not a browser layout proof under arbitrary
CSS (for example `white-space: pre-wrap`) or a generic HTML sanitizer. The raw
output remains available for applications with stricter requirements.

### md4c tasks: our adapter was wrong

The Rust adapter passed `0x2000`, which the pinned md4c header defines as
`MD_FLAG_WIKILINKS`. `MD_FLAG_TASKLISTS` is `0x0800`. That explains the literal
`[x]` text in the old output; it was not an md4c parser bug.

A native behavioral self-test now checks that enabling the overlap configuration
renders checked and unchecked task inputs while leaving `[[page]]` literal;
disabling tasks must preserve task syntax. It failed with zero checkboxes before
the fix and passes afterward. It runs before every new verification/measurement.
Publication also checks recorded effective md4c flags, independently of matching
HTML: unused but incorrectly enabled features still invalidate option parity.

All original `gfm_overlap/*` and `task_lists/*` timing records are superseded as
matched-option comparisons. They remain immutable historical evidence. The only
public row using those flags, `gfm_overlap/gfm-tables`, was rerun for **all five
parsers**, three full runs. Seven unaffected public cases retain their original
samples. No old and new parser cells are mixed within one row. Other affected
historical cases need a fresh timing run before reuse.

### Bun tables: a minimized upstream behavior bug

This input reproduces the alignment error in Bun commit `76e9dcc6`:

```markdown
| A | B |
| --- | ---: |
| one | two |

| C | D |
| :---: | --- |
| three | four |
```

The first table should retain an unaligned first column and right-aligned second
column. Bun instead centers the first column and removes the second alignment,
matching the last table. Rendering the first table alone works. Reversing the
tables reverses the borrowed alignment. All four other parsers preserve each
table's own alignment.

This matches the pinned source: `is_table_underline` writes a shared
`table_alignments` array and `render_blocks` reads that array when rendering
cells. The [GFM table specification](https://github.github.com/gfm/#tables-extension-)
defines alignment from each table's delimiter row. This changes presentation and
must not be normalized away. The standalone adapter uses unchanged Bun parser
sources; this audit does not claim to have tested a later Bun release.

### Ferromark references: deliberate fallback, relevant tradeoff

The 1 MiB fixture sets `ResourceLimit::ReferenceResolutionWork` in Ferromark's
parse report. Its 32,768-unit budget counts bracket records and label bytes over
the whole document. Once exhausted, reference syntax remains literal.

In the archived fixture, Ferromark resolves 404 links to the charter URL; each
other parser resolves 564. That is less completed work, so a speed ratio would
be misleading. The behavior is documented in `src/limits.rs` and already covered
by resource-limit tests; it is not explained by whitespace or missing syntax
options. Whether the fixed budget should scale with input size is a separate
parser-design question that needs adversarial tests. This audit preserves the
protection rather than silently disabling it to improve a comparison.

### Task lists: comparable work, different renderer contracts

The [GFM task-list specification](https://github.github.com/gfm/#task-list-items-extension-)
requires a semantic checkbox with the appropriate state. The corrected outputs
agree on task labels, order, count, nesting in these fixtures, and checked states.
The differences do not demonstrate failed task recognition:

- Ferromark and pulldown-cmark put the checkbox inside the first paragraph of a
  loose list item; their space-versus-newline difference is now accepted.
- Bun and md4c put it before that paragraph and add `task-list-item` and
  `task-list-item-checkbox` classes.
- Comrak also puts it before the paragraph, without those classes.
- In the tight-list mixed fixture, Ferromark, pulldown-cmark, and Comrak agree;
  Bun and md4c agree with one another, differing in classes and checkbox spacing.

Checkbox position relative to a paragraph can change layout; classes can be
application styling hooks. These deserve disclosure, not a blanket bug label.
A task-workload benchmark can include all five with those differences stated,
while reporting emitted bytes and latency. It should be labeled as a workload
comparison with different HTML contracts, separate from the equivalent-output
headline tables. Normalized agreeing subgroups are also usable for explicitly
labeled comparisons; never select a subgroup because it favors Ferromark.

## Consequences for publication

Keep three questions separate: **same requested options**, **same completed
Markdown work**, and **same HTML contract**. Whitespace variations alone should
not prevent timing. Missing links and incorrect table alignment remain correctness
or policy diagnostics. Task renderer conventions permit a disclosed workload
comparison but do not establish identical HTML.

The 34/42 figure is output eligibility, not a claim that all 34 cases have fresh
published timing samples. The newly admitted two historical CommonMark fixtures
have not been timed in this correction. The eight public rows retain the full
three-run protocol, with the affected GFM row replaced from new evidence.

Reproduce the grouping without rebuilding parsers:

```sh
python3 benchmarks/bun-comparison/audit.py \
  docs/reports/2026-09-11-benchmark-refresh/native \
  docs/reports/2026-09-11-benchmark-refresh/native-corrected \
  --check docs/reports/2026-09-11-benchmark-refresh/parity-groups.json
```

After preparing the native harness, use `ferromark-bun-comparison render 1`
(tables), `render 4` (tasks), or `render 0` (CommonMark), passing Markdown on
standard input, to reproduce the small examples and inspect all five outputs.

Validation after the correction: the native option self-test, 15 Python
verification/publication/grouping tests, all 119 repository contract tests,
`cargo test --locked --all-features`, workspace Clippy, and formatting passed.
The grouping audit reproduces from the saved HTML; all 39 original evidence
hashes remain unchanged and the expanded manifest verifies all 57 files.
Homepage type checking and the production build passed, including six
prerendered pages and generated-number checks.
