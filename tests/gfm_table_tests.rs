//! GFM Tables extension tests.
//!
//! Tests based on the GFM spec (https://github.github.com/gfm/#tables-extension-)
//! Examples 198-205, plus additional edge cases.

use ferromark::to_html;
use ferromark::to_html_with_options;
use ferromark::Options;

// === GFM Spec Examples ===

/// Example 198: Basic table with header, delimiter, and body rows.
#[test]
fn gfm_example_198_basic_table() {
    let input = "| foo | bar |\n| --- | --- |\n| baz | bim |\n";
    let expected = "<table>\n<thead>\n<tr>\n<th>foo</th>\n<th>bar</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>baz</td>\n<td>bim</td>\n</tr>\n</tbody>\n</table>\n";
    assert_eq!(to_html(input), expected);
}

/// Example 199: Alignment (center, right), no leading pipes on delimiter/body.
#[test]
fn gfm_example_199_alignment() {
    let input = "| abc | defghi |\n:-: | -----------:\nbar | baz\n";
    let expected = "<table>\n<thead>\n<tr>\n<th align=\"center\">abc</th>\n<th align=\"right\">defghi</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td align=\"center\">bar</td>\n<td align=\"right\">baz</td>\n</tr>\n</tbody>\n</table>\n";
    assert_eq!(to_html(input), expected);
}

/// Example 200: Escaped pipes and pipes in code spans.
#[test]
fn gfm_example_200_escaped_pipes() {
    let input = "| f\\|oo  |\n| ------ |\n| b `\\|` az |\n| b **\\|** im |\n";
    let expected = "<table>\n<thead>\n<tr>\n<th>f|oo</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>b <code>|</code> az</td>\n</tr>\n<tr>\n<td>b <strong>|</strong> im</td>\n</tr>\n</tbody>\n</table>\n";
    assert_eq!(to_html(input), expected);
}

/// Example 201: Table terminated by blockquote.
#[test]
fn gfm_example_201_blockquote_terminates() {
    let input = "| abc | def |\n| --- | --- |\n| bar | baz |\n> bar\n";
    let expected = "<table>\n<thead>\n<tr>\n<th>abc</th>\n<th>def</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>bar</td>\n<td>baz</td>\n</tr>\n</tbody>\n</table>\n<blockquote>\n<p>bar</p>\n</blockquote>\n";
    assert_eq!(to_html(input), expected);
}

/// Example 202: Table with trailing data row and blank-line termination.
#[test]
fn gfm_example_202_blank_line_terminates() {
    let input = "| abc | def |\n| --- | --- |\n| bar | baz |\nbar\n\nbar\n";
    let expected = "<table>\n<thead>\n<tr>\n<th>abc</th>\n<th>def</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>bar</td>\n<td>baz</td>\n</tr>\n<tr>\n<td>bar</td>\n<td></td>\n</tr>\n</tbody>\n</table>\n<p>bar</p>\n";
    assert_eq!(to_html(input), expected);
}

/// Example 203: Cell count mismatch → not a table.
#[test]
fn gfm_example_203_cell_count_mismatch() {
    let input = "| abc | def |\n| --- |\n| bar |\n";
    let expected = "<p>| abc | def |\n| --- |\n| bar |</p>\n";
    assert_eq!(to_html(input), expected);
}

/// Example 204: Variable row lengths (fewer → empty, more → ignored).
#[test]
fn gfm_example_204_variable_row_lengths() {
    let input = "| abc | def |\n| --- | --- |\n| bar |\n| bar | baz | boo |\n";
    let expected = "<table>\n<thead>\n<tr>\n<th>abc</th>\n<th>def</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>bar</td>\n<td></td>\n</tr>\n<tr>\n<td>bar</td>\n<td>baz</td>\n</tr>\n</tbody>\n</table>\n";
    assert_eq!(to_html(input), expected);
}

/// Example 205: Empty body (header only, no data rows).
#[test]
fn gfm_example_205_empty_body() {
    let input = "| abc | def |\n| --- | --- |\n";
    let expected = "<table>\n<thead>\n<tr>\n<th>abc</th>\n<th>def</th>\n</tr>\n</thead>\n</table>\n";
    assert_eq!(to_html(input), expected);
}

// === Additional Edge Cases ===

