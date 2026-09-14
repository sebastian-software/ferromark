# Ferromark v2

A lean, arena-allocated Markdown parser and HTML renderer, derived from
[OX-Content](https://github.com/ubugeeei-prod/ox-content). This is a local development
baseline for Ferromark v2, with a fresh Git history and unpublished packages.
The API is not compatible with Ferromark v1 and is not a stable v2 release.

```rust
use ferromark::{Allocator, HtmlRenderer, Parser};

let source = "Hello, **world**!";
let allocator = Allocator::for_source_len(source.len());
let document = Parser::new(&allocator, source).parse().unwrap();
let html = HtmlRenderer::new().render(&document);
assert_eq!(html, "<p>Hello, <strong>world</strong>!</p>\n");
```

Use `ParserOptions::gfm()` with `Parser::with_options` for GFM, or
`ParserOptions { mdx: true, ..ParserOptions::gfm() }` to combine GFM and MDX.
Footnotes, math, definition lists, and other extensions remain configurable.
The allocator and source must outlive the document. Reuse/reset an allocator only after its documents
have been dropped. HTML options and renderer hooks remain available directly.

The workspace contains four core crates and a small re-export facade:

| Crate | Responsibility |
| --- | --- |
| `ferromark` | Public entry point |
| `ferromark_allocator` | Arena allocation and buffer helpers |
| `ferromark_ast` | Nodes, source spans, visitors |
| `ferromark_parser` | Markdown to AST |
| `ferromark_renderer` | AST to HTML |

CommonMark/GFM, the existing optional syntax extensions, rendering options, and
core tests are retained. Site generation, JavaScript frameworks, bindings, editor
services, and the upstream profiler are removed. See the precise
[cleanup boundary](docs/fork.md) and [source provenance](UPSTREAM.md).

Build and verify with the pinned Rust toolchain:

```sh
cargo fmt --all --check
cargo test --workspace --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo bench --workspace --no-run --locked
```

Try the retained stdin rendering example:

```sh
printf 'Hello, **world**!\n' | cargo run --quiet --locked -p ferromark_renderer --example render_stdin
```

Seven self-contained Criterion suites cover parsing, reference prepasses, tables,
pipe scans, rendering, headings, and sanitized URLs. Run individual suites with
`cargo bench -p ferromark_parser --bench table_pipes --locked` or
`cargo bench -p ferromark_renderer --bench renderer --locked`.
The [optimization roadmap](docs/optimization-roadmap.md) records candidates for
later measured ports from the Ferramenta projects. No such port is included yet.

The [broad Markdown comparison](docs/reports/2026-09-14-broad-markdown/INTERPRETATION.md)
measures 57 cases from 37 bytes to 114 KB: short comments, real documentation,
and Wikipedia-derived prose. It separates input size, content, output agreement,
and fresh/reused lifecycles. [Full tables and raw data](docs/reports/2026-09-14-broad-markdown/README.md)
and the [earlier synthetic diagnostic comparison](docs/reports/2026-09-13-current-ferromark/README.md)
use the same pinned parser binaries.

The source is MIT licensed; the original copyright notice is preserved in
[LICENSE](LICENSE). CommonMark and GFM specification fixtures carry their own
[CC-BY-SA attribution](crates/ferromark_renderer/tests/spec_fixtures/README.md).
The [benchmark corpus sources](benchmarks/broad-comparison/README.md) retain their
separate MIT, Apache, CC BY, or CC BY-SA licenses and attribution.
