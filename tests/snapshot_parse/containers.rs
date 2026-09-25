use ferromark::allocator::Allocator;
use ferromark::parser::{Parser, ParserOptions};
use ferromark::renderer::{HtmlRenderer, HtmlRendererOptions};

use super::check;

fn check_container(name: &str, source: &str, options: ParserOptions) {
    check(name, source, options.clone());

    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, options)
        .parse()
        .expect("container characterization input should parse");
    let html = HtmlRenderer::with_options(HtmlRendererOptions::commonmark()).render(&document);
    insta::with_settings!({
        snapshot_path => "../snapshots/parser",
        prepend_module_to_snapshot => false,
        description => source.to_string(),
        omit_expression => true,
    }, {
        insta::assert_snapshot!(format!("{name}_html"), html);
    });
}

// These exact trees pin the source coordinates around stripped container
// prefixes, lazy lines, tab expansion and transparent comments.

#[test]
fn snapshot_quote_lazy_continuation_after_nested_list() {
    check_container(
        "container_quote_lazy_continuation_after_nested_list",
        "> - first\n>   - nested\n>     child\nlazy continuation\n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_quote_ends_after_fence_before_lazy_line() {
    check_container(
        "container_quote_ends_after_fence_before_lazy_line",
        "> ```\n> code\n> ```\nlazy paragraph\n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_nested_marker_tab_preserves_existing_columns() {
    check_container(
        "container_nested_marker_tab_preserves_existing_columns",
        "> -\tfoo\n>\n>   bar\n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_comments_inside_containers_keep_original_spans() {
    check_container(
        "container_comments_keep_original_spans",
        "> first\n// private quote comment\n> **second**\n\n- first\n// private list comment\n  **second**\n",
        ParserOptions {
            line_comments: true,
            ..ParserOptions::gfm_spec()
        },
    );
}