/// Table inside a blockquote.
#[test]
fn table_in_blockquote() {
    let input = "> | a | b |\n> | - | - |\n> | c | d |\n";
    let expected = "<blockquote>\n<table>\n<thead>\n<tr>\n<th>a</th>\n<th>b</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>c</td>\n<td>d</td>\n</tr>\n</tbody>\n</table>\n</blockquote>\n";
    assert_eq!(to_html(input), expected);
}

/// Table inside a list item.
#[test]
fn table_in_list_item() {
    let input = "- | a | b |\n  | - | - |\n  | c | d |\n";
    let expected = "<ul>\n<li>\n<table>\n<thead>\n<tr>\n<th>a</th>\n<th>b</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>c</td>\n<td>d</td>\n</tr>\n</tbody>\n</table>\n</li>\n</ul>\n";
    assert_eq!(to_html(input), expected);
}

/// Inline content in cells (emphasis, code, links).
#[test]
fn table_with_inline_content() {
    let input = "| *em* | **strong** | `code` |\n| --- | --- | --- |\n| [link](url) | ![img](src) | a |\n";
    let expected = "<table>\n<thead>\n<tr>\n<th><em>em</em></th>\n<th><strong>strong</strong></th>\n<th><code>code</code></th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td><a href=\"url\">link</a></td>\n<td><img src=\"src\" alt=\"img\" /></td>\n<td>a</td>\n</tr>\n</tbody>\n</table>\n";
    assert_eq!(to_html(input), expected);
}

/// Table preceded by paragraph text (retroactive header).
#[test]
fn table_preceded_by_paragraph() {
    let input = "Paragraph text\nHeader\n| --- |\ndata\n";
    let expected = "<p>Paragraph text</p>\n<table>\n<thead>\n<tr>\n<th>Header</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>data</td>\n</tr>\n</tbody>\n</table>\n";
    assert_eq!(to_html(input), expected);
}

/// Single-column table.
#[test]
fn single_column_table() {
    let input = "| a |\n| - |\n| b |\n";
    let expected = "<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>b</td>\n</tr>\n</tbody>\n</table>\n";
    assert_eq!(to_html(input), expected);
}

/// All alignment types in one table.
#[test]
fn all_alignment_types() {
    let input = "| a | b | c | d |\n| --- | :--- | :---: | ---: |\n| 1 | 2 | 3 | 4 |\n";
    let expected = "<table>\n<thead>\n<tr>\n<th>a</th>\n<th align=\"left\">b</th>\n<th align=\"center\">c</th>\n<th align=\"right\">d</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>1</td>\n<td align=\"left\">2</td>\n<td align=\"center\">3</td>\n<td align=\"right\">4</td>\n</tr>\n</tbody>\n</table>\n";
    assert_eq!(to_html(input), expected);
}

/// Table terminated by ATX heading.
#[test]
fn table_terminated_by_heading() {
    let input = "| a | b |\n| - | - |\n| c | d |\n# Heading\n";
    let expected = "<table>\n<thead>\n<tr>\n<th>a</th>\n<th>b</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>c</td>\n<td>d</td>\n</tr>\n</tbody>\n</table>\n<h1>Heading</h1>\n";
    assert_eq!(to_html(input), expected);
}

/// Table terminated by fenced code block.
#[test]
fn table_terminated_by_code_fence() {
    let input = "| a | b |\n| - | - |\n| c | d |\n```\ncode\n```\n";
    let expected = "<table>\n<thead>\n<tr>\n<th>a</th>\n<th>b</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>c</td>\n<td>d</td>\n</tr>\n</tbody>\n</table>\n<pre><code>code\n</code></pre>\n";
    assert_eq!(to_html(input), expected);
}

/// Table terminated by thematic break.
#[test]
fn table_terminated_by_thematic_break() {
    let input = "| a | b |\n| - | - |\n| c | d |\n---\n";
    let expected = "<table>\n<thead>\n<tr>\n<th>a</th>\n<th>b</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>c</td>\n<td>d</td>\n</tr>\n</tbody>\n</table>\n<hr />\n";
    assert_eq!(to_html(input), expected);
}

/// Table disabled via options.
#[test]
fn table_disabled_via_options() {
    let input = "| a | b |\n| - | - |\n| c | d |\n";
    let options = Options {
        tables: false,
        ..Options::default()
    };
    let result = to_html_with_options(input, &options);
    // Without tables, this should be a paragraph (with thematic break from ---)
    assert!(!result.contains("<table>"), "Should not contain table: {result}");
}

