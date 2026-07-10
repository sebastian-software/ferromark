use ferromark::{FencedCodeBlock, FencedCodeRenderer, Options, TrustedHtml, to_html_with_renderer};

struct RustCodeRenderer;

impl FencedCodeRenderer for RustCodeRenderer {
    fn render(&mut self, block: FencedCodeBlock<'_>) -> Option<TrustedHtml> {
        if block.language != Some("rust") {
            return None;
        }

        let mut escaped = Vec::with_capacity(block.code.len());
        ferromark::escape::escape_text_into(&mut escaped, block.code.as_bytes());
        let escaped = String::from_utf8(escaped).expect("HTML escaping preserves UTF-8");

        // TrustedHtml is written verbatim. A production renderer must escape
        // every untrusted value it includes, as this example does for `code`.
        Some(TrustedHtml::from_trusted(format!(
            "<pre class=\"highlighted\"><code>{escaped}</code></pre>\n"
        )))
    }
}

fn main() {
    let markdown = "```rust\nlet answer = 42;\n```";
    let html = to_html_with_renderer(markdown, &Options::default(), &mut RustCodeRenderer);
    print!("{html}");
}
