use ferromark_allocator::Allocator;
use ferromark_parser::{Parser, ParserOptions};
use ferromark_renderer::{HtmlRenderer, HtmlRendererOptions};

fn render(source: &str) -> String {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, ParserOptions::default())
        .parse()
        .unwrap();
    HtmlRenderer::with_options(HtmlRendererOptions {
        autolink_urls: false,
        link_target_blank: false,
        autolink_target_blank: false,
        ..HtmlRendererOptions::new()
    })
    .render(&document)
}

#[test]
fn matches_cmark_unicode_url_and_leading_bom_serialization() {
    assert_eq!(
        render("[中文](https://例え.テスト/道) and `é`\n"),
        "<p><a href=\"https://%E4%BE%8B%E3%81%88.%E3%83%86%E3%82%B9%E3%83%88/%E9%81%93\">中文</a> and <code>é</code></p>\n",
    );
    assert_eq!(
        render("\u{feff} BOM and nbsp\u{00a0}space\n"),
        "<p>BOM and nbsp\u{00a0}space</p>\n",
    );
}

#[test]
fn keeps_existing_percent_escapes_and_valid_ipv6_brackets() {
    assert_eq!(
        render("[x](https://例え.テスト/a%23b)\n"),
        "<p><a href=\"https://%E4%BE%8B%E3%81%88.%E3%83%86%E3%82%B9%E3%83%88/a%23b\">x</a></p>\n",
    );
    assert_eq!(
        render("[x](https://[::1]/道)\n"),
        "<p><a href=\"https://[::1]/%E9%81%93\">x</a></p>\n",
    );
}
