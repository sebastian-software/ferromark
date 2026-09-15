//! HTML normalizer used by the spec conformance tests.
//!
//! Expected spec HTML and actual renderer HTML both pass through
//! [`normalize_html`] before comparison, so differences that do not change
//! how a document renders are ignored:
//!
//! - insignificant whitespace between block-level tags (kept verbatim
//!   inside `<pre>`, `<code>`, and raw-text elements),
//! - attribute order, quoting style, and boolean attribute form,
//! - XHTML-style self-closing void tags (`<br />` vs `<br>`),
//! - entity spelling and UTF-8 URL spelling (reserved ASCII escapes remain distinct).
//!
//! It also strips output that ox-content adds on purpose and that the spec
//! never emits: slug `id` attributes on headings and `target`/`rel` on
//! links. Everything else — tag structure, attribute values, text content —
//! must match exactly.

#[path = "text_codec.rs"]
mod text_codec;

use text_codec::{
    decode_entities, encode_attr_into, encode_char_into, encode_text_into, normalize_url,
};

const VOID_TAGS: [&str; 3] = ["br", "hr", "img"];
const BLOCK_TAGS: [&str; 21] = [
    "blockquote",
    "body",
    "div",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "hr",
    "html",
    "li",
    "ol",
    "p",
    "pre",
    "table",
    "tbody",
    "td",
    "th",
    "thead",
    "tr",
];

struct Normalizer {
    out: String,
    literal_depth: usize,
    skip_leading_ws: bool,
    raw_element: Option<String>,
}

/// Normalizes an HTML fragment for spec comparison.
pub fn normalize_html(input: &str) -> String {
    let mut n = Normalizer {
        out: String::new(),
        literal_depth: 0,
        skip_leading_ws: true,
        raw_element: None,
    };
    let bytes = input.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if let Some(name) = &n.raw_element {
            // Raw text/RCDATA may contain '<' without starting nested markup.
            let end = bytes[i..]
                .windows(name.len() + 2)
                .enumerate()
                .find_map(|(offset, part)| {
                    (part[..2] == *b"</"
                        && part[2..].eq_ignore_ascii_case(name.as_bytes())
                        && matches!(
                            bytes.get(i + offset + part.len()),
                            Some(b'>' | b'/' | b' ' | b'\t' | b'\n' | b'\r' | b'\x0c')
                        ))
                    .then_some(offset)
                });
            let end = end.map_or(bytes.len(), |offset| i + offset);
            n.push_text(&input[i..end]);
            i = end;
            n.raw_element = None;
            if i == bytes.len() {
                break;
            }
        }
        if bytes[i] == b'<' {
            if let Some(consumed) = n.push_markup(&input[i..]) {
                i += consumed;
                continue;
            }
            // Not a well-formed tag: treat the `<` as literal text.
            n.push_text("<");
            i += 1;
            continue;
        }
        let end = input[i..]
            .find('<')
            .map_or(input.len(), |offset| i + offset);
        n.push_text(&input[i..end]);
        i = end;
    }

    while n.literal_depth == 0 && (n.out.ends_with(' ') || n.out.ends_with('\n')) {
        n.out.pop();
    }
    n.out
}

impl Normalizer {
    /// Handles `<...` markup. Returns consumed byte length, or `None` when
    /// the input is not well-formed markup.
    fn push_markup(&mut self, rest: &str) -> Option<usize> {
        for (open, close) in [
            ("<!--", "-->"),
            ("<![CDATA[", "]]>"),
            ("<?", "?>"),
            ("<!", ">"),
        ] {
            if let Some(after_open) = rest.strip_prefix(open) {
                let end = after_open.find(close)?;
                let total = open.len() + end + close.len();
                self.out.push_str(&rest[..total]);
                return Some(total);
            }
        }
        self.push_tag(rest)
    }

