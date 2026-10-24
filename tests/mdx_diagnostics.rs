#![cfg(feature = "jsx")]

use ferromark::{Allocator, MdxDiagnosticKind as Kind, ParseErrorKind, Parser, ParserOptions};
use std::num::NonZeroUsize;

fn options() -> ParserOptions {
    ParserOptions {
        mdx: true,
        mdx_compatible: true,
        ..ParserOptions::gfm()
    }
}

#[test]
fn mismatches_report_complete_tags_and_related_openers() {
    for source in [
        "<Card>\n<Badge>\n</Card>\n",
        "é前\n\n<Card>\r\n<Badge>\r\n</Card>\r\n",
        "> <Card>\n> <Badge>\n> </Card>\n",
        "- <Card>\r\n  <Badge>\r\n  </Card>\r\n",
        "> - <Card>\n>   <Badge>\n>   </Card>\n",
        "<Outer>\n  > <Card>\n  > <Badge>\n  > </Card>\n</Outer>\n",
        "é <Card><Badge></Card>",
        "\u{feff}\0\n\n> <Card>\n> <Badge>\n> </Card>\n",
    ] {
        let arena = Allocator::new();
        let error = Parser::with_options(&arena, source, options())
            .parse()
            .unwrap_err();
        assert!(matches!(error.kind(), ParseErrorKind::InvalidMdx { .. }));
        let detail = error.mdx_diagnostic().unwrap();
        assert_eq!(
            detail.kind,
            Kind::MismatchedClosingTag,
            "{source:?}: {error}"
        );
        assert_eq!(detail.span, error.span());
        assert_eq!(detail.span.source_text(source), "</Card>", "{source:?}");
        assert_eq!(
            detail.related_span.unwrap().source_text(source),
            "<Badge>",
            "{source:?}"
        );
    }
}

#[test]
fn collects_only_independent_standalone_unmatched_closers() {
    for source in [
        "</First>\n\n</Second>\n",
        "é\n\n</First>\r\n\r\n</Second>\r\n",
        "\u{feff}\0\n\n</First>\n\n</Second>\n",
    ] {
        let arena = Allocator::new();
        let failed = Parser::with_options(&arena, source, options())
            .parse_with_diagnostics(NonZeroUsize::new(8).unwrap())
            .unwrap_err();
        assert!(!failed.stopped_early, "{failed:?}");
        assert_eq!(failed.errors.len(), 2);
        for (error, tag) in failed.errors.iter().zip(["</First>", "</Second>"]) {
            let detail = error.mdx_diagnostic().unwrap();
            assert_eq!(detail.kind, Kind::UnexpectedClosingTag);
            assert_eq!(detail.span.source_text(source), tag);
            assert_eq!(error.span(), detail.span);
            assert!(detail.related_span.is_none());
        }
        let first = Parser::with_options(&arena, source, options())
            .parse()
            .unwrap_err();
        assert_eq!(first.span(), failed.errors[0].span());
    }
}

#[test]
fn recovery_stops_at_ambiguity_and_respects_the_limit() {
    for (source, kind) in [
        ("x </First>\n\n</Second>", Kind::UnexpectedClosingTag),
        ("> </First>\n\n</Second>", Kind::UnexpectedClosingTag),
        ("- </First>\n\n</Second>", Kind::UnexpectedClosingTag),
        ("</First bad>\n\n</Second>", Kind::InvalidClosingTag),
        ("</A..B>\n\n</Second>", Kind::InvalidClosingTag),
        (
            "<Card><Badge></Card>\n\n</Second>",
            Kind::MismatchedClosingTag,
        ),
        ("<Card>\ntext", Kind::UnclosedElement),
        ("{x +}\n\n</Second>", Kind::InvalidExpression),
        ("<Card bad={x +} />\n\n</Second>", Kind::InvalidOpeningTag),
        (
            "export const value: string = 'x'\n\n</Second>",
            Kind::InvalidModule,
        ),
    ] {
        let arena = Allocator::new();
        let failed = Parser::with_options(&arena, source, options())
            .parse_with_diagnostics(NonZeroUsize::new(8).unwrap())
            .unwrap_err();
        assert_eq!(failed.errors.len(), 1, "{source:?}");
        assert!(failed.stopped_early, "{source:?}");
        assert_eq!(
            failed.errors[0].mdx_diagnostic().unwrap().kind,
            kind,
            "{source:?}"
        );
        if kind == Kind::UnclosedElement {
            assert_eq!(
                failed.errors[0]
                    .mdx_diagnostic()
                    .unwrap()
                    .related_span
                    .unwrap()
                    .source_text(source),
                "<Card>"
            );
        }
    }
    let arena = Allocator::new();
    let fragments = Parser::with_options(&arena, "</>\n\n</Å名>", options())
        .parse_with_diagnostics(NonZeroUsize::new(4).unwrap())
        .unwrap_err();
    assert_eq!(fragments.errors.len(), 2);
    assert!(!fragments.stopped_early);
    let failed = Parser::with_options(&arena, "</First>\n\n</Second>", options())
        .parse_with_diagnostics(NonZeroUsize::new(1).unwrap())
        .unwrap_err();
    assert_eq!(failed.errors.len(), 1);
    assert!(failed.stopped_early);
}

#[test]
fn collection_keeps_successful_ast_validation_and_nesting_contracts() {
    let source = "\u{feff}<Card>\r\n  > é **body** {value}\r\n</Card>\r\n";
    let arena = Allocator::new();
    let plain = Parser::with_options(&arena, source, options())
        .parse()
        .unwrap();
    let collected = Parser::with_options(&arena, source, options())
        .parse_with_diagnostics(NonZeroUsize::new(4).unwrap())
        .unwrap();
    assert_eq!(format!("{plain:?}"), format!("{collected:?}"));
    let failed = Parser::with_options(
        &arena,
        "> > > body",
        ParserOptions {
            max_nesting_depth: 2,
            ..options()
        },
    )
    .parse_with_diagnostics(NonZeroUsize::new(4).unwrap())
    .unwrap_err();
    assert!(failed.stopped_early);
    assert!(matches!(
        failed.errors[0].kind(),
        ParseErrorKind::NestingTooDeep { .. }
    ));
    assert!(failed.errors[0].mdx_diagnostic().is_none());
}

#[test]
fn related_openers_include_fragment_and_javascript_attribute_boundaries() {
    for (source, expected) in [
        ("<Card><></Card>", "<>"),
        ("<>", "<>"),
        (
            r#"<Card><Badge label=">" check={a < b ? a : b}></Card>"#,
            r#"<Badge label=">" check={a < b ? a : b}>"#,
        ),
        (
            "<Card title={a < b ? a : b}>",
            "<Card title={a < b ? a : b}>",
        ),
        (
            "<Card>\n<Badge\n label=\"<inside>\"\n>\n</Card>",
            "<Badge\n label=\"<inside>\"\n>",
        ),
    ] {
        let arena = Allocator::new();
        let error = Parser::with_options(&arena, source, options())
            .parse()
            .unwrap_err();
        assert_eq!(
            error
                .mdx_diagnostic()
                .unwrap()
                .related_span
                .unwrap()
                .source_text(source),
            expected,
            "{source:?}: {error}"
        );
    }
}
