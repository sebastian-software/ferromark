use super::Segment;
use super::expr::find_expression_end;
use super::jsx_tag::parse_jsx_tag;

/// Split MDX input into typed segments.
///
/// The splitter is a line-based state machine that categorises each region of
/// the input as one of: ESM (`import`/`export`), JSX block tag, expression, or
/// Markdown.  Only block-level constructs are detected — inline JSX inside
/// paragraphs is left for the Markdown parser.
///
/// The returned `Vec<Segment>` covers the entire input (no bytes are dropped).
pub fn split(input: &str) -> Vec<Segment<'_>> {
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut segments: Vec<Segment<'_>> = Vec::new();
    let mut pos = 0;
    let mut md_start: Option<usize> = None;
    let mut tag_stack: Vec<String> = Vec::new();
    // Track whether the previous line was non-blank markdown content.
    // ESM cannot interrupt a paragraph (requires blank line before it).
    let mut in_paragraph = false;

    while pos < len {
        let line_start = pos;
        let first_non_ws = skip_whitespace_offset(bytes, pos);

        if first_non_ws >= len {
            // Remaining is whitespace-only — treat as markdown
            extend_markdown(&mut md_start, line_start);
            break;
        }

        let first = bytes[first_non_ws];

        // Detect blank lines (only newline after whitespace) — reset paragraph state
        if first == b'\n' || first == b'\r' {
            in_paragraph = false;
            extend_markdown(&mut md_start, line_start);
            pos = next_line(bytes, pos);
            continue;
        }

        // 1. Closing tag: `</`
        if first == b'<' && first_non_ws + 1 < len && bytes[first_non_ws + 1] == b'/' {
            if let Some(tag_info) = parse_jsx_tag(&bytes[first_non_ws..]) {
                if tag_info.is_closing {
                    let end = first_non_ws + tag_info.end_offset;
                    // Flow JSX requires no trailing non-whitespace content on the line
                    if has_trailing_content(bytes, end) {
                        // Fall through to markdown
                    } else {
                        flush_markdown(input, &mut md_start, line_start, &mut segments);
                        let seg_end = consume_trailing_newline(bytes, end);
                        segments.push(Segment::JsxBlockClose(&input[line_start..seg_end]));
                        if !tag_info.name.is_empty() {
                            if let Some(top_pos) = tag_stack
                                .iter()
                                .rposition(|n| n == tag_info.name)
                            {
                                tag_stack.remove(top_pos);
                            }
                        }
                        pos = seg_end;
                        in_paragraph = false;
                        continue;
                    }
                }
            }
            // Fall through to markdown
        }

        // 2. ESM: `import ` or `export ` at column 0, not interrupting a paragraph
        if pos == first_non_ws && !in_paragraph {
            if let Some(esm_end) = try_esm(bytes, pos) {
                flush_markdown(input, &mut md_start, line_start, &mut segments);
                segments.push(Segment::Esm(&input[pos..esm_end]));
                pos = esm_end;
                in_paragraph = false;
                continue;
            }
        }

        // 3. Expression: `{` as first non-whitespace
        if first == b'{' {
            if let Some(expr_len) = find_expression_end(&bytes[first_non_ws..]) {
                let end = first_non_ws + expr_len;
                // Flow expression requires no trailing non-whitespace content
                if !has_trailing_content(bytes, end) {
                    flush_markdown(input, &mut md_start, line_start, &mut segments);
                    let seg_end = consume_trailing_newline(bytes, end);
                    segments.push(Segment::Expression(&input[line_start..seg_end]));
                    pos = seg_end;
                    in_paragraph = false;
                    continue;
                }
                // Trailing content → treat as markdown
            }
            // Unterminated expression → treat as markdown
        }

        // 4. JSX opening/self-closing tag: `<` followed by letter or `>`
        if first == b'<'
            && first_non_ws + 1 < len
            && (bytes[first_non_ws + 1].is_ascii_alphabetic()
                || bytes[first_non_ws + 1] == b'>')
        {
            if let Some(tag_info) = parse_jsx_tag(&bytes[first_non_ws..]) {
                let end = first_non_ws + tag_info.end_offset;
                // Flow JSX requires no trailing non-whitespace content on the line
                if !has_trailing_content(bytes, end) {
                    flush_markdown(input, &mut md_start, line_start, &mut segments);
                    let seg_end = consume_trailing_newline(bytes, end);
                    let slice = &input[line_start..seg_end];
                    if tag_info.is_self_closing {
                        segments.push(Segment::JsxBlockSelfClose(slice));
                    } else {
                        if !tag_info.name.is_empty() {
                            tag_stack.push(tag_info.name.to_string());
                        }
                        segments.push(Segment::JsxBlockOpen(slice));
                    }
                    pos = seg_end;
                    in_paragraph = false;
                    continue;
                }
                // Trailing content → treat as markdown
            }
            // Invalid JSX → fall through to markdown
        }

        // 5. Otherwise → Markdown
        extend_markdown(&mut md_start, line_start);
        in_paragraph = true;
        pos = next_line(bytes, pos);
    }

    // Flush any remaining markdown
    if let Some(start) = md_start {
        if start < len {
            segments.push(Segment::Markdown(&input[start..len]));
        }
    }

    segments
}

