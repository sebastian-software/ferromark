//! Application work missing from a body-only renderer, included in time and heap.
//! These adapters use public events/AST/hooks, without a second Markdown parse.
use std::collections::VecDeque;
use std::fmt;
use std::sync::Mutex;

use comrak::adapters::{HeadingAdapter, HeadingMeta};
use comrak::nodes::{NodeValue, Sourcepos};
use ferromark::{Heading, ParseResult};
use pulldown_cmark::{BlockQuoteKind, Event, Parser, Tag, TagEnd};

// Parsed destinations have already undergone Markdown entity decoding. Decode
// again for the same conservative attribute policy as Ferromark's defaults.
// Unknown absolute schemes are rejected; relative URLs and this allowlist pass.
fn allowed_url(url: &str) -> bool {
    let decoded = html_escape::decode_html_entities(url);
    let Some(end) = decoded.find([':', '/', '?', '#']) else {
        return true;
    };
    if decoded.as_bytes()[end] != b':' {
        return true;
    }
    let scheme: String = decoded[..end]
        .chars()
        .filter(|c| !c.is_ascii_whitespace() && !c.is_ascii_control())
        .map(|c| c.to_ascii_lowercase())
        .collect();
    // A non-scheme colon (for example in a relative path) is not an absolute URL.
    if !scheme.starts_with(|c: char| c.is_ascii_alphabetic())
        || !scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
    {
        return true;
    }
    matches!(
        scheme.as_str(),
        "http"
            | "https"
            | "mailto"
            | "ftp"
            | "geo"
            | "irc"
            | "ircs"
            | "matrix"
            | "sms"
            | "tel"
            | "xmpp"
    )
}

// Borrow the complete raw YAML/TOML block, including its final newline. No YAML
// parsing is requested in this workload. An unclosed delimiter stays Markdown.
fn front_matter(input: &str) -> (Option<&str>, &str) {
    let Some(first_end) = input.find('\n') else {
        return (None, input);
    };
    let delimiter = input[..first_end].trim_end_matches([' ', '\t', '\r']);
    if !matches!(delimiter, "---" | "+++") {
        return (None, input);
    }
    let start = first_end + 1;
    let mut offset = start;
    for line in input[start..].split_inclusive('\n') {
        if line.trim_end_matches([' ', '\t', '\r', '\n']) == delimiter {
            return (Some(&input[start..offset]), &input[offset + line.len()..]);
        }
        offset += line.len();
    }
    (None, input)
}

#[derive(Default)]
struct Headings {
    slugs: comrak::Anchorizer,
    items: Vec<Heading>,
    collect: bool,
}

impl Headings {
    fn add(&mut self, level: u8, text: String) -> String {
        let id = self.slugs.anchorize(&text);
        if self.collect {
            self.items.push(Heading {
                level,
                id: Some(id.clone()),
                text,
            });
        }
        id
    }
}

struct ComrakHeadings(Mutex<Headings>);

impl HeadingAdapter for ComrakHeadings {
    fn enter(
        &self,
        output: &mut dyn fmt::Write,
        heading: &HeadingMeta,
        _: Option<Sourcepos>,
    ) -> fmt::Result {
        let id = self
            .0
            .lock()
            .unwrap()
            .add(heading.level, heading.content.clone());
        write!(
            output,
            "<h{} id=\"{}\">",
            heading.level,
            html_escape::encode_double_quoted_attribute(&id)
        )
    }

    fn exit(&self, output: &mut dyn fmt::Write, heading: &HeadingMeta) -> fmt::Result {
        writeln!(output, "</h{}>", heading.level)
    }
}

pub fn comrak<'a>(
    input: &'a str,
    options: &comrak::Options<'_>,
    metadata: bool,
) -> ParseResult<'a> {
    let (front_matter, body) = if metadata {
        front_matter(input)
    } else {
        (None, input)
    };
    let arena = comrak::Arena::new();
    let root = comrak::parse_document(&arena, body, options);
    for node in root.descendants() {
        if let NodeValue::Link(link) | NodeValue::Image(link) = &mut node.data_mut().value
            && !allowed_url(&link.url)
        {
            link.url.clear();
        }
    }
    let headings = ComrakHeadings(Mutex::new(Headings {
        collect: metadata,
        ..Headings::default()
    }));
    let mut plugins = comrak::options::Plugins::default();
    plugins.render.heading_adapter = Some(&headings);
    let mut html = String::new();
    comrak::format_html_with_plugins(root, options, &mut html, &plugins).unwrap();
    ParseResult {
        html,
        front_matter,
        headings: headings.0.into_inner().unwrap().items,
        resource_limits: Default::default(),
    }
}

