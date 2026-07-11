use ferromark::{Options, Profile, RenderPolicy, to_html_with_options};

#[test]
fn essentials_should_enable_everyday_markdown_extensions() {
    let options = Options::from(Profile::Essentials);

    assert!(options.tables && options.strikethrough && options.task_lists);
}

#[test]
fn essentials_should_disable_expensive_and_specialized_features() {
    let options = Options::from(Profile::Essentials);

    assert!(
        !options.allow_html
            && !options.allow_link_refs
            && !options.autolink_literals
            && !options.footnotes
            && !options.front_matter
            && !options.heading_ids
            && !options.math
            && !options.callouts
            && !options.highlight
            && !options.subscript
            && !options.superscript
    );
}

#[test]
fn extended_should_match_the_existing_default_contract() {
    assert_eq!(Options::from(Profile::Extended), Options::default());
}

#[test]
fn full_should_enable_every_supported_feature() {
    let options = Options::from(Profile::Full);

    assert!(
        options.allow_html
            && options.allow_link_refs
            && options.tables
            && options.strikethrough
            && options.highlight
            && options.superscript
            && options.subscript
            && options.task_lists
            && options.autolink_literals
            && options.disallowed_raw_html
            && options.footnotes
            && options.front_matter
            && options.heading_ids
            && options.math
            && options.callouts
    );
}

#[test]
fn profiles_should_keep_untrusted_rendering() {
    for profile in [Profile::Essentials, Profile::Extended, Profile::Full] {
        assert_eq!(
            Options::from(profile).render_policy,
            RenderPolicy::Untrusted
        );
    }
}

#[test]
fn essentials_should_render_its_extension_set() {
    let markdown = "~~done~~\n\n- [x] shipped\n\n| A | B |\n| - | - |\n| 1 | 2 |";
    let html = to_html_with_options(markdown, &Options::from(Profile::Essentials));

    assert!(
        html.contains("<del>done</del>")
            && html.contains("type=\"checkbox\"")
            && html.contains("<table>")
    );
}

#[test]
fn essentials_should_leave_reference_links_unresolved() {
    let markdown = "Read [the guide][guide].\n\n[guide]: https://example.com";
    let html = to_html_with_options(markdown, &Options::from(Profile::Essentials));

    assert!(!html.contains("href=\"https://example.com\""));
}

#[test]
fn extended_should_resolve_reference_links() {
    let markdown = "Read [the guide][guide].\n\n[guide]: https://example.com";
    let html = to_html_with_options(markdown, &Options::from(Profile::Extended));

    assert!(html.contains("href=\"https://example.com\""));
}

#[test]
fn full_should_render_specialized_extensions() {
    let markdown = "==mark== and H~2~O and x^2^ and $a+b$.";
    let html = to_html_with_options(markdown, &Options::from(Profile::Full));

    assert!(
        html.contains("<mark>mark</mark>")
            && html.contains("<sub>2</sub>")
            && html.contains("<sup>2</sup>")
            && html.contains("class=\"language-math math-inline\"")
    );
}
