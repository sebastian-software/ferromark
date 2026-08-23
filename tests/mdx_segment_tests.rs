#![cfg(feature = "mdx")]

use ferromark::mdx::{
    MdxDiagnosticCode, Segment, render, render_with_options, segment, segment_spanned,
    segment_strict, source_location,
};
use ferromark::{CodeBlockKind, Options};

// ── Helper ───────────────────────────────────────────────────────────

/// Extract the inner &str from any Segment variant.
fn seg_str<'a>(seg: &Segment<'a>) -> &'a str {
    seg.as_str()
}

// ── Basic segmentation ───────────────────────────────────────────────

#[test]
fn pure_markdown_document() {
    let input = "# Hello\n\nSome paragraph.\n";
    let segs = segment(input);
    assert_eq!(segs, vec![Segment::Markdown(input)]);
}

#[test]
fn empty_input() {
    let segs = segment("");
    assert!(segs.is_empty());
}

#[test]
fn whitespace_only() {
    let input = "   \n\n   \n";
    let segs = segment(input);
    assert_eq!(segs, vec![Segment::Markdown(input)]);
}

#[test]
fn spanned_segments_cover_input_with_exact_byte_ranges() {
    let input = "import A from 'a'\r\n\r\n# Grüß\r\n\r\n<Card />\r\n\r\n{user.name}\r\n";
    let segments = segment_spanned(input);

    assert_eq!(segments.len(), 5);
    assert!(matches!(segments[0].segment, Segment::Esm(_)));
    assert!(matches!(segments[1].segment, Segment::Markdown(_)));
    assert!(matches!(segments[2].segment, Segment::JsxBlockSelfClose(_)));
    assert!(matches!(segments[3].segment, Segment::Markdown(_)));
    assert!(matches!(segments[4].segment, Segment::Expression(_)));

    let mut expected_start = 0;
    let mut reconstructed = String::new();
    for spanned in &segments {
        assert_eq!(spanned.range.start_usize(), expected_start);
        assert_eq!(
            spanned.range.slice_str(input.as_bytes()).unwrap(),
            spanned.segment.as_str()
        );
        assert_eq!(
            spanned.range.end_usize() - spanned.range.start_usize(),
            spanned.segment.as_str().len()
        );
        expected_start = spanned.range.end_usize();
        reconstructed.push_str(spanned.segment.as_str());
    }

    assert_eq!(expected_start, input.len());
    assert_eq!(reconstructed, input);
}

#[test]
fn spanned_segments_match_existing_segment_api() {
    let input = "---\ntitle: Hello\n---\n\n<Layout>\n\n# Heading\n\n</Layout>\n";
    let existing = segment(input);
    let spanned = segment_spanned(input);

    assert_eq!(
        spanned
            .into_iter()
            .map(|segment| segment.segment)
            .collect::<Vec<_>>(),
        existing
    );
}

#[test]
fn fenced_code_keeps_mdx_constructs_in_a_single_markdown_segment() {
    let input =
        "   ```jsx\n<Card />\n{value}\nimport A from 'a'\nexport { A }\n  ```\n\n<Live />\n";
    let segments = segment_spanned(input);

    assert_eq!(segments.len(), 2);
    assert_eq!(
        segments[0].segment,
        Segment::Markdown(
            "   ```jsx\n<Card />\n{value}\nimport A from 'a'\nexport { A }\n  ```\n\n"
        )
    );
    assert_eq!(segments[0].range.start_usize(), 0);
    assert_eq!(
        segments[0].range.end_usize(),
        segments[0].segment.as_str().len()
    );
    assert_eq!(
        segments[1].segment,
        Segment::JsxBlockSelfClose("<Live />\n")
    );
}

#[test]
fn fenced_code_closes_only_with_the_matching_marker_and_length() {
    let input = "````\n<Card />\n```\n{unterminated\n````\n";

    assert_eq!(segment(input), vec![Segment::Markdown(input)]);
    assert!(segment_strict(input).is_ok());
}

#[test]
fn fenced_code_allows_root_esm_immediately_after_its_closing_fence() {
    let input = "```\n<Card />\n```\nimport A from 'a'\nexport { A }\n";
    let segments = segment(input);

    assert_eq!(
        segments,
        vec![
            Segment::Markdown("```\n<Card />\n```\n"),
            Segment::Esm("import A from 'a'\n"),
            Segment::Esm("export { A }\n"),
        ]
    );
    assert_eq!(segment_strict(input).unwrap(), segment_spanned(input));
}

#[test]
fn fenced_code_fence_boundaries_match_the_markdown_parser() {
    let crlf_input = "```\r\n<Card />\r\n```\r\nimport A from 'a'\r\n";
    assert_eq!(segment(crlf_input), vec![Segment::Markdown(crlf_input)]);
    assert!(segment_strict(crlf_input).is_ok());

    let tab_closing_input = "```\n<Card />\n\t```\nimport A from 'a'\n";
    assert_eq!(
        segment(tab_closing_input),
        vec![
            Segment::Markdown("```\n<Card />\n\t```\n"),
            Segment::Esm("import A from 'a'\n"),
        ]
    );
    assert_eq!(
        segment_strict(tab_closing_input).unwrap(),
        segment_spanned(tab_closing_input)
    );
}

#[test]
fn tilde_fenced_code_does_not_create_mdx_segments_or_strict_diagnostics() {
    let input = "~~~tsx\n</Card>\n{unterminated\nimport A from 'a'\n~~~\n";

    assert_eq!(segment(input), vec![Segment::Markdown(input)]);
    assert_eq!(segment_strict(input).unwrap(), segment_spanned(input));
}

#[test]
fn rendering_fenced_mdx_like_syntax_keeps_it_as_escaped_code() {
    let input = "```jsx\n<Card />\n{value}\nimport A from 'a'\n```\n";
    let output = render(input);

    assert!(output.body.contains("&lt;Card /&gt;"));
    assert!(output.body.contains("{value}"));
    assert!(output.body.contains("import A from 'a'"));
    assert!(!output.body.contains("\n<Card />\n"));
}

#[test]
fn semicolonless_esm_stops_before_backtick_and_tilde_fences() {
    for (input, esm) in [
        (
            "import A from 'a'\n```jsx\n<Card />\n```\n",
            "import A from 'a'\n",
        ),
        ("export { A }\n~~~jsx\n<Card />\n~~~\n", "export { A }\n"),
    ] {
        assert_eq!(
            segment(input),
            vec![Segment::Esm(esm), Segment::Markdown(&input[esm.len()..])]
        );
        assert_eq!(segment_strict(input).unwrap(), segment_spanned(input));

        let output = render(input);
        assert!(output.body.contains("&lt;Card /&gt;"));
        assert!(!output.body.contains("\n<Card />\n"));
    }
}

#[test]
fn list_fenced_code_keeps_mdx_constructs_opaque() {
    for input in [
        "- ```jsx\n  <Card />\n  {value}\n  import A from 'a'\n  ```\n",
        "1. ~~~jsx\n   <Card />\n   {value}\n   export { value }\n   ~~~\n",
    ] {
        assert_eq!(segment(input), vec![Segment::Markdown(input)]);
        assert_eq!(segment_strict(input).unwrap(), segment_spanned(input));

        let output = render(input);
        assert!(output.body.contains("&lt;Card /&gt;"));
        assert!(output.body.contains("{value}"));
        assert!(!output.body.contains("\n<Card />\n"));
    }
}

#[test]
fn list_fence_closer_restores_post_list_mdx_detection() {
    let list = "- ```jsx\n  <Card />\n  ```\n";
    let input = "- ```jsx\n  <Card />\n  ```\nimport A from \"a\"\n<Live />\n{value}\n";

    assert_eq!(
        segment(input),
        vec![
            Segment::Markdown(list),
            Segment::Esm("import A from \"a\"\n"),
            Segment::JsxBlockSelfClose("<Live />\n"),
            Segment::Expression("{value}\n"),
        ]
    );
    assert_eq!(segment_strict(input).unwrap(), segment_spanned(input));

    let output = render(input);
    assert_eq!(output.esm, vec!["import A from \"a\"\n"]);
    assert!(output.body.contains("&lt;Card /&gt;"));
    assert!(output.body.contains("<Live />"));
    assert!(output.body.contains("{value}"));

    let diagnostics = segment_strict(&format!("{list}</Unmatched>\n")).unwrap_err();
    assert_eq!(
        diagnostics[0].code,
        MdxDiagnosticCode::UnexpectedJsxClosingTag
    );
}