struct PulldownEvents<'a> {
    parser: Parser<'a>,
    pending: VecDeque<Event<'a>>,
    headings: Headings,
}

// User HTML becomes text; only adapter-owned fixed callout markup becomes HTML.
fn policy(event: Event<'_>) -> Event<'_> {
    match event {
        Event::Html(text) | Event::InlineHtml(text) => Event::Text(text),
        Event::Start(Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        }) => {
            let dest_url = if allowed_url(&dest_url) {
                dest_url
            } else {
                "".into()
            };
            Event::Start(Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            })
        }
        Event::Start(Tag::Image {
            link_type,
            dest_url,
            title,
            id,
        }) => {
            let dest_url = if allowed_url(&dest_url) {
                dest_url
            } else {
                "".into()
            };
            Event::Start(Tag::Image {
                link_type,
                dest_url,
                title,
                id,
            })
        }
        Event::Start(Tag::BlockQuote(Some(kind))) => {
            let (class, title) = match kind {
                BlockQuoteKind::Note => ("note", "Note"),
                BlockQuoteKind::Tip => ("tip", "Tip"),
                BlockQuoteKind::Important => ("important", "Important"),
                BlockQuoteKind::Warning => ("warning", "Warning"),
                BlockQuoteKind::Caution => ("caution", "Caution"),
            };
            Event::Html(format!("<div class=\"markdown-alert markdown-alert-{class}\">\n<p class=\"markdown-alert-title\">{title}</p>\n").into())
        }
        Event::End(TagEnd::BlockQuote(Some(_))) => Event::Html("</div>\n".into()),
        other => other,
    }
}

impl<'a> Iterator for PulldownEvents<'a> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(event) = self.pending.pop_front() {
            return Some(event);
        }
        let event = self.parser.next()?;
        if let Event::Start(Tag::Heading {
            level,
            classes,
            attrs,
            ..
        }) = event
        {
            // The start tag needs its slug before HTML output. Buffer only this
            // heading, not the document, while collecting its plain text.
            let mut text = String::new();
            for child in self.parser.by_ref() {
                match &child {
                    Event::Text(value) | Event::Code(value) => text.push_str(value),
                    Event::SoftBreak | Event::HardBreak => text.push(' '),
                    _ => {}
                }
                let done = matches!(child, Event::End(TagEnd::Heading(_)));
                self.pending.push_back(policy(child));
                if done {
                    break;
                }
            }
            let id = self.headings.add(level as u8, text);
            Some(Event::Start(Tag::Heading {
                level,
                id: Some(id.into()),
                classes,
                attrs,
            }))
        } else {
            Some(policy(event))
        }
    }
}

pub fn pulldown(input: &str, options: pulldown_cmark::Options, metadata: bool) -> ParseResult<'_> {
    let (front_matter, body) = if metadata {
        front_matter(input)
    } else {
        (None, input)
    };
    let mut events = PulldownEvents {
        parser: Parser::new_ext(body, options),
        pending: VecDeque::new(),
        headings: Headings {
            collect: metadata,
            ..Headings::default()
        },
    };
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, &mut events);
    ParseResult {
        html,
        front_matter,
        headings: events.headings.items,
        resource_limits: Default::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_front_matter_is_borrowed_and_unclosed_blocks_stay_markdown() {
        let input = "---\r\ntitle: Hello\r\n---\r\n# Body";
        assert_eq!(front_matter(input), (Some("title: Hello\r\n"), "# Body"));
        assert_eq!(front_matter("+++\na = 1\n+++"), (Some("a = 1\n"), ""));
        for input in ["---\nunclosed\n# Body", "----\nx\n---\n", " ---\nx\n---\n"] {
            assert_eq!(front_matter(input), (None, input));
        }
    }

    #[test]
    fn url_policy_rejects_obfuscated_and_unknown_schemes_and_keeps_relative_links() {
        for url in [
            "javascript:alert(1)",
            "JaVaScRiPt:foo",
            "javas&#99;ript:foo",
            "java\t\nscript:foo",
            "data:image/png;base64,abc",
            "custom:foo",
            "vbscript:foo",
        ] {
            assert!(!allowed_url(url), "{url}");
        }
        for url in [
            "/docs",
            "//example.org/docs",
            "#heading",
            "?q=x:y",
            "path/file:part",
            "https://example.org",
            "mailto:x@example.org",
            "tel:123",
            "geo:1,2",
        ] {
            assert!(allowed_url(url), "{url}");
        }
    }
}
