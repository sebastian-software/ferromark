//! Continuous checks for the gaps discovered by the September 2026 audit.
#[path = "spec_support/normalize.rs"]
mod normalize;
#[path = "spec_support/spec_txt.rs"]
mod spec_txt;

use ferromark_allocator::Allocator;
use ferromark_parser::{Parser, ParserOptions};
use ferromark_renderer::{HtmlRenderer, HtmlRendererOptions};

fn render(source: &str, gfm: bool) -> String {
    let allocator = Allocator::new();
    let mut options = if gfm {
        ParserOptions::gfm()
    } else {
        ParserOptions::default()
    };
    options.footnotes = false;
    let document = Parser::with_options(&allocator, source, options)
        .parse()
        .expect("valid Markdown");
    HtmlRenderer::with_options(HtmlRendererOptions {
        autolink_urls: false,
        autolink_target_blank: false,
        link_target_blank: false,
        disallow_raw_html: gfm,
        ..HtmlRendererOptions::new()
    })
    .render(&document)
}

#[test]
fn all_commonmark_examples_preserve_meaning_with_crlf_and_cr() {
    let examples = spec_txt::parse_spec(include_str!("spec_fixtures/commonmark-0.31.2-spec.txt"));
    assert_eq!(examples.len(), 652);
    for example in examples {
        let lf = render(&example.markdown, false);
        for ending in ["\r\n", "\r"] {
            let input = example.markdown.replace('\n', ending);
            let output = render(&input, false)
                .replace("\r\n", "\n")
                .replace('\r', "\n");
            assert_eq!(
                output, lf,
                "example {} ({}) with {ending:?}",
                example.number, example.section
            );
        }
    }
}

#[test]
fn all_current_gfm_extension_examples_match() {
    // Separately attributed frozen 2026-09-14 extraction; old fixtures stay intact.
    let examples: serde_json::Value = serde_json::from_str(include_str!(
        "../../../benchmarks/compatibility-audit/fixtures/gfm-examples.json"
    ))
    .unwrap();
    let examples = examples.as_array().unwrap();
    assert_eq!(examples.len(), 677);
    let mut checked = 0;
    for example in examples {
        let section = example["section"].as_str().unwrap();
        if !section.contains("(extension)") {
            continue;
        }
        let actual = render(example["markdown"].as_str().unwrap(), true);
        assert_eq!(
            normalize::normalize_html(&actual),
            normalize::normalize_html(example["html"].as_str().unwrap()),
            "GFM example {} ({section})",
            example["example"]
        );
        checked += 1;
    }
    assert_eq!(checked, 28);
}

#[test]
fn commonmark_gfm_exceptions_keep_their_expected_links() {
    // These three differences are intentional profile choices, not an allowance
    // for arbitrary output changes or panics on known-failure example numbers.
    let examples = spec_txt::parse_spec(include_str!("spec_fixtures/commonmark-0.31.2-spec.txt"));
    for (number, expected) in [
        (
            608,
            "<p>&lt; <a href=\"https://foo.bar\">https://foo.bar</a> &gt;</p>\n",
        ),
        (
            611,
            "<p><a href=\"https://example.com\">https://example.com</a></p>\n",
        ),
        (
            612,
            "<p><a href=\"mailto:foo@bar.example.com\">foo@bar.example.com</a></p>\n",
        ),
    ] {
        let example = &examples[number - 1];
        assert_eq!(
            normalize::normalize_html(&render(&example.markdown, true)),
            normalize::normalize_html(expected),
            "example {number}"
        );
    }
}
