# Line-comment optimization attempts

1. Added a process-wide `memmem` finder for `//` and a necessary-marker fast
   rejection in `without_line_comments`; this should skip the per-physical-line
   scan for slash-free prose while preserving the existing slow path whenever
   a marker is present. `cargo fmt --all --check`, parser unit tests (51), and
   all 20 line-comment renderer tests passed.

2. L2 alternative: `parse_paragraph` now records the first eligible comment
   position it already discovers while advancing lines and passes it to
   paragraph/setext reconstruction. The helper returns a borrowed slice when
   that known comment is trailing outside `content_end`, and otherwise copies
   the verified prefix with one source-map entry per physical line before
   handling the first and later comment lines. Reference-definition callers
   retain independent detection using repeated `memchr('/')` checks, avoiding
   process-global finder setup. `cargo fmt --all --check`, all 20 line-comment
   renderer tests, and 54 parser unit tests passed.

   Correction: the first version still treated `None` as unknown inside the
   helper and rescanned comment-free paragraphs. Discovery now happens only in
   `without_line_comments` (reference callers); `_with_first` treats `None` as
   proven absent and borrows immediately. Marker detection is clamped to the
   requested range so a slash pair beginning at `end - 1` cannot leak in.
   Added coverage for trailing comments before blank lines, blocks, and EOF,
   plus slash-heavy ordinary prose. The 22 line-comment renderer tests and 54
   parser unit tests pass.
