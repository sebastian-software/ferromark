//! Cross-change review coverage for the performance audit PRs.
#![cfg(feature = "mdx")]

use ferromark::{Options, mdx};

#[test]
fn document_identity_and_delayed_footnotes_survive_mdx_boundaries_together() {
    let options = ferromark::options!(Options::default();
        footnotes: true,
        inline_footnotes: true,
        front_matter: true,);
    let source = "---\ntitle: Überprüfung\n---\n\n# foo\n\nFirst^[inline]. See[^a].\n\n<Panel>\n\n# foo-1\n\nBody **bold**.\n\n</Panel>\n\n# foo\n\n[^a]: Note with [^b].\n\n[^b]: Nested café.\n";
    let result = mdx::render_with_options(source, &options);
    assert_eq!(result.front_matter, Some("title: Überprüfung\n"));
    for id in ["foo", "foo-1", "foo-2"] {
        assert_eq!(
            result.body.matches(&format!("<h1 id=\"{id}\"")).count(),
            1,
            "{}",
            result.body
        );
    }
    assert_eq!(
        result.body.matches("<section data-footnotes").count(),
        1,
        "{}",
        result.body
    );
    assert_eq!(
        result.body.matches("<li id=\"user-content-fn-a\"").count(),
        1
    );
    assert_eq!(
        result.body.matches("<li id=\"user-content-fn-b\"").count(),
        1
    );
    assert!(result.body.contains("Nested café."), "{}", result.body);
    assert!(
        result.body.contains("data-footnote-ref>3</a>"),
        "{}",
        result.body
    );
    assert!(
        result.body.find("</Panel>").unwrap()
            < result.body.find("<section data-footnotes").unwrap()
    );
}
