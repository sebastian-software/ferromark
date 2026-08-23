use std::fmt::Write;

use crate::{Options, RenderPolicy};

use super::{
    Segment,
    expr::find_expression_end,
    jsx_tag::{TagInfo, parse_jsx_tag},
};

const COMPONENT_BODY_INDENT: &str = "      ";
const HTML_VOID_ELEMENTS: [&str; 14] = [
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

#[derive(Clone, Copy)]
enum HtmlTextKind {
    Raw,
    Rcdata,
}

/// Error returned when a component name cannot be used as a JavaScript binding.
///
/// Future releases may add validation failures. Downstream matches must include a wildcard arm.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ComponentNameError {
    /// The name is empty.
    Empty,
    /// The first character cannot start a JavaScript identifier.
    InvalidStart(char),
    /// A later character cannot continue a JavaScript identifier.
    InvalidContinue(char),
    /// The name is reserved in JavaScript module code.
    ReservedWord,
}

impl std::fmt::Display for ComponentNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => f.write_str("component name cannot be empty"),
            Self::InvalidStart(ch) => write!(
                f,
                "component name starts with {ch:?}, which is not valid in a JavaScript identifier"
            ),
            Self::InvalidContinue(ch) => write!(
                f,
                "component name contains {ch:?}, which is not valid in a JavaScript identifier"
            ),
            Self::ReservedWord => f.write_str("component name is reserved by JavaScript"),
        }
    }
}

impl std::error::Error for ComponentNameError {}

fn validate_component_name(name: &str) -> Result<(), ComponentNameError> {
    let mut chars = name.chars();
    let first = chars.next().ok_or(ComponentNameError::Empty)?;
    if !is_identifier_start(first) {
        return Err(ComponentNameError::InvalidStart(first));
    }
    for ch in chars {
        if !is_identifier_continue(ch) {
            return Err(ComponentNameError::InvalidContinue(ch));
        }
    }
    if is_reserved_word(name) {
        return Err(ComponentNameError::ReservedWord);
    }
    Ok(())
}

#[inline]
fn is_identifier_start(ch: char) -> bool {
    ch == '$' || ch == '_' || unicode_ident::is_xid_start(ch)
}

#[inline]
fn is_identifier_continue(ch: char) -> bool {
    ch == '$'
        || ch == '_'
        || ch == '\u{200c}'
        || ch == '\u{200d}'
        || unicode_ident::is_xid_continue(ch)
}

fn is_reserved_word(name: &str) -> bool {
    matches!(
        name,
        "arguments"
            | "await"
            | "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "enum"
            | "eval"
            | "export"
            | "extends"
            | "false"
            | "finally"
            | "for"
            | "function"
            | "if"
            | "implements"
            | "import"
            | "in"
            | "instanceof"
            | "interface"
            | "let"
            | "new"
            | "null"
            | "package"
            | "private"
            | "protected"
            | "public"
            | "return"
            | "static"
            | "super"
            | "switch"
            | "this"
            | "throw"
            | "true"
            | "try"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
            | "yield"
    )
}

/// Rendered MDX output with extracted metadata.
pub struct MdxOutput<'a> {
    /// Rendered body: Markdown→HTML, JSX/expressions passed through.
    pub body: String,
    /// ESM statements (import/export), in document order.
    pub esm: Vec<&'a str>,
    /// Front matter content (if present in first Markdown segment).
    pub front_matter: Option<&'a str>,
}

impl MdxOutput<'_> {
    /// Wrap the rendered output as a JSX/TSX component module.
    ///
    /// Produces a complete module with ESM statements at the top and a named
    /// export function that returns the body wrapped in a fragment.
    ///
    /// ```text
    /// import { Card } from './card'
    /// export const meta = { title: 'About' }
    ///
    /// export function About() {
    ///   return (
    ///     <>
    ///       <h1 id="about">About</h1>
    ///       ...
    ///     </>
    ///   );
    /// }
    /// ```
    ///
    /// Literal braces in rendered Markdown are emitted as HTML character
    /// references so they cannot be interpreted as JSX expressions. MDX flow
    /// expressions remain executable, HTML comments are omitted, and whitespace
    /// inside `<pre>` elements is preserved.
    pub fn to_component(&self, name: &str) -> Result<String, ComponentNameError> {
        validate_component_name(name)?;
        let mut out = String::with_capacity(self.body.len() + self.esm.len() * 40 + 80);

        for esm in &self.esm {
            out.push_str(esm.trim_end());
            out.push('\n');
        }
        if !self.esm.is_empty() {
            out.push('\n');
        }

        let _ = writeln!(out, "export function {name}() {{");
        out.push_str("  return (\n    <>\n");

        let body = self.body.trim();
        if !body.is_empty() {
            write_component_body(&mut out, body);
            out.push('\n');
        }

        out.push_str("    </>\n  );\n}\n");
        Ok(out)
    }
}

