use ferromark_allocator::Allocator;
use ferromark_parser::{Parser, ParserOptions};
use ferromark_renderer::{HtmlRenderer, HtmlRendererOptions};

fn render(source: &str) -> String {
    render_with_options(source, ParserOptions::gfm())
}

fn render_with_options(source: &str, parser_options: ParserOptions) -> String {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, parser_options)
        .parse()
        .expect("fixture input should parse");
    let options = HtmlRendererOptions { autolink_urls: false, ..HtmlRendererOptions::new() };
    HtmlRenderer::with_options(options).render(&document)
}

#[test]
fn single_tilde_strikethrough_stays_literal_when_gfm_is_disabled() {
    assert_eq!(render_with_options("~there~\n", ParserOptions::default()), "<p>~there~</p>\n");
}

#[test]
fn gfm_official_strikethrough_examples_491_and_493() {
    assert_eq!(
        render("~~Hi~~ Hello, ~there~ world!\n"),
        "<p><del>Hi</del> Hello, <del>there</del> world!</p>\n"
    );
    assert_eq!(render("This will ~~~not~~~ strike.\n"), "<p>This will ~~~not~~~ strike.</p>\n");
}

#[test]
fn gfm_official_extended_autolinks_examples_633_to_635() {
    assert_eq!(
        render(
            "mailto:foo@bar.baz\n\nmailto:a.b-c_d@a.b\n\nmailto:a.b-c_d@a.b.\n\nmailto:a.b-c_d@a.b/\n\nmailto:a.b-c_d@a.b-\n\nmailto:a.b-c_d@a.b_\n\nxmpp:foo@bar.baz\n\nxmpp:foo@bar.baz.\n"
        ),
        "<p><a href=\"mailto:foo@bar.baz\">mailto:foo@bar.baz</a></p>\n<p><a href=\"mailto:a.b-c_d@a.b\">mailto:a.b-c_d@a.b</a></p>\n<p><a href=\"mailto:a.b-c_d@a.b\">mailto:a.b-c_d@a.b</a>.</p>\n<p><a href=\"mailto:a.b-c_d@a.b\">mailto:a.b-c_d@a.b</a>/</p>\n<p>mailto:a.b-c_d@a.b-</p>\n<p>mailto:a.b-c_d@a.b_</p>\n<p><a href=\"xmpp:foo@bar.baz\">xmpp:foo@bar.baz</a></p>\n<p><a href=\"xmpp:foo@bar.baz\">xmpp:foo@bar.baz</a>.</p>\n"
    );
    assert_eq!(
        render(
            "xmpp:foo@bar.baz/txt\n\nxmpp:foo@bar.baz/txt@bin\n\nxmpp:foo@bar.baz/txt@bin.com\n"
        ),
        "<p><a href=\"xmpp:foo@bar.baz/txt\">xmpp:foo@bar.baz/txt</a></p>\n<p><a href=\"xmpp:foo@bar.baz/txt@bin\">xmpp:foo@bar.baz/txt@bin</a></p>\n<p><a href=\"xmpp:foo@bar.baz/txt@bin.com\">xmpp:foo@bar.baz/txt@bin.com</a></p>\n"
    );
    assert_eq!(
        render("xmpp:foo@bar.baz/txt/bin\n"),
        "<p><a href=\"xmpp:foo@bar.baz/txt\">xmpp:foo@bar.baz/txt</a>/bin</p>\n"
    );
}

#[test]
fn gfm_extended_autolinks_require_contiguous_addresses() {
    assert_eq!(
        render("mailto: prose a@b.com\n\nxmpp: prose a@b.com\n"),
        "<p>mailto: prose <a href=\"mailto:a@b.com\">a@b.com</a></p>\n<p>xmpp: prose <a href=\"mailto:a@b.com\">a@b.com</a></p>\n"
    );
}

#[test]
fn gfm_single_tilde_ignores_escaped_and_code_span_tildes() {
    assert_eq!(render("~a\\~ b~\n"), "<p><del>a~ b</del></p>\n");
    assert_eq!(render("~a `~` b~\n"), "<p><del>a <code>~</code> b</del></p>\n");
}

#[test]
fn explicit_subscript_keeps_priority_over_single_tilde_strikethrough() {
    let options =
        ParserOptions { strikethrough: true, subscript: true, ..ParserOptions::default() };
    assert_eq!(render_with_options("~there~\n", options), "<p><sub>there</sub></p>\n");
}
