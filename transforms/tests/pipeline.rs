#![allow(clippy::panic, clippy::unwrap_used)]

use std::cell::Cell;
use std::error::Error;
use std::fmt;
use std::rc::Rc;

use ferromark::ast::{Document, Node, Span};
use ferromark::{Allocator, HtmlRenderer, HtmlRendererOptions, Parser, ParserOptions, parse};
use ferromark_transforms::{
    BoxError, TextReplacementError, TransformContext, TransformPass, TransformPipeline, text_runs,
};

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
    called: Rc<Cell<bool>>,
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
        self.called.set(true);
        append_to_text(document, context, "partial");
        Err(Box::new(TestFailure))
    }
}

struct CountCalls(Rc<Cell<usize>>);

impl TransformPass for CountCalls {
    fn name(&self) -> &'static str {
        "later-pass"
    }

    fn apply<'arena>(
        &mut self,
        _document: &mut Document<'arena>,
        _context: &TransformContext<'arena>,
    ) -> Result<(), BoxError> {
        self.0.set(self.0.get() + 1);
        Ok(())
    }
}

#[test]
fn empty_pipeline_keeps_rendered_output_unchanged() {
    let source = "A **small** Markdown document with https://example.test/a--b...";
    let arena = Allocator::new();
    let mut document = parse(&arena, source).unwrap();
    let context = TransformContext::new(&arena, source);
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
fn no_op_pass_preserves_output_across_parser_and_renderer_profiles() {
    let source = "# Heading\n\nParagraph :white_check_mark: with https://example.test/a--b... and **bold**.\n\n| A | B |\n| - | - |\n| one | [link](/target) |\n";
    let mut pipeline = TransformPipeline::new();
    pipeline.add(Noop);

    for parser_options in [ParserOptions::default(), ParserOptions::gfm()] {
        let arena = Allocator::new();
        let mut document = Parser::with_options(&arena, source, parser_options)
            .parse()
            .unwrap();
        let context = TransformContext::new(&arena, source);

        for renderer_options in [
            HtmlRendererOptions::default(),
            HtmlRendererOptions {
                source_spans: true,
                ..HtmlRendererOptions::default()
            },
        ] {
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
    let context = TransformContext::new(&arena, source);
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
    let context = TransformContext::new(&arena, source);
    let failing_called = Rc::new(Cell::new(false));
    let later_calls = Rc::new(Cell::new(0));
    let mut pipeline = TransformPipeline::new();
    pipeline.add(FailingPass {
        called: Rc::clone(&failing_called),
    });
    pipeline.add(CountCalls(Rc::clone(&later_calls)));

    let error = pipeline.run(&mut document, &context).unwrap_err();

    assert!(failing_called.get());
    assert_eq!(error.pass_index(), 0);
    assert_eq!(error.pass_name(), "failing-pass");
    assert_eq!(
        error.to_string(),
        "transform pass 0 (\"failing-pass\") failed: expected failure"
    );
    assert_eq!(later_calls.get(), 0);
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
    let context = TransformContext::new(&arena, source);
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
    let context = TransformContext::new(&arena, source);
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
fn invalid_text_range_leaves_the_ast_unchanged() {
    let source = "日本語";
    let arena = Allocator::new();
    let mut document = parse(&arena, source).unwrap();
    let context = TransformContext::new(&arena, source);
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
fn url_protection_is_explicit_and_matches_renderer_ranges() {
    let source = "https://host/:smile: and ftp://host/file.";
    let arena = Allocator::new();
    let context = TransformContext::new(&arena, source);

    let ranges = context.protected_url_ranges(source);
    assert_eq!(ranges.len(), 1);
    assert_eq!(ranges[0], 0..source.find(": and").unwrap());

    let ranges = context.protected_url_ranges_with_patterns(source, &["ftp://"]);
    assert_eq!(ranges.len(), 1);
    assert_eq!(ranges[0], source.rfind("ftp://").unwrap()..source.len() - 1);
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
            let context = TransformContext::new(&arena, source);
            pipeline.run(&mut document, &context).unwrap();
            assert_eq!(renderer.render(&document), format!("<p>{source}!</p>\n"));
        }
        arena.reset();
    }
}
