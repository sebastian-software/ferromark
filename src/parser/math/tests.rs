#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use crate::allocator::Allocator;
use crate::parser::{Parser, ParserOptions};

use super::{math_block_close, scan_inline_math_close};

fn math_parser<'a>(allocator: &'a Allocator, source: &'a str) -> Parser<'a> {
    Parser::with_options(
        allocator,
        source,
        ParserOptions {
            math: true,
            ..ParserOptions::gfm()
        },
    )
}

/// The memo answers for every opener a scan passes, so it has to agree with
/// the plain scan from each of them — cold, and again once filled.
#[test]
fn memoized_inline_closers_agree_with_the_plain_scan() {
    for source in [
        "$a $a $a ",
        "$1 $1 $1 ",
        "$$a $$a $$a ",
        "$a$b$c$",
        "$$a$$b$$",
        "a $x$ b $y$ c",
        "$a `$` b$ $c$",
        "`$a$` $b$ `$c$`",
        "\\$a$ $b\\$ $c$",
        "$a 1$ $2 b$ $3$",
        "$",
        "$$",
        "$ $ $",
        "",
    ] {
        let allocator = Allocator::new();
        let parser = math_parser(&allocator, source);
        let bytes = source.as_bytes();
        for open_len in [1usize, 2] {
            for (from, _) in source.match_indices('$') {
                let from = from + open_len;
                if from > source.len() {
                    continue;
                }
                let expected = scan_inline_math_close(bytes, from, open_len);
                assert_eq!(
                    parser.inline_math_close(source, from, open_len),
                    expected,
                    "cold {source:?} from {from} at width {open_len}"
                );
                assert_eq!(
                    parser.inline_math_close(source, from, open_len),
                    expected,
                    "warm {source:?} from {from} at width {open_len}"
                );
            }
        }
    }
}

/// Same contract for the display-math terminator, which every `$$` line
/// asks about from its own offset.
#[test]
fn memoized_block_closers_agree_with_the_plain_scan() {
    for source in [
        "$$ a\n$$ a\n$$ a\n",
        "$$\na\n$$\n$$\nb\n$$\n",
        "$$ a\n$$\n",
        "\\$$\n$$\n",
        "no math here\n",
    ] {
        let allocator = Allocator::new();
        let parser = math_parser(&allocator, source);
        let bytes = source.as_bytes();
        for from in 0..=source.len() {
            let expected = math_block_close(bytes, from);
            assert_eq!(
                parser.math_block_close_from(from),
                expected,
                "cold {source:?} from {from}"
            );
            assert_eq!(
                parser.math_block_close_from(from),
                expected,
                "warm {source:?} from {from}"
            );
        }
    }
}
