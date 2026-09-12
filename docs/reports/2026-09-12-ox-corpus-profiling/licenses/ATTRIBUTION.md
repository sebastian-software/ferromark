# Source attribution

The benchmark inputs come from the projects below. Exact original paths,
repository URLs, revisions, and file hashes are recorded in
[`corpus-manifest.json`](../corpus-manifest.json).

- [Vue documentation](https://github.com/vuejs/docs/tree/b75d188ab16bf83bd1f364a77dfd2315be8f3fa4),
  copyright 2019-present Yuxi (Evan) You and Vue documentation contributors.
  The text is licensed under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).
  The original notice is preserved in `vue-docs-LICENSE`.
- [Vite documentation](https://github.com/vitejs/vite/tree/99bd9d1d46153fa939f4a304cc0177db42e28776),
  copyright 2019-present VoidZero Inc. and Vite contributors, MIT license.
  See `vite-docs-LICENSE`.
- [The Rust Programming Language](https://github.com/rust-lang/book/tree/1500248d8f230566e4ec9f27fcbb8fe9e2898ab1),
  The Rust Project Developers. The original MIT and Apache 2.0 notices are
  preserved in `rust-book-LICENSE-MIT` and `rust-book-LICENSE-APACHE`.
- [TypeScript documentation](https://github.com/microsoft/TypeScript-Website/tree/61332a778fe41c95724b5f3ffd139f37ea41a267),
  Microsoft and TypeScript-Website contributors,
  [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).
  See the original `typescript-handbook-LICENSE`.
- [Ox Content benchmark fixtures and normalization helpers](https://github.com/ubugeeei-prod/ox-content/tree/026d1859d1c35e5fb1ea65e7e855b428a918b9bb),
  copyright 2024 ubugeeei, MIT license. See `ox-content-LICENSE`.

Archived HTML outputs and excerpts are mechanical transformations of the
original Markdown by the named engines and the documented normalizer.
Concatenation joins files in sorted order with an added newline. The normalizer
was modified to handle UTF-8 entity-window boundaries and HTML void elements.
No image files are redistributed. Nothing here implies endorsement by the
source projects or their contributors.

Ox's fetch script labels every source as MIT or MIT/Apache. At the frozen
revisions, the Vue and TypeScript root notices instead specify CC BY 4.0.
This archive preserves and attributes the actual notices rather than copying
those SPDX labels. Original license-file bytes and unified-diff context are
preserved, including upstream line endings and patch context whitespace.