#[test]
fn semicolonless_esm_stops_before_container_fences() {
    for (input, esm) in [
        (
            "import A from 'a'\n- ```jsx\n  <Card />\n  ```\n",
            "import A from 'a'\n",
        ),
        (
            "export { A }\n1. ~~~jsx\n   <Card />\n   ~~~\n",
            "export { A }\n",
        ),
    ] {
        assert_eq!(
            segment(input),
            vec![Segment::Esm(esm), Segment::Markdown(&input[esm.len()..])]
        );
        assert_eq!(segment_strict(input).unwrap(), segment_spanned(input));

        let output = render(input);
        assert_eq!(output.esm, vec![esm]);
        assert!(output.body.contains("&lt;Card /&gt;"));
    }
}

#[test]
fn semicolonless_multiline_esm_stops_before_continuation_fence_owners() {
    for (input, esm) in [
        (
            "export {\n  A\n}\n- item\n  ```jsx\n  <Card />\n  ```\n",
            "export {\n  A\n}\n",
        ),
        (
            "export {\n  A\n}\n1. item\n   ~~~jsx\n   <Card />\n   ~~~\n",
            "export {\n  A\n}\n",
        ),
        (
            "export {\n  A\n}\n> item\n> ```jsx\n> <Card />\n> ```\n",
            "export {\n  A\n}\n",
        ),
    ] {
        assert_eq!(
            segment(input),
            vec![Segment::Esm(esm), Segment::Markdown(&input[esm.len()..])]
        );
        assert_eq!(segment_strict(input).unwrap(), segment_spanned(input));

        let output = render(input);
        assert_eq!(output.esm, vec![esm]);
        assert!(output.body.contains("item"));
        assert!(output.body.contains("&lt;Card /&gt;"));
    }
}

#[test]
fn semicolonless_esm_stops_before_a_list_without_a_continuation_fence() {
    let input = "export { A }\n- ordinary list item\n";

    assert_eq!(
        segment(input),
        vec![
            Segment::Esm("export { A }\n"),
            Segment::Markdown("- ordinary list item\n"),
        ]
    );
    assert_eq!(segment_strict(input).unwrap(), segment_spanned(input));
}

#[test]
fn complete_semicolonless_esm_does_not_swallow_following_markdown() {
    let input = "export const x = 1\nSome prose paragraph.\nMore prose.\n\nAfter blank.\n";

    assert_eq!(
        segment(input),
        vec![
            Segment::Esm("export const x = 1\n"),
            Segment::Markdown("Some prose paragraph.\nMore prose.\n\nAfter blank.\n"),
        ]
    );
    assert_eq!(segment_strict(input).unwrap(), segment_spanned(input));

    let output = render(input);
    assert_eq!(output.esm, vec!["export const x = 1\n"]);
    assert!(output.body.contains("Some prose paragraph."));
    assert!(output.body.contains("More prose."));

    let component = output.to_component("Page").unwrap();
    assert!(component.contains("<p>Some prose paragraph."));
    assert!(!component.starts_with("export const x = 1\nSome prose"));
}

#[test]
fn complete_semicolonless_esm_preserves_markdown_and_mdx_boundaries() {
    for (following, expected) in [
        ("Ordinary prose.\n", Segment::Markdown("Ordinary prose.\n")),
        ("- list item\n", Segment::Markdown("- list item\n")),
        ("> quote\n", Segment::Markdown("> quote\n")),
        (
            "![diagram](diagram.png)\n",
            Segment::Markdown("![diagram](diagram.png)\n"),
        ),
        ("# Heading\n", Segment::Markdown("# Heading\n")),
        ("<Card />\n", Segment::JsxBlockSelfClose("<Card />\n")),
        ("{value}\n", Segment::Expression("{value}\n")),
    ] {
        let input = format!("export const x = 1\n{following}");
        assert_eq!(
            segment(&input),
            vec![Segment::Esm("export const x = 1\n"), expected]
        );
        assert_eq!(segment_strict(&input).unwrap(), segment_spanned(&input));
    }

    let crlf = "export const x = 1\r\nOrdinary prose.\r\n";
    assert_eq!(
        segment(crlf),
        vec![
            Segment::Esm("export const x = 1\r\n"),
            Segment::Markdown("Ordinary prose.\r\n"),
        ]
    );

    let fence = "export const x = 1\n```jsx\n<Card />\n```\n";
    assert_eq!(
        segment(fence),
        vec![
            Segment::Esm("export const x = 1\n"),
            Segment::Markdown("```jsx\n<Card />\n```\n"),
        ]
    );
}

#[test]
fn multiline_esm_keeps_lexically_required_continuations() {
    for (input, esm) in [
        (
            "import {\n  A as B,\n} from \"module\"\nMarkdown.\n",
            "import {\n  A as B,\n} from \"module\"\n",
        ),
        (
            "import Widget\nfrom \"module\"\nMarkdown.\n",
            "import Widget\nfrom \"module\"\n",
        ),
        (
            "export {\n  A as B,\n}\nfrom \"module\"\nMarkdown.\n",
            "export {\n  A as B,\n}\nfrom \"module\"\n",
        ),
        (
            "export const value = {\n  title: \"brace } in a string\",\n  template: `value: ${name}`,\n  // } in a comment\n}\nMarkdown.\n",
            "export const value = {\n  title: \"brace } in a string\",\n  template: `value: ${name}`,\n  // } in a comment\n}\n",
        ),
        (
            "export const total = 1 +\n  2\nMarkdown.\n",
            "export const total = 1 +\n  2\n",
        ),
        (
            "export default\nComponent\nMarkdown.\n",
            "export default\nComponent\n",
        ),
        (
            "export const note = /* comment\n */ `value: ${name}`\nMarkdown.\n",
            "export const note = /* comment\n */ `value: ${name}`\n",
        ),
    ] {
        assert_eq!(
            segment(input),
            vec![Segment::Esm(esm), Segment::Markdown(&input[esm.len()..])]
        );
        assert_eq!(segment_strict(input).unwrap(), segment_spanned(input));
    }
}

#[test]
fn esm_regex_literals_do_not_change_delimiter_state() {
    for esm in [
        "export const closing = /\\}/g\n",
        "export const class_closing = /[}]/g\n",
    ] {
        let input = format!("{esm}Markdown.\n");
        assert_eq!(
            segment(&input),
            vec![Segment::Esm(esm), Segment::Markdown("Markdown.\n")]
        );
        assert_eq!(segment_strict(&input).unwrap(), segment_spanned(&input));
        assert_eq!(render(&input).esm, vec![esm]);
    }
}

#[test]
fn esm_terminal_regex_literals_end_without_flags_or_member_access() {
    for esm in [
        "export const matcher = /}/\n",
        "export default /}/\n",
        "export const matcher = () => /}/\n",
        "export const escaped = /\\//\n",
        "export const class_closing = /[}]/\n",
        "export const empty_match = /(?:)/\n",
        "export const flagged = /}/gi\n",
        "export const member = /}/.source\n",
        "export const cross_line_member = /}/\n.test(value)\n",
        "export const cross_line_flagged_member = /}/gi\n.test(value)\n",
        "export const cross_line_optional_member = /}/\n?.test(value)\n",
        "export const cross_line_index = /}/\n[Symbol.match](value)\n",
        "export const cross_line_call = /}/\n(value)\n",
        "export const cross_line_tag = /}/\n`value`\n",
        "export const ratio = value / divisor\n",
        "export const continued = value /\n  divisor\n",
        "export function matches(value) { if (value) /}/\n}\n",
    ] {
        let input = format!("{esm}Markdown.\n");
        assert_eq!(
            segment(&input),
            vec![Segment::Esm(esm), Segment::Markdown("Markdown.\n")]
        );
        assert_eq!(segment_strict(&input).unwrap(), segment_spanned(&input));

        let output = render(&input);
        assert_eq!(output.esm, vec![esm]);
        assert!(output.body.contains("Markdown."));
        assert!(output.to_component("Page").unwrap().starts_with(esm));
    }
}

#[test]
fn esm_statement_context_regex_literals_do_not_change_delimiter_state() {
    for esm in [
        "export function after_condition(value) {\n  if (value) /\\}/.test(value)\n}\n",
        "export function after_block(value) {\n  if (value) {}\n  /\\}/.test(value)\n}\n",
    ] {
        let input = format!("{esm}Markdown.\n");
        assert_eq!(
            segment(&input),
            vec![Segment::Esm(esm), Segment::Markdown("Markdown.\n")]
        );
        assert_eq!(segment_strict(&input).unwrap(), segment_spanned(&input));
        assert_eq!(render(&input).esm, vec![esm]);
    }
}

