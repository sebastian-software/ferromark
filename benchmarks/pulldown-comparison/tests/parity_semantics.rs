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

#[test]
fn plain_table_benchmark_should_contain_only_table_structure_and_text() {
    let input = include_str!("../../../benches/fixtures/tables-plain.md");
    let options = pulldown_options(ParityConfig::TablesOnly);
    assert_eq!(options, PulldownOptions::ENABLE_TABLES);
    let mut tables = 0;
    let mut cells = 0;
    for event in pulldown_cmark::Parser::new_ext(input, options) {
        use pulldown_cmark::{Event, Tag, TagEnd};
        match event {
            Event::Start(Tag::Table(_)) => tables += 1,
            Event::Start(Tag::TableCell) => cells += 1,
            Event::Start(Tag::TableHead | Tag::TableRow)
            | Event::End(
                TagEnd::Table | TagEnd::TableHead | TagEnd::TableRow | TagEnd::TableCell,
            ) => {}
            Event::Text(text) => {
                assert!(text.chars().all(|ch| ch.is_ascii_alphabetic() || ch == ' '))
            }
            other => panic!("plain table benchmark contains unrelated syntax: {other:?}"),
        }
    }
    assert_eq!((tables, cells), (80, 320));
    let (ferromark, pulldown) = render_both(input, ParityConfig::TablesOnly);
    for html in [ferromark, pulldown] {
        assert_eq!(html.matches("<table>").count(), 80);
        assert_eq!(html.matches("<td>").count(), 160);
        assert_eq!(html.matches("<th>").count(), 160);
    }
}

#[test]
fn ordinary_table_options_should_render_commonmark_emphasis() {
    let input = include_str!("../../../benches/fixtures/tables-commonmark-inline.md");
    let config = ParityConfig::TablesOnly;
    let options = ferromark_options(config);
    assert!(options.tables);
    assert!(!options.strikethrough && !options.task_lists);
    assert_eq!(pulldown_options(config), PulldownOptions::ENABLE_TABLES);
    assert!(!input.contains('~'));
    let (ferromark, pulldown) = render_both(input, config);
    for html in [ferromark, pulldown] {
        assert_eq!(html.matches("<table>").count(), 80);
        assert_eq!(html.matches("<th>").count(), 160);
        assert_eq!(html.matches("<td>").count(), 160);
        assert_eq!(html.matches("<em>").count(), 40);
        assert_eq!(html.matches("<strong>").count(), 40);
        assert!(!html.contains("<del>"));
    }
}

#[test]
fn table_link_column_should_render_links_with_ordinary_table_options() {
    let input = include_str!("../../../benches/fixtures/tables-links.md");
    let (ferromark, pulldown) = render_both(input, ParityConfig::TablesOnly);
    for html in [ferromark, pulldown] {
        assert_eq!(html.matches("<table>").count(), 80);
        assert_eq!(html.matches("<th>").count(), 160);
        assert_eq!(html.matches("<td>").count(), 160);
        assert_eq!(
            html.matches("<a href=\"https://example.com/guide\">Guide</a>")
                .count(),
            80
        );
        assert_eq!(html.matches("<em>").count(), 20);
        assert_eq!(html.matches("<strong>").count(), 20);
    }
}
