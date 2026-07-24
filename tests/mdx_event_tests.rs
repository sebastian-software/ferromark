#![cfg(feature = "mdx")]

use ferromark::mdx::{
    MDX_EVENT_STREAM_VERSION, MdxDiagnosticCode, MdxEvent, parse_events, parse_events_strict,
};
use ferromark::{BlockEvent, InlineEvent};

fn source<'a>(input: &'a str, event: &MdxEvent) -> Option<&'a str> {
    event
        .source_range()
        .map(|range| range.slice_str(input.as_bytes()).unwrap())
}

#[test]
fn semantic_stream_composes_flow_blocks_and_inline_mdx_with_absolute_ranges() {
    let input = "\
---
title: Hello
---
import Card from './Card'

# Hello *world*

<Card />

> Translate {user.name} with <Badge>care</Badge>.

`literal`
";
    let stream = parse_events(input);

    assert_eq!(stream.version, MDX_EVENT_STREAM_VERSION);
    let Some(MdxEvent::FrontMatter { range, content }) = stream.events.first() else {
        panic!("expected front matter as the first event");
    };
    assert_eq!(
        range.slice_str(input.as_bytes()).unwrap(),
        "---\ntitle: Hello\n---\n"
    );
    assert_eq!(
        content.slice_str(input.as_bytes()).unwrap(),
        "title: Hello\n"
    );
    let strict = parse_events_strict(input);
    assert!(strict.is_ok(), "{strict:?}");
    assert!(
        stream
            .events
            .iter()
            .any(|event| matches!(event, MdxEvent::Esm(_)))
    );
    assert!(
        stream
            .events
            .iter()
            .any(|event| matches!(event, MdxEvent::FlowJsxSelfClose(_)))
    );
    assert!(
        stream
            .events
            .iter()
            .any(|event| matches!(event, MdxEvent::Block(BlockEvent::BlockQuoteStart { .. })))
    );
    assert!(
        stream
            .events
            .iter()
            .any(|event| matches!(event, MdxEvent::Inline(InlineEvent::MdxExpression(_))))
    );
    assert!(
        stream
            .events
            .iter()
            .any(|event| matches!(event, MdxEvent::Inline(InlineEvent::MdxJsxOpen(_))))
    );
    assert!(
        stream
            .events
            .iter()
            .any(|event| matches!(event, MdxEvent::Inline(InlineEvent::MdxJsxClose(_))))
    );
    assert!(
        stream
            .events
            .iter()
            .any(|event| matches!(event, MdxEvent::Inline(InlineEvent::Code(_))))
    );

    let expression = stream
        .events
        .iter()
        .find(|event| matches!(event, MdxEvent::Inline(InlineEvent::MdxExpression(_))))
        .unwrap();
    assert_eq!(source(input, expression), Some("{user.name}"));

    for range in stream.events.iter().filter_map(MdxEvent::source_range) {
        assert!(range.end_usize() <= input.len());
        assert!(input.is_char_boundary(range.start_usize()));
        assert!(input.is_char_boundary(range.end_usize()));
    }
}

