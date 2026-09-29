//! Run with `cargo run --features ferriki --example ferriki -- /path/to/assets/shiki`.

#[cfg(feature = "ferriki")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{self, Write};
    use std::path::Path;

    use ferromark::ferriki::{Highlighter, StandardAssetCatalogs};
    use ferromark::{Allocator, FerrikiHighlightHooks, HtmlRenderer, Parser};

    let asset_root = std::env::args().nth(1).ok_or("pass a Ferriki asset root")?;
    let assets = StandardAssetCatalogs::load_from_root(Path::new(&asset_root))?;
    let mut highlighter = Highlighter::builder()
        .with_assets(assets)
        .load_languages(["rust"])
        .load_themes(["nord"])
        .build()?;
    let source = "```rust\nfn main() {}\n```\n\n    indented & plain\n\n```unknown\n<plain>\n```";
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, source).parse()?;
    let mut renderer = HtmlRenderer::new();
    let mut hooks = FerrikiHighlightHooks::new(&mut highlighter, "nord");
    io::stdout().write_all(renderer.render_with_hooks(&document, &mut hooks).as_bytes())?;
    Ok(())
}

#[cfg(not(feature = "ferriki"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    Err("enable the `ferriki` Cargo feature to run this example".into())
}
