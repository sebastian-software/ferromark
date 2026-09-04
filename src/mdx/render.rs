use std::{collections::HashMap, fmt::Write};

use crate::{
    BlockEvent, BlockParser, FootnoteStore, HtmlWriter, LinkRefStore, Options, RenderPolicy,
};

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
const HTML_INTRINSIC_ELEMENTS: &str = concat!(
    "a abbr acronym address applet area article aside audio b base bdi bdo big blink blockquote ",
    "body br button canvas caption center cite code col colgroup data datalist dd del details dfn ",
    "dialog dir div dl dt em embed fieldset figcaption figure font footer form frameset h1 h2 h3 ",
    "h4 h5 h6 head header hgroup hr html i iframe img input ins kbd label legend li link main map ",
    "mark marquee menu menuitem meta meter nav nobr noembed noframes noscript object ol optgroup ",
    "option output p param picture plaintext pre progress q rb rp rt rtc ruby s samp script search ",
    "section select slot small source span strike strong style sub summary sup table tbody td ",
    "template textarea tfoot th thead time title tr track tt u ul var video wbr xmp",
);

#[derive(Clone, Copy)]
enum HtmlTextKind {
    Raw,
    Rcdata,
}

#[derive(Clone, Copy)]
enum ComponentTagEffect {
    None,
    PreOpen,
    PreClose(u32),
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
#[derive(Debug)]
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
    let mut html_openings = Vec::new();
    let mut declaration_ends = DeclarationEnds::default();
    let mut declaration_scan_work = DeclarationScanWork::default();
    // A crossed owner is balanced at the first mismatched closer. Remember that
    // its eventual source closer must be consumed if no newer live owner claims it.
    let mut synthetic_closings = HashMap::new();

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
            if let Some(end) = comment_end(bytes, pos) {
                pos = end;
                continue;
            }

