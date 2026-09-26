#![allow(clippy::panic, clippy::unwrap_used)]

use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use ferromark::ast::{Document, Node, Span, Text};
use ferromark::{Allocator, HtmlRenderer, HtmlRendererOptions, Parser, ParserOptions, parse};
use ferromark_transforms::{
    BoxError, TextReplacementError, TransformContext, TransformError, TransformPass,
    TransformPipeline, text_runs,
};

fn default_context<'arena>(
    allocator: &'arena Allocator,
    source: &'arena str,
) -> TransformContext<'arena> {
    let options = HtmlRendererOptions::new();
    TransformContext::new(allocator, source, &options)
}

fn append_to_text<'arena>(
    document: &mut Document<'arena>,
    context: &TransformContext<'arena>,
    suffix: &str,
) {
    for node in &mut document.children {
        let Node::Paragraph(paragraph) = node else {
            continue;
        };
        let Some(Node::Text(text)) = paragraph.children.last_mut() else {
            continue;
        };
        let value = format!("{}{suffix}", text.value);
        context.replace_text_value(text, &value);
    }
}

fn anchor(html: &str) -> &str {
    let start = html.find("<a ").unwrap();
    let end = html.find("</a>").unwrap() + "</a>".len();
    &html[start..end]
}

struct Append(&'static str);

impl TransformPass for Append {
    fn name(&self) -> &'static str {
        self.0
    }

    fn apply<'arena>(
        &mut self,
        document: &mut Document<'arena>,
        context: &TransformContext<'arena>,
    ) -> Result<(), BoxError> {
        append_to_text(document, context, self.0);
        Ok(())
    }
}

struct Noop;

impl TransformPass for Noop {
    fn name(&self) -> &'static str {
        "no-op"
    }

    fn apply<'arena>(
        &mut self,
        _document: &mut Document<'arena>,
        _context: &TransformContext<'arena>,
    ) -> Result<(), BoxError> {
        Ok(())
    }
}

#[derive(Debug)]
struct TestFailure;

impl fmt::Display for TestFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("expected failure")
    }
}

impl Error for TestFailure {}

struct FailingPass {
    called: Arc<AtomicBool>,
}

impl TransformPass for FailingPass {
    fn name(&self) -> &'static str {
        "failing-pass"
    }

    fn apply<'arena>(
        &mut self,
        document: &mut Document<'arena>,
        context: &TransformContext<'arena>,
    ) -> Result<(), BoxError> {
        self.called.store(true, Ordering::Relaxed);
        append_to_text(document, context, "partial");
        Err(Box::new(TestFailure))
    }
}

struct CountCalls(Arc<AtomicUsize>);

impl TransformPass for CountCalls {
    fn name(&self) -> &'static str {
        "later-pass"
    }

    fn apply<'arena>(
        &mut self,
        _document: &mut Document<'arena>,
        _context: &TransformContext<'arena>,
    ) -> Result<(), BoxError> {
        self.0.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}

#[test]
fn empty_pipeline_keeps_rendered_output_unchanged() {
    let source = "A **small** Markdown document with https://example.test/a--b...";
    let arena = Allocator::new();
    let mut document = parse(&arena, source).unwrap();
    let context = default_context(&arena, source);
    let mut pipeline = TransformPipeline::new();

    assert!(pipeline.is_empty());
    pipeline.run(&mut document, &context).unwrap();

    let mut renderer = HtmlRenderer::new();
    assert_eq!(
        renderer.render(&document),
        "<p>A <strong>small</strong> Markdown document with <a href=\"https://example.test/a--b\" target=\"_blank\" rel=\"noopener noreferrer\">https://example.test/a--b</a>...</p>\n"
    );
}

#[test]
fn pipeline_and_errors_are_send() {
    fn assert_send<T: Send>() {}

    assert_send::<TransformPipeline>();
    assert_send::<TransformError>();
}

#[test]
fn no_op_pass_preserves_output_across_parser_and_renderer_profiles() {
    let source = "# Heading\n\nParagraph :white_check_mark: with https://example.test/a--b... and **bold**.\n\n| A | B |\n| - | - |\n| one | [link](/target) |\n";
    let mut pipeline = TransformPipeline::new();
    pipeline.add(Noop);

    for parser_options in [ParserOptions::default(), ParserOptions::gfm()] {
        let arena = Allocator::new();
        let mut document = Parser::with_options(&arena, source, parser_options)
            .parse()
            .unwrap();
        for renderer_options in [
            HtmlRendererOptions::default(),
            HtmlRendererOptions {
                source_spans: true,
                ..HtmlRendererOptions::default()
            },
        ] {
            let context = TransformContext::new(&arena, source, &renderer_options);
            let mut renderer = HtmlRenderer::with_options(renderer_options);
            let before = renderer.render(&document);
            pipeline.run(&mut document, &context).unwrap();
            assert_eq!(renderer.render(&document), before);
        }
    }
}

