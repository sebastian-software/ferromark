//! Pinned reference outputs for the deterministic complex-input oracle corpus.
#[path = "spec_support/normalize.rs"]
mod normalize;

use ferromark::allocator::Allocator;
use ferromark::parser::{Parser, ParserOptions};
use ferromark::renderer::{HtmlRenderer, HtmlRendererOptions};

#[test]
fn complex_cmark_oracle_cases_match_reference_output() {
    // Frozen cmark 0.31.1 / cmark-gfm 0.29.0.gfm.13 output. The report records
    // source revisions, build flags, binary hashes, and profile configuration.
    // Keep this independent of the fixture generator so corpus changes cannot
    // silently rewrite expected HTML. Specification tests remain separate.
    compare_reference_cases(
        include_str!(
            "../../../docs/reports/2026-09-14-correctness-fixes/raw/cmark-oracle/results.json"
        ),
        106,
    );
}

#[test]
fn tilde_binding_combinations_match_reference_output() {
    compare_reference_cases(
        include_str!(
            "../../../docs/reports/2026-09-14-reference-compatibility/raw/bindings-before-aligned/results.json"
        ),
        256,
    );
}

fn compare_reference_cases(json: &str, count: usize) {
    let cases: serde_json::Value = serde_json::from_str(json).unwrap();
    let cases = cases.as_array().unwrap();
    assert_eq!(cases.len(), count);
    let mut failures = Vec::new();
    for case in cases {
        let source = case["markdown"].as_str().unwrap();
        let gfm = case["profile"] == "gfm";
        let parser = if gfm {
            ParserOptions::gfm_spec()
        } else {
            ParserOptions::commonmark()
        };
        let options = if gfm {
            HtmlRendererOptions::gfm()
        } else {
            HtmlRendererOptions::commonmark()
        };
        let arena = Allocator::new();
        let doc = Parser::with_options(&arena, source, parser)
            .parse()
            .unwrap();
        let html = HtmlRenderer::with_options(options).render(&doc);
        let reference = case["cmark_actual"].as_str().unwrap();
        // These two pinned cmark-gfm cases emit forbidden nested anchors.
        // Preserve that raw reference output, but assert the GFM links rule
        // instead of copying the oracle bug (see the correction report).
        let expected = match case["id"].as_str().unwrap() {
            "binding-1-31-link" | "binding-2-31-link" => {
                assert!(reference.contains("<del><a href=\"u\">"));
                "<p>[<del><a href=\"u\">x</a> [x]</del>](target)</p>\n"
            }
            _ => reference,
        };
        // cmark spells the two emitted checkbox boolean attributes with
        // empty values. Canonicalize only these exact generated input tags;
        // leave raw HTML, text, and all general normalizer rules untouched.
        let expected = expected
            .replace(
                "<input type=\"checkbox\" checked=\"\" disabled=\"\" />",
                "<input type=\"checkbox\" checked disabled>",
            )
            .replace(
                "<input type=\"checkbox\" disabled=\"\" />",
                "<input type=\"checkbox\" disabled>",
            );
        if normalize::normalize_html(&html) != normalize::normalize_html(&expected) {
            failures.push(format!(
                "{}: {source:?}\nexpected: {expected:?}\nactual: {html:?}",
                case["id"]
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
