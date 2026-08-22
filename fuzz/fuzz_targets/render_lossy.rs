#![no_main]

use ferromark::{Options, parse_with_options, to_html_into_with_options, to_html_with_options};
use libfuzzer_sys::fuzz_target;

// Keep standalone fuzz runs bounded as well as the scheduled libFuzzer job.
const MAX_INPUT_BYTES: usize = 64 * 1024;

#[allow(clippy::field_reassign_with_default)]
fn all_render_options() -> Options {
    let mut options = Options::default();
    options.definition_lists = true;
    options.footnotes = true;
    options.front_matter = true;
    options.highlight = true;
    options.inline_footnotes = true;
    options.line_comments = true;
    options.math = true;
    options.subscript = true;
    options.superscript = true;
    options
}

fuzz_target!(|input: &[u8]| {
    if input.len() > MAX_INPUT_BYTES {
        return;
    }

    let markdown = String::from_utf8_lossy(input);
    let options = all_render_options();

    let html = to_html_with_options(&markdown, &options);
    let parsed = parse_with_options(&markdown, &options);

    let mut reused_output = Vec::new();
    to_html_into_with_options(&markdown, &mut reused_output, &options);

    assert_eq!(parsed.html, html);
    assert_eq!(reused_output, html.as_bytes());
});
