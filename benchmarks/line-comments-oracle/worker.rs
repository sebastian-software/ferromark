use std::fmt::Write as _;

use ferromark_v1 as v1;
use ferromark_v2 as v2;

fn v1_html(source: &str, comments: bool) -> String {
    let mut options = v1::Options::commonmark();
    options.allow_html = true;
    options.tables = true;
    options.strikethrough = true;
    options.task_lists = true;
    options.autolink_literals = true;
    options.render_policy = v1::RenderPolicy::Trusted;
    options.disallowed_raw_html = false;
    options.heading_ids = false;
    options.line_comments = comments;
    v1::to_html_with_options(source, &options)
}

fn v2_html(source: &str, comments: bool) -> String {
    let mut options = v2::ParserOptions::gfm_spec();
    options.line_comments = comments;
    let allocator = v2::Allocator::for_source_len(source.len());
    let document = v2::Parser::with_options(&allocator, source, options)
        .parse()
        .unwrap_or_else(|error| panic!("v2 parse failed for {source:?}: {error:?}"));
    v2::HtmlRenderer::with_options(v2::HtmlRendererOptions {
        sanitize: false,
        disallow_raw_html: false,
        autolink_urls: false,
        heading_ids: false,
        ..v2::HtmlRendererOptions::gfm()
    })
    .render(&document)
}

fn hex(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len() * 2);
    for byte in value.as_bytes() {
        write!(encoded, "{byte:02x}").unwrap();
    }
    encoded
}

fn cases() -> &'static [(&'static str, &'static str)] {
    &[
        ("default-shaped", "// private note\n"),
        ("only-comments", "// first\n   // second\n//\n"),
        ("paragraph", "first\n// private\nsecond\n"),
        ("blank-lines", "first\n\n// private\n\nsecond\n"),
        ("ordinary", "https://example.com\nText // ordinary\n\\// escaped\n"),
        ("indent", "   // hidden\n    // code\n"),
        ("fence", "```\n// code\n```\n"),
        ("raw-html", "<div>\n// html content\n</div>\n"),
        ("quote", "> first\n// private\n> second\n"),
        ("list", "- first\n// private\n- second\n"),
        ("explicit-quote", "> // visible\n"),
        ("setext", "Heading\n// private\n---\n"),
        ("table", "A | B\n// private\n- | -\n1 | 2\n"),
        ("crlf", "// private\r\nvisible\r\n"),
        ("reference-before", "// private\n[target]: /url\n\nvisible\n"),
        ("reference-after-comment", "[target]: /url\n// private\n\n[target]\n"),
        (
            "reference-destination-title-comment",
            "[target]:\n// before destination\n/url\n// before title\n\"title\"\n\n[target]\n",
        ),
        ("list-explicit", "- // visible\n"),
        ("nested-explicit", "- > // visible\n"),
        ("list-fence", "- ```\n  // code\n  ```\n"),
        ("quote-fence", "> ```\n> // code\n> ```\n"),
        ("nested-list-quote-fence", "- > ```\n  > // code\n  > ```\n"),
        ("list-html", "- <div>\n  // html\n  </div>\n"),
        ("quote-html", "> <div>\n> // html\n> </div>\n"),
        ("indented-code", "    // code\n    next\n"),
        ("indented-code-after-comment", "// private\n    // code\n"),
        ("comment-eof", "visible\n// private"),
        ("comment-between-definitions", "[x]: /url\n// private\n[x]\n"),
        ("comment-table-body", "A | B\n- | -\n1 | 2\n// private\n3 | 4\n"),
        ("list-indented-code", "- first\n      // code\n      next\n"),
        ("quote-indented-code", ">     // code\n>     next\n"),
        ("definition-title", "[target]: /url\n// private\n\"title\"\n\n[target]\n"),
        ("repeated-definitions", "[target]: /one\n// private\n[target]: /two\n\n[target]\n"),
        ("quote-definition-title", "> [target]: /url\n// private\n> \"title\"\n\n[target]\n"),
        ("list-definition-title", "- [target]: /url\n  // private\n  \"title\"\n\n[target]\n"),
        ("list-table-comment", "- A | B\n  // private\n  - | -\n  1 | 2\n"),
        ("quote-table-comment", "> A | B\n// private\n> - | -\n> 1 | 2\n"),
        ("root-comment-indented-code", "// private\n    // code\n    next\n"),
        ("root-comment-before-html", "// private\n<div>\n// html\n</div>\n"),
        (
            "comment-after-paragraph-before-definition",
            "first\n// private\n[target]: /url\n\n[target]\n",
        ),
    ]
}

fn main() {
    println!("PROFILE\tv1=commonmark+trusted+extensions+line_comments\tv2=gfm_spec+line_comments");
    let mut different = 0usize;
    for &(name, source) in cases() {
        let v1 = v1_html(source, true);
        let v2 = v2_html(source, true);
        if v1 == v2 {
            println!("CASE\t{name}\tOK");
        } else {
            different += 1;
            println!("CASE\t{name}\tDIFF\t{}\t{}", hex(&v1), hex(&v2));
        }
    }

    let list_with_comment = "- [target]: /url\n  // private\n  \"title\"\n\n[target]\n";
    let list_without_comment = "- [target]: /url\n  \"title\"\n\n[target]\n";
    for (name, source) in [
        ("list-definition-title-comments-off", list_with_comment),
        ("list-definition-title-comment-removed", list_without_comment),
    ] {
        let v1 = v1_html(source, false);
        let v2 = v2_html(source, false);
        let status = if v1 == v2 { "OK" } else { "DIFF" };
        println!("BASELINE\t{name}\t{status}\t{}\t{}", hex(&v1), hex(&v2));
    }

    println!("SUMMARY\t{}\t{}\t{different}", cases().len(), cases().len() - different);
    if different != 1 {
        std::process::exit(2);
    }
}
