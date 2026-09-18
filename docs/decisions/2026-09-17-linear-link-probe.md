# Nested bracket text is parsed once, where it stands

## Decision

`parse_link` no longer probes bracket text that holds another bracket by
parsing it on its own and then letting the literal-bracket fallback parse the
same bytes again. It pushes the literal bracket speculatively, parses the
text in place with the surrounding content and offsets
(`Parser::parse_bracket_text`), and decides afterwards:

- The text produced a link, so the bracket around it cannot be one: the nodes
  already sitting in the caller's vector are exactly what the fallback would
  have produced, and nothing is redone.
- The text produced no link and the bracket resolves: the same nodes become
  that link's children and the speculative bracket is dropped.
- The text produced no link and nothing resolves: the bracket is literal and
  the nodes stay as they are.

The trailing text run of the region is deliberately left unpushed and its
start is handed back, so the caller's own run scan produces the one text node
that reaches past the closing bracket — the node shape the fallback produced.

The walk runs only where it is the whole of the work. Bracket text holding
any marker other than `[` is handed to the probe *before* a byte of it is
parsed, and so is text the probe has already judged: its cached verdict
stands and the fallback is what the parse of it has to be. The walk records
its own verdict in that same cache, exactly as the probe would, and an
attempt that has to be abandoned restores what it found.

The walk for a bracket's `]` keeps what it learns. `scan_balanced` is one
walk, and where a second walk over the same bytes is coming
(`scan_balanced_matched`, `record_bracket_matches`) it records the match of
every opener it passes, keyed by the address of the scan start and of the
content end. A nested run and a run of openers with nothing to close them
both cost one walk in total instead of one per opener.

Both live under the nesting cap of
[the inline cap record](2026-09-17-inline-nesting-cap.md): the in-place walk
opens one inline context per bracket level, at the offset the probe opened it
at, so `ParseErrorKind::NestingTooDeep` is raised for exactly the same input,
with the same span and depth.

## Why the output cannot change

The in-place walk is the literal-bracket fallback, run once instead of twice,
so the question is only whether parsing a region where it stands can differ
from parsing it on its own. Two guards keep that from happening, and the
probe path is still there for everything they exclude:

- **Only `[` is dispatched inside the region.** The walk asks for the next
  inline marker and gives up on any marker that is not `[`. Every other
  marker is one that reaches past the closing bracket or pairs with
  something outside it: emphasis and the other delimiter runs pair across a
  bracket, a code span, an autolink, raw HTML, an image, an inline note and
  an MDX expression can all close after it, and a line ending folds into the
  run that follows. None of them can occur in a region this walk decides, so
  no delimiter is pushed and no construct is cut short. Nothing in the
  emphasis pairing is touched.
- **A construct that ends past the closing bracket gives up too.** Brackets
  inside the region are balanced, because that is what `scan_balanced`
  returned, so a `[` in the region closes in the region. Its destination or
  its label can still read past the `]` — `[[a](u]x)]` is one — and that is
  only known afterwards, so the walk checks and hands the region back to the
  probe, which sees exactly what it saw before.

Inside those bounds the two parses make the same decisions byte for byte.
The marker scan is position-local; `has_closer_from` is settled for both by
the balance (a `[` in the region has a raw `]` after it inside the region);
`scan_balanced` from a position inside the region returns the same `]`; the
reference lookups take the same slices; and a scan that succeeds on its own
reads the same bytes in the same order in place, so it succeeds identically.
A scan that only succeeds in place is one that read past the `]`, which is
the second guard above. The one remaining difference is the region's last
text run, which a parse on its own ends at the region end: it is held back
and pushed only where the region becomes link children.

The recorded bracket matches are the same walk's answers. Every decision
`scan_balanced` makes — escape, code span, autolink, raw HTML — depends on
the position alone and never on where the walk started, so two walks that
both reach a position normally agree from there on. The `]` that returns a
walk to an opener's own depth is the `]` a walk starting after that opener
stops at, and a suffix one walk ends unbalanced is one such a walk ends
unbalanced too. An opener inside a region a walk skipped whole is never
recorded, so a scan starting inside a code span still walks for itself.

