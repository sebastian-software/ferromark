use crate::Options;

use super::{Segment, segment};

/// Rendered MDX output with extracted metadata.
pub struct MdxOutput<'a> {
    /// Rendered body: Markdown→HTML, JSX/expressions passed through.
    pub body: String,
    /// ESM statements (import/export), in document order.
    pub esm: Vec<&'a str>,
    /// Front matter content (if present in first Markdown segment).
    pub front_matter: Option<&'a str>,
}

/// Render MDX to HTML body with default options.
pub fn render(input: &str) -> MdxOutput<'_> {
    render_with_options(input, &mdx_default_options())
}

/// Render MDX to HTML body with custom Markdown options.
pub fn render_with_options<'a>(input: &'a str, options: &Options) -> MdxOutput<'a> {
    let segments = segment(input);
    let mut body = String::with_capacity(input.len());
    let mut esm: Vec<&'a str> = Vec::new();
    let mut front_matter: Option<&'a str> = None;

    for seg in &segments {
        match seg {
            Segment::Esm(s) => {
                esm.push(s);
            }
            Segment::Markdown(s) => {
                let result = crate::parse_with_options(s, options);
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

    MdxOutput {
        body,
        esm,
        front_matter,
    }
}

fn mdx_default_options() -> Options {
    Options {
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
}
