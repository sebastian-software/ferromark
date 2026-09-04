use ferromark::{Options, Renderer, to_html_with_options};

fn opts() -> Options {
    ferromark::options!(Options::default();
        footnotes: true,)
}

fn render(input: &str) -> String {
    to_html_with_options(input, &opts())
}

// --- Basic footnote ---

#[test]
fn basic_footnote() {
    let result = render("Text[^1].\n\n[^1]: Footnote content.");
    assert!(
        result.contains("<sup><a href=\"#user-content-fn-1\""),
        "Missing footnote ref: {result}"
    );
    assert!(
        result.contains("id=\"user-content-fnref-1\""),
        "Missing fnref id: {result}"
    );
    assert!(
        result.contains("data-footnote-ref>1</a></sup>"),
        "Missing ref number: {result}"
    );
    assert!(
        result.contains("<section data-footnotes class=\"footnotes\">"),
        "Missing footnote section: {result}"
    );
    assert!(
        result.contains("<li id=\"user-content-fn-1\">"),
        "Missing footnote li: {result}"
    );
    assert!(
        result.contains("Footnote content."),
        "Missing footnote content: {result}"
    );
    assert!(
        result.contains("</ol>\n</section>\n"),
        "Missing section close: {result}"
    );
}

#[test]
fn footnote_backref() {
    let result = render("Text[^a].\n\n[^a]: Note.");
    assert!(
        result.contains("href=\"#user-content-fnref-a\" class=\"data-footnote-backref\""),
        "Missing backref: {result}"
    );
    assert!(result.contains("\u{21a9}"), "Missing ↩ symbol: {result}");
}

#[test]
fn nested_references_extend_document_order_and_terminate_cycles() {
    let result = render("Text[^a].\n\n[^a]: A references[^b].\n\n[^b]: B references[^a].");

    let section = result
        .split_once("<section data-footnotes class=\"footnotes\">\n")
        .map(|(_, section)| section)
        .expect("footnote section");
    let a_pos = section.find("<li id=\"user-content-fn-a\">").unwrap();
    let b_pos = section.find("<li id=\"user-content-fn-b\">").unwrap();
    assert!(
        a_pos < b_pos,
        "nested definition should be queued after a: {result}"
    );
    assert!(
        section[a_pos..b_pos].contains("href=\"#user-content-fn-b\""),
        "a should retain its nested reference: {result}"
    );
    assert!(
        section[a_pos..b_pos].contains("data-footnote-ref>2</a></sup>"),
        "b should receive its document ordinal: {result}"
    );
    assert!(
        section[a_pos..b_pos].contains("id=\"user-content-fnref-b\""),
        "b's backlink target should remain in the nested reference: {result}"
    );
    assert!(
        section[b_pos..].contains("href=\"#user-content-fn-a\""),
        "b should retain the cycle edge back to a: {result}"
    );
    assert!(
        section[b_pos..].contains("data-footnote-ref>1</a></sup>"),
        "a should retain ordinal 1 through the cycle: {result}"
    );
    assert!(
        section[b_pos..].contains("href=\"#user-content-fnref-b\""),
        "b's backlink should target its nested reference: {result}"
    );
    assert_eq!(result.matches("<li id=\"user-content-fn-").count(), 2);
}

#[test]
fn reusable_renderer_resets_nested_document_numbering() {
    let input = "Text[^a].\n\n[^a]: A references[^b].\n\n[^b]: B.";
    let follow_up = "Text[^b].\n\n[^b]: Follow-up.";
    let mut renderer = Renderer::with_options(opts());

    assert_eq!(renderer.render(input), render(input));
    assert_eq!(renderer.render(follow_up), render(follow_up));
}

// --- Multiple footnotes with numbering ---

#[test]
fn multiple_footnotes_numbered_by_appearance() {
    let result = render("First[^b] second[^a].\n\n[^a]: Note A.\n\n[^b]: Note B.");
    // [^b] appears first in text, so it should be number 1
    assert!(
        result.contains("data-footnote-ref>1</a></sup>"),
        "First ref not numbered 1: {result}"
    );
    assert!(
        result.contains("data-footnote-ref>2</a></sup>"),
        "Second ref not numbered 2: {result}"
    );
    // In the footnote section, order should follow appearance: b first, then a
    let b_pos = result.find("fn-b").unwrap();
    let a_pos = result.find("fn-a").unwrap();
    // The inline refs come first, then the section. Check the section order.
    let section_start = result.find("<section").unwrap();
    let b_in_section = result[section_start..].find("fn-b").unwrap();
    let a_in_section = result[section_start..].find("fn-a").unwrap();
    assert!(
        b_in_section < a_in_section,
        "Footnotes not ordered by first appearance: b@{b_pos} a@{a_pos}"
    );
}

