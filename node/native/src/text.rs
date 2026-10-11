//! Bulk text analysis on an already parsed and transformed native tree.

use ferromark::ast::{Document, Span};
use ferromark::{JsxRendererOptions, TextExtractionOptions};
use napi::bindgen_prelude::{Error, Result, Status};
use napi_derive::napi;

use crate::jsx::SourceRange;

#[napi(object)]
#[derive(Default)]
pub struct TextOptions {
    pub include_block_code: Option<bool>,
    pub heading_ids: Option<bool>,
    pub heading_offset: Option<f64>,
    pub heading_id_prefix: Option<String>,
    pub omit_title_heading: Option<String>,
    pub callouts: Option<bool>,
}

#[napi(object)]
pub struct TextRange {
    pub start: u32,
    pub end: u32,
}

#[napi(object)]
pub struct TextSection {
    pub heading: String,
    pub level: u32,
    pub parent_index: Option<u32>,
    pub id: Option<String>,
    pub heading_span: Option<SourceRange>,
    pub source_span: Option<SourceRange>,
    pub text_range: TextRange,
    pub text: String,
}

#[napi(object)]
pub struct TextExtraction {
    pub text: String,
    pub sections: Vec<TextSection>,
    pub omitted_title_heading_span: Option<SourceRange>,
}

pub(crate) fn extract(
    document: &Document<'_>,
    include_block_code: bool,
    rendering: JsxRendererOptions,
) -> Result<TextExtraction> {
    let extracted = document.extract_text(&TextExtractionOptions {
        include_block_code,
        rendering,
    });
    Ok(TextExtraction {
        text: extracted.text,
        sections: extracted
            .sections
            .into_iter()
            .map(|section| {
                Ok(TextSection {
                    heading: section.heading,
                    level: u32::from(section.level),
                    parent_index: section.parent_index.map(output_offset).transpose()?,
                    id: section.id,
                    heading_span: section.heading_span.map(range),
                    source_span: section.source_span.map(range),
                    text_range: TextRange {
                        start: output_offset(section.text_range.start)?,
                        end: output_offset(section.text_range.end)?,
                    },
                    text: section.text,
                })
            })
            .collect::<Result<Vec<_>>>()?,
        omitted_title_heading_span: extracted.omitted_title_heading.map(range),
    })
}

fn output_offset(offset: usize) -> Result<u32> {
    u32::try_from(offset).map_err(|_| {
        Error::new(
            Status::GenericFailure,
            "Extracted text exceeds the supported byte range",
        )
    })
}

fn range(span: Span) -> SourceRange {
    SourceRange {
        start: span.start,
        end: span.end,
    }
}
