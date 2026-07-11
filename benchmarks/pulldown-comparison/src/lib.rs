//! Focused, md4c-independent ferromark versus pulldown-cmark harness.

mod allocation;
mod corpus;
mod metadata;
mod model;

use ferromark::{Options as FerromarkOptions, RenderPolicy};
use pulldown_cmark::{Options as PulldownOptions, Parser, html};

pub use corpus::{Corpus, CorpusData};
pub use metadata::{EnvironmentMetadata, RunMeasurement, RunMetadata};
pub use model::{ParserKind, RunConfig};

/// Exact shared feature configurations used by comparison benchmarks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParityConfig {
    /// CommonMark with trusted raw HTML and no extensions.
    CommonMark,
    /// CommonMark plus tables, strikethrough, and task lists.
    GfmOverlap,
    /// GFM overlap plus footnotes, math, superscript, and callouts.
    ExtendedOverlap,
}

/// Return the explicit ferromark options for a parity configuration.
pub fn ferromark_options(config: ParityConfig) -> FerromarkOptions {
    let (tables, strikethrough, task_lists, footnotes, math, superscript, callouts) = match config {
        ParityConfig::CommonMark => (false, false, false, false, false, false, false),
        ParityConfig::GfmOverlap => (true, true, true, false, false, false, false),
        ParityConfig::ExtendedOverlap => (true, true, true, true, true, true, true),
    };

    FerromarkOptions {
        render_policy: RenderPolicy::Trusted,
        allow_html: true,
        allow_link_refs: true,
        tables,
        strikethrough,
        highlight: false,
        superscript,
        subscript: false,
        task_lists,
        autolink_literals: false,
        disallowed_raw_html: false,
        footnotes,
        front_matter: false,
        heading_ids: false,
        math,
        callouts,
    }
}

/// Return the explicit pulldown-cmark options for a parity configuration.
pub fn pulldown_options(config: ParityConfig) -> PulldownOptions {
    match config {
        ParityConfig::CommonMark => PulldownOptions::empty(),
        ParityConfig::GfmOverlap => {
            PulldownOptions::ENABLE_TABLES
                | PulldownOptions::ENABLE_STRIKETHROUGH
                | PulldownOptions::ENABLE_TASKLISTS
        }
        ParityConfig::ExtendedOverlap => {
            PulldownOptions::ENABLE_TABLES
                | PulldownOptions::ENABLE_STRIKETHROUGH
                | PulldownOptions::ENABLE_TASKLISTS
                | PulldownOptions::ENABLE_FOOTNOTES
                | PulldownOptions::ENABLE_MATH
                | PulldownOptions::ENABLE_SUPERSCRIPT
                | PulldownOptions::ENABLE_GFM
        }
    }
}

/// Render with ferromark into a reusable output buffer.
pub fn render_ferromark_into(input: &str, config: ParityConfig, output: &mut Vec<u8>) {
    output.clear();
    ferromark::to_html_into_with_options(input, output, &ferromark_options(config));
}

/// Render with pulldown-cmark into a reusable output buffer.
pub fn render_pulldown_into(input: &str, config: ParityConfig, output: &mut String) {
    output.clear();
    html::push_html(output, Parser::new_ext(input, pulldown_options(config)));
}

/// Render one diagnostic configuration with Ferromark.
pub fn render_ferromark_config_into(input: &str, config: RunConfig, output: &mut Vec<u8>) {
    output.clear();
    ferromark::to_html_into_with_options(input, output, &config.ferromark_options());
}

/// Render one diagnostic parity configuration with pulldown-cmark.
///
/// # Errors
///
/// Returns an error when a Ferromark-only product profile is selected.
pub fn render_pulldown_config_into(
    input: &str,
    config: RunConfig,
    output: &mut String,
) -> Result<(), &'static str> {
    let Some(options) = config.pulldown_options() else {
        return Err("pulldown-cmark only supports parity configurations");
    };
    output.clear();
    html::push_html(output, Parser::new_ext(input, options));
    Ok(())
}
pub use allocation::{AllocationSnapshot, CountingAllocator, MeasurementWindow};
