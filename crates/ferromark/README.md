# ferromark

Markdown parsing and HTML rendering with an arena-allocated syntax tree. This is the main Rust entry point for Ferromark.

This package is part of [Ferromark](https://github.com/sebastian-software/ferromark).
See the [API documentation](https://docs.rs/ferromark) and
[v2 migration guide](https://github.com/sebastian-software/ferromark/blob/main/docs/migration-v2.md).

```rust
fn main() -> Result<(), ferromark::ParseError> {
    let html = ferromark::to_html("Hello, **world**!")?;
    assert_eq!(html, "<p>Hello, <strong>world</strong>!</p>\n");
    Ok(())
}
```

## License and attribution

MIT licensed. Ferromark v2 derives from
[OX-Content](https://github.com/ubugeeei-prod/ox-content) by ubugeeei.
The original copyright and permission notice is included in [LICENSE](LICENSE).