/// Write an HTML/MDX body as JSX without changing its rendered text semantics.
///
/// This deliberately operates on tokens instead of applying global replacements:
/// tags and MDX flow expressions must remain executable, while braces in HTML text
/// nodes are literal characters. The emitter also tracks `<pre>` ownership so its
/// presentation indentation never becomes part of preformatted content.
fn write_component_body(out: &mut String, body: &str) {
    let bytes = body.as_bytes();
    let mut pos = 0;
    let mut at_line_start = true;
    let mut pre_depth = 0_u32;
    let mut void_openings = [0_usize; HTML_VOID_ELEMENTS.len()];
    let mut void_named_component_openings = Vec::new();

    while pos < bytes.len() {
        if pre_depth > 0 && bytes[pos] != b'<' {
            let end = bytes[pos..]
                .iter()
                .position(|byte| *byte == b'<')
                .map_or(bytes.len(), |offset| pos + offset);
            write_preformatted_text(out, &body[pos..end]);
            at_line_start = body[pos..end].ends_with(['\n', '\r']);
            pos = end;
            continue;
        }

        if let Some(line_break_len) = line_break_len(bytes, pos) {
            out.push_str(&body[pos..pos + line_break_len]);
            pos += line_break_len;
            at_line_start = true;
            continue;
        }

        if bytes[pos..].starts_with(b"<!--") {
            pos = comment_end(bytes, pos).unwrap_or(bytes.len());
            continue;
        }

        if bytes[pos] == b'<' && matches!(bytes.get(pos + 1), Some(b'!' | b'?')) {
            if let Some(end) = declaration_end(bytes, pos) {
                pos = end;
                continue;
            }

            // An incomplete or declaration-like near-match is not an HTML node.
            // Escape only its opening delimiter so later real JSX/HTML nodes are
            // still processed instead of truncating the rest of the component.
            indent_component_line(out, at_line_start, pre_depth);
            out.push_str("&lt;");
            at_line_start = false;
            pos += 1;
            continue;
        }

        if bytes[pos] == b'<'
            && let Some(tag) = parse_jsx_tag(&bytes[pos..])
        {
            let text_element = if !tag.is_closing && !tag.is_self_closing {
                html_text_element(tag.name)
            } else {
                None
            };
            indent_component_line(out, at_line_start, pre_depth);
            if let Some((_, canonical_name)) = text_element {
                write_html_text_opening_tag(
                    out,
                    &body[pos..pos + tag.end_offset],
                    tag.name,
                    canonical_name,
                );
            } else {
                write_component_tag(
                    out,
                    &body[pos..pos + tag.end_offset],
                    &tag,
                    &mut void_openings,
                    &mut void_named_component_openings,
                );
            }
            at_line_start = false;

            if tag.name == "pre" && !tag.is_self_closing {
                if tag.is_closing {
                    pre_depth = pre_depth.saturating_sub(1);
                } else {
                    pre_depth = pre_depth.saturating_add(1);
                }
            }

            pos += tag.end_offset;
            if let Some((text_kind, canonical_name)) = text_element {
                if let Some((text_end, tag_end)) = raw_text_end(bytes, pos, tag.name) {
                    write_jsx_string_child(
                        out,
                        &body[pos..text_end],
                        matches!(text_kind, HtmlTextKind::Rcdata),
                    );
                    write_html_text_closing_tag(out, &body[text_end..tag_end], canonical_name);
                    pos = tag_end;
                } else {
                    write_jsx_string_child(
                        out,
                        &body[pos..],
                        matches!(text_kind, HtmlTextKind::Rcdata),
                    );
                    pos = bytes.len();
                }
                // The raw text's line breaks are escaped inside the string child,
                // so the generated JSX source is still on the opening tag's line.
                at_line_start = false;
            }
            continue;
        }

        if at_line_start
            && pre_depth == 0
            && bytes[pos] == b'{'
            && let Some(end) = standalone_expression_end(bytes, pos)
        {
            indent_component_line(out, true, pre_depth);
            out.push_str(&body[pos..end]);
            at_line_start = body[pos..end].ends_with(['\n', '\r']);
            pos = end;
            continue;
        }

        indent_component_line(out, at_line_start, pre_depth);
        at_line_start = false;

        match bytes[pos] {
            b'{' => {
                out.push_str("&#123;");
                pos += 1;
            }
            b'}' => {
                out.push_str("&#125;");
                pos += 1;
            }
            _ => {
                let ch = body[pos..]
                    .chars()
                    .next()
                    .expect("position is inside the body");
                out.push(ch);
                pos += ch.len_utf8();
            }
        }
    }
}

