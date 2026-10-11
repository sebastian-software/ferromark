use ferromark::{
    Allocator, HtmlRenderer, Parser, ParserOptions, ReferenceKind as Kind,
    ReferenceProvenance as Provenance, ReferenceSnapshot,
    ast::{Link, Node, Span, Text},
};

#[test]
fn markdown_forms_resolve_without_collecting_definitions_or_unresolved_syntax() {
    let source = "é [inline](/a(b)c) [full][id] [id][] [id] ![alt *text*](img.png) ![id][] ![id] ![full][id] <https://example.test/x> <a@example.test> www.example.test [missing]\n\n[id]: /shared \"Shared\"\n";
    let allocator = Allocator::new();
    let document = Parser::with_options(
        &allocator,
        source,
        ParserOptions {
            autolinks: true,
            ..ParserOptions::default()
        },
    )
    .parse()
    .unwrap();
    let refs = document.collect_references(source);
    assert_eq!(
        refs.iter()
            .map(|r| (r.kind, r.destination.as_str()))
            .collect::<Vec<_>>(),
        vec![
            (Kind::Link, "/a(b)c"),
            (Kind::Link, "/shared"),
            (Kind::Link, "/shared"),
            (Kind::Link, "/shared"),
            (Kind::Image, "img.png"),
            (Kind::Image, "/shared"),
            (Kind::Image, "/shared"),
            (Kind::Image, "/shared"),
            (Kind::Autolink, "https://example.test/x"),
            (Kind::Autolink, "mailto:a@example.test"),
            (Kind::Autolink, "http://www.example.test"),
        ]
    );
    assert_eq!(refs[4].text.as_deref(), Some("alt text"));
    assert_eq!(refs[1].title.as_deref(), Some("Shared"));
    assert_eq!(refs[0].span.unwrap().start, 3);
    assert_eq!(refs[1].span.unwrap().source_text(source), "[full][id]");
    assert!(
        refs.iter()
            .all(|r| r.provenance == Provenance::Source && r.original_span.is_none())
    );
    assert!(
        refs.windows(2)
            .all(|pair| pair[0].span.unwrap().start < pair[1].span.unwrap().start)
    );
}

#[test]
fn source_ranges_survive_escapes_entities_containers_and_crlf_normalization() {
    let source = "\u{feff}> é &amp; \\* www.example.test/a?q=x&amp;y=z [**é**](/a\\(b\\))\r\n>\r\n> [other](/same) [again](/same)\r\n";
    let allocator = Allocator::new();
    let document = Parser::with_options(
        &allocator,
        source,
        ParserOptions {
            autolinks: true,
            ..ParserOptions::default()
        },
    )
    .parse()
    .unwrap();
    let refs = document.collect_references(source);
    assert_eq!(refs.len(), 4);
    assert_eq!(refs[0].kind, Kind::Autolink);
    assert_eq!(refs[0].destination, "http://www.example.test/a?q=x&y=z");
    assert_eq!(
        refs[0].span.unwrap().source_text(source),
        "www.example.test/a?q=x&amp;y=z"
    );
    assert_eq!(refs[1].kind, Kind::Link);
    assert_eq!(refs[1].destination, "/a(b)");
    assert_eq!(refs[1].text.as_deref(), Some("é"));
    assert_eq!(
        refs[1].span.unwrap().source_text(source),
        "[**é**](/a\\(b\\))"
    );
    assert_eq!(refs[2].destination, refs[3].destination);
    assert_ne!(refs[2].span, refs[3].span);
}

#[test]
fn code_and_raw_html_are_excluded_and_syntax_options_apply() {
    let source = "`[inline](/ignore)`\n\n```md\n[fenced](/ignore)\n```\n\n    [indented](/ignore)\n\n<a href=\"/raw\">raw</a>\n\nwww.example.test <https://example.test> [ref]\n\n[ref]: /reference\n";
    let allocator = Allocator::new();
    let options = ParserOptions {
        autolinks: false,
        allow_link_refs: false,
        ..ParserOptions::default()
    };
    let document = Parser::with_options(&allocator, source, options)
        .parse()
        .unwrap();
    let refs = document.collect_references(source);
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].destination, "https://example.test");
    assert_eq!(refs[0].kind, Kind::Autolink);
}

