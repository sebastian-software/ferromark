use ferromark_pulldown_comparison::{
    ParityConfig, ferromark_options, pulldown_options, render_ferromark_into, render_pulldown_into,
};
use pulldown_cmark::Options as PulldownOptions;

fn render_both(input: &str, config: ParityConfig) -> (String, String) {
    let mut ferromark = Vec::new();
    render_ferromark_into(input, config, &mut ferromark);
    let ferromark = String::from_utf8(ferromark).expect("ferromark HTML should be UTF-8");

    let mut pulldown = String::new();
    render_pulldown_into(input, config, &mut pulldown);
    (ferromark, pulldown)
}

#[test]
fn commonmark_should_preserve_raw_html_and_resolve_references() {
    let input = "<span>raw</span> and [guide][g].\n\n[g]: https://example.com";
    let (ferromark, pulldown) = render_both(input, ParityConfig::CommonMark);

    for html in [&ferromark, &pulldown] {
        assert!(html.contains("<span>raw</span>") && html.contains("https://example.com"));
    }
}

#[test]
fn gfm_overlap_should_enable_the_shared_extension_set() {
    let input = "~~done~~\n\n- [x] shipped\n\n| A | B |\n| - | - |\n| 1 | 2 |";
    let (ferromark, pulldown) = render_both(input, ParityConfig::GfmOverlap);

    for html in [&ferromark, &pulldown] {
        assert!(
            html.contains("<del>done</del>")
                && html.contains("type=\"checkbox\"")
                && html.contains("<table>")
        );
    }
}

#[test]
fn extended_overlap_should_enable_every_declared_shared_feature() {
    let input =
        "> [!NOTE]\n> Shared callout.\n\nA ^shared^ superscript and $x+y$.[^n]\n\n[^n]: Note.";
    let (ferromark, pulldown) = render_both(input, ParityConfig::ExtendedOverlap);

    for (parser, html) in [("ferromark", &ferromark), ("pulldown-cmark", &pulldown)] {
        assert!(
            html.contains("markdown-alert-note")
                && html.contains("<sup>shared</sup>")
                && html.contains("footnote"),
            "{parser} did not render the declared extended overlap:\n{html}"
        );
    }
    assert!(ferromark.contains("math-inline"));
    assert!(pulldown.contains("math math-inline"));
}

#[test]
fn commonmark_should_not_enable_gfm_extensions() {
    let ferromark = ferromark_options(ParityConfig::CommonMark);
    let pulldown = pulldown_options(ParityConfig::CommonMark);

    assert!(
        !ferromark.tables
            && !ferromark.strikethrough
            && !ferromark.task_lists
            && pulldown.is_empty()
    );
}

#[test]
fn extended_overlap_should_list_every_pulldown_feature_explicitly() {
    let options = pulldown_options(ParityConfig::ExtendedOverlap);
    let expected = PulldownOptions::ENABLE_TABLES
        | PulldownOptions::ENABLE_STRIKETHROUGH
        | PulldownOptions::ENABLE_TASKLISTS
        | PulldownOptions::ENABLE_FOOTNOTES
        | PulldownOptions::ENABLE_MATH
        | PulldownOptions::ENABLE_SUPERSCRIPT
        | PulldownOptions::ENABLE_GFM;

    assert_eq!(options, expected);
}