#[test]
fn reference_consumer_can_collect_translatable_prose_without_code_or_mdx() {
    let input = "\
# Hello *world*

> Translate {user.name} with <Badge>care</Badge>.

`literal`
";
    let stream = parse_events(input);
    let units = stream
        .events
        .iter()
        .filter_map(|event| match event {
            MdxEvent::Inline(InlineEvent::Text(range)) => {
                Some(range.slice_str(input.as_bytes()).unwrap())
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(
        units,
        vec!["Hello ", "world", "Translate ", " with ", "care", "."]
    );
}

#[test]
fn links_keep_absolute_urls_and_reference_definitions() {
    let input = "\
[inline](/docs) and [reference][guide].

<Divider />

[guide]: /guide \"Guide\"
";
    let stream = parse_events(input);

    let inline_url = stream.events.iter().find_map(|event| match event {
        MdxEvent::Inline(InlineEvent::LinkStart { url, .. }) => {
            Some(url.slice_str(input.as_bytes()).unwrap())
        }
        _ => None,
    });
    let reference_index = stream.events.iter().find_map(|event| match event {
        MdxEvent::Inline(InlineEvent::LinkStartRef { def_index }) => Some(*def_index),
        _ => None,
    });

    assert_eq!(inline_url, Some("/docs"));
    let definition = stream.link_reference(reference_index.unwrap()).unwrap();
    assert_eq!(definition.url, b"/guide");
    assert_eq!(definition.title.as_deref(), Some(b"Guide".as_slice()));
}

#[test]
fn markdown_block_boundaries_remain_ordered_and_balanced() {
    let input = "# Heading\n\n- first\n- second\n\nParagraph.\n";
    let stream = parse_events(input);
    let mut heading_depth = 0;
    let mut paragraph_depth = 0;
    let mut list_depth = 0;
    let mut item_depth = 0;

    for event in &stream.events {
        match event {
            MdxEvent::Block(BlockEvent::HeadingStart { .. }) => heading_depth += 1,
            MdxEvent::Block(BlockEvent::HeadingEnd { .. }) => heading_depth -= 1,
            MdxEvent::Block(BlockEvent::ParagraphStart) => paragraph_depth += 1,
            MdxEvent::Block(BlockEvent::ParagraphEnd) => paragraph_depth -= 1,
            MdxEvent::Block(BlockEvent::ListStart { .. }) => list_depth += 1,
            MdxEvent::Block(BlockEvent::ListEnd { .. }) => list_depth -= 1,
            MdxEvent::Block(BlockEvent::ListItemStart { .. }) => item_depth += 1,
            MdxEvent::Block(BlockEvent::ListItemEnd) => item_depth -= 1,
            _ => {}
        }

        assert!(heading_depth >= 0);
        assert!(paragraph_depth >= 0);
        assert!(list_depth >= 0);
        assert!(item_depth >= 0);
    }

    assert_eq!(heading_depth, 0);
    assert_eq!(paragraph_depth, 0);
    assert_eq!(list_depth, 0);
    assert_eq!(item_depth, 0);
}

#[test]
fn contiguous_multiline_paragraphs_share_one_inline_parse() {
    let input = "Paragraph with *emphasis\ncontinued* and {name}.\n";
    let stream = parse_events(input);
    let inline = stream
        .events
        .iter()
        .filter_map(|event| match event {
            MdxEvent::Inline(event) => Some(event),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(
        inline
            .iter()
            .filter(|event| matches!(event, InlineEvent::EmphasisStart))
            .count(),
        1
    );
    assert_eq!(
        inline
            .iter()
            .filter(|event| matches!(event, InlineEvent::EmphasisEnd))
            .count(),
        1
    );
    assert!(
        inline
            .iter()
            .any(|event| matches!(event, InlineEvent::SoftBreak))
    );
    assert!(
        inline
            .iter()
            .any(|event| matches!(event, InlineEvent::MdxExpression(range)
            if range.slice_str(input.as_bytes()).unwrap() == "{name}"))
    );
}

#[test]
fn strict_stream_reports_diagnostics_while_permissive_stream_recovers_as_text() {
    let input = "# Heading\n\n{user.name\n";
    let permissive = parse_events(input);

    assert!(permissive.events.iter().any(|event| {
        matches!(event, MdxEvent::Inline(InlineEvent::Text(range))
            if range.slice_str(input.as_bytes()).unwrap() == "{user.name")
    }));

    let diagnostics = parse_events_strict(input).unwrap_err();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].code,
        MdxDiagnosticCode::UnterminatedExpression
    );
}

#[test]
fn strict_diagnostics_after_front_matter_keep_absolute_ranges() {
    let input = "---\ntitle: Test\n---\n{user.name\n";
    let diagnostics = parse_events_strict(input).unwrap_err();

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].code,
        MdxDiagnosticCode::UnterminatedExpression
    );
    assert_eq!(
        diagnostics[0]
            .primary_range
            .slice_str(input.as_bytes())
            .unwrap(),
        "{user.name\n"
    );
}

#[test]
fn strict_stream_preserves_inline_mdx_text_recovery() {
    let input = "Paragraph with {unterminated inline expression.\n";
    let stream = parse_events_strict(input).unwrap();

    assert!(stream.events.iter().any(|event| {
        matches!(event, MdxEvent::Inline(InlineEvent::Text(range))
            if range.slice_str(input.as_bytes()).unwrap()
                == "Paragraph with {unterminated inline expression.")
    }));
}