    fn push_tag(&mut self, rest: &str) -> Option<usize> {
        let bytes = rest.as_bytes();
        let closing = bytes.get(1) == Some(&b'/');
        let name_start = if closing { 2 } else { 1 };
        let mut i = name_start;
        while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-') {
            i += 1;
        }
        if i == name_start {
            return None;
        }
        let name = rest[name_start..i].to_ascii_lowercase();

        let mut attrs: Vec<(String, String)> = Vec::new();
        let mut self_closing = false;
        loop {
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            match bytes.get(i) {
                None => return None,
                Some(b'>') => {
                    i += 1;
                    break;
                }
                Some(b'/') if bytes.get(i + 1) == Some(&b'>') => {
                    self_closing = true;
                    i += 2;
                    break;
                }
                Some(_) => {
                    let (attr, consumed) = parse_attr(&rest[i..])?;
                    attrs.push(attr);
                    i += consumed;
                }
            }
        }

        self.emit_tag(&name, closing, self_closing, attrs);
        Some(i)
    }

    fn emit_tag(
        &mut self,
        name: &str,
        closing: bool,
        self_closing: bool,
        mut attrs: Vec<(String, String)>,
    ) {
        let is_block = BLOCK_TAGS.contains(&name);
        if is_block && self.literal_depth == 0 {
            while self.out.ends_with(' ') {
                self.out.pop();
            }
        }

        if closing {
            self.out.push_str("</");
            self.out.push_str(name);
            self.out.push('>');
            if matches!(name, "pre" | "code" | "textarea" | "script" | "style") {
                self.literal_depth = self.literal_depth.saturating_sub(1);
            }
        } else {
            attrs.retain(|(attr_name, _)| !is_stripped_attr(name, attr_name));
            attrs.sort();
            self.out.push('<');
            self.out.push_str(name);
            for (attr_name, value) in &attrs {
                self.out.push(' ');
                self.out.push_str(attr_name);
                self.out.push_str("=\"");
                let mut decoded = decode_entities(value);
                if matches!(attr_name.as_str(), "href" | "src") {
                    decoded = normalize_url(&decoded);
                }
                encode_attr_into(&mut self.out, &decoded);
                self.out.push('"');
            }
            if self_closing && !VOID_TAGS.contains(&name) {
                self.out.push_str(" /");
            }
            self.out.push('>');
            if matches!(name, "pre" | "code" | "textarea" | "script" | "style") {
                self.literal_depth += 1;
                if matches!(name, "textarea" | "script" | "style") {
                    self.raw_element = Some(name.to_owned());
                }
            }
        }

        if is_block && self.literal_depth == 0 {
            self.skip_leading_ws = true;
        }
    }

    fn push_text(&mut self, raw: &str) {
        let decoded = if matches!(self.raw_element.as_deref(), Some("script" | "style")) {
            raw.to_owned()
        } else {
            decode_entities(raw)
        };
        if self.literal_depth > 0 {
            encode_text_into(&mut self.out, &decoded);
            if !decoded.is_empty() {
                self.skip_leading_ws = false;
            }
            return;
        }

        for ch in decoded.chars() {
            if ch.is_ascii_whitespace() {
                if !self.skip_leading_ws && !self.out.ends_with(' ') {
                    self.out.push(' ');
                }
            } else {
                self.skip_leading_ws = false;
                encode_char_into(&mut self.out, ch);
            }
        }
    }
}

fn is_stripped_attr(tag: &str, attr: &str) -> bool {
    match tag {
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => attr == "id",
        "a" => attr == "target" || attr == "rel",
        _ => false,
    }
}

