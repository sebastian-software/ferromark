# ARCH-EXP-027: Linear text and attribute escaping on ARM64

**Status:** Adopted for ARM64 with NEON; other targets deferred
**Date:** 2026-09-12

## Decision

Bound repeated HTML escape searches on ARM64 with NEON. Search a shared
128-byte prefix inline, then use an outlined continuation. Tails of at least
256 bytes use the existing joint byte-set scanner; shorter tails use a bounded
memchr window. Text and attribute writers retain their existing output loops.

The prior long path repeatedly searched the entire remaining suffix for `<>&`
before checking quotes. Dense quotes without `<>&` therefore made total work
quadratic. The joint scan stops at the next member of the complete escape set;
the bounded short tail cannot introduce unbounded lookahead. Each output
iteration advances past the escape, making the complete write linear.

The byte set and replacements do not change. Attributes still escape single
quotes while text does not. Existing ByteSet and memchr backends do the search;
there is no new unsafe code, allocation policy or output-capacity reservation.

## Scope and evidence

ARM is the primary performance target. Other architectures retain the merged
baseline's implementation, including its quote-heavy quadratic case. The
initial portable window integration and subsequent x86 SIMD experiments are
not part of the adopted change. Shared CI runners provide correctness coverage
and diagnostic timing signals, not a controlled throughput reference for ARM.

The [report](../reports/2026-09-12-linear-html-escaping/REPORT.md) preserves failing
work-count reproducers, source snapshots, native measurements, A/A controls and
output guards. The final scoped source produces exactly the recorded ARM
candidate's machine-code section in the original harness. The report includes
source reconstruction and a repeatable code-identity check. Work-count tests
apply to the optimized NEON target; scalar output oracles cover all targets.
Release builds contain no counters.

This follows [ARCH-EXP-026](ARCH-EXP-026-neon-text-escape-integration.md), whose
five exact patches remain rejected. The selected ARM integration avoids their
multi-percent counterexamples. Small repeated costs in inline and deep-emphasis
controls remain disclosed; no workload frequencies are assumed to average them
away. The measurements do not prove that every input on every ARM machine is
faster. Published engine rankings and memory claims are outside this experiment.