/// If we're already accumulating markdown, do nothing.
/// Otherwise, mark `pos` as the start of a new markdown region.
fn extend_markdown(md_start: &mut Option<usize>, pos: usize) {
    if md_start.is_none() {
        *md_start = Some(pos);
    }
}

/// Flush accumulated markdown into `segments` and reset the accumulator.
fn flush_markdown<'a>(
    input: &'a str,
    md_start: &mut Option<usize>,
    current_pos: usize,
    segments: &mut Vec<Segment<'a>>,
) {
    if let Some(start) = md_start.take() {
        if start < current_pos {
            segments.push(Segment::Markdown(&input[start..current_pos]));
        }
    }
}

/// Advance past the current line (past `\n` or to EOF).
fn next_line(bytes: &[u8], mut pos: usize) -> usize {
    let len = bytes.len();
    while pos < len && bytes[pos] != b'\n' {
        pos += 1;
    }
    if pos < len {
        pos + 1 // skip `\n`
    } else {
        len
    }
}

/// Return the offset of the first non-whitespace byte at or after `pos`.
fn skip_whitespace_offset(bytes: &[u8], mut pos: usize) -> usize {
    let len = bytes.len();
    while pos < len && (bytes[pos] == b' ' || bytes[pos] == b'\t') {
        pos += 1;
    }
    pos
}

/// Check if there is non-whitespace content between `pos` and the end of the
/// current line. Used to distinguish flow (block) constructs from inline ones:
/// `<x />` is flow, but `<x />.` has trailing content and is text/inline.
fn has_trailing_content(bytes: &[u8], mut pos: usize) -> bool {
    let len = bytes.len();
    while pos < len && (bytes[pos] == b' ' || bytes[pos] == b'\t') {
        pos += 1;
    }
    pos < len && bytes[pos] != b'\n' && bytes[pos] != b'\r'
}

/// Consume optional trailing whitespace + a single newline after a construct.
fn consume_trailing_newline(bytes: &[u8], mut pos: usize) -> usize {
    let len = bytes.len();
    // Skip trailing spaces/tabs
    while pos < len && (bytes[pos] == b' ' || bytes[pos] == b'\t') {
        pos += 1;
    }
    // Consume newline
    if pos < len && bytes[pos] == b'\r' {
        pos += 1;
    }
    if pos < len && bytes[pos] == b'\n' {
        pos += 1;
    }
    pos
}

