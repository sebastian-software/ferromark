//! Static Markdown projection from an existing prepared tree and owned source.

use ferromark::{MarkdownProjectionDiagnosticKind as Kind, MarkdownProjectionOptions};
use napi::bindgen_prelude::{Error, Result, Status};
use napi_derive::napi;

use crate::jsx::{ParsedJsxDocument, SourceRange};

#[napi(object)]
pub struct ProjectMarkdownOptions {
    /// Preserve recognized metadata by default, or omit its complete block.
    pub front_matter: Option<String>,
    /// Omit only the first top-level H1 if its display text matches this title.
    pub omit_title_heading: Option<String>,
}

#[napi(object)]
pub struct MarkdownProjectionDiagnostic {
    pub code: String,
    pub message: String,
    /// Inclusive original UTF-8 byte offset.
    pub start: u32,
    /// Exclusive original UTF-8 byte offset.
    pub end: u32,
}

#[napi(object)]
pub struct MarkdownProjectionResult {
    pub markdown: String,
    pub diagnostics: Vec<MarkdownProjectionDiagnostic>,
    pub omitted_title_heading_span: Option<SourceRange>,
}

pub(crate) fn project_markdown(
    parsed: &ParsedJsxDocument,
    options: Option<ProjectMarkdownOptions>,
) -> Result<MarkdownProjectionResult> {
    let (omit_front_matter, omit_title_heading) = match options {
        Some(options) => (
            match options.front_matter.as_deref() {
                None | Some("preserve") => false,
                Some("omit") => true,
                _ => {
                    return Err(Error::new(
                        Status::InvalidArg,
                        "frontMatter must be 'preserve' or 'omit'",
                    ));
                }
            },
            options.omit_title_heading,
        ),
        None => (false, None),
    };
    parsed.with_document(|source, document| {
        let result = document
            .project_markdown(
                source,
                &MarkdownProjectionOptions {
                    omit_front_matter,
                    omit_title_heading,
                },
            )
            .map_err(|error| Error::new(Status::InvalidArg, error.to_string()))?;
        Ok(MarkdownProjectionResult {
            markdown: result.markdown,
            diagnostics: result
                .diagnostics
                .into_iter()
                .map(|diagnostic| {
                    let (code, message) = diagnostic_message(diagnostic.kind);
                    MarkdownProjectionDiagnostic {
                        code: code.into(),
                        message: message.into(),
                        start: diagnostic.span.start,
                        end: diagnostic.span.end,
                    }
                })
                .collect(),
            omitted_title_heading_span: result.omitted_title_heading.map(|span| SourceRange {
                start: span.start,
                end: span.end,
            }),
        })
    })
}

fn diagnostic_message(kind: Kind) -> (&'static str, &'static str) {
    match kind {
        Kind::EsmOmitted => (
            "esm-omitted",
            "Module source was omitted without importing or executing it.",
        ),
        Kind::JsxWrapperUnwrapped => (
            "jsx-wrapper-unwrapped",
            "JSX tags and attributes were discarded; only static source children remain. Component output may differ.",
        ),
        Kind::ExpressionOmitted => (
            "expression-omitted",
            "A JavaScript expression was omitted without evaluation.",
        ),
        Kind::SelfClosingElementOmitted => (
            "self-closing-element-omitted",
            "A self-closing JSX element has no known static children and was omitted.",
        ),
        Kind::RawHtmlOmitted => (
            "raw-html-omitted",
            "Raw HTML was omitted. Block HTML may include its entire body.",
        ),
        Kind::UnsupportedJsxOmitted => (
            "unsupported-jsx-omitted",
            "JSX source boundaries could not be verified; the entire element was omitted.",
        ),
        Kind::RawTextElementOmitted => (
            "raw-text-element-omitted",
            "An intrinsic script, style, title, or textarea element was omitted with its body.",
        ),
        Kind::FrontMatterOmitted => (
            "front-matter-omitted",
            "Recognized front matter was omitted by request.",
        ),
        Kind::TitleHeadingOmitted => (
            "title-heading-omitted",
            "A matching top-level title heading was omitted by request.",
        ),
    }
}
