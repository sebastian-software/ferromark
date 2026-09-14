# Linear URL punctuation trimming

Hypothesis: repeated trailing bracket validation scans the same prefix once per
removed closer. Caching counts lazily for each of the three bracket types changes
O(n * trailing closers) work to at most three O(n) counts plus O(trailing closers).
Ordinary URLs without a trailing closer should remain a constant-time exit.

Variant: lazy per-bracket opening/closing counts. Include the current closer in
the initial count; remove it iff closes > opens; decrement cached closes after
each removal. Plain trailing punctuation changes no bracket counts. Initial
counting for another bracket type uses the already shortened URL.

Correctness: preserve the original implementation as a test oracle. Exhaustively
compare all tails through length six from nine bracket/punctuation/letter bytes,
with multiple start offsets; compare long mixed closer/punctuation runs after a
Unicode URL. Existing autolink snapshots and conformance output are unchanged.
Tests were added and passed before the production replacement.

Status: ready for parent timing. Patch archived as renderer-linear-trim.patch.