#[test]
fn esm_other_statement_blocks_allow_following_regex_literals() {
    // The continuation scanner deliberately does not validate JavaScript
    // statement grammar. These exercise its lexical statement-block context:
    // all four block introducers must leave a following regex literal opaque.
    for esm in [
        "export function after_else(value) { if (value) {} else {} /\\}/.test(value) }\n",
        "export function after_try() { try {} /\\}/.test(value) }\n",
        "export function after_finally() { try {} finally {} /\\}/.test(value) }\n",
        "export function after_do() { do {} /\\}/.test(value) }\n",
        "export function after_all_blocks(value) { if (value) {} else {} /\\}/.test(value); try {} finally {} /\\}/.test(value); do {} /\\}/.test(value) }\n",
    ] {
        let input = format!("{esm}Markdown.\n");
        assert_eq!(
            segment(&input),
            vec![Segment::Esm(esm), Segment::Markdown("Markdown.\n")]
        );
        assert_eq!(segment_strict(&input).unwrap(), segment_spanned(&input));
        assert_eq!(render(&input).esm, vec![esm]);
    }
}

#[test]
fn esm_expression_closers_do_not_start_regex_literals() {
    for esm in [
        "export const ratio = divide(value) / divisor\n",
        "export const ratio = ({ value: 1 }) / divisor\n",
        "export function ratio(value) { return ({ value }) / divisor }\n",
    ] {
        let input = format!("{esm}Markdown.\n");
        assert_eq!(
            segment(&input),
            vec![Segment::Esm(esm), Segment::Markdown("Markdown.\n")]
        );
        assert_eq!(segment_strict(&input).unwrap(), segment_spanned(&input));
    }
}

#[test]
fn esm_object_property_keywords_do_not_start_statement_blocks() {
    for keyword in ["else", "try", "finally", "do"] {
        for esm in [
            format!("export const value = {{ {keyword}: {{ a: 1 }} / 2 }}\n"),
            format!("export const value = {{ {keyword}, value: {{ a: 1 }} / 2 }}\n"),
            format!("export const value = {{ {keyword}() {{}} / 2 }}\n"),
            format!("export const value = {{ [{keyword}]: {{ a: 1 }} / 2 }}\n"),
            format!("export const value = {{ outer: {{ {keyword}: {{ a: 1 }} / 2 }} }}\n"),
        ] {
            let input = format!("{esm}Markdown.\n");
            assert_eq!(
                segment(&input),
                vec![Segment::Esm(&esm), Segment::Markdown("Markdown.\n")]
            );
            assert_eq!(segment_strict(&input).unwrap(), segment_spanned(&input));
            assert_eq!(render(&input).esm, vec![esm.as_str()]);
        }
    }
}

#[test]
fn esm_for_await_keeps_its_control_condition_context() {
    for esm in [
        "export async function iterate(items) { for await (const item of items) {} /\\}/.test(items) }\n",
        "export function iterate(items) { for (const item of items) {} /\\}/.test(items) }\n",
        "export async function pending(task) { return await task / divisor }\n",
        "export async function matches(value) { return await /\\}/.test(value) }\n",
    ] {
        let input = format!("{esm}Markdown.\n");
        assert_eq!(
            segment(&input),
            vec![Segment::Esm(esm), Segment::Markdown("Markdown.\n")]
        );
        assert_eq!(segment_strict(&input).unwrap(), segment_spanned(&input));
        assert_eq!(render(&input).esm, vec![esm]);
    }
}

#[test]
fn esm_braceless_control_bodies_preserve_regex_and_division_contexts() {
    for esm in [
        "export function nested(first, second) {\n  if (first)\n    if (second) /\\}/.test(second)\n    else /\\}/.test(first)\n  else /\\}/.test(second)\n}\n",
        "export function loops(items, value) { for (const item of items) /\\}/.test(item); while (value) /\\}/.test(value); with (items) /\\}/.test(items) }\n",
        "export function division(value, ratio, divisor) { if (value) ratio / divisor; else\n  ratio / divisor; do /\\}/.test(value) while (value) }\n",
    ] {
        let input = format!("{esm}Markdown.\n");
        assert_eq!(
            segment(&input),
            vec![Segment::Esm(esm), Segment::Markdown("Markdown.\n")]
        );
        assert_eq!(segment_strict(&input).unwrap(), segment_spanned(&input));
        assert_eq!(render(&input).esm, vec![esm]);
    }
}

#[test]
fn esm_braceless_bodies_do_not_leak_block_context_past_their_end() {
    for esm in [
        "export function after_else(value) { if (value) {} else value; ({ value: 1 }) / 2 }\n",
        "export function after_asi(value) { if (value) {} else\n  value\n  ({ value: 1 }) / 2 }\n",
        "export function after_nested_if(value) { if (value) if (value) value; ({ value: 1 }) / 2 }\n",
        "export function after_loop(value) { while (value) value; ({ value: 1 }) / 2 }\n",
        "export function after_do(value) { do value; ({ value: 1 }) / 2 while (value) }\n",
        "export function after_else_regex(value) { if (value) {} else /\\}/.test(value); ({ value: 1 }) / 2 }\n",
        "export function after_else_block(value) { if (value) {} else {} /\\}/.test(value) }\n",
    ] {
        let input = format!("{esm}Markdown.\n");
        assert_eq!(
            segment(&input),
            vec![Segment::Esm(esm), Segment::Markdown("Markdown.\n")]
        );
        assert_eq!(segment_strict(&input).unwrap(), segment_spanned(&input));
        assert_eq!(render(&input).esm, vec![esm]);
    }
}

#[test]
fn incomplete_esm_falls_back_to_markdown_and_is_strictly_diagnosed() {
    for input in [
        "import Widget\nOrdinary prose.\n",
        "export default\nSome prose.\n",
        "export const item = {\n# Heading\n",
        "export const total = 1 +\n- list item\n",
    ] {
        assert_eq!(segment(input), vec![Segment::Markdown(input)]);
        let diagnostics = segment_strict(input).unwrap_err();
        assert_eq!(diagnostics[0].code, MdxDiagnosticCode::IncompleteEsm);
    }
}

#[test]
fn long_multiline_esm_scan_remains_single_segment() {
    let mut input = String::from("export {\n");
    for index in 0..10_000 {
        input.push_str(&format!("  value_{index},\n"));
    }
    input.push_str("} from \"module\"\nMarkdown.\n");

    let segments = segment(&input);
    assert_eq!(segments.len(), 2);
    assert!(matches!(segments[0], Segment::Esm(_)));
    assert_eq!(
        segments[0].as_str().len() + segments[1].as_str().len(),
        input.len()
    );
}

#[test]
fn container_exit_keeps_following_markdown_in_the_paragraph() {
    for input in [
        "- ```jsx\n  <Card />\nordinary list exit\nimport A from 'a'\n",
        "> ```jsx\n> <Card />\nordinary quote exit\nexport { A }\n",
    ] {
        assert_eq!(segment(input), vec![Segment::Markdown(input)]);

        let output = render(input);
        assert!(output.esm.is_empty());
        assert!(output.body.contains("ordinary"));
        assert!(output.body.contains("import A from 'a'") || output.body.contains("export { A }"));

        let diagnostics = segment_strict(input).unwrap_err();
        assert_eq!(diagnostics[0].code, MdxDiagnosticCode::InvalidEsmPosition);
    }
}

#[test]
fn blockquote_fence_delimiter_still_allows_root_esm() {
    let input = "> ```jsx\n> <Card />\n> ```\nimport A from 'a'\n";

    assert_eq!(
        segment(input),
        vec![
            Segment::Markdown("> ```jsx\n> <Card />\n> ```\n"),
            Segment::Esm("import A from 'a'\n"),
        ]
    );
    assert_eq!(segment_strict(input).unwrap(), segment_spanned(input));

    let output = render(input);
    assert_eq!(output.esm, vec!["import A from 'a'\n"]);
    assert!(output.body.contains("&lt;Card /&gt;"));
}

#[test]
fn empty_container_fences_restore_root_mdx() {
    for fence in ["- ```\n  ```\n", "1. ~~~\n   ~~~\n", "> ```\n> ```\n"] {
        let input = format!("{fence}import A from 'a'\n<Live />\n{{value}}\n");

        assert_eq!(
            segment(&input),
            vec![
                Segment::Markdown(&input[..fence.len()]),
                Segment::Esm("import A from 'a'\n"),
                Segment::JsxBlockSelfClose("<Live />\n"),
                Segment::Expression("{value}\n"),
            ]
        );
        assert_eq!(segment_strict(&input).unwrap(), segment_spanned(&input));

        let output = render(&input);
        assert_eq!(output.esm, vec!["import A from 'a'\n"]);
        assert!(output.body.contains("<Live />"));
        assert!(output.body.contains("{value}"));
    }
}