fn write_component_tag<'a>(
    out: &mut String,
    source: &str,
    tag: &TagInfo<'a>,
    void_openings: &mut [usize; HTML_VOID_ELEMENTS.len()],
    void_named_component_openings: &mut Vec<&'a str>,
) {
    if tag.is_closing {
        if void_named_component_openings.last() == Some(&tag.name) {
            void_named_component_openings.pop();
            out.push_str(source);
            return;
        }

        if let Some(index) = html_void_element_index(tag.name) {
            void_openings[index] = void_openings[index].saturating_sub(1);
            return;
        }
        if let Some(index) = html_void_element_index_ignore_ascii_case(tag.name)
            && void_openings[index] > 0
        {
            void_openings[index] -= 1;
            return;
        }

        out.push_str(source);
        return;
    }

    if let Some(index) = html_void_element_index(tag.name) {
        void_openings[index] += 1;
        if tag.is_self_closing {
            out.push_str(source);
            return;
        }

        let before_close = &source[..source.len() - 1];
        out.push_str(before_close);
        if before_close.ends_with(char::is_whitespace) {
            out.push('/');
        } else {
            out.push_str(" /");
        }
        out.push('>');
        return;
    }

    if !tag.is_self_closing && html_void_element_index_ignore_ascii_case(tag.name).is_some() {
        void_named_component_openings.push(tag.name);
    }
    out.push_str(source);
}

fn html_void_element_index(name: &str) -> Option<usize> {
    HTML_VOID_ELEMENTS
        .iter()
        .position(|element| *element == name)
}

fn html_void_element_index_ignore_ascii_case(name: &str) -> Option<usize> {
    HTML_VOID_ELEMENTS
        .iter()
        .position(|element| element.eq_ignore_ascii_case(name))
}

fn html_text_element(name: &str) -> Option<(HtmlTextKind, &'static str)> {
    // Lowercase-leading and all-uppercase names use HTML intrinsic semantics in
    // JSX. Preserve PascalCase/mixed-leading-uppercase names as MDX components.
    let html_syntax = name.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        || name
            .bytes()
            .all(|byte| !byte.is_ascii_alphabetic() || byte.is_ascii_uppercase());
    if !html_syntax {
        return None;
    }

    for (canonical, kind) in [
        ("script", HtmlTextKind::Raw),
        ("style", HtmlTextKind::Raw),
        ("xmp", HtmlTextKind::Raw),
        ("iframe", HtmlTextKind::Raw),
        ("noembed", HtmlTextKind::Raw),
        ("noframes", HtmlTextKind::Raw),
        ("textarea", HtmlTextKind::Rcdata),
        ("title", HtmlTextKind::Rcdata),
    ] {
        if name.eq_ignore_ascii_case(canonical) {
            return Some((kind, canonical));
        }
    }

    None
}

fn indent_component_line(out: &mut String, at_line_start: bool, pre_depth: u32) {
    if at_line_start && pre_depth == 0 {
        out.push_str(COMPONENT_BODY_INDENT);
    }
}

fn write_preformatted_text(out: &mut String, text: &str) {
    write_jsx_string_child(out, text, true);
}

fn write_jsx_string_child(out: &mut String, text: &str, decode_entities: bool) {
    let decoded = if decode_entities {
        crate::render::decode_entities_commonmark(text)
    } else {
        std::borrow::Cow::Borrowed(text)
    };
    out.push_str("{\"");

    for ch in decoded.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{0008}' => out.push_str("\\b"),
            '\u{000c}' => out.push_str("\\f"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            ch if ch.is_control() => {
                let _ = write!(out, "\\u{:04x}", ch as u32);
            }
            _ => out.push(ch),
        }
    }

    out.push_str("\"}");
}

fn write_html_text_closing_tag(out: &mut String, source: &str, opening_name: &str) {
    debug_assert!(source.starts_with("</"));
    debug_assert!(source.len() >= opening_name.len() + 2);
    out.push_str("</");
    out.push_str(opening_name);
    out.push_str(&source[opening_name.len() + 2..]);
}

fn write_html_text_opening_tag(
    out: &mut String,
    source: &str,
    original_name: &str,
    canonical_name: &str,
) {
    debug_assert!(source.starts_with('<'));
    debug_assert!(source.len() > original_name.len());
    out.push('<');
    out.push_str(canonical_name);
    out.push_str(&source[original_name.len() + 1..]);
}

fn raw_text_end(bytes: &[u8], start: usize, tag_name: &str) -> Option<(usize, usize)> {
    let mut pos = start;

    while let Some(offset) = bytes[pos..].iter().position(|byte| *byte == b'<') {
        pos += offset;
        if let Some(tag) = parse_jsx_tag(&bytes[pos..])
            && tag.is_closing
            && tag.name.eq_ignore_ascii_case(tag_name)
        {
            return Some((pos, pos + tag.end_offset));
        }
        pos += 1;
    }

    None
}

fn line_break_len(bytes: &[u8], pos: usize) -> Option<usize> {
    match bytes[pos] {
        b'\n' => Some(1),
        b'\r' if bytes.get(pos + 1) == Some(&b'\n') => Some(2),
        b'\r' => Some(1),
        _ => None,
    }
}

