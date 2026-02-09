use ferromark::{to_html, to_html_with_options, Options};

fn html(input: &str) -> String {
    to_html(input)
}

fn html_no_callouts(input: &str) -> String {
    let mut opts = Options::default();
    opts.callouts = false;
    to_html_with_options(input, &opts)
}

// --- All 5 callout types ---

#[test]
fn callout_note() {
    let out = html("> [!NOTE]\n> This is a note.");
    assert!(out.contains("markdown-alert markdown-alert-note"));
    assert!(out.contains("markdown-alert-title\">Note</p>"));
    assert!(out.contains("This is a note."));
    assert!(out.contains("</div>"));
    assert!(!out.contains("<blockquote>"));
}

#[test]
fn callout_tip() {
    let out = html("> [!TIP]\n> This is a tip.");
    assert!(out.contains("markdown-alert-tip"));
    assert!(out.contains(">Tip</p>"));
}

#[test]
fn callout_important() {
    let out = html("> [!IMPORTANT]\n> This is important.");
    assert!(out.contains("markdown-alert-important"));
    assert!(out.contains(">Important</p>"));
}

#[test]
fn callout_warning() {
    let out = html("> [!WARNING]\n> This is a warning.");
    assert!(out.contains("markdown-alert-warning"));
    assert!(out.contains(">Warning</p>"));
}

#[test]
fn callout_caution() {
    let out = html("> [!CAUTION]\n> This is a caution.");
    assert!(out.contains("markdown-alert-caution"));
    assert!(out.contains(">Caution</p>"));
}

// --- Case insensitivity ---

#[test]
fn callout_case_insensitive_lower() {
    let out = html("> [!note]\n> Content.");
    assert!(out.contains("markdown-alert-note"));
}

#[test]
fn callout_case_insensitive_mixed() {
    let out = html("> [!Note]\n> Content.");
    assert!(out.contains("markdown-alert-note"));
}

// --- Unknown type → regular blockquote ---

#[test]
fn unknown_type_is_regular_blockquote() {
    let out = html("> [!DANGER]\n> Content.");
    assert!(out.contains("<blockquote>"));
    assert!(!out.contains("markdown-alert"));
    // The [!DANGER] text should appear as content
    assert!(out.contains("[!DANGER]"));
}

// --- Extra text after marker → regular blockquote ---

#[test]
fn extra_text_after_marker() {
    let out = html("> [!NOTE] extra text\n> Content.");
    assert!(out.contains("<blockquote>"));
    assert!(!out.contains("markdown-alert"));
    assert!(out.contains("[!NOTE] extra text"));
}

// --- Empty callout (no content after marker) ---

#[test]
fn empty_callout() {
    let out = html("> [!NOTE]");
    assert!(out.contains("markdown-alert-note"));
    assert!(out.contains(">Note</p>"));
    assert!(out.contains("</div>"));
}

// --- Multiple paragraphs inside callout ---

#[test]
fn callout_multiple_paragraphs() {
    let out = html("> [!NOTE]\n> First paragraph.\n>\n> Second paragraph.");
    assert!(out.contains("markdown-alert-note"));
    assert!(out.contains("First paragraph."));
    assert!(out.contains("Second paragraph."));
}

// --- Inline formatting inside callout ---

#[test]
fn callout_with_inline_formatting() {
    let out = html("> [!NOTE]\n> This is **bold** and *italic*.");
    assert!(out.contains("markdown-alert-note"));
    assert!(out.contains("<strong>bold</strong>"));
    assert!(out.contains("<em>italic</em>"));
}

// --- Nested callout inside callout ---

#[test]
fn nested_callout() {
    let out = html("> [!NOTE]\n> > [!WARNING]\n> > Inner warning.");
    assert!(out.contains("markdown-alert-note"));
    assert!(out.contains("markdown-alert-warning"));
    assert!(out.contains("Inner warning."));
}

// --- Callout marker not on first line → regular blockquote ---

#[test]
fn callout_not_on_first_line() {
    let out = html("> Some text\n> [!NOTE]");
    assert!(out.contains("<blockquote>"));
    // The [!NOTE] is just content, not a callout
    assert!(!out.contains("markdown-alert"));
}

// --- Callout disabled via options ---

#[test]
fn callout_disabled() {
    let out = html_no_callouts("> [!NOTE]\n> Content.");
    assert!(out.contains("<blockquote>"));
    assert!(!out.contains("markdown-alert"));
    assert!(out.contains("[!NOTE]"));
}

// --- Trailing whitespace on marker line ---

#[test]
fn callout_trailing_whitespace() {
    let out = html("> [!NOTE]   \n> Content.");
    assert!(out.contains("markdown-alert-note"));
}

// --- Callout inside list item ---

#[test]
fn callout_inside_list() {
    let out = html("- item\n\n  > [!TIP]\n  > A tip in a list.");
    assert!(out.contains("markdown-alert-tip"));
    assert!(out.contains("A tip in a list."));
}

// --- Callout with code block inside ---

#[test]
fn callout_with_code_block() {
    let out = html("> [!NOTE]\n> ```\n> code\n> ```");
    assert!(out.contains("markdown-alert-note"));
    assert!(out.contains("<code>"));
    assert!(out.contains("code"));
}