#[test]
fn empty_container_fence_followed_by_paragraph_keeps_esm_nonflow() {
    for input in [
        "- ```\n  ```\nordinary list paragraph\nimport A from 'a'\n",
        "1. ~~~\n   ~~~\nordinary ordered paragraph\nexport { A }\n",
        "> ```\n> ```\nordinary quote paragraph\nimport A from 'a'\n",
    ] {
        assert_eq!(segment(input), vec![Segment::Markdown(input)]);

        let output = render(input);
        assert!(output.esm.is_empty());
        assert!(output.body.contains("ordinary"));

        let diagnostics = segment_strict(input).unwrap_err();
        assert_eq!(diagnostics[0].code, MdxDiagnosticCode::InvalidEsmPosition);
    }
}

#[test]
fn continuation_line_fences_restore_root_mdx_after_a_blank_exit() {
    for fence in [
        "- item\n  ```jsx\n  <Card />\n",
        "1. item\n   ~~~jsx\n   <Card />\n",
    ] {
        let input = format!("{fence}\nimport A from 'a'\n<Live />\n{{value}}\n");
        let markdown_end = fence.len() + 1;

        assert_eq!(
            segment(&input),
            vec![
                Segment::Markdown(&input[..markdown_end]),
                Segment::Esm("import A from 'a'\n"),
                Segment::JsxBlockSelfClose("<Live />\n"),
                Segment::Expression("{value}\n"),
            ]
        );
        assert_eq!(segment_strict(&input).unwrap(), segment_spanned(&input));

        let output = render(&input);
        assert_eq!(output.esm, vec!["import A from 'a'\n"]);
        assert!(output.body.contains("<Live />"));
        assert!(output.body.contains("{value}"));
    }
}

#[test]
fn continuation_line_fence_container_exit_keeps_esm_nonflow() {
    for input in [
        "- item\n  ```jsx\n  <Card />\nordinary list exit\nimport A from 'a'\n",
        "1. item\n   ~~~jsx\n   <Card />\nordinary ordered exit\nexport { A }\n",
    ] {
        assert_eq!(segment(input), vec![Segment::Markdown(input)]);
        assert!(render(input).esm.is_empty());

        let diagnostics = segment_strict(input).unwrap_err();
        assert_eq!(diagnostics[0].code, MdxDiagnosticCode::InvalidEsmPosition);
    }
}

#[test]
fn rejected_backtick_opener_does_not_shift_container_fence_boundaries() {
    for input in [
        "``` invalid ` info\n- ```jsx\n  <Card />\n  {value}\n  import A from 'a'\n  ```\n",
        "``` invalid ` info\n1. ~~~jsx\n   <Card />\n   {value}\n   export { value }\n   ~~~\n",
    ] {
        assert_eq!(segment(input), vec![Segment::Markdown(input)]);
        assert_eq!(segment_strict(input).unwrap(), segment_spanned(input));

        let output = render(input);
        assert!(output.esm.is_empty());
        assert!(output.body.contains("&lt;Card /&gt;"));
        assert!(!output.body.contains("\n<Card />\n"));
    }
}

#[test]
fn code_block_kind_fenced_pattern_remains_source_compatible() {
    let kind = CodeBlockKind::Fenced { info: None };
    assert!(matches!(kind, CodeBlockKind::Fenced { info: None }));
}

#[test]
fn container_exit_does_not_skip_a_new_root_fence() {
    let input = "- ```jsx\n  <Card />\n~~~\nimport A from 'a'\n";

    assert_eq!(segment(input), vec![Segment::Markdown(input)]);
    assert!(segment_strict(input).is_ok());

    let output = render(input);
    assert!(output.esm.is_empty());
    assert!(output.body.contains("import A from 'a'"));
}

#[test]
fn container_exit_does_not_treat_a_new_matching_root_fence_as_its_closer() {
    for input in [
        "- ```jsx\n  <Card />\n```\nimport A from 'a'\n```\n",
        "1. ~~~jsx\n   <Card />\n~~~\nexport { value }\n~~~\n",
    ] {
        assert_eq!(segment(input), vec![Segment::Markdown(input)]);
        assert_eq!(segment_strict(input).unwrap(), segment_spanned(input));

        let output = render(input);
        assert!(output.esm.is_empty());
        assert!(
            output.body.contains("import A from 'a'") || output.body.contains("export { value }")
        );
    }
}

// ── Strict diagnostics ──────────────────────────────────────────────

#[test]
fn strict_mode_returns_the_existing_spanned_segments_for_valid_mdx() {
    let input = "import A from 'a'\n\n<Card>\n{user.name}\n</Card>\n";

    assert_eq!(segment_strict(input).unwrap(), segment_spanned(input));
}

#[test]
fn strict_mode_keeps_permissive_segmentation_unchanged() {
    let input = "<Card bad=>\n";

    assert_eq!(segment(input), vec![Segment::JsxBlockOpen(input)]);
    assert_eq!(
        segment_strict(input).unwrap_err()[0].code,
        MdxDiagnosticCode::InvalidJsxTag
    );
}

#[test]
fn strict_mode_reports_unterminated_expression_with_its_source_range() {
    let input = "# Heading\n\n{user.name\n";
    let diagnostics = segment_strict(input).unwrap_err();

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].code,
        MdxDiagnosticCode::UnterminatedExpression
    );
    assert_eq!(
        diagnostics[0]
            .primary_range
            .slice_str(input.as_bytes())
            .unwrap(),
        "{user.name\n"
    );
    assert!(diagnostics[0].message.contains("expected `}`"));
}

#[test]
fn strict_mode_reports_unterminated_jsx_tag_with_its_source_range() {
    let input = "<Card title=\"example\"";
    let diagnostics = segment_strict(input).unwrap_err();

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, MdxDiagnosticCode::UnterminatedJsxTag);
    assert_eq!(
        diagnostics[0]
            .primary_range
            .slice_str(input.as_bytes())
            .unwrap(),
        input
    );
    assert!(diagnostics[0].message.contains("expected `>`"));
}

#[test]
fn strict_mode_accumulates_independent_diagnostics() {
    let input = "<Card bad=>\n\n{user.name\n";
    let diagnostics = segment_strict(input).unwrap_err();

    assert_eq!(
        diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>(),
        vec![
            MdxDiagnosticCode::InvalidJsxTag,
            MdxDiagnosticCode::UnterminatedExpression,
        ]
    );
}

#[test]
fn strict_mode_reports_mismatched_jsx_tags_with_related_opening_range() {
    let input = "<Outer>\n<Inner>\n</Outer>\n";
    let diagnostics = segment_strict(input).unwrap_err();

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].code,
        MdxDiagnosticCode::MismatchedJsxClosingTag
    );
    assert_eq!(
        diagnostics[0]
            .primary_range
            .slice_str(input.as_bytes())
            .unwrap(),
        "</Outer>"
    );
    assert_eq!(
        diagnostics[0]
            .related_range
            .unwrap()
            .slice_str(input.as_bytes())
            .unwrap(),
        "<Inner>"
    );
}

#[test]
fn strict_mode_reports_an_unmatched_jsx_closing_tag() {
    let input = "</Card>\n";
    let diagnostics = segment_strict(input).unwrap_err();

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].code,
        MdxDiagnosticCode::UnexpectedJsxClosingTag
    );
    assert_eq!(
        diagnostics[0]
            .primary_range
            .slice_str(input.as_bytes())
            .unwrap(),
        "</Card>"
    );
}

#[test]
fn strict_mode_reports_invalid_esm_boundaries() {
    let input = "Paragraph\nimport A from 'a'\n";
    let diagnostics = segment_strict(input).unwrap_err();

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, MdxDiagnosticCode::InvalidEsmPosition);
    assert_eq!(
        diagnostics[0]
            .primary_range
            .slice_str(input.as_bytes())
            .unwrap(),
        "import"
    );
}

#[test]
fn strict_mode_does_not_validate_javascript_inside_esm() {
    let input = "export const = ;\n";

    assert!(segment_strict(input).is_ok());
}

#[test]
fn strict_mode_diagnostics_use_utf8_byte_ranges_and_crlf_locations() {
    let input = "Grüß\r\n\r\n<Card bad=>\r\n";
    let diagnostics = segment_strict(input).unwrap_err();
    let diagnostic = &diagnostics[0];

    assert_eq!(
        diagnostic
            .primary_range
            .slice_str(input.as_bytes())
            .unwrap(),
        "<Card bad=>"
    );
    assert_eq!(
        source_location(input, diagnostic.primary_range.start_usize()),
        ferromark::mdx::SourceLocation { line: 3, column: 1 }
    );
}

// ── ESM ──────────────────────────────────────────────────────────────

#[test]
fn single_import() {
    let input = "import React from 'react'\n\n# Hello\n";
    let segs = segment(input);
    assert_eq!(segs[0], Segment::Esm("import React from 'react'\n"));
    assert_eq!(segs[1], Segment::Markdown("\n# Hello\n"));
}

