//! Hostile inputs must return `Result` errors or a document, never abort.

use std::panic::{AssertUnwindSafe, catch_unwind};

use ferromark::allocator::Allocator;
use ferromark::parser::{ParseErrorKind, Parser, ParserOptions};

fn parse_or_err(source: &str, options: ParserOptions) -> Result<(), String> {
    let allocator = Allocator::new();
    Parser::with_options(&allocator, source, options)
        .parse()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[test]
fn malformed_markdown_does_not_abort() {
    let options = ParserOptions::gfm();
    let cases = [
        "",
        "*",
        "**",
        "[",
        "](",
        "![",
        "`",
        "```",
        "|",
        "> ",
        "- \n- \n- ",
        "<script>",
        "\\",
        "\u{1F600}***\u{3042}",
        &"> ".repeat(8),
        &format!("{}emphasis*", "*".repeat(32)),
    ];

    for source in cases {
        let outcome = catch_unwind(AssertUnwindSafe(|| parse_or_err(source, options.clone())));
        assert!(outcome.is_ok(), "parser aborted on {source:?}");
    }
}

#[test]
fn nesting_limit_returns_an_error_instead_of_aborting() {
    let source = "> ".repeat(120) + "too deep";
    let allocator = Allocator::new();
    let error = Parser::with_options(&allocator, &source, ParserOptions::gfm())
        .parse()
        .expect_err("deeply nested quotes should fail closed");
    assert!(matches!(
        error.kind(),
        ParseErrorKind::NestingTooDeep { max_depth: 100, .. }
    ));
}

#[test]
fn deep_inline_brackets_return_an_error_instead_of_aborting() {
    // Issue #349: 16 KB of `[` overflowed an 8 MB stack because link text,
    // image alt text and inline notes re-entered inline parsing without
    // counting. A stack overflow aborts instead of unwinding, so
    // `catch_unwind` here proves nothing on its own — the assertion is that
    // the cap turns each of these into an ordinary `Err`.
    let depth = 20_000;
    let cases = [
        "[".repeat(depth) + "a" + &"]".repeat(depth),
        "[".repeat(depth) + "a" + &"](u)".repeat(depth),
        "![".repeat(depth) + "a" + &"](u)".repeat(depth),
        "[ ".repeat(depth) + &"]".repeat(depth),
        "^[".repeat(depth) + "a" + &"]".repeat(depth),
    ];

    for source in cases {
        let error = parse_or_err(&source, ParserOptions::gfm())
            .expect_err("deeply nested inline brackets should fail closed");
        assert!(
            error.contains("nesting too deep"),
            "expected the nesting cap, got {error}"
        );
    }
}

#[test]
fn deep_emphasis_nesting_returns_an_error_instead_of_aborting() {
    // Issue #371: pairing `*`×n `a` `*`×n is iterative, so the parse itself
    // survived; the tree it built was n/2 levels deep and overflowed the
    // stack when the renderer walked it. 4 KB was enough for a 1 MB stack.
    let count = 20_000;
    let cases = [
        "*".repeat(count) + "a" + &"*".repeat(count),
        "_".repeat(count) + "a" + &"_".repeat(count),
        "**".repeat(count) + "a" + &"**".repeat(count),
        " ~~x".repeat(count) + " a " + &"x~~ ".repeat(count),
    ];

    for source in cases {
        let error = parse_or_err(&source, ParserOptions::gfm())
            .expect_err("deeply nested emphasis should fail closed");
        assert!(
            error.contains("nesting too deep"),
            "expected the nesting cap, got {error}"
        );
    }
}
