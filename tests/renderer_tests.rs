use ferromark::{Options, Renderer, to_html_with_options};

#[test]
fn paragraph_fragments_preserve_line_endings_indentation_and_document_reset() {
    let options = Options::commonmark();
    let mut renderer = Renderer::with_options(options.clone());
    for (input, expected) in [
        (
            "first\nsecond\n\nthird",
            "<p>first\nsecond</p>\n<p>third</p>\n",
        ),
        (
            "first\r\nsecond\r\n\r\nthird",
            "<p>first\nsecond</p>\n<p>third</p>\n",
        ),
        (
            "first  \nsecond\n\nthird",
            "<p>first<br />\nsecond</p>\n<p>third</p>\n",
        ),
        (
            "first\\\nsecond\n\nthird",
            "<p>first<br />\nsecond</p>\n<p>third</p>\n",
        ),
        (
            "> first\n> second\n\nthird",
            "<blockquote>\n<p>first\nsecond</p>\n</blockquote>\n<p>third</p>\n",
        ),
        (
            "first\n  second\n\nthird",
            "<p>first\nsecond</p>\n<p>third</p>\n",
        ),
        ("first \t\n\nsecond", "<p>first</p>\n<p>second</p>\n"),
        (
            "one &amp; two\n\n*three*",
            "<p>one &amp; two</p>\n<p><em>three</em></p>\n",
        ),
        ("", ""),
        ("final", "<p>final</p>\n"),
    ] {
        assert_eq!(to_html_with_options(input, &options), expected, "{input:?}");
        assert_eq!(renderer.render(input), expected, "reused: {input:?}");
    }
}

#[test]
fn single_paragraph_preserves_references_escapes_policy_and_session_reset() {
    for policy in [
        ferromark::RenderPolicy::Trusted,
        ferromark::RenderPolicy::Untrusted,
    ] {
        let options = ferromark::options!(Options::commonmark(); render_policy: policy);
        let mut renderer = Renderer::with_options(options.clone());
        let raw_html = if matches!(policy, ferromark::RenderPolicy::Trusted) {
            "<p>A &amp; B <i>raw</i>.</p>\n"
        } else {
            "<p>A &amp; B &lt;i&gt;raw&lt;/i&gt;.</p>\n"
        };
        for (input, expected) in [
            (
                "[target]: /guide \"Guide\"\n\n**bold** with [a link][target] and \\*literal\\*. \t\n",
                "<p><strong>bold</strong> with <a href=\"/guide\" title=\"Guide\">a link</a> and *literal*.</p>\n",
            ),
            ("A &amp; B <i>raw</i>.", raw_html),
            ("[target] and *em*.", "<p>[target] and <em>em</em>.</p>\n"),
            ("first\nsecond", "<p>first\nsecond</p>\n"),
            ("final", "<p>final</p>\n"),
        ] {
            assert_eq!(to_html_with_options(input, &options), expected);
            assert_eq!(renderer.render(input), expected);
        }
    }
}

#[test]
fn reusable_renderer_matches_fresh_rendering_across_documents() {
    let options = ferromark::options!(Options::default();
        footnotes: true,
        inline_footnotes: true,
        highlight: true,
        definition_lists: true,
    );
    let mut renderer = Renderer::with_options(options.clone());
    let documents = [
        "# Same\n\n# Same\n\n[link][target]\n\n[target]: /first",
        "# Same\n\n[link][target]",
        "A reference[^note].\n\n[^note]: Footnote text.",
        "An inline note.^[Inline *content*.]",
        "| A | B |\n| - | - |\n| 1 | 2 |",
        "> [!NOTE]\n> Reusable callout",
        "Term\n: Definition",
        "==highlighted==",
        "",
    ];

    for document in documents {
        assert_eq!(
            renderer.render(document),
            to_html_with_options(document, &options),
            "session output differed for {document:?}",
        );
    }
}

#[test]
fn document_local_ids_references_and_footnotes_do_not_leak() {
    let options = ferromark::options!(Options::default();
        footnotes: true,
    );
    let mut renderer = Renderer::with_options(options);

    let first = renderer.render(
        "# Repeated\n\n# Repeated\n\n[resolved][ref]\n\nNote[^a].\n\n[ref]: /one\n\n[^a]: First",
    );
    assert!(first.contains("id=\"repeated-1\""));
    assert!(first.contains("href=\"/one\""));
    assert!(first.contains("data-footnotes"));

    let second = renderer.render("# Repeated\n\n[unresolved][ref]\n\nNo footnote.");
    assert!(second.contains("id=\"repeated\""));
    assert!(second.contains("[unresolved][ref]"));
    assert!(!second.contains("href=\"/one\""));
    assert!(!second.contains("data-footnotes"));
}

#[test]
fn render_into_reuses_and_replaces_the_output_buffer() {
    let mut renderer = Renderer::new();
    let mut output = Vec::with_capacity(256);
    let allocation = output.as_ptr();

    renderer.render_into("# First", &mut output);
    assert_eq!(output, b"<h1 id=\"first\">First</h1>\n");
    assert_eq!(output.as_ptr(), allocation);

    renderer.render_into("Second", &mut output);
    assert_eq!(output, b"<p>Second</p>\n");
    assert_eq!(output.as_ptr(), allocation);
}