#[test]
fn multiple_imports() {
    let input = "import A from 'a'\nimport B from 'b'\n\n# Title\n";
    let segs = segment(input);
    assert_eq!(segs[0], Segment::Esm("import A from 'a'\n"));
    assert_eq!(segs[1], Segment::Esm("import B from 'b'\n"));
    assert_eq!(segs[2], Segment::Markdown("\n# Title\n"));
}

#[test]
fn import_with_braces() {
    let input = "import { useState, useEffect } from 'react'\n\nContent\n";
    let segs = segment(input);
    assert_eq!(
        segs[0],
        Segment::Esm("import { useState, useEffect } from 'react'\n")
    );
}

#[test]
fn import_side_effect() {
    let input = "import './styles.css'\n\nContent\n";
    let segs = segment(input);
    assert_eq!(segs[0], Segment::Esm("import './styles.css'\n"));
}

#[test]
fn export_const() {
    let input = "export const meta = { title: 'hi' }\n\nText\n";
    let segs = segment(input);
    assert_eq!(
        segs[0],
        Segment::Esm("export const meta = { title: 'hi' }\n")
    );
    assert_eq!(segs[1], Segment::Markdown("\nText\n"));
}

#[test]
fn export_default() {
    let input = "export default function Layout({ children }) {\n  return children\n}\n\n# Page\n";
    let segs = segment(input);
    assert_eq!(
        segs[0],
        Segment::Esm("export default function Layout({ children }) {\n  return children\n}\n")
    );
    assert_eq!(segs[1], Segment::Markdown("\n# Page\n"));
}

#[test]
fn import_not_at_column_zero_is_markdown() {
    let input = "  import Foo from 'foo'\n";
    let segs = segment(input);
    assert_eq!(segs, vec![Segment::Markdown(input)]);
}

// ── JSX blocks ───────────────────────────────────────────────────────

#[test]
fn jsx_open_close_with_markdown() {
    let input = "<Wrapper>\n\n## Heading\n\n</Wrapper>\n";
    let segs = segment(input);
    assert_eq!(segs[0], Segment::JsxBlockOpen("<Wrapper>\n"));
    assert_eq!(segs[1], Segment::Markdown("\n## Heading\n\n"));
    assert_eq!(segs[2], Segment::JsxBlockClose("</Wrapper>\n"));
}

#[test]
fn jsx_self_closing() {
    let input = "<Image src=\"hero.png\" alt=\"Hero\" />\n";
    let segs = segment(input);
    assert_eq!(segs, vec![Segment::JsxBlockSelfClose(input)]);
}

#[test]
fn jsx_with_expression_attribute() {
    let input = "<Button onClick={() => alert('hi')}>\nClick\n</Button>\n";
    let segs = segment(input);
    assert_eq!(
        segs[0],
        Segment::JsxBlockOpen("<Button onClick={() => alert('hi')}>\n")
    );
    assert_eq!(segs[1], Segment::Markdown("Click\n"));
    assert_eq!(segs[2], Segment::JsxBlockClose("</Button>\n"));
}

#[test]
fn jsx_with_spread_props() {
    let input = "<Card {...props}>\nContent\n</Card>\n";
    let segs = segment(input);
    assert_eq!(segs[0], Segment::JsxBlockOpen("<Card {...props}>\n"));
    assert_eq!(segs[2], Segment::JsxBlockClose("</Card>\n"));
}

#[test]
fn jsx_member_expression() {
    let input = "<Layout.Header>\nTitle\n</Layout.Header>\n";
    let segs = segment(input);
    assert_eq!(segs[0], Segment::JsxBlockOpen("<Layout.Header>\n"));
    assert_eq!(segs[2], Segment::JsxBlockClose("</Layout.Header>\n"));
}

#[test]
fn nested_jsx_components() {
    let input = "<Outer>\n<Inner />\nContent\n</Outer>\n";
    let segs = segment(input);
    assert_eq!(segs[0], Segment::JsxBlockOpen("<Outer>\n"));
    assert_eq!(segs[1], Segment::JsxBlockSelfClose("<Inner />\n"));
    assert_eq!(segs[2], Segment::Markdown("Content\n"));
    assert_eq!(segs[3], Segment::JsxBlockClose("</Outer>\n"));
}

#[test]
fn deeply_nested_jsx() {
    let input = "<A>\n<B>\n<C />\ntext\n</B>\n</A>\n";
    let segs = segment(input);
    assert_eq!(segs[0], Segment::JsxBlockOpen("<A>\n"));
    assert_eq!(segs[1], Segment::JsxBlockOpen("<B>\n"));
    assert_eq!(segs[2], Segment::JsxBlockSelfClose("<C />\n"));
    assert_eq!(segs[3], Segment::Markdown("text\n"));
    assert_eq!(segs[4], Segment::JsxBlockClose("</B>\n"));
    assert_eq!(segs[5], Segment::JsxBlockClose("</A>\n"));
}

#[test]
fn fragment_open_close() {
    let input = "<>\nHello\n</>\n";
    let segs = segment(input);
    assert_eq!(segs[0], Segment::JsxBlockOpen("<>\n"));
    assert_eq!(segs[1], Segment::Markdown("Hello\n"));
    assert_eq!(segs[2], Segment::JsxBlockClose("</>\n"));
}

#[test]
fn jsx_with_boolean_attribute() {
    let input = "<Modal open>\nBody\n</Modal>\n";
    let segs = segment(input);
    assert_eq!(segs[0], Segment::JsxBlockOpen("<Modal open>\n"));
}

#[test]
fn jsx_with_multiple_attributes() {
    let input = "<Card variant=\"outlined\" size=\"lg\" onClick={handler}>\nContent\n</Card>\n";
    let segs = segment(input);
    assert_eq!(
        segs[0],
        Segment::JsxBlockOpen("<Card variant=\"outlined\" size=\"lg\" onClick={handler}>\n")
    );
}

// ── Expressions ──────────────────────────────────────────────────────

#[test]
fn simple_expression() {
    let input = "{variable}\n\nText\n";
    let segs = segment(input);
    assert_eq!(segs[0], Segment::Expression("{variable}\n"));
    assert_eq!(segs[1], Segment::Markdown("\nText\n"));
}

#[test]
fn expression_with_nested_braces() {
    let input = "{items.map(i => <li key={i}>{i}</li>)}\n";
    let segs = segment(input);
    assert_eq!(segs[0], Segment::Expression(input));
}

#[test]
fn expression_with_template_literal() {
    let input = "{`Hello ${name}`}\n";
    let segs = segment(input);
    assert_eq!(segs[0], Segment::Expression(input));
}

#[test]
fn expression_with_string_containing_brace() {
    let input = "{\"value: }\"}\n";
    let segs = segment(input);
    assert_eq!(segs[0], Segment::Expression(input));
}

#[test]
fn expression_with_comment() {
    let input = "{/* comment */ value}\n";
    let segs = segment(input);
    assert_eq!(segs[0], Segment::Expression(input));
}

// ── Real-world MDX patterns ─────────────────────────────────────────

#[test]
fn docusaurus_style_document() {
    let input = "\
import Tabs from '@theme/Tabs'
import TabItem from '@theme/TabItem'

# Installation

Install the package:

<Tabs>

<TabItem value=\"npm\">

```bash
npm install ferromark
```

</TabItem>

<TabItem value=\"yarn\">

```bash
yarn add ferromark
```

</TabItem>

</Tabs>
";
    let segs = segment(input);
    assert_eq!(segs[0], Segment::Esm("import Tabs from '@theme/Tabs'\n"));
    assert_eq!(
        segs[1],
        Segment::Esm("import TabItem from '@theme/TabItem'\n")
    );
    // Markdown: "# Installation\n\nInstall the package:\n\n"
    assert!(matches!(segs[2], Segment::Markdown(_)));

    // Count JSX open/close pairs
    let opens: Vec<_> = segs
        .iter()
        .filter(|s| matches!(s, Segment::JsxBlockOpen(_)))
        .collect();
    let closes: Vec<_> = segs
        .iter()
        .filter(|s| matches!(s, Segment::JsxBlockClose(_)))
        .collect();
    assert_eq!(opens.len(), 3); // Tabs + 2x TabItem
    assert_eq!(closes.len(), 3);
}

