use ferromark::{Options, to_html_with_options};

#[test]
fn crlf_blank_lines_separate_paragraphs_and_forward_reference_definitions() {
    for padding in [
        "".to_owned(),
        " \t".to_owned(),
        " ".repeat(15),
        " ".repeat(16),
        " ".repeat(32),
    ] {
        let input = format!("[x]\n{padding}\n[x]: /safe \"Title\"\n[y]: /other\n");
        let expected = "<p><a href=\"/safe\" title=\"Title\">x</a></p>\n";
        for ending in ["\n", "\r\n"] {
            assert_eq!(
                to_html_with_options(&input.replace('\n', ending), &Options::commonmark()),
                expected
            );
        }
    }
}

#[test]
fn crlf_blank_lines_inside_containers_preserve_paragraph_boundaries() {
    for input in ["> first\n>\n> second\n", "- first\n  \n  second\n"] {
        let options = Options::commonmark();
        assert_eq!(
            to_html_with_options(&input.replace('\n', "\r\n"), &options),
            to_html_with_options(input, &options)
        );
    }
}

#[test]
fn crlf_code_fences_close_before_forward_reference_definitions() {
    for fence in ["```", "~~~"] {
        let input = format!("{fence}rust\r\ncode\r\n{fence} \t\r\n\r\n[x]\r\n\r\n[x]: /safe\r\n");
        let html = to_html_with_options(&input, &Options::commonmark());
        assert!(
            html.starts_with("<pre><code class=\"language-rust\">"),
            "{html}"
        );
        assert!(
            html.ends_with("</code></pre>\n<p><a href=\"/safe\">x</a></p>\n"),
            "{html}"
        );
    }
}

#[test]
fn crlf_blank_lines_end_html_blocks_before_reference_definitions() {
    for tag in ["<div>", "<custom>"] {
        let input = format!("{tag}\r\nraw\r\n\r\n[x]\r\n\r\n[x]: /safe\r\n");
        let html = to_html_with_options(&input, &Options::commonmark());
        assert!(html.ends_with("<p><a href=\"/safe\">x</a></p>\n"), "{html}");
    }
}
