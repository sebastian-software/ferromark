# ARCH-EXP-019: Native table and short-document hotspots

**Date:** 2026-09-11

**Hypothesis:** the remaining native comparison gaps have different causes.
Fresh short documents pay fixed allocation and paragraph-copy costs; short table
cells repeatedly pay delimiter-recognition and short escape-scan setup costs.

**Method:** reproduce the gaps using the unchanged five-parser native driver and
its pinned compiler, generic CPU target, release profile, native libraries, and
shared mimalloc. Profile both Rust parsers, screen independent changes against a
frozen source snapshot, check exact output before timing, and confirm combinations
on larger and adversarial controls. Finally rebuild the real repository and
remeasure it with the original native driver.

**Decision:** retain lazy content/reference scratch and emphasis opener stacks;
borrow a complete single-range paragraph after normal block parsing; reserve a
small minimum for nonempty owned HTML; use scalar membership lookup for sub-vector
NEON searches; and recognize delimiter cells directly in one pass. Preserve byte
validation after the table column limit, including its existing bounded behavior.

The paragraph shortcut requires fresh document state and the exact whole-document
paragraph event sequence. It still invokes the complete inline parser and retains
subsequent footnote emission and resource-limit reporting. Lists, blockquotes,
multiline paragraphs, and other event shapes keep their existing stateful path.
The preallocated event buffers from ARCH-EXP-006 and the SSE2 path remain intact.
No public event/API, syntax option, dependency, or unsafe load contract changes.

**Rejected approaches:** indiscriminately smaller or larger event buffers, forced
inlining of the inline renderer, inline storage for every emphasis stack, and a
delimiter rewrite that first searches for cell separators. A long-delimiter
control exposed a regression in that last approach; removing both redundant
passes produced the retained recognizer.

**Evidence and limits:** the [profiling and optimization report](../reports/2026-09-11-native-hotspot-optimization.md)
contains raw samples, each candidate, retained-source hashes, production results,
and validation. These are Apple M1 Pro measurements of fresh owned rendering.
Small residual differences are not reliable universal rankings. The later [complete publication](../reports/2026-09-11-native-optimization-publication.md)
refreshes README/homepage figures with the regular publication protocol. Focused
diagnostics remain separately sourced evidence.
