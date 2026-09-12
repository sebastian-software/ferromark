# HTML block parsing after inline emission optimization

This investigation starts from merged main `f36b0a3ab809af07be029c66413af8314984abc5`
(PR #309). Fresh profiles confirm
that the TypeScript compiler-options reference still spends most visible samples
in block parsing. The accepted change accelerates root HTML continuations,
recognition of known block tags, and searches for HTML end markers.

[RESULTS.md](RESULTS.md) contains generated measurements, including every timed
control. [ARCH-EXP-023](../../arch/ARCH-EXP-023-html-block-continuations.md) records
the decision and compatibility boundary.

## Experiments and implementation

The initial experiments separately test repeated root continuation processing,
deferring opening-line lookahead, and targeted end-marker search. Profiles then
localize additional samples to the linear scan of the known block-tag list;
a fourth hypothesis tests grouping that unchanged list by length and first byte.

The selected continuation loop starts inside HTML-block recognition, after it
consumes the opening line. An initial integration called the loop from the general
parser dispatch and repeatedly slowed the short-fence control. Moving the entry
to HTML recognition removes that extra check from ordinary Markdown lines. It emits the original line-based events directly and updates the
same per-line definition state. HTML inside containers still uses normal prefix
matching. Blank termination returns through the existing HTML-line handler, which
also preserves the parser's handling of carriage returns and trailing whitespace.
The private renderer can still coalesce contiguous ranges as before.

The tag lookup retains all known block tags and its heading-tag rule. It folds
only the initial byte for dispatch, then compares the remaining candidates with
the original case-insensitive equality check. End searches remain bounded by the
physical line; split terminators are not joined across newlines.

The exploratory alternatives are retained even when slower. A repeated call to
the original HTML-line handler saves less work than direct range emission.
Iterating all newline positions adds no further benefit in these screens.
The opening-prefix experiment has little standalone benefit. General substring
search substantially helps long lines but adds costs on some short-block cases;
the retained candidate-byte search handles these short terminators with less setup.

## Correctness boundary

Frozen comparisons cover specification and extension examples, renderer reuse,
all original Ox corpus inputs, deterministic inline combinations, and additional
HTML/container combinations. The HTML guard compares exact public block events,
full MDX event streams and spanned segments, rendered HTML, headings, and resource
reports. It includes trusted/untrusted rendering, the raw-HTML filter, disabled
HTML, source-only comments, definition lists, lists, blockquotes, footnotes,
CRLF, bare carriage returns, NULs, UTF-8, EOF, and marker fragments.

These are behavior-preservation checks, not a claim that every baseline result
is normative Markdown. Known differences from Ox remain in the corpus admission
results. Every timed input has an exact baseline output check; exploratory
short-line/long-line cases are identified as synthetic.

Repository regression tests cover each HTML-block kind, line ranges, transitions
back to Markdown, source comments, container termination, unterminated blocks,
split markers, renderer reuse, policies and the complete known block-tag set.
They pass on the frozen baseline before the implementation and on production.

## Measurement boundaries

The optimization harness uses native Rust with fresh render state and owned HTML
except explicitly named reuse controls. It uses the system allocator and Rust
1.97.1 on an Apple M1 Pro. The aarch64-apple-darwin default CPU is `apple-m1`, not
`generic`. Standalone builds run outside repository Cargo configuration and remove
inherited Rust flags. Release settings are optimization level 3, fat LTO, one
codegen unit and panic abort.

Screens are exploratory. Production confirmation rebuilds the formatted source
and uses three fresh process pairs, alternating order over longer windows. No
build, test, profiler or second timing suite runs during a timed comparison.
Small variations are not confidence intervals or universal speed claims.

Heap observations count allocator requests during fresh rendering with output
still live. The suite retains large ordinary documents and adds HTML cases at
64 KiB and 8 MiB. This measures neither process RSS nor input storage. No new heap
buffer or reservation is introduced by the implementation.

The fresh Ox comparison uses the existing independent native corpus harness and
normalization boundary. Both engines create and drop their parser state and owned
HTML. Ox's growing arena is primary; presized results remain separate. The Ox
revision and corpus inputs are pinned. Unequal output supports diagnostics only.
Do not subtract these latencies from the optimization harness's absolute times.
README and homepage tables remain independently dated publication snapshots.

Profiles retain two captures per document/implementation, using optimized builds
with debug information and frame pointers. Sampling is diagnostic and is separate
from timing. The baseline sampling driver predates the added full MDX snapshot
field; that operation is not used by its render loop. Both sampling builds pass
the original frozen output checks. The final profiles attribute inlined
continuation work to `try_html_block_start`; its samples include processing the
block body, not only recognizing the opening tag.

## Reproduction and validation

[REPRODUCE.md](REPRODUCE.md) explains how to reconstruct the frozen baseline and
production from Git plus the archive, verify checksums and replay the output
oracles. `metadata.json` identifies source and executable hashes. Exploratory
patches remain separate from `implementation.patch` and the formatted production
source. Duplicated full outputs for unequal corpus concatenations are omitted;
all original-file mismatches and every corpus input remain reproducible.

The production implementation is commit `f2197eb8312b57fd227fd5950e232eff1d687a4d`. Subsequent
report commits do not alter the measured Rust source.

Validation logs include the all-feature Rust tests, all-target/all-feature Clippy
with warnings denied, the format check, and an independent archive replay.
