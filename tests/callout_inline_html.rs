//! Callout bodies render their inline content on the inline path.
//!
//! The marker paragraph of a callout is rendered by a dedicated routine that
//! strips `[!KIND]`. It used to hand every non-text child to the block
//! renderer, so an inline raw HTML node inside the body took the HTML
//! *block* path and gained a line break after every fragment:
//! `> [!NOTE]\n> <b>x</b> y` rendered `<p><b>\nx</b>\n y</p>`, while the
//! hooks path rendered the same document as `<p><b>x</b> y</p>`.

use ferromark::{
    Allocator, HtmlRenderer, HtmlRendererOptions, NoHtmlRenderHooks, Parser, ParserOptions,
};

fn render(source: &str, options: HtmlRendererOptions) -> (String, String) {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, ParserOptions::gfm())
        .parse()
        .expect("source should parse");
    let mut renderer = HtmlRenderer::with_options(options.clone());
    let plain = renderer.render(&document);
    let mut hooked = HtmlRenderer::with_options(options);
    let with_hooks = hooked.render_with_hooks(&document, &mut NoHtmlRenderHooks);
    (plain, with_hooks)
}

#[test]
fn inline_html_in_a_callout_body_stays_inline() {
    let (plain, with_hooks) = render("> [!NOTE]\n> <b>x</b> y\n", HtmlRendererOptions::new());
    assert_eq!(
        plain,
        "<blockquote class=\"ox-callout ox-callout--note\">\n<p class=\"ox-callout-title\">Note</p>\n<p><b>x</b> y</p>\n</blockquote>\n"
    );
    assert_eq!(plain, with_hooks);
}

#[test]
fn sanitized_inline_html_in_a_callout_body_stays_inline() {
    let options = HtmlRendererOptions {
        sanitize: true,
        ..HtmlRendererOptions::new()
    };
    let (plain, with_hooks) = render("> [!TIP]\n> <b>x</b> y\n", options);
    assert!(plain.contains("<p>&lt;b&gt;x&lt;/b&gt; y</p>"), "{plain}");
    assert_eq!(plain, with_hooks);
}

#[test]
fn a_custom_soft_break_reaches_every_callout_line() {
    let options = HtmlRendererOptions {
        soft_break: "<br />\n".into(),
        ..HtmlRendererOptions::new()
    };
    let (plain, with_hooks) = render("> [!NOTE]\n> a\n> b\n> *c*\n> d\n", options);
    assert!(
        plain.contains("<p>a<br />\nb<br />\n<em>c</em><br />\nd</p>"),
        "{plain}"
    );
    assert_eq!(plain, with_hooks);
}