/// Escaped pipe at end of cell.
#[test]
fn escaped_pipe_at_end() {
    let input = "| a\\| | b |\n| --- | --- |\n| c\\| | d |\n";
    let expected = "<table>\n<thead>\n<tr>\n<th>a|</th>\n<th>b</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>c|</td>\n<td>d</td>\n</tr>\n</tbody>\n</table>\n";
    assert_eq!(to_html(input), expected);
}

/// Table at end of input without trailing newline.
#[test]
fn table_at_eof() {
    let input = "| a | b |\n| - | - |\n| c | d |";
    let expected = "<table>\n<thead>\n<tr>\n<th>a</th>\n<th>b</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>c</td>\n<td>d</td>\n</tr>\n</tbody>\n</table>\n";
    assert_eq!(to_html(input), expected);
}

/// Multiple tables separated by paragraph.
#[test]
fn multiple_tables() {
    let input = "| a |\n| - |\n| b |\n\nText\n\n| c |\n| - |\n| d |\n";
    let expected = "<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>b</td>\n</tr>\n</tbody>\n</table>\n<p>Text</p>\n<table>\n<thead>\n<tr>\n<th>c</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>d</td>\n</tr>\n</tbody>\n</table>\n";
    assert_eq!(to_html(input), expected);
}

/// Delimiter row with only dashes (no colons = no alignment).
#[test]
fn delimiter_all_none_alignment() {
    let input = "| a | b |\n| --- | --- |\n| c | d |\n";
    let result = to_html(input);
    // No align attributes
    assert!(result.contains("<th>a</th>"));
    assert!(result.contains("<td>c</td>"));
    assert!(!result.contains("align="));
}

/// Delimiter row with left alignment.
#[test]
fn delimiter_left_alignment() {
    let input = "| a |\n| :--- |\n| b |\n";
    let result = to_html(input);
    assert!(result.contains("<th align=\"left\">a</th>"));
    assert!(result.contains("<td align=\"left\">b</td>"));
}

// === Official cmark-gfm extensions.txt tests ===
// Tests from https://github.com/github/cmark-gfm/blob/master/test/extensions.txt

/// cmark-gfm ext: Multi-row table body.
#[test]
fn cmark_ext_multi_row_body() {
    let input = "| abc | def |\n| --- | --- |\n| ghi | jkl |\n| mno | pqr |\n";
    let expected = "<table>\n<thead>\n<tr>\n<th>abc</th>\n<th>def</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>ghi</td>\n<td>jkl</td>\n</tr>\n<tr>\n<td>mno</td>\n<td>pqr</td>\n</tr>\n</tbody>\n</table>\n";
    assert_eq!(to_html(input), expected);
}

/// cmark-gfm ext: Inline formatting in table cells.
#[test]
fn cmark_ext_inline_formatting_in_cells() {
    let input = "Hello!\n\n| _abc_ | \u{30BB}\u{30F3} |\n| ----- | ---- |\n| 1. Block elements inside cells don't work. | |\n| But _**inline elements do**_. | x |\n\nHi!\n";
    let expected = "<p>Hello!</p>\n<table>\n<thead>\n<tr>\n<th><em>abc</em></th>\n<th>\u{30BB}\u{30F3}</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>1. Block elements inside cells don't work.</td>\n<td></td>\n</tr>\n<tr>\n<td>But <em><strong>inline elements do</strong></em>.</td>\n<td>x</td>\n</tr>\n</tbody>\n</table>\n<p>Hi!</p>\n";
    assert_eq!(to_html(input), expected);
}

/// cmark-gfm ext: Edge cases on table recognition.
#[test]
fn cmark_ext_table_recognition_edge_cases() {
    // Single line is not a table
    assert_eq!(
        to_html("| Not enough table | to be considered table |\n"),
        "<p>| Not enough table | to be considered table |</p>\n"
    );

    // Two lines without delimiter is not a table
    assert_eq!(
        to_html("| Not enough table | to be considered table |\n| Not enough table | to be considered table |\n"),
        "<p>| Not enough table | to be considered table |\n| Not enough table | to be considered table |</p>\n"
    );

    // Header-only table (just enough)
    let input = "| Just enough table | to be considered table |\n| ----------------- | ---------------------- |\n";
    let expected = "<table>\n<thead>\n<tr>\n<th>Just enough table</th>\n<th>to be considered table</th>\n</tr>\n</thead>\n</table>\n";
    assert_eq!(to_html(input), expected);
}

