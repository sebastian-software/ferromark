use ferromark::{Options, RenderPolicy, to_html_with_options};

#[test]
fn minimal_should_disable_every_optional_syntax_feature() {
    let options = Options::minimal();

    assert_eq!(
        options,
        Options {
            render_policy: RenderPolicy::Untrusted,
            allow_html: false,
            allow_link_refs: false,
            tables: false,
            strikethrough: false,
            highlight: false,
            superscript: false,
            subscript: false,
            task_lists: false,
            autolink_literals: false,
            disallowed_raw_html: false,
            footnotes: false,
            front_matter: false,
            heading_ids: false,
            math: false,
            callouts: false,
        }
    );
}

#[test]
fn commonmark_should_enable_only_commonmark_syntax_boundaries() {
    let options = Options::commonmark();

    assert!(options.allow_html && options.allow_link_refs);
    assert_eq!(
        options,
        Options {
            allow_html: true,
            allow_link_refs: true,
            ..Options::minimal()
        }
    );
}

#[test]
fn gfm_should_extend_commonmark_with_exact_gfm_extensions() {
    let options = Options::gfm();

    assert_eq!(
        options,
        Options {
            tables: true,
            strikethrough: true,
            task_lists: true,
            autolink_literals: true,
            disallowed_raw_html: true,
            ..Options::commonmark()
        }
    );
}

#[test]
fn syntax_presets_should_keep_untrusted_rendering() {
    for options in [Options::minimal(), Options::commonmark(), Options::gfm()] {
        assert_eq!(options.render_policy, RenderPolicy::Untrusted);
    }
}

#[test]
fn default_should_retain_the_existing_feature_mix() {
    assert_eq!(
        Options::default(),
        Options {
            render_policy: RenderPolicy::Untrusted,
            allow_html: true,
            allow_link_refs: true,
            tables: true,
            strikethrough: true,
            highlight: false,
            superscript: false,
            subscript: false,
            task_lists: true,
            autolink_literals: false,
            disallowed_raw_html: true,
            footnotes: false,
            front_matter: false,
            heading_ids: true,
            math: false,
            callouts: true,
        }
    );
}

#[test]
fn minimal_should_leave_reference_links_unresolved() {
    let markdown = "Read [the guide][guide].\n\n[guide]: https://example.com";
    let html = to_html_with_options(markdown, &Options::minimal());

    assert!(!html.contains("href=\"https://example.com\""));
}

#[test]
fn commonmark_should_resolve_reference_links() {
    let markdown = "Read [the guide][guide].\n\n[guide]: https://example.com";
    let html = to_html_with_options(markdown, &Options::commonmark());

    assert!(html.contains("href=\"https://example.com\""));
}

#[test]
fn commonmark_should_parse_html_without_implicitly_trusting_it() {
    let markdown = "<div>content</div>";
    let untrusted = to_html_with_options(markdown, &Options::commonmark());
    let trusted = to_html_with_options(
        markdown,
        &Options {
            render_policy: RenderPolicy::Trusted,
            ..Options::commonmark()
        },
    );

    assert_eq!(untrusted, "&lt;div&gt;content&lt;/div&gt;");
    assert_eq!(trusted, "<div>content</div>");
}

#[test]
fn gfm_should_render_its_extension_set() {
    let markdown =
        "~~done~~ at https://example.com\n\n- [x] shipped\n\n| A | B |\n| - | - |\n| 1 | 2 |";
    let html = to_html_with_options(markdown, &Options::gfm());

    assert!(
        html.contains("<del>done</del>")
            && html.contains("<a href=\"https://example.com\">")
            && html.contains("type=\"checkbox\"")
            && html.contains("<table>")
    );
}
