use std::fmt::Write;

use crate::Options;

use super::{Segment, segment};

/// Error returned when a component name cannot be used as a JavaScript binding.
#[derive(Debug, Clone, PartialEq, Eq)]
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
            for line in body.lines() {
                if line.is_empty() {
                    out.push('\n');
                } else {
                    out.push_str("      ");
                    out.push_str(line);
                    out.push('\n');
                }
            }
        }

        out.push_str("    </>\n  );\n}\n");
        Ok(out)
    }
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
