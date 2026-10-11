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
fn autolink_ranges_cover_expanding_entities_and_coalesced_text() {
    let source = "&nLt;\\* https://example.test/x?value=&fjlig; then https://other.test/end\n";
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, ParserOptions::gfm())
        .parse()
        .unwrap();
    let references = document.collect_references(source);
    assert_eq!(references.len(), 2);
    assert_eq!(references[0].destination, "https://example.test/x?value=fj");
    assert_eq!(
        references[0].span.unwrap().source_text(source),
        "https://example.test/x?value=&fjlig;"
    );
    assert_eq!(
        references[1].span.unwrap().source_text(source),
        "https://other.test/end"
    );
}

#[test]
fn lazy_autolink_ranges_verify_multiline_container_runs() {
    let source = "\u{feff}> - > &nLt;\\* https://same.test/a?x=&fjlig; \r\n>   > \\> https://same.test/a?x=&fjlig;\r\n>   > third  \r\n>   > https://last.test/x\\?q=&amp;\r\n";
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, ParserOptions::gfm())
        .parse()
        .unwrap();
    let references = document.collect_references(source);
    assert_eq!(references.len(), 3);
    assert_eq!(references[0].destination, references[1].destination);
    assert_eq!(references[2].destination, "https://last.test/x?q=&");
    assert_eq!(
        references
            .iter()
            .map(|reference| reference.span.unwrap().source_text(source))
            .collect::<Vec<_>>(),
        vec![
            "https://same.test/a?x=&fjlig;",
            "https://same.test/a?x=&fjlig;",
            "https://last.test/x\\?q=&amp;"
        ]
    );
    assert_ne!(references[0].span, references[1].span);
    assert_eq!(
        ReferenceSnapshot::capture(&document, source).collect(&document, source),
        references
    );
}

#[test]
fn lazy_autolink_ranges_do_not_confuse_code_bracket_angle_or_repeated_links() {
    let source = "`https://same.test/x` &nLt;\\* https://same.test/x [https://same.test/x](https://same.test/x) <https://same.test/x> &amp; https://same.test/x\n\n&#104;ttps://same.test/x https://same.test/x";
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, ParserOptions::gfm())
        .parse()
        .unwrap();
    let references = document.collect_references(source);
    let expected = vec![
        "https://same.test/x",
        "[https://same.test/x](https://same.test/x)",
        "<https://same.test/x>",
        "https://same.test/x",
        "&#104;ttps://same.test/x",
        "https://same.test/x",
    ];
    assert_eq!(
        references
            .iter()
            .map(|reference| reference.span.unwrap().source_text(source))
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(references[1].kind, Kind::Link);
    assert!(
        references
            .iter()
            .enumerate()
            .all(|(index, reference)| index == 1 || reference.kind == Kind::Autolink)
    );
    assert!(
        references
            .windows(2)
            .all(|pair| pair[0].span.unwrap().end <= pair[1].span.unwrap().start)
    );
}

#[test]
fn snapshots_keep_verified_autolink_ranges_separate_from_parser_spans() {
    let source = "&nLt;\\* https://example.test/x?value=&fjlig;";
    let allocator = Allocator::new();
    let mut document = Parser::with_options(&allocator, source, ParserOptions::gfm())
        .parse()
        .unwrap();
    let references = document.collect_references(source);
    let source_span = references[0].span.unwrap();
    let snapshot = ReferenceSnapshot::capture(&document, source);
    assert_eq!(snapshot.collect(&document, source), references);
    let Node::Paragraph(paragraph) = &mut document.children[0] else {
        panic!()
    };
    let link = paragraph
        .children
        .iter_mut()
        .find_map(|node| match node {
            Node::Link(link) => Some(link),
            _ => None,
        })
        .unwrap();
    assert_ne!(link.span, source_span);
    link.url = "/changed";
    let changed = snapshot.collect(&document, source);
    assert_eq!(changed[0].provenance, Provenance::Transformed);
    assert_eq!(changed[0].span, None);
    assert_eq!(changed[0].original_span, Some(source_span));
}

#[test]
fn bare_autolink_ranges_are_unavailable_when_source_verification_disagrees() {
    let source = "&amp; https://example.test";
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, source, ParserOptions::gfm())
        .parse()
        .unwrap();
    let references = document.collect_references("wrong https://example.test");
    assert_eq!(references.len(), 1);
    assert_eq!(references[0].span, None);
}