#[test]
fn pipeline_runs_passes_in_the_order_they_were_added() {
    let source = "text";
    let arena = Allocator::new();
    let mut document = parse(&arena, source).unwrap();
    let context = default_context(&arena, source);
    let mut pipeline = TransformPipeline::new();
    pipeline.add(Append("first"));
    pipeline.add(Append("second"));

    assert_eq!(pipeline.len(), 2);
    pipeline.run(&mut document, &context).unwrap();

    let Node::Paragraph(paragraph) = &document.children[0] else {
        panic!("expected a paragraph");
    };
    let Node::Text(text) = &paragraph.children[0] else {
        panic!("expected text");
    };
    assert_eq!(text.value, "textfirstsecond");
    assert_eq!(text.span, Span::new(0, 4));
}

#[test]
fn first_error_identifies_the_pass_and_stops_later_passes() {
    let source = "text";
    let arena = Allocator::new();
    let mut document = parse(&arena, source).unwrap();
    let context = default_context(&arena, source);
    let failing_called = Arc::new(AtomicBool::new(false));
    let later_calls = Arc::new(AtomicUsize::new(0));
    let mut pipeline = TransformPipeline::new();
    pipeline.add(FailingPass {
        called: Arc::clone(&failing_called),
    });
    pipeline.add(CountCalls(Arc::clone(&later_calls)));

    let error = pipeline.run(&mut document, &context).unwrap_err();

    assert!(failing_called.load(Ordering::Relaxed));
    assert_eq!(error.pass_index(), 0);
    assert_eq!(error.pass_name(), "failing-pass");
    assert_eq!(
        error.to_string(),
        "transform pass 0 (\"failing-pass\") failed: expected failure"
    );
    assert_eq!(later_calls.load(Ordering::Relaxed), 0);
    // A failing pass may have changed the document; the pipeline does not roll it back.
    let Node::Paragraph(paragraph) = &document.children[0] else {
        panic!("expected a paragraph");
    };
    let Node::Text(text) = &paragraph.children[0] else {
        panic!("expected text");
    };
    assert_eq!(text.value, "textpartial");
}

#[test]
fn text_runs_coalesce_adjacent_nodes_and_keep_each_segment_span() {
    let source = ":white_check_mark:";
    let arena = Allocator::new();
    let document = parse(&arena, source).unwrap();
    let Node::Paragraph(paragraph) = &document.children[0] else {
        panic!("expected a paragraph");
    };

    let runs = text_runs(&paragraph.children).collect::<Vec<_>>();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].value(), source);
    assert_eq!(runs[0].source_span(), Span::new(0, source.len() as u32));
    assert_eq!(runs[0].segments().count(), 5);
}

#[test]
fn replacing_a_range_across_text_segments_keeps_its_bounding_source_span() {
    let source = ":white_check_mark:";
    let arena = Allocator::new();
    let mut document = parse(&arena, source).unwrap();
    let context = default_context(&arena, source);
    let Node::Paragraph(paragraph) = &mut document.children[0] else {
        panic!("expected a paragraph");
    };
    let node_range = text_runs(&paragraph.children).next().unwrap().node_range();

    let span = context
        .replace_text_range(&mut paragraph.children, node_range, 0..source.len(), "✅")
        .unwrap();

    assert_eq!(span, Span::new(0, source.len() as u32));
    assert_eq!(paragraph.children.len(), 1);
    let Node::Text(text) = &paragraph.children[0] else {
        panic!("expected replacement text");
    };
    assert_eq!(text.value, "✅");
    assert_eq!(text.span, Span::new(0, source.len() as u32));
}

#[test]
fn insertion_gets_an_empty_span_and_unchanged_text_keeps_source_spans() {
    let source = "hello";
    let arena = Allocator::new();
    let mut document = parse(&arena, source).unwrap();
    let context = default_context(&arena, source);
    let Node::Paragraph(paragraph) = &mut document.children[0] else {
        panic!("expected a paragraph");
    };
    let node_range = text_runs(&paragraph.children).next().unwrap().node_range();

    context
        .replace_text_range(&mut paragraph.children, node_range, 2..2, "!")
        .unwrap();

    assert_eq!(paragraph.children.len(), 3);
    let Node::Text(prefix) = &paragraph.children[0] else {
        panic!("expected prefix");
    };
    let Node::Text(inserted) = &paragraph.children[1] else {
        panic!("expected inserted text");
    };
    let Node::Text(suffix) = &paragraph.children[2] else {
        panic!("expected suffix");
    };
    assert_eq!(prefix.span, Span::new(0, 5));
    assert_eq!(inserted.span, Span::empty());
    assert_eq!(suffix.span, Span::new(0, 5));
    assert_eq!(context.generated_text("also inserted").span, Span::empty());
}