/// Try to parse an ESM block (`import`/`export`) starting at `pos`.
///
/// ESM statements can span multiple lines (e.g. multiline imports).
/// We accumulate lines until we hit a blank line or a line that doesn't
/// look like a continuation.
///
/// Returns the byte offset after the ESM block, or `None` if not ESM.
fn try_esm(bytes: &[u8], pos: usize) -> Option<usize> {
    let len = bytes.len();
    let rest = &bytes[pos..];

    let is_import = rest.starts_with(b"import ")
        || rest.starts_with(b"import\t")
        || rest.starts_with(b"import{")
        || (rest.len() >= 7 && rest.starts_with(b"import\""))
        || (rest.len() >= 7 && rest.starts_with(b"import'"));
    let is_export = rest.starts_with(b"export ")
        || rest.starts_with(b"export\t")
        || rest.starts_with(b"export{");

    if !is_import && !is_export {
        return None;
    }

    // Reject dynamic imports: `import(`, `import (`, `import.`
    if is_import {
        let after_import = skip_whitespace_offset(rest, 6); // "import".len() == 6
        if after_import < rest.len() && (rest[after_import] == b'(' || rest[after_import] == b'.') {
            return None;
        }
    }

    // Find the end of the ESM statement.
    // Simple heuristic: accumulate lines until we see a blank line,
    // or the first line ends with a semicolon/newline.
    let mut end = next_line(bytes, pos);

    // Check if the statement might be multiline (has an unclosed `{` or
    // uses `from` keyword that hasn't appeared yet, etc.)
    // Simple approach: if line doesn't end with `;` or contain `from`,
    // keep going until blank line.
    loop {
        if end >= len {
            break;
        }
        // Check for blank line
        let next_first_non_ws = skip_whitespace_offset(bytes, end);
        if next_first_non_ws >= len || bytes[next_first_non_ws] == b'\n' {
            break;
        }
        // If this line starts with a keyword that begins a new statement, stop
        let next_rest = &bytes[end..];
        if next_rest.starts_with(b"import ")
            || next_rest.starts_with(b"export ")
            || next_rest.starts_with(b"<")
            || next_rest.starts_with(b"{")
            || next_rest.starts_with(b"#")
        {
            break;
        }
        // Check if previous line ended with a semicolon (before the newline)
        let prev_line_end = if end >= 2 && bytes[end - 2] == b'\r' {
            end - 2
        } else if end >= 1 {
            end - 1
        } else {
            break;
        };
        // Walk back past whitespace to find the last significant char
        let mut check = prev_line_end;
        while check > pos && (bytes[check - 1] == b' ' || bytes[check - 1] == b'\t') {
            check -= 1;
        }
        if check > pos && bytes[check - 1] == b';' {
            break;
        }
        // Continue accumulating
        end = next_line(bytes, end);
    }

    Some(end)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Basic ────────────────────────────────────────────────────────

    #[test]
    fn pure_markdown() {
        let input = "# Hello\n\nWorld\n";
        let segs = split(input);
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0], Segment::Markdown(input));
    }

    #[test]
    fn empty_input() {
        let segs = split("");
        assert!(segs.is_empty());
    }

    // ── ESM: valid forms ─────────────────────────────────────────────

    #[test]
    fn import_then_markdown() {
        let input = "import Foo from 'foo'\n\n# Hello\n";
        let segs = split(input);
        assert_eq!(segs[0], Segment::Esm("import Foo from 'foo'\n"));
        assert_eq!(segs[1], Segment::Markdown("\n# Hello\n"));
    }

    #[test]
    fn export_statement() {
        let input = "export const x = 1\n\nText\n";
        let segs = split(input);
        assert_eq!(segs[0], Segment::Esm("export const x = 1\n"));
    }

    #[test]
    fn multiple_imports() {
        let input = "import A from 'a'\nimport B from 'b'\n\n# Title\n";
        let segs = split(input);
        assert_eq!(segs[0], Segment::Esm("import A from 'a'\n"));
        assert_eq!(segs[1], Segment::Esm("import B from 'b'\n"));
    }

    #[test]
    fn import_default() {
        let segs = split("import a from \"b\"\n\nc\n");
        assert_eq!(segs[0], Segment::Esm("import a from \"b\"\n"));
    }

    #[test]
    fn import_namespace() {
        let segs = split("import * as a from \"b\"\n\nc\n");
        assert_eq!(segs[0], Segment::Esm("import * as a from \"b\"\n"));
    }

    #[test]
    fn import_destructured() {
        let segs = split("import {a} from \"b\"\n\nc\n");
        assert_eq!(segs[0], Segment::Esm("import {a} from \"b\"\n"));
    }

    #[test]
    fn import_side_effect() {
        let segs = split("import \"a\"\n\nc\n");
        assert_eq!(segs[0], Segment::Esm("import \"a\"\n"));
    }

    #[test]
    fn import_side_effect_single_quote() {
        let segs = split("import 'a'\n\nc\n");
        assert_eq!(segs[0], Segment::Esm("import 'a'\n"));
    }

    #[test]
    fn export_var() {
        let segs = split("export var a = 1\n\nb\n");
        assert_eq!(segs[0], Segment::Esm("export var a = 1\n"));
    }

    #[test]
    fn export_const() {
        let segs = split("export const a = \"\"\n\nb\n");
        assert_eq!(segs[0], Segment::Esm("export const a = \"\"\n"));
    }

    #[test]
    fn export_let() {
        let segs = split("export let a = \"\"\n\nb\n");
        assert_eq!(segs[0], Segment::Esm("export let a = \"\"\n"));
    }

    #[test]
    fn export_default() {
        let segs = split("export default a = 1\n\nb\n");
        assert_eq!(segs[0], Segment::Esm("export default a = 1\n"));
    }

    #[test]
    fn export_function() {
        let segs = split("export function a() {}\n\nb\n");
        assert_eq!(segs[0], Segment::Esm("export function a() {}\n"));
    }

    #[test]
    fn export_class() {
        let segs = split("export class a {}\n\nb\n");
        assert_eq!(segs[0], Segment::Esm("export class a {}\n"));
    }

    #[test]
    fn export_from() {
        let segs = split("export {a} from \"b\"\n\nc\n");
        assert_eq!(segs[0], Segment::Esm("export {a} from \"b\"\n"));
    }

    #[test]
    fn export_star_from() {
        let segs = split("export * from \"a\"\n\nb\n");
        assert_eq!(segs[0], Segment::Esm("export * from \"a\"\n"));
    }

    #[test]
    fn export_multiline() {
        let input = "export {\n  a\n} from \"b\"\n\nc\n";
        let segs = split(input);
        assert_eq!(segs[0], Segment::Esm("export {\n  a\n} from \"b\"\n"));
    }

    // ── ESM: NOT ESM (false positive protection) ─────────────────────

    #[test]
    fn not_esm_impossible() {
        // Word starting with "im" — not `import `
        let segs = split("impossible\n");
        assert_eq!(segs[0], Segment::Markdown("impossible\n"));
    }

    #[test]
    fn not_esm_exporting() {
        // Word starting with "export" — not `export `
        let segs = split("exporting\n");
        assert_eq!(segs[0], Segment::Markdown("exporting\n"));
    }

    #[test]
    fn not_esm_import_dot() {
        // `import.meta` is property access, not ESM
        let segs = split("import.meta.url\n");
        assert_eq!(segs[0], Segment::Markdown("import.meta.url\n"));
    }

    #[test]
    fn not_esm_dynamic_import() {
        // `import("a")` is dynamic import, not ESM
        let segs = split("import(\"a\")\n");
        assert_eq!(segs[0], Segment::Markdown("import(\"a\")\n"));
    }

    #[test]
    fn not_esm_dynamic_import_space() {
        // `import ('a')` is dynamic import with space
        let segs = split("import ('a')\n");
        assert_eq!(segs[0], Segment::Markdown("import ('a')\n"));
    }

    #[test]
    fn not_esm_indented() {
        // Indented import is not ESM
        let segs = split("  import a from \"b\"\n");
        assert_eq!(segs[0], Segment::Markdown("  import a from \"b\"\n"));
    }

    #[test]
    fn not_esm_interrupts_paragraph() {
        // ESM cannot interrupt a paragraph — needs blank line before
        let segs = split("a\nimport a from \"b\"\n");
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0], Segment::Markdown("a\nimport a from \"b\"\n"));
    }

    #[test]
    fn not_esm_interrupts_paragraph_export() {
        let segs = split("a\nexport default c\n");
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0], Segment::Markdown("a\nexport default c\n"));
    }

    #[test]
    fn esm_after_blank_line() {
        // After a blank line, ESM is valid again
        let segs = split("a\n\nimport a from \"b\"\n\nc\n");
        assert!(matches!(segs[0], Segment::Markdown(_)));
        assert_eq!(segs[1], Segment::Esm("import a from \"b\"\n"));
    }

    // ── JSX: flow ────────────────────────────────────────────────────

    #[test]
    fn jsx_open_close() {
        let input = "<Wrapper>\n\n## Heading\n\n</Wrapper>\n";
        let segs = split(input);
        assert_eq!(segs[0], Segment::JsxBlockOpen("<Wrapper>\n"));
        assert_eq!(segs[1], Segment::Markdown("\n## Heading\n\n"));
        assert_eq!(segs[2], Segment::JsxBlockClose("</Wrapper>\n"));
    }

    #[test]
    fn jsx_self_closing() {
        let input = "<Image src=\"foo.png\" />\n";
        let segs = split(input);
        assert_eq!(segs[0], Segment::JsxBlockSelfClose(input));
    }

    #[test]
    fn fragment_open_close() {
        let input = "<>\nHello\n</>\n";
        let segs = split(input);
        assert_eq!(segs[0], Segment::JsxBlockOpen("<>\n"));
        assert_eq!(segs[1], Segment::Markdown("Hello\n"));
        assert_eq!(segs[2], Segment::JsxBlockClose("</>\n"));
    }

    #[test]
    fn jsx_with_attributes() {
        let input = "<Button onClick={handleClick} variant=\"primary\">\nClick me\n</Button>\n";
        let segs = split(input);
        assert_eq!(
            segs[0],
            Segment::JsxBlockOpen("<Button onClick={handleClick} variant=\"primary\">\n")
        );
    }

    #[test]
    fn nested_jsx_components() {
        let input = "<Outer>\n<Inner />\n</Outer>\n";
        let segs = split(input);
        assert_eq!(segs[0], Segment::JsxBlockOpen("<Outer>\n"));
        assert_eq!(segs[1], Segment::JsxBlockSelfClose("<Inner />\n"));
        assert_eq!(segs[2], Segment::JsxBlockClose("</Outer>\n"));
    }

    #[test]
    fn jsx_self_closing_with_leading_spaces() {
        // Leading whitespace still counts as flow
        let segs = split("   <a />\n");
        assert_eq!(segs[0], Segment::JsxBlockSelfClose("   <a />\n"));
    }

    #[test]
    fn jsx_with_markdown_inside() {
        let input = "<a>\nb\n</a>\n";
        let segs = split(input);
        assert_eq!(segs[0], Segment::JsxBlockOpen("<a>\n"));
        assert_eq!(segs[1], Segment::Markdown("b\n"));
        assert_eq!(segs[2], Segment::JsxBlockClose("</a>\n"));
    }

    #[test]
    fn jsx_with_list_inside() {
        let input = "<a>\n- b\n</a>\n";
        let segs = split(input);
        assert_eq!(segs[0], Segment::JsxBlockOpen("<a>\n"));
        assert_eq!(segs[1], Segment::Markdown("- b\n"));
        assert_eq!(segs[2], Segment::JsxBlockClose("</a>\n"));
    }

    // ── JSX: NOT flow (inline/text) ──────────────────────────────────

    #[test]
    fn jsx_trailing_content_is_markdown() {
        // `<x />.` — trailing content makes entire line text/markdown
        let segs = split("<x />.\n");
        assert_eq!(segs[0], Segment::Markdown("<x />.\n"));
    }

    #[test]
    fn jsx_leading_text_is_markdown() {
        // `a <x />` — leading text makes it inline
        let segs = split("a <x />\n");
        assert_eq!(segs[0], Segment::Markdown("a <x />\n"));
    }

    #[test]
    fn close_tag_trailing_content_is_markdown() {
        let segs = split("</a>.\n");
        assert_eq!(segs[0], Segment::Markdown("</a>.\n"));
    }

    // ── Expression: flow ─────────────────────────────────────────────

    #[test]
    fn expression_block() {
        let input = "{variable}\n\nHello\n";
        let segs = split(input);
        assert_eq!(segs[0], Segment::Expression("{variable}\n"));
    }

    #[test]
    fn complex_expression() {
        let input = "{items.map(i => <li>{i}</li>)}\n";
        let segs = split(input);
        assert_eq!(segs[0], Segment::Expression(input));
    }

    #[test]
    fn empty_expression() {
        let segs = split("{}\n");
        assert_eq!(segs[0], Segment::Expression("{}\n"));
    }

    #[test]
    fn expression_with_leading_spaces() {
        let segs = split("  { a }\n");
        assert_eq!(segs[0], Segment::Expression("  { a }\n"));
    }

    #[test]
    fn expression_with_trailing_spaces() {
        let segs = split("{ a } \t\n");
        assert_eq!(segs[0], Segment::Expression("{ a } \t\n"));
    }

    #[test]
    fn expression_multiline() {
        let segs = split("{\n  1 + 1\n}\n");
        assert_eq!(segs[0], Segment::Expression("{\n  1 + 1\n}\n"));
    }

    #[test]
    fn expression_with_comment() {
        let segs = split("{/**/}\n");
        assert_eq!(segs[0], Segment::Expression("{/**/}\n"));
    }

    // ── Expression: NOT flow (inline/text) ───────────────────────────

    #[test]
    fn expression_trailing_content_is_markdown() {
        // `{ a } b` — trailing text makes it text/markdown
        let segs = split("{ a } b\n");
        assert_eq!(segs[0], Segment::Markdown("{ a } b\n"));
    }

    #[test]
    fn expression_in_paragraph_is_markdown() {
        let segs = split("a {b} c\n");
        assert_eq!(segs[0], Segment::Markdown("a {b} c\n"));
    }

    // ── Mixed document ───────────────────────────────────────────────

    #[test]
    fn mixed_document() {
        let input = "\
import A from 'a'

# Title

<Card>

Some **text**.

</Card>
";
        let segs = split(input);
        assert_eq!(segs[0], Segment::Esm("import A from 'a'\n"));
        assert_eq!(segs[1], Segment::Markdown("\n# Title\n\n"));
        assert_eq!(segs[2], Segment::JsxBlockOpen("<Card>\n"));
        assert_eq!(segs[3], Segment::Markdown("\nSome **text**.\n\n"));
        assert_eq!(segs[4], Segment::JsxBlockClose("</Card>\n"));
    }

    // ── Defensive ────────────────────────────────────────────────────

    #[test]
    fn invalid_jsx_becomes_markdown() {
        let segs = split("< 5\n");
        assert_eq!(segs[0], Segment::Markdown("< 5\n"));
    }

    #[test]
    fn unterminated_expression_becomes_markdown() {
        let segs = split("{unterminated\n");
        assert_eq!(segs[0], Segment::Markdown("{unterminated\n"));
    }
}
