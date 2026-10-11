//! Asset-free preparation and read-only document reference extraction.

use napi::bindgen_prelude::Result;
use napi_derive::napi;

use crate::jsx::{SourceRange, prepare_jsx_document};
use crate::jsx_compiler::{PreparationOptions, PreparedJsxDocument, validate_preparation_options};
use crate::{Options, input::Utf8Input};

/// Parses and runs native passes without loading highlighting assets.
#[napi(catch_unwind)]
pub fn prepare_document(
    #[napi(ts_arg_type = "string | Uint8Array")] markdown: Utf8Input,
    options: Option<Options>,
    preparation_options: Option<PreparationOptions>,
) -> Result<PreparedJsxDocument> {
    validate_preparation_options(options.as_ref())?;
    let preparation = preparation_options.unwrap_or_default();
    PreparedJsxDocument::without_highlighting(prepare_jsx_document(
        markdown,
        options,
        preparation.format.as_deref(),
        preparation.track_references.unwrap_or(false),
    )?)
}

/// One reference occurrence from the prepared tree.
#[napi(object)]
pub struct DocumentReference {
    pub kind: String,
    pub destination: String,
    pub text: Option<String>,
    pub title: Option<String>,
    pub element: Option<String>,
    pub attribute: Option<String>,
    pub span: Option<SourceRange>,
    pub provenance: String,
    pub original_span: Option<SourceRange>,
}

#[napi]
impl PreparedJsxDocument {
    /// Collects Markdown references and literal MDX href/src attributes on demand.
    #[napi(catch_unwind)]
    pub fn collect_references(&self) -> Result<Vec<DocumentReference>> {
        Ok(self
            .parsed
            .collect_references()?
            .into_iter()
            .map(|reference| DocumentReference {
                kind: match reference.kind {
                    ferromark::ReferenceKind::Link => "link",
                    ferromark::ReferenceKind::Image => "image",
                    ferromark::ReferenceKind::Autolink => "autolink",
                    ferromark::ReferenceKind::MdxAttribute => "mdxAttribute",
                }
                .to_owned(),
                destination: reference.destination,
                text: reference.text,
                title: reference.title,
                element: reference.element,
                attribute: reference.attribute,
                span: reference.span.map(range),
                original_span: reference.original_span.map(range),
                provenance: match reference.provenance {
                    ferromark::ReferenceProvenance::Source => "source",
                    ferromark::ReferenceProvenance::Transformed => "transformed",
                }
                .to_owned(),
            })
            .collect())
    }
}

fn range(span: ferromark::ast::Span) -> SourceRange {
    SourceRange {
        start: span.start,
        end: span.end,
    }
}