            // Bound malformed-comment recovery just like incomplete
            // declarations so a later real element is not swallowed. The
            // ordinary text path below also encodes its braces and closing `>`.
            indent_component_line(out, at_line_start, pre_depth);
            out.push_str("&lt;");
            at_line_start = false;
            pos += 1;
            continue;
        }

        if bytes[pos] == b'<' && matches!(bytes.get(pos + 1), Some(b'!' | b'?')) {
            if let Some(end) = declaration_end(
                bytes,
                pos,
                &mut declaration_ends,
                &mut declaration_scan_work,
            ) {
                pos = end;
                continue;
            }

            // An incomplete or declaration-like near-match is not an HTML node.
            // Escape its opening delimiter and let the ordinary text path encode
            // the remaining JSX-significant bytes. Later real JSX/HTML nodes are
            // still processed instead of being swallowed by malformed recovery.
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
            let tag_effect = if let Some((_, canonical_name)) = text_element {
                write_html_opening_tag(
                    out,
                    &body[pos..pos + tag.end_offset],
                    tag.name,
                    canonical_name,
                );
                ComponentTagEffect::None
            } else {
                write_component_tag(
                    out,
                    &body[pos..pos + tag.end_offset],
                    &tag,
                    &mut html_openings,
                    &mut synthetic_closings,
                )
            };
            at_line_start = false;

            match tag_effect {
                ComponentTagEffect::PreOpen => pre_depth = pre_depth.saturating_add(1),
                ComponentTagEffect::PreClose(count) => {
                    pre_depth = pre_depth.saturating_sub(count);
                }
                ComponentTagEffect::None => {}
            }

            pos += tag.end_offset;
            if let Some((text_kind, canonical_name)) = text_element {
                if let Some((text_end, tag_end)) = raw_text_end(bytes, pos, tag.name) {
                    write_jsx_string_child(
                        out,
                        &body[pos..text_end],
                        matches!(text_kind, HtmlTextKind::Rcdata),
                    );
                    write_synthetic_closing_tag(out, canonical_name);
                    pos = tag_end;
                } else {
                    write_jsx_string_child(
                        out,
                        &body[pos..],
                        matches!(text_kind, HtmlTextKind::Rcdata),
                    );
                    write_synthetic_closing_tag(out, canonical_name);
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
            b'>' => {
                out.push_str("&gt;");
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
    html_openings: &mut Vec<(&'static str, Option<&'a str>)>,
    synthetic_closings: &mut HashMap<(&'static str, bool), usize>,
) -> ComponentTagEffect {
    if tag.is_closing {
        if let Some(canonical_name) = html_element_ignore_ascii_case(tag.name) {
            let component_syntax = !uses_html_intrinsic_syntax(tag.name);
            let top_matches = html_openings
                .last()
                .is_some_and(|(opening_canonical, _)| *opening_canonical == canonical_name);
            let matching_index = if top_matches && is_all_uppercase_html_name(tag.name) {
                Some(html_openings.len() - 1)
            } else {
                html_openings
                    .iter()
                    .rposition(|(opening_canonical, component_name)| {
                        *opening_canonical == canonical_name
                            && component_name.is_some() == component_syntax
                    })
                    .or_else(|| {
                        html_openings.iter().rposition(|(opening_canonical, _)| {
                            *opening_canonical == canonical_name
                        })
                    })
            };
            if let Some(matching_index) = matching_index {
                let mut closed_pre_count = 0;
                while html_openings.len() > matching_index + 1 {
                    let (intervening_name, component_name) = html_openings
                        .pop()
                        .expect("an intervening HTML opening was just counted");
                    write_synthetic_closing_tag(out, component_name.unwrap_or(intervening_name));
                    *synthetic_closings
                        .entry((intervening_name, component_name.is_some()))
                        .or_default() += 1;
                    closed_pre_count +=
                        u32::from(component_name.is_none() && intervening_name == "pre");
                }
                let (_, component_name) = html_openings
                    .pop()
                    .expect("matching HTML opening was just located");
                write_closing_tag(out, source, component_name.unwrap_or(canonical_name));
                closed_pre_count += u32::from(component_name.is_none() && canonical_name == "pre");
                return if closed_pre_count > 0 {
                    ComponentTagEffect::PreClose(closed_pre_count)
                } else {
                    ComponentTagEffect::None
                };
            }
            let exact_key = (canonical_name, component_syntax);
            let fallback_key = (canonical_name, !component_syntax);
            let synthetic_key = synthetic_closings
                .get(&exact_key)
                .is_some_and(|count| *count > 0)
                .then_some(exact_key)
                .or_else(|| {
                    synthetic_closings
                        .get(&fallback_key)
                        .is_some_and(|count| *count > 0)
                        .then_some(fallback_key)
                });
            if let Some(synthetic_key) = synthetic_key {
                let count = synthetic_closings
                    .get_mut(&synthetic_key)
                    .expect("a synthetic closing was just located");
                *count -= 1;
                return ComponentTagEffect::None;
            }
            if html_text_element_ignore_ascii_case(tag.name).is_some() {
                return ComponentTagEffect::None;
            }
            if HTML_VOID_ELEMENTS.contains(&canonical_name) {
                return ComponentTagEffect::None;
            }
            if uses_html_intrinsic_syntax(tag.name) {
                write_closing_tag(out, source, canonical_name);
                return if canonical_name == "pre" {
                    ComponentTagEffect::PreClose(1)
                } else {
                    ComponentTagEffect::None
                };
            }
        }
        out.push_str(source);
        return ComponentTagEffect::None;
    }

    if let Some(canonical_name) = html_void_element(tag.name) {
        write_html_void_opening_tag(out, source, tag.name, canonical_name, tag.is_self_closing);
        return ComponentTagEffect::None;
    }

    if let Some(canonical_name) = html_element(tag.name) {
        write_html_opening_tag(out, source, tag.name, canonical_name);
        if !tag.is_self_closing {
            html_openings.push((canonical_name, None));
        }
        return if canonical_name == "pre" && !tag.is_self_closing {
            ComponentTagEffect::PreOpen
        } else {
            ComponentTagEffect::None
        };
    }

    if !tag.is_self_closing
        && let Some(canonical_name) = html_element_ignore_ascii_case(tag.name)
    {
        html_openings.push((canonical_name, Some(tag.name)));
    }
    out.push_str(source);
    ComponentTagEffect::None
}

fn write_html_void_opening_tag(
    out: &mut String,
    source: &str,
    original_name: &str,
    canonical_name: &str,
    is_self_closing: bool,
) {
    debug_assert!(source.starts_with('<'));
    debug_assert!(source.len() > original_name.len());
    out.push('<');
    out.push_str(canonical_name);

    let after_name = &source[original_name.len() + 1..];
    if is_self_closing {
        out.push_str(after_name);
        return;
    }

    let before_close = &after_name[..after_name.len() - 1];
    out.push_str(before_close);
    if before_close.ends_with(char::is_whitespace) {
        out.push('/');
    } else {
        out.push_str(" /");
    }
    out.push('>');
}

fn html_void_element(name: &str) -> Option<&'static str> {
    if !uses_html_intrinsic_syntax(name) {
        return None;
    }

    HTML_VOID_ELEMENTS
        .iter()
        .find(|element| element.eq_ignore_ascii_case(name))
        .copied()
}

fn html_element(name: &str) -> Option<&'static str> {
    uses_html_intrinsic_syntax(name)
        .then(|| html_element_ignore_ascii_case(name))
        .flatten()
}

fn html_element_ignore_ascii_case(name: &str) -> Option<&'static str> {
    HTML_INTRINSIC_ELEMENTS
        .split_ascii_whitespace()
        .find(|element| element.eq_ignore_ascii_case(name))
}

fn html_text_element(name: &str) -> Option<(HtmlTextKind, &'static str)> {
    if !uses_html_intrinsic_syntax(name) {
        return None;
    }

    html_text_element_ignore_ascii_case(name)
}

fn html_text_element_ignore_ascii_case(name: &str) -> Option<(HtmlTextKind, &'static str)> {
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

fn uses_html_intrinsic_syntax(name: &str) -> bool {
    // Lowercase-leading and all-uppercase HTML names are emitted as JSX
    // intrinsics. Preserve PascalCase/mixed-leading-uppercase names as MDX
    // components.
    name.as_bytes().first().is_some_and(u8::is_ascii_lowercase) || is_all_uppercase_html_name(name)
}

fn is_all_uppercase_html_name(name: &str) -> bool {
    name.bytes()
        .all(|byte| !byte.is_ascii_alphabetic() || byte.is_ascii_uppercase())
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

fn write_closing_tag(out: &mut String, source: &str, opening_name: &str) {
    debug_assert!(source.starts_with("</"));
    debug_assert!(source.len() >= opening_name.len() + 2);
    out.push_str("</");
    out.push_str(opening_name);
    out.push_str(&source[opening_name.len() + 2..]);
}

fn write_synthetic_closing_tag(out: &mut String, opening_name: &str) {
    out.push_str("</");
    out.push_str(opening_name);
    out.push('>');
}

fn write_html_opening_tag(
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
        if let Some(end) = html_text_closing_tag_end(bytes, pos, tag_name) {
            return Some((pos, end));
        }
        pos += 1;
    }

    None
}

/// Locate an HTML raw-text/RCDATA end tag, including parse-error forms that
/// HTML still closes but JSX rejects, such as `</textarea/>` or end-tag attrs.
fn html_text_closing_tag_end(bytes: &[u8], start: usize, tag_name: &str) -> Option<usize> {
    let name_start = start.checked_add(2)?;
    let name_end = name_start.checked_add(tag_name.len())?;
    if !bytes.get(start..name_start)?.eq(b"</")
        || !bytes
            .get(name_start..name_end)?
            .eq_ignore_ascii_case(tag_name.as_bytes())
    {
        return None;
    }

    match bytes.get(name_end) {
        Some(b'>') => return Some(name_end + 1),
        Some(byte) if byte.is_ascii_whitespace() || *byte == b'/' => {}
        _ => return None,
    }

    let mut pos = name_end + 1;
    let mut quote = None;
    while pos < bytes.len() {
        if let Some(delimiter) = quote {
            if bytes[pos] == delimiter {
                quote = None;
            }
        } else {
            match bytes[pos] {
                b'\'' | b'"' => quote = Some(bytes[pos]),
                b'>' => return Some(pos + 1),
                b'<' => return None,
                _ => {}
            }
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

#[derive(Clone, Copy)]
struct GenericDeclarationEnd {
    start: usize,
    outcome: GenericDeclarationScan,
}

/// Cached generic declaration ends for a single component body.
///
/// Entries are built from right to left. A complete declaration nested in an
/// internal subset can therefore be skipped as a balanced unit. An inner scan
/// which reaches EOF proves that an enclosing declaration with additional
/// bracket depth cannot close either. Ordinary-tag boundaries are reusable at
/// every subset depth; quote and declaration boundaries remain context-local
/// because an enclosing subset can assign different meaning to the same byte.
struct GenericDeclarationEnds {
    entries: Vec<GenericDeclarationEnd>,
    next_entry: usize,
    #[cfg(test)]
    scanned_bytes: usize,
}

#[derive(Default)]
struct DeclarationEnds {
    generic: Option<GenericDeclarationEnds>,
    processing_instructions: Option<DeclarationTerminatorEnds>,
    cdata: Option<DeclarationTerminatorEnds>,
}

struct DeclarationTerminatorEnds {
    ends: Vec<usize>,
    next_end: usize,
}

impl DeclarationTerminatorEnds {
    fn new(bytes: &[u8], terminator: &[u8], work: &mut DeclarationScanWork) -> Self {
        let mut ends = Vec::new();
        let mut pos = 0;
        while pos + terminator.len() <= bytes.len() {
            work.visit();
            if bytes[pos..].starts_with(terminator) {
                ends.push(pos + terminator.len());
            }
            pos += 1;
        }
        Self { ends, next_end: 0 }
    }

    fn end_after(&mut self, content_start: usize) -> Option<usize> {
        while self
            .ends
            .get(self.next_end)
            .is_some_and(|end| *end < content_start)
        {
            self.next_end += 1;
        }
        self.ends.get(self.next_end).copied()
    }
}

#[derive(Default)]
struct DeclarationScanWork {
    #[cfg(test)]
    scanned_bytes: usize,
}

impl DeclarationScanWork {
    #[inline]
    fn visit(&mut self) {
        #[cfg(test)]
        {
            self.scanned_bytes += 1;
        }
    }
}

impl GenericDeclarationEnds {
    fn new(bytes: &[u8], first_start: usize) -> Self {
        let mut entries = Vec::new();
        for start in first_start..bytes.len() {
            if is_generic_declaration_start(bytes, start) {
                entries.push(GenericDeclarationEnd {
                    start,
                    outcome: GenericDeclarationScan::BoundaryFailure { reusable: false },
                });
            }
        }

        let mut work = DeclarationScanWork::default();
        for entry_index in (0..entries.len()).rev() {
            let outcome = generic_declaration_scan(
                bytes,
                entries[entry_index].start,
                Some((&entries, entry_index + 1)),
                &mut work,
            );
            entries[entry_index].outcome = outcome;
        }

        Self {
            entries,
            next_entry: 0,
            #[cfg(test)]
            scanned_bytes: bytes.len().saturating_sub(first_start) + work.scanned_bytes,
        }
    }

    fn end_at(&mut self, start: usize) -> Option<usize> {
        while self
            .entries
            .get(self.next_entry)
            .is_some_and(|entry| entry.start < start)
        {
            self.next_entry += 1;
        }

        let outcome = self
            .entries
            .get(self.next_entry)
            .filter(|entry| entry.start == start)
            .map_or(
                GenericDeclarationScan::BoundaryFailure { reusable: false },
                |entry| entry.outcome,
            );
        match outcome {
            GenericDeclarationScan::Complete { end, .. } => Some(end),
            GenericDeclarationScan::BoundaryFailure { .. } | GenericDeclarationScan::EofFailure => {
                None
            }
        }
    }

    #[cfg(test)]
    fn scanned_bytes(&self) -> usize {
        self.scanned_bytes
    }

    #[cfg(test)]
    fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

fn declaration_end(
    bytes: &[u8],
    start: usize,
    ends: &mut DeclarationEnds,
    work: &mut DeclarationScanWork,
) -> Option<usize> {
    if bytes[start..].starts_with(b"<![CDATA[") {
        return ends
            .cdata
            .get_or_insert_with(|| DeclarationTerminatorEnds::new(bytes, b"]]>", work))
            .end_after(start + 9);
    }

    if bytes[start..].starts_with(b"<?") {
        return ends
            .processing_instructions
            .get_or_insert_with(|| DeclarationTerminatorEnds::new(bytes, b"?>", work))
            .end_after(start + 2);
    }

    if !is_generic_declaration_start(bytes, start) {
        return None;
    }

    ends.generic
        .get_or_insert_with(|| GenericDeclarationEnds::new(bytes, start))
        .end_at(start)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GenericDeclarationScan {
    Complete {
        end: usize,
        /// Whether the declaration's internal subset brackets are balanced
        /// without borrowing a closing bracket from an enclosing subset.
        independently_balanced: bool,
    },
    BoundaryFailure {
        /// The same ordinary-tag boundary also terminates every enclosing
        /// declaration, regardless of its current subset depth.
        reusable: bool,
    },
    EofFailure,
}

fn is_generic_declaration_start(bytes: &[u8], start: usize) -> bool {
    bytes.get(start) == Some(&b'<')
        && bytes.get(start + 1) == Some(&b'!')
        && bytes.get(start + 2).is_some_and(u8::is_ascii_alphabetic)
}

fn generic_declaration_scan(
    bytes: &[u8],
    start: usize,
    cached_entries: Option<(&[GenericDeclarationEnd], usize)>,
    work: &mut DeclarationScanWork,
) -> GenericDeclarationScan {
    let mut pos = start + 2;
    let mut bracket_depth = 0_u32;
    let mut independently_balanced = true;
    let mut quote = None;
    let mut next_cached_entry = cached_entries.map_or(0, |(_, entry_index)| entry_index);

    while pos < bytes.len() {
        work.visit();
        if let Some(delimiter) = quote {
            if bytes[pos] == delimiter {
                quote = None;
            } else if bytes[pos] == b'<' && bracket_depth == 0 {
                // A new top-level tag cannot belong to an unclosed declaration
                // literal. Bound recovery here so a later quote and `>` cannot
                // make the following node look like the declaration's suffix.
                return GenericDeclarationScan::BoundaryFailure { reusable: false };
            }
            pos += 1;
            continue;
        }

        if bracket_depth > 0 && bytes[pos..].starts_with(b"<!--") {
            let Some(end) = comment_end(bytes, pos) else {
                return GenericDeclarationScan::EofFailure;
            };
            pos = end;
            continue;
        }
        if bracket_depth > 0 && bytes[pos..].starts_with(b"<?") {
            let Some(end) = bytes[pos + 2..]
                .windows(2)
                .position(|window| window == b"?>")
                .map(|offset| pos + 2 + offset + 2)
            else {
                return GenericDeclarationScan::EofFailure;
            };
            pos = end;
            continue;
        }
        if bracket_depth > 0 && conditional_section_content_start(bytes, pos).is_some() {
            let Some(end) = conditional_section_end(bytes, pos) else {
                return GenericDeclarationScan::EofFailure;
            };
            pos = end;
            continue;
        }

        if bracket_depth > 0
            && is_generic_declaration_start(bytes, pos)
            && let Some((entries, _)) = cached_entries
        {
            while entries
                .get(next_cached_entry)
                .is_some_and(|entry| entry.start < pos)
            {
                next_cached_entry += 1;
            }
            if let Some(entry) = entries
                .get(next_cached_entry)
                .filter(|entry| entry.start == pos)
            {
                match entry.outcome {
                    GenericDeclarationScan::EofFailure => {
                        return GenericDeclarationScan::EofFailure;
                    }
                    GenericDeclarationScan::BoundaryFailure { reusable: true } => {
                        return GenericDeclarationScan::BoundaryFailure { reusable: true };
                    }
                    GenericDeclarationScan::BoundaryFailure { reusable: false }
                    | GenericDeclarationScan::Complete {
                        independently_balanced: false,
                        ..
                    } => {}
                    GenericDeclarationScan::Complete {
                        end,
                        independently_balanced: true,
                    } => {
                        pos = end;
                        continue;
                    }
                }
            }
        }

        match bytes[pos] {
            b'\'' | b'"' => quote = Some(bytes[pos]),
            b'[' => {
                let Some(next_depth) = bracket_depth.checked_add(1) else {
                    return GenericDeclarationScan::BoundaryFailure { reusable: false };
                };
                bracket_depth = next_depth;
            }
            b']' if bracket_depth > 0 => bracket_depth -= 1,
            b']' => independently_balanced = false,
            b'<' => {
                // Internal subsets contain nested declarations and processing
                // instructions. An ordinary tag is instead the recovery
                // boundary for an incomplete outer declaration.
                if !matches!(bytes.get(pos + 1), Some(b'!' | b'?')) {
                    return GenericDeclarationScan::BoundaryFailure { reusable: true };
                }
                if bracket_depth == 0 {
                    return GenericDeclarationScan::BoundaryFailure { reusable: false };
                }
            }
            b'>' if bracket_depth == 0 => {
                return GenericDeclarationScan::Complete {
                    end: pos + 1,
                    independently_balanced,
                };
            }
            _ => {}
        }
        pos += 1;
    }

    GenericDeclarationScan::EofFailure
}

#[cfg(test)]
fn generic_declaration_end_uncached(bytes: &[u8], start: usize) -> Option<usize> {
    match generic_declaration_scan(bytes, start, None, &mut DeclarationScanWork::default()) {
        GenericDeclarationScan::Complete { end, .. } => Some(end),
        GenericDeclarationScan::BoundaryFailure { .. } | GenericDeclarationScan::EofFailure => None,
    }
}

fn conditional_section_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut pos = conditional_section_content_start(bytes, start)?;
    let mut depth = 1_u32;

    while pos < bytes.len() {
        if bytes[pos..].starts_with(b"<!--") {
            pos = comment_end(bytes, pos)?;
            continue;
        }
        if bytes[pos..].starts_with(b"<?") {
            pos = bytes[pos + 2..]
                .windows(2)
                .position(|window| window == b"?>")
                .map(|offset| pos + 2 + offset + 2)?;
            continue;
        }
        if bytes[pos..].starts_with(b"<![CDATA[") {
            pos = bytes[pos + 9..]
                .windows(3)
                .position(|window| window == b"]]>")
                .map(|offset| pos + 9 + offset + 3)?;
            continue;
        }
        if let Some(content_start) = conditional_section_content_start(bytes, pos) {
            depth = depth.checked_add(1)?;
            pos = content_start;
            continue;
        }
        if bytes[pos..].starts_with(b"]]>") {
            depth -= 1;
            pos += 3;
            if depth == 0 {
                return Some(pos);
            }
            continue;
        }

        match bytes[pos] {
            b'\'' | b'"' => pos = declaration_quote_end(bytes, pos)?,
            _ => pos += 1,
        }
    }

    None
}

fn conditional_section_content_start(bytes: &[u8], start: usize) -> Option<usize> {
    if !bytes[start..].starts_with(b"<![") {
        return None;
    }

    let mut pos = skip_ascii_whitespace(bytes, start + 3);
    if bytes[pos..].starts_with(b"INCLUDE") {
        pos += b"INCLUDE".len();
    } else if bytes[pos..].starts_with(b"IGNORE") {
        pos += b"IGNORE".len();
    } else {
        return None;
    }
    pos = skip_ascii_whitespace(bytes, pos);

    (bytes.get(pos) == Some(&b'[')).then_some(pos + 1)
}

fn declaration_quote_end(bytes: &[u8], start: usize) -> Option<usize> {
    let quote = bytes[start];
    bytes[start + 1..]
        .iter()
        .position(|byte| *byte == quote)
        .map(|offset| start + 1 + offset + 1)
}

fn skip_ascii_whitespace(bytes: &[u8], mut pos: usize) -> usize {
    while bytes.get(pos).is_some_and(u8::is_ascii_whitespace) {
        pos += 1;
    }
    pos
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
    let mut parsed_markdown = Vec::new();
    let mut link_refs = LinkRefStore::new();
    let mut footnote_store = FootnoteStore::new();
    let mut esm: Vec<&'a str> = Vec::new();
    let mut front_matter: Option<&'a str> = None;

    // Parse each Markdown slice once. Stores are assembled in source order
    // before rendering so forward references resolve without changing the
    // first-definition-wins rule.
    for segment in &segments {
        match segment {
            Segment::Esm(s) => esm.push(s),
            Segment::Markdown(source) => {
                if front_matter.is_none() && options.front_matter {
                    front_matter = crate::extract_front_matter(source).map(|(value, _)| value);
                }

                let markdown = crate::markdown_without_front_matter(source, options);
                let source_offset = (markdown.as_ptr() as usize)
                    .checked_sub(input.as_ptr() as usize)
                    .expect("MDX Markdown segment must borrow from its input");
                // These buffers live until their segment is rendered. Small
                // JSX-separated paragraphs usually need only a few events.
                let mut events = Vec::with_capacity((markdown.len() / 16).max(8));
                let mut parser =
                    BlockParser::new_with_options(markdown.as_bytes(), options.clone());
                parser.parse(&mut events);
                crate::fixup_list_tight(&mut events);
                if options.allow_link_refs {
                    link_refs.merge_first_wins(parser.take_link_refs());
                }
                if options.footnotes {
                    let mut notes = parser.take_footnote_store();
                    notes.shift_ranges(crate::range::offset_to_u32(source_offset));
                    footnote_store.merge_first_wins(notes);
                }
                parsed_markdown.push(ParsedMarkdown {
                    source: markdown,
                    events,
                });
            }
            Segment::JsxBlockOpen(s)
            | Segment::JsxBlockClose(s)
            | Segment::JsxBlockSelfClose(s)
            | Segment::Expression(s) => {
                let _ = s;
            }
        }
    }

    let mut body_writer = HtmlWriter::with_capacity_for(input.len());
    let mut inline_parser = crate::InlineParser::new();
    let mut inline_events = Vec::with_capacity(64);
    let mut render_state = crate::RenderState::new(options);
    let mut footnote_numbers = crate::FootnoteNumbers::new(0);
    let mut markdown_started = false;
    let mut parsed_markdown = parsed_markdown.into_iter();
    inline_parser.begin_document();
    render_state.reset(options);
    footnote_numbers.reset_document(footnote_store.len());

    for segment in &segments {
        match segment {
            Segment::Esm(_) => {}
            Segment::Markdown(_) => {
                let parsed = parsed_markdown
                    .next()
                    .expect("each Markdown segment has one parsed event stream");
                crate::render_events_with_state(
                    parsed.source.as_bytes(),
                    &parsed.events,
                    &mut body_writer,
                    options,
                    &link_refs,
                    options.footnotes.then_some(&footnote_store),
                    &mut inline_parser,
                    &mut inline_events,
                    &mut render_state,
                    &mut footnote_numbers,
                );
                markdown_started = true;
            }
            Segment::JsxBlockOpen(s)
            | Segment::JsxBlockClose(s)
            | Segment::JsxBlockSelfClose(s)
            | Segment::Expression(s) => {
                body_writer.write_string(s.trim());
                body_writer.write_byte(b'\n');
            }
        }
    }

    if markdown_started && (options.footnotes || options.inline_footnotes) {
        crate::render_footnotes_with_state(
            input.as_bytes(),
            &mut body_writer,
            options,
            &link_refs,
            &footnote_store,
            &mut inline_parser,
            &mut inline_events,
            &mut render_state,
            &mut footnote_numbers,
        );
    }

    let body = body_writer
        .into_string()
        .expect("rendering from a UTF-8 MDX string must produce UTF-8 HTML");

    Ok(MdxOutput {
        body,
        esm,
        front_matter,
    })
}

struct ParsedMarkdown<'a> {
    source: &'a str,
    events: Vec<BlockEvent>,
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
    fn to_component_canonicalizes_all_uppercase_html_void_elements() {
        let body = "<AREA data-kind=\"upper\"></area><BASE ></BASE><BR></br><COL span={2}></COL><EMBED type=\"example/test\"></EMBED><HR></hr><IMG src=\"image.png\"></img><INPUT disabled></INPUT><LINK rel=\"help\"></LINK><META name=\"kind\"></META><PARAM name=\"example\"></PARAM><SOURCE src=\"media.mp4\"></SOURCE><TRACK kind=\"captions\"></TRACK><WBR/></wbr>";
        let out = MdxOutput {
            body: body.to_owned(),
            esm: vec![],
            front_matter: None,
        };
        let component = out.to_component("Page").unwrap();

        assert_eq!(out.body, body);
        for expected in [
            "<area data-kind=\"upper\" />",
            "<base />",
            "<br />",
            "<col span={2} />",
            "<embed type=\"example/test\" />",
            "<hr />",
            "<img src=\"image.png\" />",
            "<input disabled />",
            "<link rel=\"help\" />",
            "<meta name=\"kind\" />",
            "<param name=\"example\" />",
            "<source src=\"media.mp4\" />",
            "<track kind=\"captions\" />",
            "<wbr/>",
        ] {
            assert!(
                component.contains(expected),
                "missing {expected:?}: {component}"
            );
        }
        for omitted in HTML_VOID_ELEMENTS.map(str::to_ascii_uppercase) {
            assert!(
                !component.contains(&format!("<{omitted}")),
                "retained uppercase opener {omitted:?}: {component}"
            );
            assert!(
                !component.contains(&format!("</{omitted}")),
                "retained void closer {omitted:?}: {component}"
            );
        }
    }

    #[test]
    fn to_component_pairs_void_named_components_across_closing_case() {
        let mut body = String::new();
        for name in HTML_VOID_ELEMENTS {
            let pascal = format!("{}{}", name[..1].to_ascii_uppercase(), &name[1..]);
            let upper = name.to_ascii_uppercase();
            let _ = write!(
                body,
                "<{pascal} data-kind=\"component\" ></{upper}   ><{upper} data-kind=\"intrinsic\"></{pascal}>"
            );
        }
        let out = MdxOutput {
            body: body.clone(),
            esm: vec![],
            front_matter: None,
        };
        let component = out.to_component("Page").unwrap();

        assert_eq!(out.body, body);
        for name in HTML_VOID_ELEMENTS {
            let pascal = format!("{}{}", name[..1].to_ascii_uppercase(), &name[1..]);
            assert!(
                component.contains(&format!(
                    "<{pascal} data-kind=\"component\" ></{pascal}   >"
                )),
                "component pair for {name:?} was not retained: {component}"
            );
            assert!(
                component.contains(&format!("<{name} data-kind=\"intrinsic\" />")),
                "intrinsic pair for {name:?} was not normalized: {component}"
            );
        }
    }

    #[test]
    fn to_component_leaves_void_near_matches_and_custom_elements_unchanged() {
        let body = "<Brave data-kind=\"component\"></Brave><BR-X></BR-X><br-x></br-x><brr></BRR><ImgExtra></ImgExtra>";
        let out = MdxOutput {
            body: body.to_owned(),
            esm: vec![],
            front_matter: None,
        };
        let component = out.to_component("Page").unwrap();

        assert!(component.contains(body), "{component}");
    }

    #[test]
    fn to_component_distinguishes_html_void_names_from_pascal_case_components() {
        let body = "<bR data-kind=\"mixed\"></BR><iMg src=\"mixed.png\"></IMG><Br data-kind=\"component\"></Br><Img src=\"component.png\"></Img><BRAVO></BRAVO><BR-X></BR-X><custom-img></custom-img>";
        let out = MdxOutput {
            body: body.to_owned(),
            esm: vec![],
            front_matter: None,
        };
        let component = out.to_component("Page").unwrap();

        assert!(
            component.contains("<br data-kind=\"mixed\" />"),
            "{component}"
        );
        assert!(
            component.contains("<img src=\"mixed.png\" />"),
            "{component}"
        );
        assert!(
            component.contains("<Br data-kind=\"component\"></Br>"),
            "{component}"
        );
        assert!(
            component.contains("<Img src=\"component.png\"></Img>"),
            "{component}"
        );
        assert!(component.contains("<BRAVO></BRAVO>"), "{component}");
        assert!(component.contains("<BR-X></BR-X>"), "{component}");
        assert!(
            component.contains("<custom-img></custom-img>"),
            "{component}"
        );
    }

    #[test]
    fn to_component_canonicalizes_uppercase_void_tags_from_rendered_markdown() {
        let out = render("a<BR>b\n\n<IMG src=\"image.png\">\n\nAfter.");
        assert!(out.body.contains("<p>a<BR>b</p>"), "{}", out.body);
        assert!(out.body.contains("<IMG src=\"image.png\">"), "{}", out.body);

        let component = out.to_component("Page").unwrap();
        assert!(component.contains("<p>a<br />b</p>"), "{component}");
        assert!(
            component.contains("<img src=\"image.png\" />"),
            "{component}"
        );
        assert!(component.contains("<p>After.</p>"), "{component}");
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
        assert!(component.contains("&lt;!DOCTYPE"), "{component}");
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
    fn to_component_does_not_extend_incomplete_declarations_into_following_tags() {
        for line_break in ["\n", "\r\n"] {
            let body = format!(
                "<!DOCTYPE html{line_break}<p>after doctype</p>{line_break}<?pi incomplete{line_break}<p>after pi</p>{line_break}<![CDATA[incomplete{line_break}<p>after cdata</p>"
            );
            let out = MdxOutput {
                body: body.clone(),
                esm: vec![],
                front_matter: None,
            };
            let component = out.to_component("Page").unwrap();

            assert_eq!(out.body, body);
            for declaration in ["&lt;!DOCTYPE html", "&lt;?pi incomplete", "&lt;![CDATA["] {
                assert!(component.contains(declaration), "{component}");
            }
            for paragraph in [
                "<p>after doctype</p>",
                "<p>after pi</p>",
                "<p>after cdata</p>",
            ] {
                assert!(component.contains(paragraph), "{component}");
            }
        }
    }

    #[test]
    fn to_component_omits_closed_multiline_declarations_at_their_own_boundary() {
        let body = "<!DOCTYPE\nhtml>\n<p>after doctype</p>\n<?pi\nvalue?>\n<p>after pi</p>\n<![CDATA[ignored\n<p>inside cdata</p>]]>\n<p>after cdata</p>";
        let out = MdxOutput {
            body: body.to_owned(),
            esm: vec![],
            front_matter: None,
        };
        let component = out.to_component("Page").unwrap();

        for omitted in ["DOCTYPE", "<?pi", "inside cdata", "CDATA"] {
            assert!(!component.contains(omitted), "{component}");
        }
        for paragraph in [
            "<p>after doctype</p>",
            "<p>after pi</p>",
            "<p>after cdata</p>",
        ] {
            assert!(component.contains(paragraph), "{component}");
        }
    }

    #[test]
    fn to_component_omits_complete_doctype_internal_subsets() {
        for line_break in ["\n", "\r\n"] {
            let body = format!(
                "<!DOCTYPE html [{line_break}<!ENTITY example \"value>still\">{line_break}<!ENTITY markup '<p data-kind=\"literal\">text</p>'>{line_break}<!-- subset ] > comment -->{line_break}<?subset ] > data?>{line_break}]>{line_break}<p>after subset</p>{line_break}<!DOCTYPE root [<![INCLUDE[<!ENTITY nested \"[value]>\">]]>]><p>after nested subset</p>{line_break}<!DOCTYPE html SYSTEM \"value>still\"><p>after system literal</p>"
            );
            let out = MdxOutput {
                body: body.clone(),
                esm: vec![],
                front_matter: None,
            };
            let component = out.to_component("Page").unwrap();

            assert_eq!(out.body, body);
            for omitted in [
                "DOCTYPE",
                "ENTITY",
                "data-kind=\"literal\"",
                "subset ] > comment",
                "subset ] > data",
                "INCLUDE",
                "value>still",
            ] {
                assert!(!component.contains(omitted), "{component}");
            }
            for paragraph in [
                "<p>after subset</p>",
                "<p>after nested subset</p>",
                "<p>after system literal</p>",
            ] {
                assert!(component.contains(paragraph), "{component}");
            }
        }
    }

    #[test]
    fn to_component_omits_conditional_sections_containing_markup() {
        for line_break in ["\n", "\r\n"] {
            let body = format!(
                "<!DOCTYPE html [{line_break}<![ INCLUDE [{line_break}<p data-end=']]>'>literal</p>{line_break}<![IGNORE[<section>nested</section><!-- comment ]]> --><?inside ]]>?><![CDATA[<em>cdata</em>]]>]]>{line_break}]]>{line_break}]>{line_break}<p>after section</p>"
            );
            let out = MdxOutput {
                body: body.clone(),
                esm: vec![],
                front_matter: None,
            };
            let component = out.to_component("Page").unwrap();

            assert_eq!(out.body, body);
            for omitted in [
                "DOCTYPE", "data-end", "literal", "nested", "comment", "<?inside", "cdata",
            ] {
                assert!(!component.contains(omitted), "{component}");
            }
            assert!(component.contains("<p>after section</p>"), "{component}");
        }
    }

    #[test]
    fn to_component_bounds_incomplete_conditional_sections() {
        for body in [
            "<!DOCTYPE html [<![INCLUDE[<section>inside</section>\n<p>after missing close</p>",
            "<!DOCTYPE html [<![IGNORE['unterminated\n<p>after quote</p>",
            "<!DOCTYPE html [<![INCLUDE[<!-- unterminated\n<p>after comment</p>",
            "<!DOCTYPE html [<![INCLUDE[<?unterminated\n<p>after pi</p>",
            "<!DOCTYPE html [<![INCLUD[near match\n<p>after near match</p>",
        ] {
            let out = MdxOutput {
                body: body.to_owned(),
                esm: vec![],
                front_matter: None,
            };
            let component = out.to_component("Page").unwrap();

            assert!(component.contains("&lt;!DOCTYPE"), "{component}");
            assert!(component.contains("<p>after"), "{component}");
        }
    }

    #[test]
    fn to_component_bounds_incomplete_doctype_internal_subsets() {
        for body in [
            "<!DOCTYPE html [<!ENTITY example \"value\">\n<p>after missing subset close</p>",
            "<!DOCTYPE html [<!ENTITY example \"unterminated\n<p>after unterminated entity</p>",
            "<!DOCTYPE html SYSTEM \"unterminated\n<p data-kind='next'>after system literal</p>",
            "<!DOCTYPE html [\r\n<section>after CRLF subset</section>",
        ] {
            let out = MdxOutput {
                body: body.to_owned(),
                esm: vec![],
                front_matter: None,
            };
            let component = out.to_component("Page").unwrap();

            assert!(component.contains("&lt;!DOCTYPE"), "{component}");
            assert!(
                component.contains("after ") || component.contains("after CRLF"),
                "{component}"
            );
            assert!(
                component.contains("<p") || component.contains("<section>"),
                "{component}"
            );
        }
    }

    #[test]
    fn generic_declaration_cache_matches_the_sequential_scanner() {
        let repeated = "<!DOCTYPE [".repeat(32);
        let bodies = [
            "<!DOCTYPE html>",
            "<!DOCTYPE html [<!ENTITY example \"value\">]><p>after</p>",
            "<!OUTER [<!INNER [<!LEAF value>]>]><p>after</p>",
            "<!DOCTYPE [<!INNER complete>",
            "<!OUTER [<!INNER ]>",
            "<!OUTER [<!INNER ]><p>after borrowed close</p>",
            "<!OUTER [<!INNER \"unterminated <p> literal\">]><p>after</p>",
            "<!DOCTYPE html [<!-- comment --><?inside value?><![INCLUDE[<p>literal</p>]]>]>",
            "<!DOCTYPE html [<!ENTITY example \"unterminated\n<p>after</p>",
            "<!near-match><p>after</p>",
            repeated.as_str(),
        ];

        for body in bodies {
            let bytes = body.as_bytes();
            let starts: Vec<_> = (0..bytes.len())
                .filter(|&start| is_generic_declaration_start(bytes, start))
                .collect();
            let mut cached = GenericDeclarationEnds::new(bytes, starts[0]);

            for start in starts {
                assert_eq!(
                    cached.end_at(start),
                    generic_declaration_end_uncached(bytes, start),
                    "cache changed the declaration boundary at {start} in {body:?}"
                );
            }
        }
    }

    #[test]
    fn repeated_unterminated_subsets_have_a_deterministic_linear_work_bound() {
        const REPEATS: usize = 8_192;
        let unit = "<!DOCTYPE [";
        let body = unit.repeat(REPEATS);
        let mut cached = GenericDeclarationEnds::new(body.as_bytes(), 0);

        assert_eq!(cached.entry_count(), REPEATS);
        for start in (0..body.len()).step_by(unit.len()) {
            assert_eq!(cached.end_at(start), None);
        }
        assert!(
            cached.scanned_bytes() <= body.len() * 3,
            "{} scan visits for {} input bytes",
            cached.scanned_bytes(),
            body.len()
        );
    }

    #[test]
    fn repeated_unterminated_subsets_before_a_tag_have_a_linear_work_bound() {
        const REPEATS: usize = 4_096;
        let unit = "<!DOCTYPE [";
        let body = format!("{}<p>after</p>", unit.repeat(REPEATS));
        let mut cached = GenericDeclarationEnds::new(body.as_bytes(), 0);

        assert_eq!(cached.entry_count(), REPEATS);
        for start in (0..unit.len() * REPEATS).step_by(unit.len()) {
            assert_eq!(cached.end_at(start), None);
        }
        assert!(
            cached.scanned_bytes() <= body.len() * 4,
            "{} scan visits for {} input bytes",
            cached.scanned_bytes(),
            body.len()
        );
    }

    #[test]
    fn repeated_unterminated_pi_and_cdata_have_a_linear_work_bound() {
        const REPEATS: usize = 4_096;

        for unit in ["<?", "<![CDATA["] {
            let body = format!("{}<p>after</p>", unit.repeat(REPEATS));
            let mut declaration_ends = DeclarationEnds::default();
            let mut work = DeclarationScanWork::default();

            for start in (0..unit.len() * REPEATS).step_by(unit.len()) {
                assert_eq!(
                    declaration_end(body.as_bytes(), start, &mut declaration_ends, &mut work),
                    None,
                );
            }
            assert!(
                work.scanned_bytes <= body.len() * 3,
                "{} scan visits for {} input bytes ({unit:?})",
                work.scanned_bytes,
                body.len()
            );
        }
    }

    #[test]
    fn declaration_terminator_cache_matches_sequential_searches() {
        let body = "<?outer <?inner?> tail <?unterminated <![CDATA[first <![CDATA[second]]> tail <![CDATA[unterminated";
        let bytes = body.as_bytes();
        let mut declaration_ends = DeclarationEnds::default();
        let mut work = DeclarationScanWork::default();

        for start in (0..bytes.len()).filter(|&start| bytes[start..].starts_with(b"<?")) {
            let expected = bytes[start + 2..]
                .windows(2)
                .position(|window| window == b"?>")
                .map(|offset| start + 2 + offset + 2);
            assert_eq!(
                declaration_end(bytes, start, &mut declaration_ends, &mut work),
                expected,
            );
        }

        for start in (0..bytes.len()).filter(|&start| bytes[start..].starts_with(b"<![CDATA[")) {
            let expected = bytes[start + 9..]
                .windows(3)
                .position(|window| window == b"]]>")
                .map(|offset| start + 9 + offset + 3);
            assert_eq!(
                declaration_end(bytes, start, &mut declaration_ends, &mut work),
                expected,
            );
        }
    }

    #[test]
    fn repeated_unterminated_subsets_keep_public_output_and_strict_parity() {
        let input = "<!DOCTYPE [".repeat(1_024);
        let output = render(&input);
        let component = output.to_component("Page").unwrap();

        assert_eq!(component.matches("&lt;!DOCTYPE [").count(), 1_024);
        assert!(!component.contains("<!DOCTYPE"), "{component}");
        assert_eq!(
            crate::mdx::segment_strict(&input).unwrap(),
            crate::mdx::segment_spanned(&input)
        );
    }

    #[test]
    fn repeated_unterminated_declarations_before_a_tag_keep_public_output_and_strict_parity() {
        for unit in ["<!DOCTYPE [", "<?", "<![CDATA["] {
            let input = format!("{}<p>after</p>", unit.repeat(256));
            let output = render(&input);
            let component = output.to_component("Page").unwrap();

            assert_eq!(component.matches("&lt;").count(), 256, "{unit:?}");
            assert!(component.contains("<p>after</p>"), "{unit:?}: {component}");
            assert_eq!(
                crate::mdx::segment_strict(&input).unwrap(),
                crate::mdx::segment_spanned(&input),
                "{unit:?}",
            );
        }
    }

    #[test]
    fn declaration_cache_preserves_nested_recovery_semantics() {
        let incomplete_outer = MdxOutput {
            body: "<!DOCTYPE [<!INNER complete><p>after outer failure</p>".to_owned(),
            esm: vec![],
            front_matter: None,
        }
        .to_component("Page")
        .unwrap();
        assert!(
            incomplete_outer.contains("&lt;!DOCTYPE ["),
            "{incomplete_outer}"
        );
        assert!(!incomplete_outer.contains("INNER"), "{incomplete_outer}");
        assert!(
            incomplete_outer.contains("<p>after outer failure</p>"),
            "{incomplete_outer}"
        );

        let context_dependent_inner = MdxOutput {
            body: "<!OUTER [<!INNER \"unterminated <p> literal\">]><p>after complete outer</p>"
                .to_owned(),
            esm: vec![],
            front_matter: None,
        }
        .to_component("Page")
        .unwrap();
        assert!(
            !context_dependent_inner.contains("OUTER"),
            "{context_dependent_inner}"
        );
        assert!(
            !context_dependent_inner.contains("INNER"),
            "{context_dependent_inner}"
        );
        assert!(
            context_dependent_inner.contains("<p>after complete outer</p>"),
            "{context_dependent_inner}"
        );

        let borrowed_subset_close = MdxOutput {
            body: "<!OUTER [<!INNER ]><p>after borrowed close</p>".to_owned(),
            esm: vec![],
            front_matter: None,
        }
        .to_component("Page")
        .unwrap();
        assert!(
            !borrowed_subset_close.contains("OUTER"),
            "{borrowed_subset_close}"
        );
        assert!(
            !borrowed_subset_close.contains("INNER"),
            "{borrowed_subset_close}"
        );
        assert!(
            borrowed_subset_close.contains("<p>after borrowed close</p>"),
            "{borrowed_subset_close}"
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
    fn to_component_escapes_complete_malformed_declaration_text() {
        for line_break in ["\n", "\r\n"] {
            for malformed in [
                "<?bad>",
                "<?bad data='value>still' {literal}>",
                "<![not-cdata]>",
                "<![not-cdata data=\"value>still\"]>",
                "<!-- no terminator>",
                "<!-- no terminator {literal}>",
                "<!-near-match>",
            ] {
                let markdown = format!("{malformed}{line_break}{line_break}<p>after</p>");
                let component = render(&markdown).to_component("Page").unwrap();

                assert!(component.contains("&lt;"), "{malformed}: {component}");
                assert!(component.contains("&gt;"), "{malformed}: {component}");
                assert!(!component.contains("{literal}"), "{malformed}: {component}");
                if malformed.contains("{literal}") {
                    assert!(component.contains("&#123;literal&#125;"), "{component}");
                }
                assert!(
                    component.contains("<p>after</p>"),
                    "{malformed}: {component}"
                );
            }
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
            "<textarea data-kind=\"example\">{\"first\\r\\n  second {value}&\\r\\n\"}</textarea><p>after textarea</p>"
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
    fn to_component_normalizes_html_text_closers_to_valid_jsx() {
        let cases = [
            ("textarea", "</textarea/>"),
            ("title", "</TiTlE / >"),
            ("script", "</SCRIPT data-kind='ignored'>"),
            ("style", "</stYLE data-kind=\"value>still\">"),
            ("xmp", "</XMP/>"),
            ("iframe", "</IFRAME / >"),
            ("noembed", "</NOEMBED data-kind='ignored'>"),
            ("noframes", "</noFrames data-kind=\"value>still\">"),
        ];

        for (name, closer) in cases {
            let body = format!("<{name}>a {{value}}&amp;{closer}b<p>after</p>");
            let out = MdxOutput {
                body,
                esm: vec![],
                front_matter: None,
            };
            let component = out.to_component("Page").unwrap();

            assert!(
                component.contains(&format!("</{name}>b<p>after</p>")),
                "{component}"
            );
            assert!(!component.contains(closer), "{component}");
        }

        let public_component = render("<textarea>a</textarea/>b")
            .to_component("Page")
            .unwrap();
        assert!(
            public_component.contains("<textarea>{\"a\"}</textarea>b"),
            "{public_component}"
        );
    }

    #[test]
    fn to_component_keeps_html_text_closer_near_matches_as_text() {
        let body = "<textarea>a</textareax>b</textarea><p>after name</p><title>c</title?>d</title><p>after punctuation</p><script>e</script data='unterminated<p>inside raw text</p>";
        let out = MdxOutput {
            body: body.to_owned(),
            esm: vec![],
            front_matter: None,
        };
        let component = out.to_component("Page").unwrap();

        assert!(
            component.contains("<textarea>{\"a</textareax>b\"}</textarea><p>after name</p>"),
            "{component}"
        );
        assert!(
            component.contains("<title>{\"c</title?>d\"}</title><p>after punctuation</p>"),
            "{component}"
        );
        assert!(
            component.contains(
                "<script>{\"e</script data='unterminated<p>inside raw text</p>\"}</script>"
            ),
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
    fn to_component_uses_html_text_end_tag_name_boundaries() {
        let out = MdxOutput {
            body: "<textarea>before</TEXTAREAX>middle</textarea extra>still</TeXtArEa><p>after</p>\n<script>before</SCRIPTS>middle</ScRiPt><p>after script</p>"
                .to_owned(),
            esm: vec![],
            front_matter: None,
        };
        let component = out.to_component("Page").unwrap();

        assert!(
            component
                .contains("<textarea>{\"before</TEXTAREAX>middle\"}</textarea>still<p>after</p>"),
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
    fn to_component_canonicalizes_rendered_case_variant_pre_and_code() {
        let out = render("<PRE><CODE>first\n  second {value}\n</CODE></PRE>\n");
        let component = out.to_component("Page").unwrap();

        assert!(
            component.contains("      <pre><code>{\"first\\n  second {value}\\n\"}</code></pre>"),
            "{component}"
        );
    }

    #[test]
    fn to_component_preserves_case_variant_pre_text_and_nested_html_kinds() {
        for body in [
            "<PRE data-kind=\"upper\"><CODE>first\r\n  second {value}&amp;\r\n</cOdE></pRe><p>after</p>",
            "<pRe><cOdE>first\n  second {value}\n</CODE><TEXTAREA>third\n  fourth &amp; {value}\n</textarea><SCRIPT>if (left < right) { run(); }</script></PRE>",
        ] {
            let out = MdxOutput {
                body: body.to_owned(),
                esm: vec![],
                front_matter: None,
            };
            let component = out.to_component("Page").unwrap();

            assert!(!component.contains("<PRE"), "{component}");
            assert!(!component.contains("<CODE"), "{component}");
            assert!(component.contains("<pre"), "{component}");
            assert!(component.contains("<code>"), "{component}");
            assert!(component.contains("second {value}"), "{component}");
            assert!(!component.contains("      second"), "{component}");
            assert!(component.contains("</code>"), "{component}");
            assert!(component.contains("</pre>"), "{component}");
        }
    }

    #[test]
    fn to_component_keeps_pascal_case_pre_and_code_components() {
        let out = render("<Pre>\n\n<Code>\n\n{value}\n\n</Code>\n\n</Pre>\n");
        let component = out.to_component("Page").unwrap();

        assert!(component.contains("      <Pre>"), "{component}");
        assert!(component.contains("      <Code>"), "{component}");
        assert!(component.contains("      {value}"), "{component}");
        assert!(component.contains("      </Code>"), "{component}");
        assert!(component.contains("      </Pre>"), "{component}");
    }

    #[test]
    fn to_component_keeps_nested_intrinsic_and_component_owners_distinct() {
        let body = "<Div><div>inner</div></Div><div><Div>component</DIV></DIV><Div><div>component first</Div><div><Div>intrinsic first</div><Pre><pre>first\n  second {value}\n</Pre>";
        let out = MdxOutput {
            body: body.to_owned(),
            esm: vec![],
            front_matter: None,
        };
        let component = out.to_component("Page").unwrap();

        assert!(
            component.contains("<Div><div>inner</div></Div>"),
            "{component}"
        );
        assert!(
            component.contains("<div><Div>component</Div></div>"),
            "{component}"
        );
        assert!(
            component.contains("<Div><div>component first</div></Div>"),
            "{component}"
        );
        assert!(
            component.contains("<div><Div>intrinsic first</Div></div>"),
            "{component}"
        );
        assert!(
            component.contains("<Pre><pre>{\"first\\n  second {value}\\n\"}</pre></Pre>"),
            "{component}"
        );
    }

    #[test]
    fn to_component_drops_source_closers_for_synthetically_closed_owners() {
        let cases = [
            ("<Div><div>a</Div></div>", "<Div><div>a</div></Div>"),
            ("<div><Div>a</div></Div>", "<div><Div>a</Div></div>"),
            (
                "<Section><div><span>a</Section></span></div>",
                "<Section><div><span>a</span></div></Section>",
            ),
            (
                "<Div><div>a</Div><div>b</div></div>",
                "<Div><div>a</div></Div><div>b</div>",
            ),
            ("<Div><DIV>a</Div></DIV>", "<Div><div>a</div></Div>"),
        ];

        for (body, expected) in cases {
            let out = MdxOutput {
                body: body.to_owned(),
                esm: vec![],
                front_matter: None,
            };
            let component = out.to_component("Page").unwrap();

            assert!(component.contains(expected), "{body}: {component}");
            assert_eq!(
                component.matches("</div>").count(),
                expected.matches("</div>").count()
            );
            assert_eq!(
                component.matches("</Div>").count(),
                expected.matches("</Div>").count()
            );
            assert_eq!(
                component.matches("</span>").count(),
                expected.matches("</span>").count()
            );
        }
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
