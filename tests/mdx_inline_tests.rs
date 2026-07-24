#![cfg(feature = "mdx")]

use ferromark::{InlineEvent, InlineParser};

fn parse(input: &str) -> Vec<InlineEvent> {
    let mut parser = InlineParser::new();
    let mut events = Vec::new();
    parser.parse_mdx(input.as_bytes(), None, &mut events);
    events
}

fn event_source<'a>(input: &'a str, event: &InlineEvent) -> Option<&'a str> {
    let range = match event {
        InlineEvent::Text(range)
        | InlineEvent::Code(range)
        | InlineEvent::MdxExpression(range)
        | InlineEvent::MdxJsxOpen(range)
        | InlineEvent::MdxJsxClose(range)
        | InlineEvent::MdxJsxSelfClose(range) => range,
        _ => return None,
    };
    range.slice_str(input.as_bytes()).ok()
}

#[test]
fn inline_expressions_are_source_ranged_between_text() {
    let input = "Hello {user.profile.name}!";
    let events = parse(input);

    assert_eq!(events.len(), 3);
    assert_eq!(event_source(input, &events[0]), Some("Hello "));
    assert_eq!(event_source(input, &events[1]), Some("{user.profile.name}"));
    assert_eq!(event_source(input, &events[2]), Some("!"));
    assert!(matches!(events[1], InlineEvent::MdxExpression(_)));
}

#[test]
fn inline_expressions_share_the_nested_brace_scanner() {
    let input = "Heading {format({ value: user.name })}";
    let events = parse(input);

    assert_eq!(events.len(), 2);
    assert_eq!(event_source(input, &events[0]), Some("Heading "));
    assert_eq!(
        event_source(input, &events[1]),
        Some("{format({ value: user.name })}")
    );
    assert!(matches!(events[1], InlineEvent::MdxExpression(_)));
}

#[test]
fn inline_jsx_pairs_and_self_closing_tags_are_typed() {
    let input = "Read <Badge>the guide</Badge> and <Icon name=\"book\" />.";
    let events = parse(input);

    let typed = events
        .iter()
        .filter_map(|event| match event {
            InlineEvent::MdxJsxOpen(_)
            | InlineEvent::MdxJsxClose(_)
            | InlineEvent::MdxJsxSelfClose(_) => event_source(input, event),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(typed, vec!["<Badge>", "</Badge>", "<Icon name=\"book\" />"]);
}

#[test]
fn inline_jsx_reuses_expression_attributes_fragments_and_member_names() {
    let input = "<><UI.Badge {...props}>{label}</UI.Badge></>";
    let events = parse(input);

    let typed = events
        .iter()
        .filter_map(|event| match event {
            InlineEvent::MdxExpression(_)
            | InlineEvent::MdxJsxOpen(_)
            | InlineEvent::MdxJsxClose(_)
            | InlineEvent::MdxJsxSelfClose(_) => event_source(input, event),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        typed,
        vec![
            "<>",
            "<UI.Badge {...props}>",
            "{label}",
            "</UI.Badge>",
            "</>"
        ]
    );
}

#[test]
fn markdown_events_continue_around_inline_mdx() {
    let input = "*Hello* {user} [guide](/guide) <Icon />";
    let events = parse(input);

    assert!(matches!(events[0], InlineEvent::EmphasisStart));
    assert!(matches!(events[2], InlineEvent::EmphasisEnd));
    assert!(
        events
            .iter()
            .any(|event| matches!(event, InlineEvent::MdxExpression(_)))
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, InlineEvent::LinkStart { .. }))
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, InlineEvent::LinkEnd))
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, InlineEvent::MdxJsxSelfClose(_)))
    );
}

#[test]
fn code_spans_and_escaped_delimiters_remain_non_mdx() {
    let input = r"`{literal} <Badge>` \{escaped\} \<Tag> {real}";
    let events = parse(input);

    assert!(
        events
            .iter()
            .any(|event| matches!(event, InlineEvent::Code(_)))
    );
    assert!(events.iter().any(|event| matches!(event, InlineEvent::MdxExpression(range) if event_source(input, &InlineEvent::MdxExpression(*range)) == Some("{real}"))));
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(
                event,
                InlineEvent::MdxExpression(_)
                    | InlineEvent::MdxJsxOpen(_)
                    | InlineEvent::MdxJsxClose(_)
                    | InlineEvent::MdxJsxSelfClose(_)
            ))
            .count(),
        1
    );
}

#[test]
fn malformed_inline_mdx_falls_back_to_text() {
    let input = "Broken {user and <Badge prop=";
    let events = parse(input);

    assert_eq!(events.len(), 1);
    assert_eq!(event_source(input, &events[0]), Some(input));
}

#[test]
fn inline_mdx_only_splits_events_added_by_this_call() {
    let mut parser = InlineParser::new();
    let mut events = Vec::new();

    parser.parse_mdx(b"First {value}", None, &mut events);
    let first_events = events.clone();
    parser.parse_mdx(b"Second <Icon />", None, &mut events);

    assert_eq!(&events[..first_events.len()], first_events);
    assert!(matches!(
        events.last(),
        Some(InlineEvent::MdxJsxSelfClose(_))
    ));
}