fn comment_end(bytes: &[u8], start: usize) -> Option<usize> {
    bytes[start + 4..]
        .windows(3)
        .position(|window| window == b"-->")
        .map(|offset| start + 4 + offset + 3)
}

fn declaration_end(bytes: &[u8], start: usize) -> Option<usize> {
    if bytes[start..].starts_with(b"<![CDATA[") {
        return bytes[start + 9..]
            .windows(3)
            .position(|window| window == b"]]>")
            .map(|offset| start + 9 + offset + 3);
    }

    if bytes[start..].starts_with(b"<?") {
        return bytes[start + 2..]
            .windows(2)
            .position(|window| window == b"?>")
            .map(|offset| start + 2 + offset + 2);
    }

    if bytes
        .get(start + 2)
        .is_none_or(|byte| !byte.is_ascii_alphabetic())
    {
        return None;
    }

    bytes[start + 2..]
        .iter()
        .position(|byte| *byte == b'>')
        .map(|offset| start + 2 + offset + 1)
}

fn standalone_expression_end(bytes: &[u8], start: usize) -> Option<usize> {
    let relative_end = find_expression_end(&bytes[start..])?;
    let end = start + relative_end;
    let mut pos = end;

    while pos < bytes.len() && bytes[pos] != b'\n' && bytes[pos] != b'\r' {
        if !bytes[pos].is_ascii_whitespace() {
            return None;
        }
        pos += 1;
    }

    Some(end)
}

/// Render MDX to HTML body with default options.
///
/// # Panics
///
/// Panics when the input exceeds [`crate::MAX_INPUT_BYTES`]. Use
/// [`try_render`] to handle the limit as an error.
pub fn render(input: &str) -> MdxOutput<'_> {
    try_render(input).unwrap_or_else(|error| panic!("{error}"))
}

/// Render MDX to HTML without panicking for oversized input.
pub fn try_render(input: &str) -> Result<MdxOutput<'_>, crate::InputSizeError> {
    try_render_with_options(input, &mdx_default_options())
}

/// Render MDX to HTML body with custom Markdown options.
///
/// # Panics
///
/// Panics when the input exceeds [`crate::MAX_INPUT_BYTES`]. Use
/// [`try_render_with_options`] to handle the limit as an error.
pub fn render_with_options<'a>(input: &'a str, options: &Options) -> MdxOutput<'a> {
    try_render_with_options(input, options).unwrap_or_else(|error| panic!("{error}"))
}

/// Render MDX to HTML with custom Markdown options without panicking for
/// oversized input.
pub fn try_render_with_options<'a>(
    input: &'a str,
    options: &Options,
) -> Result<MdxOutput<'a>, crate::InputSizeError> {
    let segments = super::try_segment(input)?;
    let mut body = String::with_capacity(input.len());
    let mut esm: Vec<&'a str> = Vec::new();
    let mut front_matter: Option<&'a str> = None;

    for seg in &segments {
        match seg {
            Segment::Esm(s) => {
                esm.push(s);
            }
            Segment::Markdown(s) => {
                let result = crate::try_parse_with_options(s, options)?;
                body.push_str(&result.html);
                if front_matter.is_none() {
                    front_matter = result.front_matter;
                }
            }
            Segment::JsxBlockOpen(s)
            | Segment::JsxBlockClose(s)
            | Segment::JsxBlockSelfClose(s)
            | Segment::Expression(s) => {
                body.push_str(s.trim());
                body.push('\n');
            }
        }
    }

    Ok(MdxOutput {
        body,
        esm,
        front_matter,
    })
}