#[test]
fn renderer_keeps_its_configuration() {
    let options = ferromark::options!(Options::default();
        heading_ids: false,
        highlight: true,
    );
    let mut renderer = Renderer::with_options(options);

    assert!(!renderer.options().heading_ids);
    assert!(renderer.options().highlight);
    assert_eq!(
        renderer.render("# Title\n\n==mark=="),
        "<h1>Title</h1>\n<p><mark>mark</mark></p>\n"
    );
}

#[test]
fn strikethrough_openers_do_not_leak_between_cells_or_documents() {
    let options = Options::gfm();
    let mut renderer = Renderer::with_options(options.clone());
    let many_openers = "~~open ".repeat(256);
    for _ in 0..3 {
        for input in [
            many_openers.as_str(),
            "plain",
            "close~~",
            "~~closed~~ then ~~open",
            "| ~~open | close~~ |\n| --- | --- |\n| ~~yes~~ | no~~ |",
            "~~outside [inside~~](/url)",
            "[~~inside](/url) outside~~",
            "`~~code~~` and ~~strike~~",
            "",
        ] {
            assert_eq!(
                renderer.render(input),
                to_html_with_options(input, &options),
                "strikethrough state leaked into {input:?}",
            );
        }
    }
    assert_eq!(renderer.render("close~~"), "<p>close~~</p>\n");
    assert_eq!(
        renderer.render("| ~~open | close~~ |\n| --- | --- |\n"),
        "<table>\n<thead>\n<tr>\n<th>~~open</th>\n<th>close~~</th>\n</tr>\n</thead>\n</table>\n",
        "strikethrough must not span table cells",
    );
}

#[test]
fn inline_extension_openers_do_not_cross_features_paragraphs_or_documents() {
    let options = ferromark::options!(Options::default();
        highlight: true, subscript: true, superscript: true,
    );
    let mut renderer = Renderer::with_options(options.clone());
    let cases = [
        (
            "~~open ~open ^open ==open",
            "<p>~~open ~open ^open ==open</p>\n",
        ),
        (
            "close~~ close~ close^ close==",
            "<p>close~~ close~ close^ close==</p>\n",
        ),
        (
            "~~strike~~ ~sub~ ^sup^ ==mark==",
            "<p><del>strike</del> <sub>sub</sub> <sup>sup</sup> <mark>mark</mark></p>\n",
        ),
        ("==open\n\nclose==", "<p>==open</p>\n<p>close==</p>\n"),
        (
            "`~sub~ ^sup^ ==mark==`",
            "<p><code>~sub~ ^sup^ ==mark==</code></p>\n",
        ),
    ];
    for _ in 0..3 {
        for (input, expected) in cases {
            assert_eq!(renderer.render(input), expected, "{input}");
        }
        for input in [
            "| ==open | close== |\n| --- | --- |\n| ^open | close^ |\n| ~open | close~ |",
            "==outside [inside==](/url) ^outside [inside^](/url) ~outside [inside~](/url)",
            "[==inside](/url) outside== [^inside](/url) outside^ [~inside](/url) outside~",
            "",
        ] {
            assert_eq!(
                renderer.render(input),
                to_html_with_options(input, &options)
            );
        }
    }
}

#[test]
fn table_cells_preserve_inline_content_across_borrowed_escaped_and_empty_cells() {
    let input = "| A | B | C |\n| --- | --- | --- |\n\
| *one* | `a\\|b` | [Guide][g] |\n\
| **two** | plain | [Other](/other) |\n\
| last |\n\n[g]: /guide\n";
    let expected = "<table>\n<thead>\n<tr>\n<th>A</th>\n<th>B</th>\n<th>C</th>\n</tr>\n</thead>\n\
<tbody>\n<tr>\n<td><em>one</em></td>\n<td><code>a|b</code></td>\n<td><a href=\"/guide\">Guide</a></td>\n</tr>\n\
<tr>\n<td><strong>two</strong></td>\n<td>plain</td>\n<td><a href=\"/other\">Other</a></td>\n</tr>\n\
<tr>\n<td>last</td>\n<td></td>\n<td></td>\n</tr>\n</tbody>\n</table>\n";
    for policy in [
        ferromark::RenderPolicy::Trusted,
        ferromark::RenderPolicy::Untrusted,
    ] {
        let options =
            ferromark::options!(Options::commonmark(); tables: true, render_policy: policy,);
        let mut renderer = Renderer::with_options(options.clone());
        for _ in 0..3 {
            assert_eq!(renderer.render(input), expected);
            assert_eq!(to_html_with_options(input, &options), expected);
            let next =
                renderer.render("| *open | close* |\n| --- | --- |\n| [Guide][g] | plain |\n");
            assert!(next.contains("<th>*open</th>\n<th>close*</th>"));
            assert!(next.contains("<td>[Guide][g]</td>"));
        }
    }
}