`link_probe_cache` stays: the probe path still memoizes its verdicts for the
regions the guards exclude, and the walk reads and writes it so that the two
paths agree on which texts have been judged already.

## The one verdict that can differ

[The emphasis bound](2026-09-17-inline-nesting-cap.md) counts the inline
contexts a parse opens, and a context is opened per sub-parse that *runs*: an
abandoned probe counts, a probe answered from `link_probe_cache` does not.
This change replaces a probe plus a re-parse of the same text with one parse,
so which sub-parses run is not the same, and at the cap that can change a
verdict.

Reading the cache before the walk and recording the walk's verdict in it
aligns every case the differential corpus found except one class, and that
one needs `wiki_links`: it is the only path that probes a slice which is not
a bracket text at the same position (a trimmed `[[…|…]]` label), so it seeds
the cache for ranges the walk never sees. With `max_nesting_depth = 4` and
every extension on, `[[[[[][]()]]]]` renders on `main` and is refused here;
with `wiki_links` off, both refuse it. Across 300,645 generated sources under
seven option sets (2.1 M renders) two sources differ, both of that class and
both under an artificially tight cap; with the cap at its default of 100 or
lifted, none do.

The band this can move is the one the cap documents as pathological: a
document has to reach `max_nesting_depth` levels of combined inline nesting
before the bound fires at all, and the count it fires on is an
over-approximation by construction — `main` refuses documents that are not
that deep because a speculative parse ran, and it accepts documents that are
because one was cached. No hand-written document is within one level of 100.
Pinning the verdict there would mean bounding the tree a parse keeps rather
than the sub-parses that happen to run, which is the emphasis bound's own
concern and not this change's.

## Why change

`probe_link_text` re-parsed the whole candidate text at every bracket level,
and every level walked for its own `]`, so nesting a link N deep cost O(N³).
Measured on the release build before the change (x86_64 Linux, best of
several runs): 8 KB of `[`×1600 `a` `](u)`×1600 took 2.56 s with the cap
lifted, and 25 KB of depth-100 groups took 42 ms under the default cap —
the cap of issue #349 stops the crash, not the denial of service below it.
A run of openers with one closer (`[`×50000 `a](u)`, 50 KB) took 1.37 s,
quadratic in the number of openers. The numbers and the paired runs are in
[the report](../reports/2026-09-17-linear-link-probe/README.md).

## Validation

Output equality first, timing after. A differential corpus of 300,645
generated bracket-heavy sources plus the hand-picked shapes around the new
path, each rendered under seven option sets — default, GFM and
all-extensions with the cap lifted, GFM at the default cap of 100, and three
at an artificially tight cap of 4 — is byte-identical in HTML *and* in the
AST `Debug` between `main` and this change, apart from the two sources of the
class above. `cargo test --workspace --all-features --locked` passes with no
`.snap.new` file, so the snapshot suites and the CommonMark 0.31.2 and GFM
conformance baselines are unchanged.

`src/parser/delimiters/tests.rs` pins the recorded matches against the plain
walk from every position of a generated token corpus, cold and after
recording. `tests/nested_links.rs` pins the node shape the literal fallback
produces, the destination that reaches past the outer bracket, the markup
that keeps pairing across the bracket, and three costs: nested groups now
cost the same per byte at depth 25 and depth 100, deep nesting past the
lifted cap finishes, and a run of openers with one closer is linear. All
three cost tests fail on `main`.

The walk also recurses less than the probe path it replaces. With the cap
lifted on a 1 MiB worker stack, `[`×N `a` `]`×N parsed to N = 900 and
overflowed by N = 950 before the change; after it, N = 1200 parses and
N = 1600 overflows. The default cap of 100 keeps every ordinary parse an
order of magnitude away from either bound, which is what the cap is for.

The MDX open-tag case in the same issue (`<A>`×40000 with `mdx: true`) is
quadratic for its own reason in `mdx_jsx`, shares nothing with this path and
is unchanged at 5.4 s; it needs its own fix.
