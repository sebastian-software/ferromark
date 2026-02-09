use ferromark::{Options, parse, parse_with_options, to_html_with_options};

#[test]
fn yaml_basic() {
    let result = parse("---\ntitle: Hello\n---\n# Content");
    assert_eq!(result.front_matter, Some("title: Hello\n"));
    assert!(result.html.contains("Content</h1>"));
}

#[test]
fn toml_basic() {
    let result = parse("+++\ntitle = \"Hello\"\n+++\n# Content");
    assert_eq!(result.front_matter, Some("title = \"Hello\"\n"));
    assert!(result.html.contains("Content</h1>"));
}

#[test]
fn no_front_matter() {
    let result = parse("# Content");
    assert_eq!(result.front_matter, None);
    assert!(result.html.contains("Content</h1>"));
}

#[test]
fn option_disabled() {
    // With front_matter: false (default), `---` is treated as thematic break
    let input = "---\ntitle: Hello\n---\n";
    let options = Options::default();
    assert!(!options.front_matter);
    let result = parse_with_options(input, &options);
    assert_eq!(result.front_matter, None);
    assert!(result.html.contains("<hr />"));
}

#[test]
fn empty_front_matter() {
    let result = parse("---\n---\nContent");
    assert_eq!(result.front_matter, Some(""));
    assert!(result.html.contains("<p>Content</p>"));
}

#[test]
fn no_closing_delimiter() {
    // Without closing delimiter, entire doc is markdown
    let result = parse("---\ntitle: Hello\nno closing");
    assert_eq!(result.front_matter, None);
    // The `---` becomes a thematic break or setext heading
}

#[test]
fn four_dashes_not_front_matter() {
    let result = parse("----\ntitle: Hello\n----\n# Content");
    assert_eq!(result.front_matter, None);
}

#[test]
fn mixed_delimiters_dont_match() {
    let result = parse("---\ntitle: Hello\n+++\n# Content");
    assert_eq!(result.front_matter, None);
}

#[test]
fn trailing_whitespace_on_delimiters() {
    let result = parse("---  \ntitle: Hello\n---\t\n# Content");
    assert_eq!(result.front_matter, Some("title: Hello\n"));
    assert!(result.html.contains("Content</h1>"));
}

#[test]
fn blank_lines_inside_front_matter() {
    let result = parse("---\ntitle: Hello\n\ndescription: World\n---\n# Content");
    assert_eq!(
        result.front_matter,
        Some("title: Hello\n\ndescription: World\n")
    );
    assert!(result.html.contains("Content</h1>"));
}

#[test]
fn crlf_line_endings() {
    let result = parse("---\r\ntitle: Hello\r\n---\r\n# Content");
    assert_eq!(result.front_matter, Some("title: Hello\r\n"));
    assert!(result.html.contains("Content</h1>"));
}

#[test]
fn to_html_with_front_matter_strips() {
    let options = Options {
        front_matter: true,
        ..Options::default()
    };
    let html = to_html_with_options("---\ntitle: Hello\n---\n# Content", &options);
    assert!(html.contains("Content</h1>"));
    assert!(!html.contains("title"));
}

#[test]
fn front_matter_at_eof_no_trailing_content() {
    let result = parse("---\ntitle: x\n---");
    assert_eq!(result.front_matter, Some("title: x\n"));
    assert_eq!(result.html, "");
}

#[test]
fn front_matter_multiline_yaml() {
    let input = "---\ntitle: Hello\nauthor: World\ntags:\n  - rust\n  - markdown\n---\n\n# Doc\n";
    let result = parse(input);
    assert_eq!(
        result.front_matter,
        Some("title: Hello\nauthor: World\ntags:\n  - rust\n  - markdown\n")
    );
    assert!(result.html.contains("Doc</h1>"));
}

#[test]
fn not_at_document_start() {
    // Front matter must be at byte 0
    let result = parse("\n---\ntitle: Hello\n---\n# Content");
    assert_eq!(result.front_matter, None);
}

#[test]
fn space_before_delimiter_not_front_matter() {
    let result = parse(" ---\ntitle: Hello\n---\n");
    assert_eq!(result.front_matter, None);
}

#[test]
fn plus_four_not_front_matter() {
    let result = parse("++++\ntitle: Hello\n++++\n# Content");
    assert_eq!(result.front_matter, None);
}