#[test]
fn nested_image_labels_and_fragment_relative_external_destinations_stay_owned() {
    let source = "[![icon](./asset.svg) **Go** `now`](#target) [root](/root) [external](https://example.test)";
    let allocator = Allocator::new();
    let document = Parser::new(&allocator, source).parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let before = renderer.render(&document);
    let refs = document.collect_references(source);
    assert_eq!(refs[0].text.as_deref(), Some("icon Go now"));
    assert_eq!(
        refs.iter()
            .map(|r| r.destination.as_str())
            .collect::<Vec<_>>(),
        vec!["#target", "./asset.svg", "/root", "https://example.test"]
    );
    assert_eq!(refs, document.collect_references(source));
    assert_eq!(before, renderer.render(&document));
}

#[test]
fn snapshot_distinguishes_retained_changed_and_generated_references() {
    let source = "[keep](/keep) [change](/old)";
    let allocator = Allocator::new();
    let mut document = Parser::new(&allocator, source).parse().unwrap();
    let snapshot = ReferenceSnapshot::capture(&document, source);
    let Node::Paragraph(paragraph) = &mut document.children[0] else {
        panic!()
    };
    let Node::Link(changed) = &mut paragraph.children[2] else {
        panic!()
    };
    let original_span = changed.span;
    changed.url = "/new";
    let mut children = allocator.new_vec();
    children.push(Node::Text(Text {
        value: "new",
        span: Span::empty(),
    }));
    paragraph.children.push(Node::Link(allocator.boxed(Link {
        url: "/created",
        title: None,
        attributes: None,
        children,
        span: Span::empty(),
    })));
    let refs = snapshot.collect(&document, source);
    let keep = refs.iter().find(|r| r.destination == "/keep").unwrap();
    let changed = refs.iter().find(|r| r.destination == "/new").unwrap();
    let generated = refs.iter().find(|r| r.destination == "/created").unwrap();
    assert_eq!(keep.provenance, Provenance::Source);
    assert!(keep.span.is_some());
    assert_eq!(changed.provenance, Provenance::Transformed);
    assert_eq!(changed.span, None);
    assert_eq!(changed.original_span, Some(original_span));
    assert_eq!(generated.kind, Kind::Link);
    assert_eq!(generated.provenance, Provenance::Transformed);
    assert_eq!((generated.span, generated.original_span), (None, None));
    assert_eq!(refs, snapshot.collect(&document, source));
}

#[cfg(feature = "jsx")]
#[test]
fn mdx_references_resolve_across_segments_and_literals_retain_attribute_origin() {
    let source = "[before][shared]\n\n<Card href=\"/card?a=1&amp;b=2\" src='/cover.png' {...props}>\n\n![inside][shared]\n\n</Card>\n\nText <a href={dynamic} src={\"also dynamic\"} href=\"/literal\" />\n\n[shared]: /resolved\n";
    let allocator = Allocator::new();
    let options = ParserOptions {
        mdx: true,
        mdx_compatible: true,
        ..ParserOptions::default()
    };
    let document = Parser::with_options(&allocator, source, options)
        .parse()
        .unwrap();
    let refs = document.collect_references(source);
    assert_eq!(
        refs.iter()
            .map(|r| r.destination.as_str())
            .collect::<Vec<_>>(),
        vec![
            "/resolved",
            "/card?a=1&b=2",
            "/cover.png",
            "/resolved",
            "/literal"
        ]
    );
    assert_eq!(refs[1].element.as_deref(), Some("Card"));
    assert_eq!(refs[1].attribute.as_deref(), Some("href"));
    assert_eq!(refs[1].kind, Kind::MdxAttribute);
    assert_eq!(refs[1].text, None);
    assert_eq!(
        refs[1].span.unwrap().source_text(source),
        "href=\"/card?a=1&amp;b=2\""
    );
    assert_eq!(refs[4].element.as_deref(), Some("a"));
}