/// cmark-gfm ext: Delimiter row alone is not a table.
#[test]
fn cmark_ext_delimiter_row_alone() {
    let input = "| ---- | --- |\n";
    let result = to_html(input);
    assert!(!result.contains("<table>"), "Delimiter row alone should not be a table: {result}");
}

/// cmark-gfm ext: Minimal single-cell table.
#[test]
fn cmark_ext_minimal_single_cell() {
    let input = "|x|\n|-|\n";
    let expected = "<table>\n<thead>\n<tr>\n<th>x</th>\n</tr>\n</thead>\n</table>\n";
    assert_eq!(to_html(input), expected);
}

/// cmark-gfm ext: Table without leading pipes (GFM style).
#[test]
fn cmark_ext_no_leading_pipes() {
    let input = "abc | def\n--- | ---\nxyz | ghi\n";
    let expected = "<table>\n<thead>\n<tr>\n<th>abc</th>\n<th>def</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>xyz</td>\n<td>ghi</td>\n</tr>\n</tbody>\n</table>\n";
    assert_eq!(to_html(input), expected);
}

/// cmark-gfm ext: 5-column alignment.
#[test]
fn cmark_ext_five_column_alignment() {
    let input = "aaa | bbb | ccc | ddd | eee\n:-- | --- | :-: | --- | --:\nfff | ggg | hhh | iii | jjj\n";
    let expected = "<table>\n<thead>\n<tr>\n<th align=\"left\">aaa</th>\n<th>bbb</th>\n<th align=\"center\">ccc</th>\n<th>ddd</th>\n<th align=\"right\">eee</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td align=\"left\">fff</td>\n<td>ggg</td>\n<td align=\"center\">hhh</td>\n<td>iii</td>\n<td align=\"right\">jjj</td>\n</tr>\n</tbody>\n</table>\n";
    assert_eq!(to_html(input), expected);
}

/// cmark-gfm ext: Header/delimiter column count mismatch rejects table.
#[test]
fn cmark_ext_header_delimiter_mismatch() {
    let input = "| a | b | c |\n| --- | --- |\n| this | isn't | okay |\n";
    let expected = "<p>| a | b | c |\n| --- | --- |\n| this | isn't | okay |</p>\n";
    assert_eq!(to_html(input), expected);
}

/// cmark-gfm ext: Body row cell count flexibility (fewer pad, more truncate).
#[test]
fn cmark_ext_body_cell_count_flexibility() {
    let input = "| a | b | c |\n| --- | --- | ---\n| x\n| a | b\n| 1 | 2 | 3 | 4 | 5 |\n";
    let expected = "<table>\n<thead>\n<tr>\n<th>a</th>\n<th>b</th>\n<th>c</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>x</td>\n<td></td>\n<td></td>\n</tr>\n<tr>\n<td>a</td>\n<td>b</td>\n<td></td>\n</tr>\n<tr>\n<td>1</td>\n<td>2</td>\n<td>3</td>\n</tr>\n</tbody>\n</table>\n";
    assert_eq!(to_html(input), expected);
}

/// cmark-gfm ext: Escaped pipes in body cells.
#[test]
fn cmark_ext_escaped_pipes_in_body() {
    let input = "| a | b |\n| --- | --- |\n| Escaped pipes are \\|okay\\|. | Like \\| this. |\n| Within `\\|code\\| is okay` too. |\n";
    let expected = "<table>\n<thead>\n<tr>\n<th>a</th>\n<th>b</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>Escaped pipes are |okay|.</td>\n<td>Like | this.</td>\n</tr>\n<tr>\n<td>Within <code>|code| is okay</code> too.</td>\n<td></td>\n</tr>\n</tbody>\n</table>\n";
    assert_eq!(to_html(input), expected);
}

/// cmark-gfm ext: Oddly-formatted delimiter (missing leading pipe).
#[test]
fn cmark_ext_oddly_formatted_delimiter() {
    let input = "| a |\n--- |\n";
    let expected = "<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n</table>\n";
    assert_eq!(to_html(input), expected);
}