fn mdx_default_options() -> Options {
    Options {
        render_policy: RenderPolicy::Trusted,
        allow_html: true,
        disallowed_raw_html: false,
        front_matter: true,
        heading_ids: true,
        ..Options::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pure_markdown() {
        let out = render("# Hello\n\nWorld\n");
        assert!(out.body.contains("<h1"));
        assert!(out.body.contains("Hello"));
        assert!(out.body.contains("<p>World</p>"));
        assert!(out.esm.is_empty());
        assert!(out.front_matter.is_none());
    }

    #[test]
    fn only_esm() {
        let out = render("import A from 'a'\nexport const x = 1\n");
        assert_eq!(out.esm.len(), 2);
        assert!(out.esm[0].contains("import A"));
        assert!(out.esm[1].contains("export const"));
        // Body may contain whitespace from blank markdown segments, but no HTML tags
        assert!(!out.body.contains('<'));
    }

    #[test]
    fn mixed_esm_markdown_jsx_expression() {
        let input = "\
import { Card } from './card'
export const meta = { title: 'Test' }

# Title

Paragraph.

<Card title=\"hello\">

## Inside

</Card>

{new Date().getFullYear()}
";
        let out = render(input);
        assert_eq!(out.esm.len(), 2);
        assert!(out.body.contains("<h1"));
        assert!(out.body.contains("<p>Paragraph.</p>"));
        assert!(out.body.contains("<Card title=\"hello\">"));
        assert!(out.body.contains("</Card>"));
        assert!(out.body.contains("<h2"));
        assert!(out.body.contains("Inside"));
        assert!(out.body.contains("new Date().getFullYear()"));
    }

    #[test]
    fn front_matter_extraction() {
        let input = "---\ntitle: Hello\n---\n\n# Heading\n";
        let out = render(input);
        assert_eq!(out.front_matter, Some("title: Hello\n"));
        assert!(out.body.contains("<h1"));
    }

    #[test]
    fn readme_render_example_extracts_front_matter_before_esm() {
        let input = r#"---
title: Hello
---

import { Card } from './card'

# Hello World

<Card title="Example">

Markdown **inside** a component.

</Card>

{new Date().getFullYear()}
"#;
        let out = render(input);

        assert_eq!(out.front_matter, Some("title: Hello\n"));
        assert_eq!(out.esm, ["import { Card } from './card'\n"]);
        assert!(out.body.contains("<h1 id=\"hello-world\">Hello World</h1>"));
        assert!(out.body.contains("<Card title=\"Example\">"));
        assert!(out.body.contains("<strong>inside</strong>"));
        assert!(out.body.contains("{new Date().getFullYear()}"));
        assert!(!out.body.contains("title: Hello"));
    }

    #[test]
    fn inline_html_passthrough() {
        let input = "Text with <sl-button>Click</sl-button> here.\n";
        let out = render(input);
        assert!(out.body.contains("<sl-button>Click</sl-button>"));
    }

    #[test]
    fn empty_input() {
        let out = render("");
        assert!(out.body.is_empty());
        assert!(out.esm.is_empty());
        assert!(out.front_matter.is_none());
    }

    #[test]
    fn jsx_trimmed_consistently() {
        let out = render("<Card>\nContent\n</Card>\n");
        // JSX tags should be trimmed and have exactly one newline
        assert!(out.body.contains("<Card>\n"));
        assert!(out.body.contains("</Card>\n"));
        // No double newlines from trailing whitespace
        assert!(!out.body.contains("<Card>\n\n"));
    }

    #[test]
    fn disallowed_html_off_by_default() {
        // script tags should pass through in MDX mode
        let input = "<script>alert('hi')</script>\n";
        let out = render(input);
        // The segmenter treats lowercase HTML as markdown (not JSX),
        // so it goes through the markdown parser. With disallowed_raw_html=false,
        // script should NOT be filtered.
        assert!(out.body.contains("<script>"));
    }

    #[test]
    fn custom_options() {
        let input = "# Heading\n\n~~struck~~\n";
        let opts = Options {
            strikethrough: true,
            allow_html: true,
            disallowed_raw_html: false,
            heading_ids: false,
            ..Options::default()
        };
        let out = render_with_options(input, &opts);
        assert!(out.body.contains("<del>struck</del>"));
        // No id attribute since heading_ids is false
        assert!(!out.body.contains("id="));
    }

    #[test]
    fn to_component_full() {
        let input = "\
import { Card } from './card'
export const meta = { title: 'Test' }

# Title

<Card>

Content

</Card>
";
        let out = render(input);
        let comp = out.to_component("About").unwrap();

        // ESM at top
        assert!(comp.starts_with("import { Card } from './card'\n"));
        assert!(comp.contains("export const meta = { title: 'Test' }\n"));

        // Named export, not default
        assert!(comp.contains("export function About() {"));
        assert!(!comp.contains("default"));

        // Fragment wrapper
        assert!(comp.contains("<>"));
        assert!(comp.contains("</>"));

        // Body indented inside fragment
        assert!(comp.contains("      <h1"));
        assert!(comp.contains("      <Card>"));
    }

    #[test]
    fn to_component_no_esm() {
        let out = render("# Hello\n");
        let comp = out.to_component("Page").unwrap();

        // Starts directly with export, no blank line
        assert!(comp.starts_with("export function Page() {"));
    }

    #[test]
    fn to_component_empty_body() {
        let out = render("import A from 'a'\n");
        let comp = out.to_component("Empty").unwrap();

        assert!(comp.contains("import A from 'a'"));
        assert!(comp.contains("<>\n    </>"));
    }

    #[test]
    fn to_component_escapes_markdown_braces_but_preserves_mdx_expressions() {
        let out = render(
            "Use {braces}, `code {braces}`, and `{'{'}literal}`.\n\n{value ?? { nested: true }}\n",
        );
        let component = out.to_component("Page").unwrap();

        assert!(component.contains(
            "<p>Use &#123;braces&#125;, <code>code &#123;braces&#125;</code>, and <code>&#123;'&#123;'&#125;literal&#125;</code>.</p>"
        ));
        assert!(component.contains("      {value ?? { nested: true }}\n"));
    }

    #[test]
    fn to_component_escapes_fenced_code_braces() {
        let out = render("```rust\nfn main() { println!(\"hi\"); }\n```\n");
        let component = out.to_component("Page").unwrap();

        assert!(component.contains("{\"fn main() { println!(\\\"hi\\\"); }\\n\"}"));
        assert!(!component.contains("fn main() { println!(\"hi\"); }"));
    }

    #[test]
    fn to_component_drops_html_comments() {
        let out = render("Before.\n\n<!-- comment with */ and {brace} -->\n\nAfter.\n");
        assert!(out.body.contains("<!-- comment with */ and {brace} -->"));

        let component = out.to_component("Page").unwrap();

        assert!(!component.contains("<!--"));
        assert!(!component.contains("comment with"));
        assert!(component.contains("<p>Before.</p>"));
        assert!(component.contains("<p>After.</p>"));
    }

    #[test]
    fn to_component_preserves_preformatted_whitespace_and_crlf() {
        let out = MdxOutput {
            body: "<pre data-kind=\"example\"><code>first\r\n  second {value}\r\n</code></pre>\r\n<p>After</p>"
                .to_owned(),
            esm: vec![],
            front_matter: None,
        };
        let component = out.to_component("Page").unwrap();

        assert!(component.contains(
            "      <pre data-kind=\"example\"><code>{\"first\\r\\n  second {value}\\r\\n\"}</code></pre>\r\n      <p>After</p>"
        ));
    }

    #[test]
    fn to_component_keeps_existing_entities_and_jsx_flow() {
        let out = render(
            "<Card value={{ nested: true }}>\n\nLiteral &#123; and {'{'}text}.\n\n</Card>\n",
        );
        let component = out.to_component("Page").unwrap();

        assert!(component.contains("      <Card value={{ nested: true }}>"));
        assert!(component.contains("Literal &#123; and &#123;'&#123;'&#125;text&#125;."));
        assert!(component.contains("      </Card>"));
    }

    #[test]
    fn to_component_keeps_escaped_comments_visible() {
        let options = Options {
            allow_html: false,
            ..Options::default()
        };
        let out = render_with_options("<!-- visible -->\n", &options);
        let component = out.to_component("Page").unwrap();

        assert!(component.contains("&lt;!-- visible --&gt;"));
    }

    #[test]
    fn to_component_preserves_entities_in_preformatted_text() {
        let out = MdxOutput {
            body: "<pre><code>&lt;&amp;&quot;&#123;&#125;&ngE;&#0;\n</code></pre>".to_owned(),
            esm: vec![],
            front_matter: None,
        };
        let component = out.to_component("Page").unwrap();

        assert!(component.contains("<pre><code>{\"<&\\\"{}≧̸�\\n\"}</code></pre>"));
    }

    #[test]
    fn to_component_preserves_raw_script_and_style_text() {
        let out = MdxOutput {
            body: "<script>const value = { text: \"<!-- &amp;\" };\n</script>\n<style>.x { color: red } /* <!-- */</style>"
                .to_owned(),
            esm: vec![],
            front_matter: None,
        };
        let component = out.to_component("Page").unwrap();

        assert!(
            component
                .contains("<script>{\"const value = { text: \\\"<!-- &amp;\\\" };\\n\"}</script>"),
            "{component}"
        );
        assert!(
            component.contains("<style>{\".x { color: red } /* <!-- */\"}</style>"),
            "{component}"
        );
    }

    #[test]
    fn to_component_self_closes_html_void_elements() {
        let out = render("Before.\n\n---\n\n![alt](image.png)\n\nline  \nbreak\n");
        let component = out.to_component("Page").unwrap();

        assert!(component.contains("<hr />"), "{component}");
        assert!(
            component.contains("<img src=\"image.png\" alt=\"alt\" />"),
            "{component}"
        );
        assert!(component.contains("line<br />\n"), "{component}");
    }

    #[test]
    fn to_component_omits_html_void_element_closing_tags() {
        let body = "<area data-kind=\"example\"></AREA   ><base ></base><br></BR><col></col><embed></embed><hr></hr><img src=\"image.png\"></IMG><input></input><link></link><meta></meta><param></param><source></source><track></track><wbr></wBr>";
        let out = MdxOutput {
            body: body.to_owned(),
            esm: vec![],
            front_matter: None,
        };
        let component = out.to_component("Page").unwrap();

        assert_eq!(out.body, body);
        for expected in [
            "<area data-kind=\"example\" />",
            "<base />",
            "<br />",
            "<col />",
            "<embed />",
            "<hr />",
            "<img src=\"image.png\" />",
            "<input />",
            "<link />",
            "<meta />",
            "<param />",
            "<source />",
            "<track />",
            "<wbr />",
        ] {
            assert!(
                component.contains(expected),
                "missing {expected:?}: {component}"
            );
        }
        for omitted in [
            "</AREA", "</base", "</BR", "</col", "</embed", "</hr", "</IMG", "</input", "</link",
            "</meta", "</param", "</source", "</track", "</wBr",
        ] {
            assert!(
                !component.contains(omitted),
                "retained {omitted:?}: {component}"
            );
        }
    }

    #[test]
    fn to_component_omits_void_closing_tag_from_rendered_markdown() {
        let out = render("a<br></br>z");
        assert!(out.body.contains("<p>a<br></br>z</p>"), "{}", out.body);

        let component = out.to_component("Page").unwrap();
        assert!(component.contains("<p>a<br />z</p>"), "{component}");
        assert!(!component.contains("</br>"), "{component}");
    }

    #[test]
    fn to_component_preserves_non_void_and_component_tag_pairs() {
        let body = "<div data-kind=\"normal\"></div><bravo></bravo><custom-br></custom-br><br><Br></Br></BR><Img></Img><br></bravo>";
        let out = MdxOutput {
            body: body.to_owned(),
            esm: vec![],
            front_matter: None,
        };
        let component = out.to_component("Page").unwrap();

        assert!(
            component.contains("<div data-kind=\"normal\"></div>"),
            "{component}"
        );
        assert!(component.contains("<bravo></bravo>"), "{component}");
        assert!(component.contains("<custom-br></custom-br>"), "{component}");
        assert!(component.contains("<br /><Br></Br>"), "{component}");
        assert!(!component.contains("</BR>"), "{component}");
        assert!(component.contains("<Img></Img>"), "{component}");
        assert!(component.contains("<br /></bravo>"), "{component}");
    }

    #[test]
    fn to_component_omits_html_declarations() {
        let out = MdxOutput {
            body:
                "<!DOCTYPE html>\n<?xml version=\"1.0\"?>\n<![CDATA[ignored > text]]>\n<p>Safe</p>"
                    .to_owned(),
            esm: vec![],
            front_matter: None,
        };
        let component = out.to_component("Page").unwrap();

        assert!(!component.contains("<!"), "{component}");
        assert!(!component.contains("<?"), "{component}");
        assert!(!component.contains("ignored"), "{component}");
        assert!(component.contains("<p>Safe</p>"), "{component}");
    }

    #[test]
    fn to_component_uses_html_declaration_boundaries_without_truncation() {
        let body = "<!DOCTYPE html data='unterminated><p>after doctype</p><?pi data='unterminated?><p>after pi</p><![CDATA[ignored > text]]><p>after cdata</p>";
        let out = MdxOutput {
            body: body.to_owned(),
            esm: vec![],
            front_matter: None,
        };
        let component = out.to_component("Page").unwrap();

        assert_eq!(out.body, body);
        assert!(!component.contains("DOCTYPE"), "{component}");
        assert!(!component.contains("<?pi"), "{component}");
        assert!(!component.contains("CDATA"), "{component}");
        for trailing in ["after doctype", "after pi", "after cdata"] {
            assert!(
                component.contains(trailing),
                "missing {trailing:?}: {component}"
            );
        }

        let rendered = render(
            "before <!DOCTYPE html data='unterminated> after declaration\n\n<p>after node</p>",
        );
        assert!(
            rendered.body.contains("after declaration"),
            "{}",
            rendered.body
        );
        let rendered_component = rendered.to_component("Page").unwrap();
        assert!(
            rendered_component.contains("after declaration"),
            "{rendered_component}"
        );
        assert!(
            rendered_component.contains("<p>after node</p>"),
            "{rendered_component}"
        );
    }

    #[test]
    fn to_component_bounds_incomplete_declarations_and_near_matches() {
        for body in [
            "<![CDATA[unterminated<p>after cdata</p>",
            "<?unterminated<p>after pi</p>",
            "<![near-match><p>after bracket near-match</p>",
            "<!-near-match><p>after punctuation near-match</p>",
        ] {
            let out = MdxOutput {
                body: body.to_owned(),
                esm: vec![],
                front_matter: None,
            };
            let component = out.to_component("Page").unwrap();

            assert!(component.contains("&lt;"), "{component}");
            assert!(component.contains("<p>after"), "{component}");
        }
    }

    #[test]
    fn to_component_preserves_textarea_and_title_text() {
        let out = MdxOutput {
            body: "<textarea data-kind=\"example\">first\r\n  second {value}&amp;\r\n</textarea>\r\n<title>first\n  second {value}&amp;\n</title>"
                .to_owned(),
            esm: vec![],
            front_matter: None,
        };
        let component = out.to_component("Page").unwrap();

        assert!(component.contains(
            "<textarea data-kind=\"example\">{\"first\\r\\n  second {value}&\\r\\n\"}</textarea>"
        ), "{component}");
        assert!(
            component.contains("<title>{\"first\\n  second {value}&\\n\"}</title>"),
            "{component}"
        );
    }

    #[test]
    fn to_component_matches_html_text_end_tags_ascii_case_insensitively() {
        let out = MdxOutput {
            body: "<textarea data-kind=\"example\">first\r\n  second {value}&amp;\r\n</TEXTAREA   ><p>after textarea</p>\n<title data-kind=\"example\">heading &amp; {value}</TiTlE><p>after title</p>\n<script>if (left < right) { run(); }</SCRIPT><p>after script</p>\n<style>.x { color: red }</StYlE><p>after style</p>"
                .to_owned(),
            esm: vec![],
            front_matter: None,
        };
        let component = out.to_component("Page").unwrap();

        assert!(component.contains(
            "<textarea data-kind=\"example\">{\"first\\r\\n  second {value}&\\r\\n\"}</textarea   ><p>after textarea</p>"
        ), "{component}");
        assert!(
            component.contains(
                "<title data-kind=\"example\">{\"heading & {value}\"}</title><p>after title</p>"
            ),
            "{component}"
        );
        assert!(
            component
                .contains("<script>{\"if (left < right) { run(); }\"}</script><p>after script</p>"),
            "{component}"
        );
        assert!(
            component.contains("<style>{\".x { color: red }\"}</style><p>after style</p>"),
            "{component}"
        );
    }

    #[test]
    fn to_component_preserves_case_variant_html_text_semantics() {
        let body = "<TEXTAREA data-kind=\"upper\">first\r\n  second {value}&amp;\r\n</textAREA><p>after textarea</p>\n<textAREA data-kind=\"mixed\">third\n  fourth &amp; {value}\n</TEXTAREA><TITLE>heading\n  continuation &amp; {value}</title><SCRIPT>if (left < right) { run(); }</script><stYLE>.x { color: red }</STYLE>";
        let out = MdxOutput {
            body: body.to_owned(),
            esm: vec![],
            front_matter: None,
        };
        let component = out.to_component("Page").unwrap();

        assert_eq!(out.body, body);
        assert!(component.contains(
            "<textarea data-kind=\"upper\">{\"first\\r\\n  second {value}&\\r\\n\"}</textarea><p>after textarea</p>"
        ), "{component}");
        assert!(
            component.contains(
                "<textarea data-kind=\"mixed\">{\"third\\n  fourth & {value}\\n\"}</textarea>"
            ),
            "{component}"
        );
        assert!(
            component.contains("<title>{\"heading\\n  continuation & {value}\"}</title>"),
            "{component}"
        );
        assert!(
            component.contains("<script>{\"if (left < right) { run(); }\"}</script>"),
            "{component}"
        );
        assert!(
            component.contains("<style>{\".x { color: red }\"}</style>"),
            "{component}"
        );
    }

    #[test]
    fn to_component_ignores_raw_text_closing_tag_near_matches() {
        let out = MdxOutput {
            body: "<textarea>before</TEXTAREAX>middle</textarea extra>still</TeXtArEa><p>after</p>\n<script>before</SCRIPTS>middle</ScRiPt><p>after script</p>"
                .to_owned(),
            esm: vec![],
            front_matter: None,
        };
        let component = out.to_component("Page").unwrap();

        assert!(
            component.contains(
                "<textarea>{\"before</TEXTAREAX>middle</textarea extra>still\"}</textarea><p>after</p>"
            ),
            "{component}"
        );
        assert!(
            component.contains("<script>{\"before</SCRIPTS>middle\"}</script><p>after script</p>"),
            "{component}"
        );
    }

    #[test]
    fn to_component_does_not_treat_component_names_as_html_text_elements() {
        let out = render(
            "<Pre>\n\n{value}\n\n</Pre>\n\n<Style>\n\n{text}\n\n</Style>\n\n<Textarea>\n\n{value}\n\n</Textarea>\n\n<Title>\n\n{text}\n\n</Title>\n",
        );
        let component = out.to_component("Page").unwrap();

        assert!(
            component.contains("      <Pre>\n      {value}\n      </Pre>"),
            "{component}"
        );
        assert!(
            component.contains("      <Style>\n      {text}\n      </Style>"),
            "{component}"
        );
        assert!(
            component.contains("      <Textarea>\n      {value}\n      </Textarea>"),
            "{component}"
        );
        assert!(
            component.contains("      <Title>\n      {text}\n      </Title>"),
            "{component}"
        );
    }

    #[test]
    fn to_component_accepts_unicode_identifier() {
        let out = render("# Hello\n");
        let component = out.to_component("Überblick").unwrap();

        assert!(component.contains("export function Überblick()"));
    }

    #[test]
    fn to_component_rejects_invalid_or_reserved_names() {
        let out = render("# Hello\n");

        assert_eq!(
            out.to_component("getting-started"),
            Err(ComponentNameError::InvalidContinue('-'))
        );
        assert_eq!(
            out.to_component("2026Report"),
            Err(ComponentNameError::InvalidStart('2'))
        );
        assert_eq!(
            out.to_component("default"),
            Err(ComponentNameError::ReservedWord)
        );
        assert_eq!(
            out.to_component("Page() {}\nexport const injected = true; //"),
            Err(ComponentNameError::InvalidContinue('('))
        );
    }
}