#[test]
fn nextjs_mdx_page() {
    let input = "\
import { Card } from '../components/card'
export const metadata = { title: 'Blog Post' }

# My Blog Post

Some introductory paragraph with **bold** and *italic*.

<Card>

## Featured Content

This is rendered as Markdown inside a React component.

- Item one
- Item two
- Item three

</Card>

## Conclusion

Thanks for reading!
";
    let segs = segment(input);

    // Verify ESM at the top
    assert!(matches!(segs[0], Segment::Esm(_)));
    assert!(matches!(segs[1], Segment::Esm(_)));

    // Find the Card open/close
    let card_open = segs
        .iter()
        .position(|s| matches!(s, Segment::JsxBlockOpen(t) if t.contains("Card")))
        .expect("should find <Card>");
    let card_close = segs
        .iter()
        .position(|s| matches!(s, Segment::JsxBlockClose(t) if t.contains("Card")))
        .expect("should find </Card>");

    // Markdown between Card tags should contain the heading and list
    let inner = &segs[card_open + 1..card_close];
    let inner_md: String = inner
        .iter()
        .filter_map(|s| match s {
            Segment::Markdown(m) => Some(*m),
            _ => None,
        })
        .collect();
    assert!(inner_md.contains("## Featured Content"));
    assert!(inner_md.contains("- Item one"));

    // Markdown after Card should contain conclusion
    let after: String = segs[card_close + 1..]
        .iter()
        .filter_map(|s| match s {
            Segment::Markdown(m) => Some(*m),
            _ => None,
        })
        .collect();
    assert!(after.contains("## Conclusion"));
}

#[test]
fn mixed_self_closing_and_block_components() {
    let input = "\
<Banner src=\"/hero.png\" />

# Welcome

<Alert type=\"info\">

Please read the docs carefully.

</Alert>

<Divider />

## Next Steps

Continue to the tutorial.
";
    let segs = segment(input);

    let self_closes: Vec<_> = segs
        .iter()
        .filter(|s| matches!(s, Segment::JsxBlockSelfClose(_)))
        .collect();
    assert_eq!(self_closes.len(), 2); // Banner + Divider

    let opens: Vec<_> = segs
        .iter()
        .filter(|s| matches!(s, Segment::JsxBlockOpen(_)))
        .collect();
    assert_eq!(opens.len(), 1); // Alert
}

#[test]
fn end_to_end_markdown_rendering() {
    let input = "\
import { Box } from './box'

# Title

Paragraph with **bold**.

<Box>

## Inside Box

- one
- two

</Box>
";
    let segs = segment(input);

    // Collect and render only Markdown segments
    let mut html_parts = Vec::new();
    for seg in &segs {
        if let Segment::Markdown(md) = seg {
            let html = ferromark::to_html(md);
            if !html.trim().is_empty() {
                html_parts.push(html);
            }
        }
    }

    let combined = html_parts.join("");
    assert!(combined.contains("<h1"), "should contain h1");
    assert!(
        combined.contains("<strong>bold</strong>"),
        "should render bold"
    );
    assert!(
        combined.contains("<h2"),
        "should contain h2 from inside Box"
    );
    assert!(
        combined.contains("<li>one</li>"),
        "should render list items"
    );
}

// ── Ported from mdxjs: ESM edge cases ────────────────────────────────
//
// Based on test cases from:
// - micromark-extension-mdxjs-esm
// - markdown-rs tests/mdx_esm.rs

#[test]
fn esm_import_default() {
    let segs = segment("import a from \"b\"\n\nc\n");
    assert_eq!(segs[0], Segment::Esm("import a from \"b\"\n"));
}

#[test]
fn esm_import_namespace() {
    let segs = segment("import * as a from \"b\"\n\nc\n");
    assert_eq!(segs[0], Segment::Esm("import * as a from \"b\"\n"));
}

#[test]
fn esm_import_destructured() {
    let segs = segment("import {a} from \"b\"\n\nc\n");
    assert_eq!(segs[0], Segment::Esm("import {a} from \"b\"\n"));
}

#[test]
fn esm_import_destructured_renamed() {
    let segs = segment("import {a as b} from \"c\"\n\nc\n");
    assert_eq!(segs[0], Segment::Esm("import {a as b} from \"c\"\n"));
}

#[test]
fn esm_import_side_effect_double_quote() {
    let segs = segment("import \"a\"\n\nc\n");
    assert_eq!(segs[0], Segment::Esm("import \"a\"\n"));
}

#[test]
fn esm_import_side_effect_single_quote() {
    let segs = segment("import 'a'\n\nc\n");
    assert_eq!(segs[0], Segment::Esm("import 'a'\n"));
}

#[test]
fn esm_export_var() {
    let segs = segment("export var a = \"\"\n\nb\n");
    assert_eq!(segs[0], Segment::Esm("export var a = \"\"\n"));
}

#[test]
fn esm_export_const() {
    let segs = segment("export const a = \"\"\n\nb\n");
    assert_eq!(segs[0], Segment::Esm("export const a = \"\"\n"));
}

#[test]
fn esm_export_let() {
    let segs = segment("export let a = \"\"\n\nb\n");
    assert_eq!(segs[0], Segment::Esm("export let a = \"\"\n"));
}

#[test]
fn esm_export_default() {
    let segs = segment("export default a = 1\n\nb\n");
    assert_eq!(segs[0], Segment::Esm("export default a = 1\n"));
}

#[test]
fn esm_export_function() {
    let segs = segment("export function a() {}\n\nb\n");
    assert_eq!(segs[0], Segment::Esm("export function a() {}\n"));
}

#[test]
fn esm_export_class() {
    let segs = segment("export class a {}\n\nb\n");
    assert_eq!(segs[0], Segment::Esm("export class a {}\n"));
}

#[test]
fn esm_export_from() {
    let segs = segment("export {a} from \"b\"\n\nc\n");
    assert_eq!(segs[0], Segment::Esm("export {a} from \"b\"\n"));
}

#[test]
fn esm_export_star_from() {
    let segs = segment("export * from \"a\"\n\nb\n");
    assert_eq!(segs[0], Segment::Esm("export * from \"a\"\n"));
}

#[test]
fn esm_export_star_as_from() {
    let segs = segment("export * as a from \"b\"\n\nc\n");
    assert_eq!(segs[0], Segment::Esm("export * as a from \"b\"\n"));
}

#[test]
fn esm_export_multiline() {
    let segs = segment("export {\n  a\n} from \"b\"\n\nc\n");
    assert_eq!(segs[0], Segment::Esm("export {\n  a\n} from \"b\"\n"));
}

#[test]
fn esm_two_imports_consecutive() {
    let segs = segment("import a from \"b\"\nimport c from \"d\"\n\ne\n");
    assert_eq!(segs[0], Segment::Esm("import a from \"b\"\n"));
    assert_eq!(segs[1], Segment::Esm("import c from \"d\"\n"));
}

#[test]
fn esm_import_then_export() {
    let segs = segment("import a from \"b\"\n\nexport default c\n\nd\n");
    assert_eq!(segs[0], Segment::Esm("import a from \"b\"\n"));
    assert!(matches!(segs[1], Segment::Markdown(_)));
    assert_eq!(segs[2], Segment::Esm("export default c\n"));
}

// ── Ported from mdxjs: NOT ESM (false positive protection) ──────────

#[test]
fn not_esm_word_starting_with_im() {
    // "impossible" starts with "im" but is not `import `
    let segs = segment("impossible\n");
    assert_eq!(segs, vec![Segment::Markdown("impossible\n")]);
}

#[test]
fn not_esm_word_starting_with_export() {
    // "exporting" starts with "export" but is not `export `
    let segs = segment("exporting\n");
    assert_eq!(segs, vec![Segment::Markdown("exporting\n")]);
}

#[test]
fn not_esm_import_dot() {
    // import.meta.url is property access
    let segs = segment("import.meta.url\n");
    assert_eq!(segs, vec![Segment::Markdown("import.meta.url\n")]);
}

#[test]
fn not_esm_dynamic_import_parens() {
    // import("a") is a dynamic import call
    let segs = segment("import(\"a\")\n");
    assert_eq!(segs, vec![Segment::Markdown("import(\"a\")\n")]);
}

#[test]
fn not_esm_dynamic_import_space_parens() {
    // import ('a') is also a dynamic import call
    let segs = segment("import ('a')\n");
    assert_eq!(segs, vec![Segment::Markdown("import ('a')\n")]);
}

#[test]
fn not_esm_indented_import() {
    let segs = segment("  import a from \"b\"\n");
    assert_eq!(segs, vec![Segment::Markdown("  import a from \"b\"\n")]);
}

#[test]
fn not_esm_interrupts_paragraph() {
    // ESM cannot interrupt a paragraph — requires blank line
    let segs = segment("a\nimport a from \"b\"\n");
    assert_eq!(segs.len(), 1);
    assert_eq!(segs[0], Segment::Markdown("a\nimport a from \"b\"\n"));
}

#[test]
fn not_esm_export_interrupts_paragraph() {
    let segs = segment("a\nexport default c\n");
    assert_eq!(segs.len(), 1);
    assert_eq!(segs[0], Segment::Markdown("a\nexport default c\n"));
}

