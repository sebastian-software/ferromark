//! Differential tests for the plain (unannotated) fenced-code path.
//!
//! `write_plain_code_block` short-circuits info tokens that
//! `plain_code_block_language` proves are already their own normalized, escaped
//! class body, and assembles the surrounding markup from merged literals.
//! `oracle` below writes the same fence the way the renderer did before either
//! change — tag, delimiter, and attribute name pushed separately, the language
//! always run through the metadata tokenizer and the escaper — so every case
//! here asserts the two produce the same bytes.

use super::*;
use crate::allocator::Allocator;
use crate::ast::Span;
use crate::parser::Parser;
use crate::renderer::html::code_annotations::normalize_code_block_info;
use crate::renderer::html::escape::write_escaped_into;
use crate::renderer::html::{CodeAnnotationSyntax, HtmlRendererOptions};

/// A body carrying bytes the escaper rewrites, so every case also covers the
/// markup around the language.
const BODY: &str = "let ok = 1 < 2 && \"3\" > '2';\n";

/// A non-empty span, so the `source_spans` profile actually emits the attribute.
const SPAN: Span = Span { start: 7, end: 41 };

/// Language tokens that should take the shortcut: the bare names that dominate
/// documentation corpora, plus punctuation-carrying names that are still plain.
const BARE_LANGUAGES: &[&str] = &[
    "ts",
    "js",
    "tsx",
    "jsx",
    "bash",
    "sh",
    "shell",
    "console",
    "rust",
    "json",
    "jsonc",
    "json5",
    "yaml",
    "yml",
    "toml",
    "html",
    "css",
    "scss",
    "md",
    "markdown",
    "mdx",
    "vue",
    "svelte",
    "python",
    "py",
    "go",
    "java",
    "kotlin",
    "swift",
    "ruby",
    "php",
    "sql",
    "diff",
    "text",
    "plaintext",
    "c",
    "c++",
    "c#",
    "f#",
    "objective-c",
    "vim.rc",
    "docker-compose.yml",
    "ts]",
    "js}",
    "a,b",
    "annotate=1",
    "ts_v2",
    "ts-2",
    "ts.2",
    "~tilde~",
    "100%",
    "x@y",
    "a\\b",
    "a/b",
    "!bang",
    "$dollar",
    "(paren)",
    "*star*",
    "+plus+",
    "?query",
    "^caret",
    "|pipe|",
    "`tick`",
];

/// Info tokens the shortcut must decline, one per rule it encodes.
const METADATA_TOKENS: &[&str] = &[
    "",
    " ",
    "\t",
    "\n",
    "\u{b}",
    "\u{c}",
    "\r",
    "\u{a0}",
    "\u{2028}",
    "\u{3000}",
    " ts ",
    "\tts\t",
    "ts\u{a0}",
    "ts{1,3}",
    "js{1}",
    "js{1-4,7}",
    "{1,3}",
    "[title]",
    "ts[config.ts]",
    "[]",
    "ts:line-numbers",
    "ts:line-numbers=2",
    "ts:no-line-numbers",
    "ts:line-links",
    "ts:line-links=auth-loader",
    "ts:wrap",
    "ts:wrap-lines",
    "ts:no-wrap",
    "ts:nowrap",
    "ts:no-wrap-lines",
    ":line-numbers",
    ":no-line-numbers",
    "c:",
    "c::",
    "ts:not-a-directive",
    "ts:line-numbers{1,3}[config.ts]",
    "ts{1,3}:line-numbers",
    "ts&amp",
    "ts<script>",
    "ts\"quoted\"",
    "ts'quoted'",
    "ts&<>\"'",
    "日本語",
    "café",
    "emoji🦀",
    "{",
    "[",
    ":",
    "ü",
];

/// Full info strings for the end-to-end matrix. These go through the fence
/// parser, so they carry the `lang`/`meta` split real documents produce.
const INFO_STRINGS: &[&str] = &[
    "",
    "ts",
    "rust",
    "ts {1,3}",
    "ts [config.ts]",
    "ts:line-numbers",
    "ts:line-numbers=4",
    "ts:no-line-numbers",
    "ts:line-numbers {1,3} [config.ts]",
    "ts annotate=\"highlight:1;warning:2\"",
    "ts markers=\"error:2\"",
    "ts{1} [a title] :line-links=anchor :wrap",
    "{1,3}",
    "[only-a-title]",
    ":line-numbers",
    "c++ {2}",
    "日本語 {1}",
];

