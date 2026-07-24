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
fn thematic_breaks_expose_exact_absolute_source_ranges() {
    let input = concat!(
        "# Intro\n\n",
        "  - - - \t\n\n",
        "> ***\n\n",
        "Setext heading\n",
        "---\n\n",
        "<Panel>\n\n",
        "___\n\n",
        "</Panel>\n",
    );
    let stream = parse_events(input);
    let breaks = stream
        .events
        .iter()
        .filter_map(|event| match event {
            MdxEvent::Block(BlockEvent::ThematicBreak(range)) => {
                Some(range.slice_str(input.as_bytes()).unwrap())
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(breaks, vec!["- - - \t", "***", "___"]);
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

#[test]
fn fenced_code_info_exposes_its_absolute_source_range() {
    let input = "<Divider />\n\n```rust,ignore\nlet value = 1;\n```\n";
    let stream = parse_events(input);
    let event = stream
        .events
        .iter()
        .find(|event| matches!(event, MdxEvent::Block(BlockEvent::CodeBlockStart { .. })))
        .unwrap();

    assert_eq!(source(input, event), Some("rust,ignore"));
}

#[test]
fn tag_only_blockquote_paragraphs_promote_around_markdown_blocks() {
    let input = "\
> <Card>
>
> Hello *world*.
>
> </Card>
";
    let stream = parse_events(input);
    let open = stream
        .events
        .iter()
        .position(|event| matches!(event, MdxEvent::FlowJsxOpen(_)))
        .unwrap();
    let paragraph = stream
        .events
        .iter()
        .position(|event| matches!(event, MdxEvent::Block(BlockEvent::ParagraphStart)))
        .unwrap();
    let close = stream
        .events
        .iter()
        .position(|event| matches!(event, MdxEvent::FlowJsxClose(_)))
        .unwrap();

    assert!(matches!(
        stream.events.first(),
        Some(MdxEvent::Block(BlockEvent::BlockQuoteStart { .. }))
    ));
    assert!(matches!(
        stream.events.last(),
        Some(MdxEvent::Block(BlockEvent::BlockQuoteEnd))
    ));
    assert!(open < paragraph && paragraph < close);
    assert_eq!(source(input, &stream.events[open]), Some("<Card>"));
    assert_eq!(source(input, &stream.events[close]), Some("</Card>"));
    assert!(!stream.events.iter().any(|event| {
        matches!(
            event,
            MdxEvent::Inline(InlineEvent::MdxJsxOpen(_))
                | MdxEvent::Inline(InlineEvent::MdxJsxClose(_))
        )
    }));
}

#[test]
fn list_and_nested_container_units_promote_with_exact_ranges() {
    let input = concat!(
        "Intro ä.\n\n",
        "-   <Badge />   \n\n",
        "1. {value}\n\n",
        "- <Open>\n",
        "- </Open>\n\n",
        "> - <Nested />\n",
    );
    let stream = parse_events(input);
    let promoted = stream
        .events
        .iter()
        .filter(|event| {
            matches!(
                event,
                MdxEvent::FlowJsxOpen(_)
                    | MdxEvent::FlowJsxClose(_)
                    | MdxEvent::FlowJsxSelfClose(_)
                    | MdxEvent::FlowExpression(_)
            )
        })
        .filter_map(|event| source(input, event))
        .collect::<Vec<_>>();

    assert_eq!(
        promoted,
        vec!["<Badge />", "{value}", "<Open>", "</Open>", "<Nested />"]
    );
    assert_eq!(
        stream
            .events
            .iter()
            .filter(|event| matches!(event, MdxEvent::Block(BlockEvent::ListItemStart { .. })))
            .count(),
        5
    );
    assert!(
        stream
            .events
            .iter()
            .filter_map(|event| match event {
                MdxEvent::Block(BlockEvent::ListStart { tight, .. }) => Some(*tight),
                _ => None,
            })
            .all(|tight| tight)
    );
}

#[test]
fn list_item_component_wrappers_promote_around_multiple_blocks() {
    let input = "\
> 1. <Panel>
>
>    Body
>
>    </Panel>
";
    let stream = parse_events(input);
    let open = stream
        .events
        .iter()
        .position(|event| matches!(event, MdxEvent::FlowJsxOpen(_)))
        .unwrap();
    let body = stream
        .events
        .iter()
        .position(|event| {
            matches!(event, MdxEvent::Inline(InlineEvent::Text(range))
                if range.slice_str(input.as_bytes()).unwrap() == "Body")
        })
        .unwrap();
    let close = stream
        .events
        .iter()
        .position(|event| matches!(event, MdxEvent::FlowJsxClose(_)))
        .unwrap();

    assert!(open < body && body < close);
    assert_eq!(source(input, &stream.events[open]), Some("<Panel>"));
    assert_eq!(source(input, &stream.events[close]), Some("</Panel>"));
    assert!(matches!(
        stream.events.first(),
        Some(MdxEvent::Block(BlockEvent::BlockQuoteStart { .. }))
    ));
    assert!(stream.events.iter().any(|event| {
        matches!(
            event,
            MdxEvent::Block(BlockEvent::ListStart { tight: false, .. })
        )
    }));
}

#[test]
fn mixed_prose_code_and_multiline_container_mdx_are_not_promoted() {
    let input = "\
> Read <Badge>new</Badge> today.
>
> `<Card />`
>
> \\<Escaped />
>
> ```mdx
> <Code />
> ```
>
>     <Indented />
>
> <Multiline
>   prop={value}
> />
";
    let stream = parse_events(input);

    assert!(!stream.events.iter().any(|event| {
        matches!(
            event,
            MdxEvent::FlowJsxOpen(_)
                | MdxEvent::FlowJsxClose(_)
                | MdxEvent::FlowJsxSelfClose(_)
                | MdxEvent::FlowExpression(_)
        )
    }));
    assert!(stream.events.iter().any(|event| {
        matches!(event, MdxEvent::Inline(InlineEvent::MdxJsxOpen(range))
            if range.slice_str(input.as_bytes()).unwrap() == "<Badge>")
    }));
    assert!(stream.events.iter().any(|event| {
        matches!(event, MdxEvent::Inline(InlineEvent::Code(range))
            if range.slice_str(input.as_bytes()).unwrap() == "<Card />")
    }));
    assert!(
        stream
            .events
            .iter()
            .any(|event| { matches!(event, MdxEvent::Block(BlockEvent::CodeBlockStart { .. })) })
    );
    assert!(stream.events.iter().any(|event| {
        matches!(event, MdxEvent::Block(BlockEvent::Code(range))
            if range.slice_str(input.as_bytes()).unwrap().contains("<Indented />"))
    }));
}

#[test]
fn semantic_normalization_does_not_change_mdx_rendering() {
    let input = "> <Card />\n";
    let event_stream = parse_events(input);
    let rendered = ferromark::mdx::render::render(input);

    assert!(
        event_stream
            .events
            .iter()
            .any(|event| matches!(event, MdxEvent::FlowJsxSelfClose(_)))
    );
    assert!(rendered.body.contains("<blockquote>"));
    assert!(
        rendered.body.contains("<Card />"),
        "container JSX must remain in rendered output: {}",
        rendered.body
    );
}