/// Parses one attribute at the start of `rest`; returns it plus consumed bytes.
fn parse_attr(rest: &str) -> Option<((String, String), usize)> {
    let bytes = rest.as_bytes();
    let mut i = 0;
    while i < bytes.len()
        && !bytes[i].is_ascii_whitespace()
        && !matches!(bytes[i], b'=' | b'>' | b'/')
    {
        i += 1;
    }
    if i == 0 {
        return None;
    }
    let name = rest[..i].to_ascii_lowercase();

    let mut j = i;
    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
        j += 1;
    }
    if bytes.get(j) != Some(&b'=') {
        return Some(((name, String::new()), i));
    }
    j += 1;
    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
        j += 1;
    }

    match bytes.get(j) {
        Some(&quote @ (b'"' | b'\'')) => {
            let value_start = j + 1;
            let end = rest[value_start..].find(quote as char)? + value_start;
            Some(((name, rest[value_start..end].to_string()), end + 1))
        }
        Some(_) => {
            let value_start = j;
            let mut end = j;
            while end < bytes.len()
                && !bytes[end].is_ascii_whitespace()
                && bytes[end] != b'>'
                && !(bytes[end] == b'/' && bytes.get(end + 1) == Some(&b'>'))
            {
                end += 1;
            }
            Some(((name, rest[value_start..end].to_string()), end))
        }
        None => None,
    }
}

#[test]
fn normalizes_equivalent_markup() {
    let spec = "<h1>Title</h1>\n<p>a <em>b</em> c</p>\n<hr />\n";
    let ours = "<h1 id=\"title\">Title</h1>\n<p>a <em>b</em>  c</p>\n<hr>\n";
    assert_eq!(normalize_html(spec), normalize_html(ours));

    let spec_link = "<p><a href=\"/caf%C3%A9\" title=\"t\">x</a></p>";
    let our_link =
        "<p><a href=\"/café\" target=\"_blank\" rel=\"noopener noreferrer\" title=\"t\">x</a></p>";
    assert_eq!(normalize_html(spec_link), normalize_html(our_link));
}

#[test]
fn keeps_meaningful_differences() {
    assert_ne!(normalize_html("<p>a b</p>"), normalize_html("<p>ab</p>"));
    assert_ne!(
        normalize_html("<p><em>a</em></p>"),
        normalize_html("<p>a</p>")
    );
    assert_ne!(
        normalize_html("<pre><code>a\n b\n</code></pre>"),
        normalize_html("<pre><code>a\nb\n</code></pre>")
    );
    assert_ne!(
        normalize_html("<p><a href=\"/x\">l</a></p>"),
        normalize_html("<p><a href=\"/y\">l</a></p>")
    );
}

#[test]
fn preserves_pre_content_verbatim() {
    let html = "<pre><code>  indented\n\ttab &amp; more  \n</code></pre>";
    let normalized = normalize_html(html);
    assert!(
        normalized.contains("  indented\n\ttab &amp; more  \n"),
        "{normalized}"
    );
}

#[test]
fn preserves_inline_code_and_raw_text_semantics() {
    for tag in ["code", "textarea", "script", "style"] {
        assert_ne!(
            normalize_html(&format!("<p><{tag}>  </{tag}></p>")),
            normalize_html(&format!("<p><{tag}></{tag}></p>")),
            "{tag} whitespace is content"
        );
    }
    assert_ne!(
        normalize_html("<p><code>a  b</code></p>"),
        normalize_html("<p><code>a b</code></p>")
    );
    assert_ne!(
        normalize_html("<script>let a='&amp;';</script>"),
        normalize_html("<script>let a='&';</script>")
    );
    assert_ne!(
        normalize_html("<script>x</scriptx>&amp;</script>"),
        normalize_html("<script>x</scriptx>&;</script>")
    );
    assert_ne!(normalize_html("<script>  "), normalize_html("<script>"));
    assert_ne!(
        normalize_html("<p><code>x</code> y</p>"),
        normalize_html("<p><code>x</code>y</p>")
    );
}

#[test]
fn preserves_reserved_and_unsafe_url_bytes() {
    for (encoded, literal) in [
        ("%23", "#"),
        ("%2F", "/"),
        ("%5C", "\\"),
        ("%26", "&"),
        ("%25", "%"),
    ] {
        assert_ne!(
            normalize_html(&format!("<a href=\"/a{encoded}b\">x</a>")),
            normalize_html(&format!("<a href=\"/a{literal}b\">x</a>"))
        );
    }
}
