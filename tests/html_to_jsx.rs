//! Trusted HTML replacements converted to JSX, and code block locations offered
//! to the HTML highlighting stage.

use ferromark::ast::Span;
use ferromark::{
    Allocator, CodeHighlightInput, HighlightedCodeBlock, HtmlRenderHooks, HtmlRenderer,
    HtmlRendererOptions, Parser, html_to_jsx,
};

#[test]
fn svg_attributes_keep_their_case_and_camel_case_hyphenated_names() {
    let svg = concat!(
        r#"<svg viewBox="0 0 10 10" class="diagram" xmlns:xlink="http://www.w3.org/1999/xlink">"#,
        r##"<marker markerWidth="3" refX="1"/><path stroke-width="2" xlink:href="#a" "##,
        r#"data-node="A" aria-label="Edge" xml:space="preserve"/></svg>"#,
    );
    assert_eq!(
        html_to_jsx(svg),
        concat!(
            r#"<svg viewBox="0 0 10 10" className="diagram" xmlnsXlink="http://www.w3.org/1999/xlink">"#,
            r##"<marker markerWidth="3" refX="1" /><path strokeWidth="2" xlinkHref="#a" "##,
            r#"data-node="A" aria-label="Edge" xmlSpace="preserve" /></svg>"#,
        )
    );
}

#[test]
fn html_attributes_text_styles_and_comments_follow_raw_html_rules() {
    assert_eq!(
        html_to_jsx(
            r#"<div class="a" tabindex="0" style="font-size:2px"><!-- note -->a &amp; b<br></div>"#
        ),
        r#"<div className="a" tabIndex="0" style={{fontSize: "2px"}}>{/* note */}{"a & b"}<br /></div>"#
    );
    assert_eq!(
        html_to_jsx("<style>.a>b { color: red; }</style>"),
        r#"<style>{".a>b { color: red; }"}</style>"#
    );
}

#[derive(Default)]
struct SpanRecorder {
    spans: Vec<Span>,
}

impl HtmlRenderHooks for SpanRecorder {
    fn highlight_code_block(
        &mut self,
        input: CodeHighlightInput<'_>,
    ) -> Option<HighlightedCodeBlock> {
        self.spans.push(input.span);
        None
    }
}

#[test]
fn the_highlighting_stage_receives_the_complete_block_span() {
    let source = "é\n\n```ts\nlet a\n```\n\n    indented\n";
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, source).parse().unwrap();
    for options in [
        HtmlRendererOptions::default(),
        HtmlRendererOptions {
            code_annotations: true,
            code_fence_metadata: true,
            ..HtmlRendererOptions::default()
        },
    ] {
        let mut hooks = SpanRecorder::default();
        let _ = HtmlRenderer::with_options(options).render_with_code_hooks(&document, &mut hooks);
        let blocks: Vec<&str> = hooks
            .spans
            .iter()
            .map(|span| &source[span.start as usize..span.end as usize])
            .collect();
        assert_eq!(blocks, ["```ts\nlet a\n```\n", "    indented\n"]);
    }
}