/// Writes one plain fence the way the renderer did before the shortcut and the
/// merged literals: every piece of markup pushed separately, and the language
/// always taken through the metadata tokenizer and the HTML escaper.
fn oracle(lang: Option<&str>, source_spans: bool, code_fence_metadata: bool) -> String {
    let language = if code_fence_metadata {
        normalize_code_block_language(lang)
    } else {
        lang.map(str::trim).filter(|lang| !lang.is_empty())
    };

    let mut expected = String::new();
    expected.push_str("<pre");
    if source_spans && SPAN.start != SPAN.end {
        expected.push_str(" data-source-span=\"");
        expected.push_str(&SPAN.start.to_string());
        expected.push('-');
        expected.push_str(&SPAN.end.to_string());
        expected.push('"');
    }
    expected.push_str("><code");
    if let Some(language) = language {
        expected.push_str(" class=\"language-");
        write_escaped_into(&mut expected, language);
        expected.push('"');
    }
    expected.push('>');
    write_escaped_into(&mut expected, BODY);
    expected.push_str("</code></pre>\n");
    expected
}

/// Holds one renderer per option combination the plain fence path branches on,
/// so the generated cases do not rebuild renderer state per token.
struct Fences {
    renderers: Vec<(bool, bool, HtmlRenderer)>,
}

impl Fences {
    fn new() -> Self {
        let mut renderers = Vec::new();
        for source_spans in [false, true] {
            for code_fence_metadata in [true, false] {
                let mut renderer = HtmlRenderer::new();
                renderer.options.source_spans = source_spans;
                renderer.options.code_fence_metadata = code_fence_metadata;
                renderers.push((source_spans, code_fence_metadata, renderer));
            }
        }
        Self { renderers }
    }

    fn assert_matches_oracle(&mut self, lang: Option<&str>) {
        let code_block = CodeBlock {
            lang,
            meta: None,
            value: BODY,
            span: SPAN,
        };

        for (source_spans, code_fence_metadata, renderer) in &mut self.renderers {
            renderer.output.clear();
            renderer.write_plain_code_block(&code_block);
            assert_eq!(
                renderer.output,
                oracle(lang, *source_spans, *code_fence_metadata),
                "fence for {lang:?} with source_spans={source_spans} \
                 code_fence_metadata={code_fence_metadata}"
            );
        }
    }
}

/// Deterministic xorshift64 so the random cases are reproducible without a
/// development dependency.
struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        let mut state = self.0;
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        self.0 = state;
        state
    }

    fn below(&mut self, bound: usize) -> usize {
        (self.next_u64() % bound as u64) as usize
    }
}

#[test]
fn bare_languages_take_the_shortcut() {
    let mut fences = Fences::new();
    for language in BARE_LANGUAGES {
        assert_eq!(
            plain_code_block_language(language),
            Some(*language),
            "{language:?} should be recognized as a plain language token"
        );
        fences.assert_matches_oracle(Some(language));
    }
}

#[test]
fn metadata_tokens_fall_back_to_the_general_route() {
    let mut fences = Fences::new();
    for token in METADATA_TOKENS {
        assert_eq!(
            plain_code_block_language(token),
            None,
            "{token:?} must not be treated as a plain language token"
        );
        fences.assert_matches_oracle(Some(token));
    }
}

#[test]
fn absent_language_matches_oracle() {
    Fences::new().assert_matches_oracle(None);
}

#[test]
fn plain_language_tokens_survive_escaping_unchanged() {
    for language in BARE_LANGUAGES {
        let Some(plain) = plain_code_block_language(language) else {
            continue;
        };
        let mut escaped = String::new();
        write_escaped_into(&mut escaped, plain);
        assert_eq!(
            escaped, plain,
            "{plain:?} is emitted unescaped, so escaping it must be the identity"
        );
    }
}

#[test]
fn bundled_fixture_info_tokens_match_oracle() {
    let mut fences = Fences::new();
    let tokens = bundled_fence_info_tokens();
    // The specification fixtures write most examples as indented blocks, so the
    // harvest is small; these two bare names pin the extractor against a silently
    // empty scan.
    assert!(
        tokens.iter().any(|token| token == "markdown")
            && tokens.iter().any(|token| token == "ruby"),
        "fence info extraction looks broken, harvested {tokens:?}"
    );

    let mut plain = 0usize;
    for token in &tokens {
        if plain_code_block_language(token).is_some() {
            plain += 1;
        }
        fences.assert_matches_oracle(Some(token));
    }

    assert!(
        plain > 0,
        "expected at least one bundled fixture fence to be a bare language name"
    );
}

