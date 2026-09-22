use ferromark::{Allocator, HtmlRenderer, NoHtmlRenderHooks, Parser, map_heading_level};

#[test]
fn heading_level_mapping_clamps_after_applying_the_offset() {
    assert_eq!(map_heading_level(1, 0), 1);
    assert_eq!(map_heading_level(3, 2), 5);
    assert_eq!(map_heading_level(6, 1), 6);
    assert_eq!(map_heading_level(1, -1), 1);
    assert_eq!(map_heading_level(6, -3), 3);
    assert_eq!(map_heading_level(0, 0), 1);
    assert_eq!(map_heading_level(9, 0), 6);
    assert_eq!(map_heading_level(1, i32::MAX), 6);
    assert_eq!(map_heading_level(6, i32::MIN), 1);
}

#[test]
fn heading_offset_is_shared_by_normal_hooked_and_fragment_rendering() {
    let source = "# Top\n\n## Middle\n\n###### Deep";
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, source).parse().expect("parses");
    let expected =
        "<h2 id=\"top\">Top</h2>\n<h3 id=\"middle\">Middle</h3>\n<h6 id=\"deep\">Deep</h6>\n";

    let mut normal = HtmlRenderer::new().with_heading_level_offset(1);
    assert_eq!(normal.render(&document), expected);

    let mut hooked = HtmlRenderer::new().with_heading_level_offset(1);
    assert_eq!(
        hooked.render_with_hooks(&document, &mut NoHtmlRenderHooks),
        expected
    );

    let mut incremental = HtmlRenderer::new().with_heading_level_offset(1);
    for (source, expected_fragment) in [
        ("# First", "<h2 id=\"first\">First</h2>\n"),
        ("## Second", "<h3 id=\"second\">Second</h3>\n"),
    ] {
        let fragment_allocator = Allocator::new();
        let fragment = Parser::new(&fragment_allocator, source)
            .parse()
            .expect("fragment parses");
        assert_eq!(
            incremental.render_incremental_fragment(&fragment),
            expected_fragment
        );
    }
    incremental.reset_incremental_state();
    let reset_allocator = Allocator::new();
    let reset_fragment = Parser::new(&reset_allocator, "# After reset")
        .parse()
        .expect("fragment parses");
    assert_eq!(
        incremental.render_incremental_fragment(&reset_fragment),
        "<h2 id=\"after-reset\">After reset</h2>\n"
    );
}