/// cmark-gfm ext: Escaping behavior in table cells.
#[test]
fn cmark_ext_escaping_behavior() {
    let input = "| a | b |\n| --- | --- |\n| \\\\ | `\\\\` |\n| \\\\\\\\ | `\\\\\\\\` |\n| \\_ | `\\_` |\n| \\| | `\\|` |\n| \\a | `\\a` |\n";
    let expected = "<table>\n<thead>\n<tr>\n<th>a</th>\n<th>b</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>\\</td>\n<td><code>\\\\</code></td>\n</tr>\n<tr>\n<td>\\\\</td>\n<td><code>\\\\\\\\</code></td>\n</tr>\n<tr>\n<td>_</td>\n<td><code>\\_</code></td>\n</tr>\n<tr>\n<td>|</td>\n<td><code>|</code></td>\n</tr>\n<tr>\n<td>\\a</td>\n<td><code>\\a</code></td>\n</tr>\n</tbody>\n</table>\n";
    assert_eq!(to_html(input), expected);
}

/// cmark-gfm ext: Embedded HTML in table cells.
#[test]
fn cmark_ext_embedded_html_in_cells() {
    let input = "| a |\n| --- |\n| <strong>hello</strong> |\n| ok <br> sure |\n";
    let expected = "<table>\n<thead>\n<tr>\n<th>a</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td><strong>hello</strong></td>\n</tr>\n<tr>\n<td>ok <br> sure</td>\n</tr>\n</tbody>\n</table>\n";
    assert_eq!(to_html(input), expected);
}

/// cmark-gfm ext: Reference-style links in table cells.
#[test]
fn cmark_ext_reference_links_in_cells() {
    let input = "Here's a link to [Freedom Planet 2][].\n\n| Here's a link to [Freedom Planet 2][] in a table header. |\n| --- |\n| Here's a link to [Freedom Planet 2][] in a table row. |\n\n[Freedom Planet 2]: http://www.freedomplanet2.com/\n";
    let expected = "<p>Here's a link to <a href=\"http://www.freedomplanet2.com/\">Freedom Planet 2</a>.</p>\n<table>\n<thead>\n<tr>\n<th>Here's a link to <a href=\"http://www.freedomplanet2.com/\">Freedom Planet 2</a> in a table header.</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>Here's a link to <a href=\"http://www.freedomplanet2.com/\">Freedom Planet 2</a> in a table row.</td>\n</tr>\n</tbody>\n</table>\n";
    assert_eq!(to_html(input), expected);
}

/// cmark-gfm ext: Sequential empty cells (double pipe).
#[test]
fn cmark_ext_sequential_empty_cells() {
    let input = "| a | b | c |\n| --- | --- | --- |\n| d || e |\n";
    let expected = "<table>\n<thead>\n<tr>\n<th>a</th>\n<th>b</th>\n<th>c</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>d</td>\n<td></td>\n<td>e</td>\n</tr>\n</tbody>\n</table>\n";
    assert_eq!(to_html(input), expected);
}

/// cmark-gfm ext: Emphasis interaction in cells.
#[test]
fn cmark_ext_emphasis_interaction() {
    let input = "| a | b |\n| --- | --- |\n|***(a)***|\n";
    let expected = "<table>\n<thead>\n<tr>\n<th>a</th>\n<th>b</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td><em><strong>(a)</strong></em></td>\n<td></td>\n</tr>\n</tbody>\n</table>\n";
    assert_eq!(to_html(input), expected);
}

/// cmark-gfm ext: Table preceded by multi-line paragraph (retroactive header).
#[test]
fn cmark_ext_table_after_multiline_paragraph() {
    let input = "123\n456\n| a | b |\n| ---| --- |\nd | e\n";
    let expected = "<p>123\n456</p>\n<table>\n<thead>\n<tr>\n<th>a</th>\n<th>b</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>d</td>\n<td>e</td>\n</tr>\n</tbody>\n</table>\n";
    assert_eq!(to_html(input), expected);
}

/// cmark-gfm regression: bare pipe + dash is not a table.
#[test]
fn cmark_regression_bare_pipe_dash() {
    let input = "|\n-|\n";
    let result = to_html(input);
    assert!(!result.contains("<table>"), "Bare pipe+dash should not be table: {result}");
}