// --- Duplicate reference ---

#[test]
fn duplicate_reference_same_number() {
    let result = render("First[^x] and again[^x].\n\n[^x]: Content X.");
    // Both references should get the same number
    let count = result.matches("data-footnote-ref>1</a></sup>").count();
    assert_eq!(
        count, 2,
        "Expected 2 refs with number 1, got {count}: {result}"
    );
}

// --- Undefined reference renders as literal text ---

#[test]
fn undefined_reference_literal() {
    let result = render("Text[^undef].");
    // Should render as literal text, not as a footnote ref
    assert!(
        !result.contains("<sup>"),
        "Undefined ref should not create sup: {result}"
    );
    assert!(
        result.contains("[^undef]"),
        "Undefined ref should be literal: {result}"
    );
}

// --- Duplicate definitions (first wins) ---

#[test]
fn duplicate_definition_first_wins() {
    let result = render("Ref[^d].\n\n[^d]: First def.\n\n[^d]: Second def.");
    assert!(
        result.contains("First def."),
        "First def should win: {result}"
    );
    // Second definition is consumed but discarded — first definition's content is used
    assert!(
        !result.contains("Second def."),
        "Second def should be discarded: {result}"
    );
}

// --- Footnotes disabled renders literal ---

#[test]
fn footnotes_disabled_literal() {
    let opts = ferromark::options!(Options::default();
        footnotes: false,);
    let result = to_html_with_options("Text[^1].\n\n[^1]: Content.", &opts);
    assert!(
        !result.contains("<sup>"),
        "Should not render footnote when disabled: {result}"
    );
    assert!(
        !result.contains("<section"),
        "Should not render section when disabled: {result}"
    );
}

// --- Multi-paragraph footnote definition ---

#[test]
fn multi_paragraph_footnote() {
    let input = "Text[^mp].\n\n[^mp]: First paragraph.\n\n    Second paragraph.";
    let result = render(input);
    assert!(
        result.contains("First paragraph."),
        "Missing first para: {result}"
    );
    assert!(
        result.contains("Second paragraph."),
        "Missing second para: {result}"
    );
}

// --- Footnote with code block ---

#[test]
fn footnote_with_code_block() {
    let input = "Text[^code].\n\n[^code]: Some text.\n\n        code here";
    let result = render(input);
    assert!(result.contains("Some text."), "Missing text: {result}");
    assert!(result.contains("code here"), "Missing code: {result}");
}

// --- Label validation ---

#[test]
fn label_with_alphanumeric() {
    let result = render("Text[^abc123].\n\n[^abc123]: Content.");
    assert!(
        result.contains("<sup>"),
        "Should resolve alphanumeric label: {result}"
    );
}

#[test]
fn label_with_dash_underscore() {
    let result = render("Text[^my-note_1].\n\n[^my-note_1]: Content.");
    assert!(
        result.contains("<sup>"),
        "Should resolve dash/underscore label: {result}"
    );
}

#[test]
fn empty_label_literal() {
    let result = render("Text[^].");
    assert!(
        !result.contains("<sup>"),
        "Empty label should not create ref: {result}"
    );
    assert!(
        result.contains("[^]"),
        "Empty label should be literal: {result}"
    );
}

// --- Footnote definition not in paragraph ---

#[test]
fn footnote_def_not_in_paragraph() {
    // Footnote definitions are block-level, they shouldn't be parsed inside paragraphs
    let result = render("Text[^1].\n\n[^1]: Note content.");
    assert!(
        result.contains("<section"),
        "Should have footnote section: {result}"
    );
}

// --- No footnotes referenced = no section ---

#[test]
fn no_refs_no_section() {
    let result = render("[^1]: Note content.\n\nJust a paragraph.");
    assert!(
        !result.contains("<section"),
        "Should not have section when no refs used: {result}"
    );
}

// --- Case insensitive labels ---

#[test]
fn case_insensitive_labels() {
    let result = render("Text[^ABC].\n\n[^abc]: Content.");
    assert!(
        result.contains("<sup>"),
        "Labels should be case-insensitive: {result}"
    );
}