#[test]
fn esm_valid_after_blank_line() {
    // After blank line, ESM detection works again
    let segs = segment("a\n\nimport a from \"b\"\n\nc\n");
    assert!(matches!(segs[0], Segment::Markdown(_)));
    assert_eq!(segs[1], Segment::Esm("import a from \"b\"\n"));
}

// ── Ported from mdxjs: JSX flow vs text ──────────────────────────────
//
// Based on test cases from:
// - micromark-extension-mdx-jsx (flow sections)
// - markdown-rs tests/mdx_jsx_flow.rs

#[test]
fn jsx_flow_self_closing() {
    let segs = segment("<a />\n");
    assert_eq!(segs[0], Segment::JsxBlockSelfClose("<a />\n"));
}

#[test]
fn jsx_flow_with_leading_spaces() {
    let segs = segment("   <a />\n");
    assert_eq!(segs[0], Segment::JsxBlockSelfClose("   <a />\n"));
}

#[test]
fn jsx_flow_open_content_close() {
    let segs = segment("<a>\nb\n</a>\n");
    assert_eq!(segs[0], Segment::JsxBlockOpen("<a>\n"));
    assert_eq!(segs[1], Segment::Markdown("b\n"));
    assert_eq!(segs[2], Segment::JsxBlockClose("</a>\n"));
}

#[test]
fn jsx_flow_with_list_content() {
    let segs = segment("<a>\n- b\n</a>\n");
    assert_eq!(segs[0], Segment::JsxBlockOpen("<a>\n"));
    assert_eq!(segs[1], Segment::Markdown("- b\n"));
    assert_eq!(segs[2], Segment::JsxBlockClose("</a>\n"));
}

#[test]
fn jsx_flow_with_all_attribute_types() {
    let segs = segment("<a b c:d e=\"\" f={/* g */} {...h} />\n");
    assert_eq!(
        segs[0],
        Segment::JsxBlockSelfClose("<a b c:d e=\"\" f={/* g */} {...h} />\n")
    );
}

#[test]
fn jsx_flow_fragment() {
    let segs = segment("<>\nb\n</>\n");
    assert_eq!(segs[0], Segment::JsxBlockOpen("<>\n"));
    assert_eq!(segs[1], Segment::Markdown("b\n"));
    assert_eq!(segs[2], Segment::JsxBlockClose("</>\n"));
}

#[test]
fn jsx_not_flow_trailing_period() {
    // Trailing non-whitespace after tag → entire line is text/markdown
    let segs = segment("<x />.\n");
    assert_eq!(segs, vec![Segment::Markdown("<x />.\n")]);
}

#[test]
fn jsx_not_flow_leading_text() {
    // Text before tag → inline, not flow
    let segs = segment("a <x />\n");
    assert_eq!(segs, vec![Segment::Markdown("a <x />\n")]);
}

#[test]
fn jsx_not_flow_close_trailing_period() {
    let segs = segment("</a>.\n");
    assert_eq!(segs, vec![Segment::Markdown("</a>.\n")]);
}

// ── Ported from mdxjs: Expression flow vs text ───────────────────────
//
// Based on test cases from:
// - micromark-extension-mdx-expression (flow sections)
// - markdown-rs tests/mdx_expression_flow.rs

#[test]
fn expr_flow_simple() {
    let segs = segment("{a}\n");
    assert_eq!(segs[0], Segment::Expression("{a}\n"));
}

#[test]
fn expr_flow_empty() {
    let segs = segment("{}\n");
    assert_eq!(segs[0], Segment::Expression("{}\n"));
}

#[test]
fn expr_flow_multiline() {
    let segs = segment("{\n}\n");
    assert_eq!(segs[0], Segment::Expression("{\n}\n"));
}

#[test]
fn expr_flow_with_trailing_spaces() {
    let segs = segment("{ a } \t\n");
    assert_eq!(segs[0], Segment::Expression("{ a } \t\n"));
}

#[test]
fn expr_flow_with_leading_spaces() {
    let segs = segment("  { a }\n");
    assert_eq!(segs[0], Segment::Expression("  { a }\n"));
}

#[test]
fn expr_flow_nested_braces() {
    let segs = segment("{b { c }}\n");
    assert_eq!(segs[0], Segment::Expression("{b { c }}\n"));
}

#[test]
fn expr_flow_comment_only() {
    let segs = segment("{/**/}\n");
    assert_eq!(segs[0], Segment::Expression("{/**/}\n"));
}

#[test]
fn expr_flow_line_comment() {
    let segs = segment("{//\n}\n");
    assert_eq!(segs[0], Segment::Expression("{//\n}\n"));
}

#[test]
fn expr_flow_complex_multiline() {
    let segs = segment("{\n  1 + 1\n}\n\n# heading\n");
    assert_eq!(segs[0], Segment::Expression("{\n  1 + 1\n}\n"));
    assert!(matches!(segs[1], Segment::Markdown(_)));
}

#[test]
fn expr_not_flow_trailing_text() {
    // Trailing text after expression → markdown, not flow
    let segs = segment("{ a } b\n");
    assert_eq!(segs, vec![Segment::Markdown("{ a } b\n")]);
}

#[test]
fn expr_not_flow_in_paragraph() {
    let segs = segment("a {b} c\n");
    assert_eq!(segs, vec![Segment::Markdown("a {b} c\n")]);
}

// ── Ported from mdxjs: Interleaving ──────────────────────────────────

#[test]
fn interleave_jsx_and_expression() {
    let segs = segment("<div>\n{asd}\n</div>\n");
    assert_eq!(segs[0], Segment::JsxBlockOpen("<div>\n"));
    assert_eq!(segs[1], Segment::Expression("{asd}\n"));
    assert_eq!(segs[2], Segment::JsxBlockClose("</div>\n"));
}

#[test]
fn interleave_markdown_esm_markdown() {
    let segs = segment("a\n\nimport a from \"b\"\n\nc\n");
    assert!(matches!(segs[0], Segment::Markdown(_)));
    assert_eq!(segs[1], Segment::Esm("import a from \"b\"\n"));
    assert!(matches!(segs[2], Segment::Markdown(_)));
}

#[test]
fn interleave_esm_then_jsx_with_expression() {
    let input = "import {Pill} from \"./comp.js\"\n\n<Pill>\n{1}\n</Pill>\n";
    let segs = segment(input);
    assert_eq!(segs[0], Segment::Esm("import {Pill} from \"./comp.js\"\n"));
    assert!(matches!(segs[1], Segment::Markdown(_))); // blank line
    assert!(matches!(segs[2], Segment::JsxBlockOpen(_)));
    assert_eq!(segs[3], Segment::Expression("{1}\n"));
    assert!(matches!(segs[4], Segment::JsxBlockClose(_)));
}

// ── Defensive: invalid constructs become markdown ────────────────────

#[test]
fn invalid_jsx_becomes_markdown() {
    let input = "< not-jsx\n";
    let segs = segment(input);
    assert_eq!(segs, vec![Segment::Markdown(input)]);
}

#[test]
fn unterminated_expression_becomes_markdown() {
    let input = "{unclosed\n";
    let segs = segment(input);
    assert_eq!(segs, vec![Segment::Markdown(input)]);
}

#[test]
fn less_than_in_prose_stays_markdown() {
    let input = "Use x < 5 in your code.\n";
    let segs = segment(input);
    assert_eq!(segs, vec![Segment::Markdown(input)]);
}

#[test]
fn number_after_less_than_stays_markdown() {
    let input = "<5 is a comparison\n";
    let segs = segment(input);
    assert_eq!(segs, vec![Segment::Markdown(input)]);
}

#[test]
fn bare_brace_in_prose_stays_markdown() {
    let input = "Use { and } in code.\n";
    let segs = segment(input);
    assert_eq!(segs, vec![Segment::Markdown(input)]);
}

#[test]
fn html_entities_not_confused_with_jsx() {
    let input = "&lt;div&gt; is not JSX.\n";
    let segs = segment(input);
    assert_eq!(segs, vec![Segment::Markdown(input)]);
}

// ── Invariants ───────────────────────────────────────────────────────

#[test]
fn all_segments_are_slices_of_input() {
    let input = "import X from 'x'\n\n# Title\n\n<Box>\nHi\n</Box>\n{expr}\n";
    let segs = segment(input);
    let input_range = input.as_ptr() as usize..input.as_ptr() as usize + input.len();
    for seg in &segs {
        let s = seg_str(seg);
        let ptr = s.as_ptr() as usize;
        assert!(
            input_range.contains(&ptr),
            "segment {:?} is not a slice of the original input",
            seg
        );
    }
}

