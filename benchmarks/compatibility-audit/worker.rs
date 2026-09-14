//! Compatibility audit only: raw HTML and the inherited normalization side by side.
use ferromark::{Allocator, HtmlRenderer, HtmlRendererOptions, Parser, ParserOptions};
use serde_json::json;
use std::io::{self, BufRead, Write};

#[allow(dead_code)]
#[path = "../../crates/ferromark_renderer/tests/spec_support/normalize.rs"]
mod inherited;

fn render(source: &str, profile: &str) -> Result<String, String> {
    let mut parser =
        if profile.starts_with("gfm") { ParserOptions::gfm() } else { ParserOptions::default() };
    let mut html = HtmlRendererOptions::new();
    if !matches!(profile, "default" | "gfm-preset") {
        html.autolink_urls = false;
        html.autolink_target_blank = false;
        html.link_target_blank = false;
        if profile.starts_with("gfm") {
            parser.footnotes = false;
        }
        html.disallow_raw_html = profile == "gfm";
    }
    if profile == "gfm-no-tagfilter" {
        html.disallow_raw_html = false;
    }
    if profile == "mdx" {
        parser.mdx = true;
    }
    let arena = Allocator::for_source_len(source.len());
    Parser::with_options(&arena, source, parser)
        .parse()
        .map(|doc| HtmlRenderer::with_options(html).render(&doc))
        .map_err(|err| format!("{err:?}"))
}

fn main() {
    let mut output = io::BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        let request: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
        let source = request["markdown"].as_str().unwrap();
        let profile = request["profile"].as_str().unwrap();
        assert!(
            ["commonmark", "gfm", "gfm-no-tagfilter", "default", "gfm-preset", "mdx"]
                .contains(&profile)
        );
        let result = std::panic::catch_unwind(|| render(source, profile));
        let (html, error) = match result {
            Ok(Ok(html)) => (html, None),
            Ok(Err(err)) => (String::new(), Some(err)),
            Err(_) => (String::new(), Some("PANIC".to_string())),
        };
        let expected = request["html"].as_str().unwrap_or("");
        writeln!(output, "{}", json!({"html":html, "error":error,
            "inherited_equal": error.is_none() && inherited::normalize_html(&html) == inherited::normalize_html(expected),
            "inherited_actual": inherited::normalize_html(&html),
            "inherited_expected": inherited::normalize_html(expected)})).unwrap();
        output.flush().unwrap();
    }
}
