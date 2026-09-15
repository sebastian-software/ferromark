# Unicode URL and leading BOM compatibility fix

## Decision

The cmark oracle treats a URL destination as a UTF-8 byte sequence: non-ASCII
bytes are serialized as uppercase `%HH` escapes. Existing `%HH` sequences are
left unchanged. This is byte serialization only; it does not perform IDNA or
host-name conversion. Valid IPv6 authority brackets remain literal because
they delimit the parsed host; brackets elsewhere continue through the normal
URL escaping path.

cmark removes one leading U+FEFF before parsing. Ferromark now follows that
behavior while preserving all later U+FEFF characters, non-breaking spaces,
literal spaces, and NUL source spans. The parser's document span still covers
the complete caller input, including a stripped BOM; node spans point into the
original source after the three-byte BOM offset.

## Regression evidence

The intentionally red first run is retained in `url-bom-before.log`. It
failed because the URL retained UTF-8 characters; the BOM assertion was not
reached. The final focused tests are:

* `cargo test --locked -p ferromark_parser --test bom_source -- --nocapture`
  — 2 passed.
* `cargo test --locked -p ferromark_parser --test nul_source -- --nocapture`
  — passed.
* `cargo test --locked -p ferromark_parser --test edge_cases -- --nocapture`
  — passed.
* `cargo test --locked -p ferromark_renderer --test edge_unicode_compat -- --nocapture`
  — 2 passed, including existing percent escapes and IPv6 plus a Unicode
  path segment.
* `cargo test --locked -p ferromark_renderer --lib escape -- --nocapture`
  — 16 passed, including all-byte scalar/SIMD differential checks and the
  escape-heavy ASCII plus Unicode suffix regression.
* `cargo test --locked -p ferromark_renderer --test edge_links -- --nocapture`
  — passed.

The follow-up span coverage includes BOM-only input, two leading BOMs (only
the first is stripped), and a BOM before a nested block quote/link containing
NUL. BOM-only input now borrows `source[3..]` instead of making a second text
copy. URL segments first find each ASCII/Unicode boundary and pass the entire
ASCII run through the existing vectorized escaper, so repeated ASCII escapes
do not restart a scan for every replacement.

Logs for the final runs are in this directory. The implementation changes are
in the parser source normalization/span map and HTML URL escape helper; no
production presets or normalizer/comparator code was changed.