#[test]
fn segments_cover_full_input() {
    let input = "import A from 'a'\n\n# Hello\n\n<Foo>\nbar\n</Foo>\n\n{x}\n";
    let segs = segment(input);
    let total: usize = segs.iter().map(|s| seg_str(s).len()).sum();
    assert_eq!(total, input.len(), "segments don't cover the full input");
}

#[test]
fn segments_are_contiguous() {
    let input = "import X from 'x'\n\n# Title\n\n<Box>\nHi\n</Box>\n{y}\n";
    let segs = segment(input);

    let base = input.as_ptr() as usize;
    let mut expected_offset = 0;
    for seg in &segs {
        let s = seg_str(seg);
        let actual_offset = s.as_ptr() as usize - base;
        assert_eq!(
            actual_offset, expected_offset,
            "gap or overlap between segments at offset {expected_offset}"
        );
        expected_offset += s.len();
    }
    assert_eq!(expected_offset, input.len());
}

#[test]
fn no_empty_segments() {
    let inputs = &[
        "# Hello\n",
        "import A from 'a'\n\n# B\n",
        "<Foo>\nbar\n</Foo>\n",
        "{x}\n",
        "<A />\n",
        "",
    ];
    for input in inputs {
        let segs = segment(input);
        for seg in &segs {
            assert!(
                !seg_str(seg).is_empty(),
                "empty segment in input: {:?}",
                input
            );
        }
    }
}

// ── Edge cases ───────────────────────────────────────────────────────

#[test]
fn input_without_trailing_newline() {
    let input = "# Hello";
    let segs = segment(input);
    assert_eq!(segs, vec![Segment::Markdown("# Hello")]);
}

#[test]
fn jsx_without_trailing_newline() {
    let input = "<Foo />";
    let segs = segment(input);
    assert_eq!(segs, vec![Segment::JsxBlockSelfClose("<Foo />")]);
}

#[test]
fn consecutive_jsx_blocks() {
    let input = "<A />\n<B />\n<C />\n";
    let segs = segment(input);
    assert_eq!(segs.len(), 3);
    assert!(
        segs.iter()
            .all(|s| matches!(s, Segment::JsxBlockSelfClose(_)))
    );
}

#[test]
fn consecutive_expressions() {
    let input = "{a}\n{b}\n{c}\n";
    let segs = segment(input);
    assert_eq!(segs.len(), 3);
    assert!(segs.iter().all(|s| matches!(s, Segment::Expression(_))));
}

#[test]
fn markdown_between_expressions() {
    let input = "{a}\n\nSome text\n\n{b}\n";
    let segs = segment(input);
    assert_eq!(segs[0], Segment::Expression("{a}\n"));
    assert!(matches!(segs[1], Segment::Markdown(_)));
    assert_eq!(segs[2], Segment::Expression("{b}\n"));
}

// ── render() integration tests ───────────────────────────────────────

#[test]
fn render_complete_document() {
    let input = "\
import { Card } from './card'
export const meta = { title: 'Test' }

# Title

Paragraph with **bold**.

<Card>

## Inside Card

- one
- two

</Card>

{new Date().getFullYear()}
";
    let out = render(input);

    // ESM extracted
    assert_eq!(out.esm.len(), 2);
    assert!(out.esm[0].contains("import { Card }"));
    assert!(out.esm[1].contains("export const meta"));

    // Body contains rendered markdown
    assert!(out.body.contains("<h1"), "should render h1");
    assert!(
        out.body.contains("<strong>bold</strong>"),
        "should render bold"
    );
    assert!(out.body.contains("<h2"), "should render h2 inside Card");
    assert!(out.body.contains("<li>one</li>"), "should render list");

    // Body contains JSX passthrough
    assert!(out.body.contains("<Card>"), "should pass through JSX open");
    assert!(
        out.body.contains("</Card>"),
        "should pass through JSX close"
    );

    // Body contains expression passthrough
    assert!(
        out.body.contains("new Date().getFullYear()"),
        "should pass through expression"
    );

    // No front matter
    assert!(out.front_matter.is_none());
}

#[test]
fn render_web_component_inline() {
    let input = "Text with <sl-button>Click</sl-button> here.\n";
    let out = render(input);
    assert!(
        out.body.contains("<sl-button>Click</sl-button>"),
        "inline HTML should pass through"
    );
}

#[test]
fn render_front_matter() {
    let input = "---\ntitle: Hello\nauthor: World\n---\n\n# Heading\n";
    let out = render(input);
    assert_eq!(out.front_matter, Some("title: Hello\nauthor: World\n"));
    assert!(out.body.contains("<h1"));
    assert!(out.body.contains("Heading"));
}

#[test]
fn render_only_markdown() {
    let input = "# Hello\n\nWorld\n";
    let out = render(input);
    assert!(out.body.contains("<h1"));
    assert!(out.body.contains("<p>World</p>"));
    assert!(out.esm.is_empty());
}

#[test]
fn render_only_esm() {
    let input = "import A from 'a'\nimport B from 'b'\n";
    let out = render(input);
    assert_eq!(out.esm.len(), 2);
    assert!(!out.body.contains('<'));
}

#[test]
fn render_docusaurus_style() {
    let input = "\
import Tabs from '@theme/Tabs'
import TabItem from '@theme/TabItem'

# Installation

Install the package:

<Tabs>

<TabItem value=\"npm\">

```bash
npm install ferromark
```

</TabItem>

<TabItem value=\"yarn\">

```bash
yarn add ferromark
```

</TabItem>

</Tabs>
";
    let out = render(input);

    // ESM
    assert_eq!(out.esm.len(), 2);
    assert!(out.esm[0].contains("Tabs"));
    assert!(out.esm[1].contains("TabItem"));

    // Rendered markdown
    assert!(out.body.contains("<h1"), "should have h1");
    assert!(out.body.contains("<code"), "should have code blocks");

    // JSX passthrough
    assert!(out.body.contains("<Tabs>"));
    assert!(out.body.contains("</Tabs>"));
    assert!(out.body.contains("<TabItem value=\"npm\">"));
    assert!(out.body.contains("</TabItem>"));
}

#[test]
fn render_with_custom_options() {
    let input = "# Heading\n\n~~struck~~\n";
    let opts = ferromark::options!(Options::default();
        strikethrough: true,
        heading_ids: false,
        allow_html: true,
        disallowed_raw_html: false,);
    let out = render_with_options(input, &opts);
    assert!(out.body.contains("<del>struck</del>"));
    assert!(!out.body.contains("id="));
}

#[test]
fn render_empty_input() {
    let out = render("");
    assert!(out.body.is_empty());
    assert!(out.esm.is_empty());
    assert!(out.front_matter.is_none());
}

// ── to_component() integration tests ─────────────────────────────────

#[test]
fn component_complete_document() {
    let input = "\
import { Card } from './card'
export const meta = { title: 'Test' }

# About

Welcome to **about**.

<Card>

## Details

</Card>

{new Date().getFullYear()}
";
    let out = render(input);
    let comp = out.to_component("About").unwrap();

    // Starts with ESM
    assert!(comp.starts_with("import { Card } from './card'\n"));
    assert!(comp.contains("export const meta = { title: 'Test' }\n"));

    // Named export function
    assert!(comp.contains("export function About() {"));
    assert!(!comp.contains("default"));

    // Fragment wrapper
    assert!(comp.contains("    <>"));
    assert!(comp.contains("    </>"));

    // Body content indented
    assert!(comp.contains("      <h1"));
    assert!(comp.contains("      <Card>"));
    assert!(comp.contains("      </Card>"));
    assert!(comp.contains("      {new Date().getFullYear()}"));

    // Ends cleanly
    assert!(comp.ends_with("  );\n}\n"));
}

#[test]
fn component_pure_markdown() {
    let input = "# Hello\n\nWorld\n";
    let out = render(input);
    let comp = out.to_component("Page").unwrap();

    // No ESM, starts with export
    assert!(comp.starts_with("export function Page() {"));
    assert!(comp.contains("      <h1"));
    assert!(comp.contains("      <p>World</p>"));
}

#[test]
fn component_docusaurus_style() {
    let input = "\
import Tabs from '@theme/Tabs'
import TabItem from '@theme/TabItem'

# Install

<Tabs>

<TabItem value=\"npm\">

```bash
npm install ferromark
```

</TabItem>

</Tabs>
";
    let out = render(input);
    let comp = out.to_component("Install").unwrap();

    assert!(comp.contains("import Tabs from '@theme/Tabs'"));
    assert!(comp.contains("import TabItem from '@theme/TabItem'"));
    assert!(comp.contains("export function Install() {"));
    assert!(comp.contains("      <Tabs>"));
    assert!(comp.contains("      <TabItem value=\"npm\">"));
    assert!(comp.contains("      </TabItem>"));
    assert!(comp.contains("      </Tabs>"));
}