#[test]
fn random_info_tokens_match_oracle() {
    // The alphabet carries every byte the shortcut rejects — the tokenizer's
    // `{`, `[`, `:`, the escaper's `&<>"'`, ASCII and non-ASCII whitespace, and
    // multi-byte code points — so the generator keeps landing on both sides of
    // the decision.
    const ALPHABET: &[char] = &[
        '{', '}', '[', ']', ':', '=', ',', '-', '&', '<', '>', '"', '\'', ' ', '\t', '\n', '\r',
        '\u{b}', '\u{c}', '\u{a0}', '\u{2028}', 'a', 'b', 'l', 'i', 'n', 'e', 'u', 'm', 'w', 'r',
        'p', 's', 'o', 't', '0', '1', '9', '.', '/', '_', 'é', '日', '🦀',
    ];

    let mut fences = Fences::new();
    let mut rng = Rng(0x2026_0915_c0de_b10c);
    let mut token = String::new();
    let mut plain = 0usize;
    let mut rejected = 0usize;

    for _ in 0..20_000 {
        token.clear();
        for _ in 0..rng.below(12) {
            token.push(ALPHABET[rng.below(ALPHABET.len())]);
        }

        if plain_code_block_language(&token).is_some() {
            plain += 1;
        } else {
            rejected += 1;
        }
        fences.assert_matches_oracle(Some(&token));
    }

    assert!(
        plain > 100 && rejected > 100,
        "the generator should reach both sides of the decision, saw {plain} plain and {rejected} rejected"
    );
}

#[test]
fn annotation_paths_still_derive_the_language_from_normalized_info() {
    // The `code_annotations` route reads its language from
    // `normalize_code_block_info`, which the shortcut does not touch. Render the
    // matrix through every syntax and confirm the emitted class still comes from
    // that routine.
    for syntax in [
        CodeAnnotationSyntax::Attribute,
        CodeAnnotationSyntax::VitePress,
        CodeAnnotationSyntax::Both,
    ] {
        for info in INFO_STRINGS {
            let source = format!("```{info}\nconst first = true;\nconst second = false;\n```\n");
            let allocator = Allocator::new();
            let document = Parser::new(&allocator, &source).parse().expect("parses");
            let html = HtmlRenderer::with_options(HtmlRendererOptions {
                code_annotations: true,
                code_annotation_syntax: syntax,
                ..Default::default()
            })
            .render(&document);

            let (lang, meta) = split_fence_info(info);
            let expected = match normalize_code_block_info(lang, meta).language {
                Some(language) => {
                    let mut expected = String::from("<code class=\"language-");
                    write_escaped_into(&mut expected, &language);
                    expected.push('"');
                    expected
                }
                None => String::from("<code>"),
            };

            assert!(
                html.contains(&expected),
                "{syntax:?} rendering of ```{info} should contain {expected:?}, got {html}"
            );
        }
    }
}

#[test]
fn default_rendering_matches_the_normalized_language() {
    for info in INFO_STRINGS {
        let source = format!("```{info}\nconst first = true;\n```\n");
        let allocator = Allocator::new();
        let document = Parser::new(&allocator, &source).parse().expect("parses");
        let html = HtmlRenderer::new().render(&document);

        let (lang, _) = split_fence_info(info);
        let expected = match normalize_code_block_language(lang) {
            Some(language) => {
                let mut expected = String::from("<code class=\"language-");
                write_escaped_into(&mut expected, language);
                expected.push_str("\">");
                expected
            }
            None => String::from("<code>"),
        };
        assert!(
            html.contains(&expected),
            "default rendering of ```{info} should contain {expected:?}, got {html}"
        );
    }
}

/// Mirrors the fence parser's info-string split: language up to the first
/// space, metadata after it.
fn split_fence_info(info: &str) -> (Option<&str>, Option<&str>) {
    let info = info.trim();
    if info.is_empty() {
        (None, None)
    } else if let Some(space) = info.find(' ') {
        (Some(&info[..space]), Some(&info[space + 1..]))
    } else {
        (Some(info), None)
    }
}

/// Collects the first info-string token of every fenced block in the bundled
/// specification fixtures and the bundled changelog benchmark input.
fn bundled_fence_info_tokens() -> Vec<String> {
    const FIXTURES: &[&str] = &[
        "tests/spec_fixtures/commonmark-0.31.2-spec.txt",
        "tests/spec_fixtures/gfm-extensions-spec.txt",
        "benches/fixtures/upstream-changelog.md",
    ];

    let mut tokens = Vec::new();
    for relative in FIXTURES {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));

        for line in text.lines() {
            let trimmed = line.trim_start();
            let Some(rest) = trimmed
                .strip_prefix("```")
                .or_else(|| trimmed.strip_prefix("~~~"))
            else {
                continue;
            };
            let info = rest.trim_start_matches(['`', '~']).trim();
            if info.is_empty() {
                continue;
            }
            let token = info.split(' ').next().unwrap_or(info);
            tokens.push(token.to_string());
        }
    }

    tokens.sort_unstable();
    tokens.dedup();
    tokens
}
