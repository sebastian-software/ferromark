#![no_main]

use ferromark::{BlockParser, InlineParser, Options};
use libfuzzer_sys::fuzz_target;

// This preserves useful structural inputs while keeping a single mutation from
// exhausting memory when the target is run without the CI libFuzzer arguments.
const MAX_INPUT_BYTES: usize = 64 * 1024;

#[allow(clippy::field_reassign_with_default)]
fn all_inline_options() -> Options {
    let mut options = Options::default();
    options.definition_lists = true;
    options.footnotes = true;
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

    let options = all_inline_options();

    let mut block_events = Vec::new();
    let mut block_parser = BlockParser::new_with_options(input, options.clone());
    block_parser.parse(&mut block_events);

    let link_references = block_parser.take_link_refs();
    let mut inline_events = Vec::new();
    let mut inline_parser = InlineParser::new();
    inline_parser.parse(
        input,
        Some(&link_references),
        options.allow_html,
        &mut inline_events,
    );
});