#[test]
fn lazy_autolink_ranges_preserve_comment_gaps_and_authored_comment_like_lines() {
    let source =
        "&amp; https://first.test\n// hidden https://hidden.test\n    // literal https://last.test";
    for comments in [false, true] {
        let allocator = Allocator::new();
        let document = Parser::with_options(
            &allocator,
            source,
            ParserOptions {
                line_comments: comments,
                ..ParserOptions::gfm()
            },
        )
        .parse()
        .unwrap();
        let references = document.collect_references(source);
        let expected = if comments {
            vec!["https://first.test", "https://last.test"]
        } else {
            vec![
                "https://first.test",
                "https://hidden.test",
                "https://last.test",
            ]
        };
        assert_eq!(
            references
                .iter()
                .map(|reference| reference.span.unwrap().source_text(source))
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(
            ReferenceSnapshot::capture(&document, source).collect(&document, source),
            references
        );
    }
    let source = "before\n// literal https://same.test\n    // literal https://same.test";
    let allocator = Allocator::new();
    let document = Parser::with_options(
        &allocator,
        source,
        ParserOptions {
            line_comments: true,
            ..ParserOptions::gfm()
        },
    )
    .parse()
    .unwrap();
    let references = document.collect_references(source);
    assert_eq!(references.len(), 1);
    assert_eq!(
        references[0].span.unwrap().start as usize,
        source.rfind("https://same.test").unwrap()
    );
}

#[test]
fn lazy_autolink_ranges_verify_formatting_attributes_tables_and_captions() {
    for source in [
        "# &amp; https://example.test {#id}\n",
        "&amp; https://example.test\n---\n",
        "*&amp; https://example.test*\n",
        "[&amp; https://example.test]{#id}\n",
        "| &amp; https://example.test/a\\|b |\n| --- |\n: &amp; https://example.test/caption {#id}\n",
        "![alt](/image)\n: &amp; https://example.test {#figure}\n",
        "> quote\n: &amp; https://example.test {#author}\n",
        "&amp; https://example.test\n: definition\n",
        "[^note]: &amp; https://example.test\n",
        "\0 例子 &nLt; https://example.test/例子/é?q=&fjlig;\n",
    ] {
        let allocator = Allocator::new();
        let document = Parser::with_options(&allocator, source, ParserOptions::ffm())
            .parse()
            .unwrap();
        let references = document.collect_references(source);
        assert!(!references.is_empty(), "{source:?}");
        for reference in references
            .iter()
            .filter(|reference| reference.kind == Kind::Autolink)
        {
            let range = reference
                .span
                .unwrap_or_else(|| panic!("missing span: {source:?}"));
            assert!(
                range.source_text(source).starts_with("https://"),
                "{source:?}"
            );
        }
        assert_eq!(
            ReferenceSnapshot::capture(&document, source).collect(&document, source),
            references
        );
    }
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
    assert_eq!(
        refs.iter()
            .map(|reference| reference.destination.as_str())
            .collect::<Vec<_>>(),
        vec!["/keep", "/new", "/created"]
    );
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
    let Node::Paragraph(paragraph) = &mut document.children[0] else {
        panic!()
    };
    paragraph.children.swap(0, 2);
    let reordered = snapshot.collect(&document, source);
    assert_eq!(
        reordered
            .iter()
            .map(|reference| reference.destination.as_str())
            .collect::<Vec<_>>(),
        vec!["/new", "/keep", "/created"]
    );
    assert_eq!(reordered[1].provenance, Provenance::Source);
    assert_eq!(
        reordered[1].span.unwrap().source_text(source),
        "[keep](/keep)"
    );
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

#[test]
fn table_references_follow_authored_rows_before_the_caption() {
    let source = "| [header](/head) |\n| --- |\n| [cell](/body) |\n: [caption](/caption)\n";
    let allocator = Allocator::new();
    let document = Parser::with_options(
        &allocator,
        source,
        ParserOptions {
            table_attributes: true,
            ..ParserOptions::gfm()
        },
    )
    .parse()
    .unwrap();
    let expected = vec!["/head", "/body", "/caption"];
    let references = document.collect_references(source);
    assert_eq!(
        references
            .iter()
            .map(|reference| reference.destination.as_str())
            .collect::<Vec<_>>(),
        expected
    );
    let snapshot = ReferenceSnapshot::capture(&document, source);
    assert_eq!(references, snapshot.collect(&document, source));
    assert_eq!(
        references[2].span.unwrap().source_text(source),
        "[caption](/caption)"
    );
}
