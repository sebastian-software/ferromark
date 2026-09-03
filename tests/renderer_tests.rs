use ferromark::{Options, Renderer, to_html_with_options};

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