#[test]
fn replacing_later_text_preserves_earlier_url_nodes_and_rendered_destination() {
    let source = "Go https://example.test/?a=1&amp;b=2 \"now\"";
    let arena = Allocator::new();
    let mut document = parse(&arena, source).unwrap();
    let renderer_options = HtmlRendererOptions::new();
    let context = TransformContext::new(&arena, source, &renderer_options);
    let mut renderer = HtmlRenderer::with_options(renderer_options);
    let before = renderer.render(&document);

    let Node::Paragraph(paragraph) = &mut document.children[0] else {
        panic!("expected a paragraph");
    };
    let run = text_runs(&paragraph.children).next().unwrap();
    let value = run.value();
    let edit_start = value.rfind('"').unwrap();
    let node_range = run.node_range();
    let mut untouched = Vec::new();
    let mut cursor = 0;
    for segment in run.segments() {
        if cursor + segment.value.len() <= edit_start {
            untouched.push((
                segment.value.as_ptr() as usize,
                segment.value.len(),
                segment.span,
            ));
        }
        cursor += segment.value.len();
    }
    drop(value);

    context
        .replace_text_range(
            &mut paragraph.children,
            node_range,
            edit_start..edit_start + 1,
            "”",
        )
        .unwrap();

    let after_segments = text_runs(&paragraph.children)
        .next()
        .unwrap()
        .segments()
        .collect::<Vec<_>>();
    assert!(after_segments.len() >= untouched.len());
    for (segment, expected) in after_segments.iter().zip(untouched.iter()) {
        assert_eq!(
            (
                segment.value.as_ptr() as usize,
                segment.value.len(),
                segment.span
            ),
            *expected
        );
    }

    let after = renderer.render(&document);
    assert!(anchor(&before).contains("href=\"https://example.test/?a=1\""));
    assert_eq!(anchor(&after), anchor(&before));
    assert!(after.contains("now") && after.contains("”"));
}

#[test]
fn invalid_text_range_leaves_the_ast_unchanged() {
    let source = "日本語";
    let arena = Allocator::new();
    let mut document = parse(&arena, source).unwrap();
    let context = default_context(&arena, source);
    let Node::Paragraph(paragraph) = &mut document.children[0] else {
        panic!("expected a paragraph");
    };
    let node_range = text_runs(&paragraph.children).next().unwrap().node_range();
    let before = match &paragraph.children[0] {
        Node::Text(text) => text.value,
        _ => "unexpected node",
    };

    let error = context
        .replace_text_range(&mut paragraph.children, node_range, 1..2, "x")
        .unwrap_err();

    assert_eq!(error, TextReplacementError::NotCharBoundary { offset: 1 });
    let after = match &paragraph.children[0] {
        Node::Text(text) => text.value,
        _ => "unexpected node",
    };
    assert_eq!(after, before);
}

#[test]
fn url_protection_uses_renderer_options_and_scans_each_text_segment() {
    let source = "ahttps://host/path. ftp://host/file.";
    let arena = Allocator::new();
    let split = "ahttps://host/path. ".len();
    let nodes = [
        Node::Text(Text {
            value: "a",
            span: Span::new(0, 1),
        }),
        Node::Text(Text {
            value: "https://host/path. ",
            span: Span::new(1, split as u32),
        }),
        Node::Text(Text {
            value: "ftp://host/file.",
            span: Span::new(split as u32, source.len() as u32),
        }),
    ];
    let run = text_runs(&nodes).next().unwrap();
    assert_eq!(run.value(), source);

    let default_options = HtmlRendererOptions::new();
    let default_context = TransformContext::new(&arena, source, &default_options);
    let default_ranges = run.protected_url_ranges(&default_context);
    assert_eq!(default_ranges.len(), 1);
    assert_eq!(default_ranges[0], 1.."ahttps://host/path".len());

    let mut custom_options = HtmlRendererOptions::new();
    custom_options.autolink_patterns = vec!["ftp://".into()].into();
    let custom_context = TransformContext::new(&arena, source, &custom_options);
    let custom_ranges = run.protected_url_ranges(&custom_context);
    assert_eq!(custom_ranges.len(), 1);
    assert_eq!(custom_ranges[0], split..source.len() - 1);

    let mut disabled_options = HtmlRendererOptions::new();
    disabled_options.autolink_urls = false;
    let disabled_context = TransformContext::new(&arena, source, &disabled_options);
    assert!(run.protected_url_ranges(&disabled_context).is_empty());

    let split_scheme = [
        Node::Text(Text {
            value: "htt",
            span: Span::new(0, 3),
        }),
        Node::Text(Text {
            value: "ps://host/path",
            span: Span::new(3, 17),
        }),
    ];
    let split_run = text_runs(&split_scheme).next().unwrap();
    assert!(split_run.protected_url_ranges(&default_context).is_empty());
}

#[test]
fn pipeline_can_be_reused_after_each_document_is_dropped() {
    let mut arena = Allocator::new();
    let mut pipeline = TransformPipeline::new();
    pipeline.add(Append("!"));
    let mut renderer = HtmlRenderer::new();

    for source in ["first", "second"] {
        {
            let mut document = parse(&arena, source).unwrap();
            let context = default_context(&arena, source);
            pipeline.run(&mut document, &context).unwrap();
            assert_eq!(renderer.render(&document), format!("<p>{source}!</p>\n"));
        }
        arena.reset();
    }
}
