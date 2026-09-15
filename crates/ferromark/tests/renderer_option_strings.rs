//! The `Cow` representation of renderer option strings must be invisible.
//!
//! `HtmlRendererOptions` holds borrowed-or-owned strings so the documented
//! defaults cost nothing. Whether a value is borrowed or owned is an allocation
//! decision only: rendered HTML must be byte-identical either way, defaults must
//! equal the same values written out explicitly, and empty values must keep
//! meaning "empty" rather than "use the default".

use std::borrow::Cow;

use ferromark::{
    Allocator, CodeAnnotationSyntax, HtmlRenderer, HtmlRendererOptions, Parser, ParserOptions,
};

/// Exercises every rendering path that reads a string-valued option: hard
/// breaks, root-absolute and `.md` link conversion, source-relative links,
/// fenced code annotation metadata, and bare-URL autolinking.
const SOURCE: &str = "\
# Options and their strings

A line ending in two spaces  \nfollowed by a hard break, and a backslash break\\\nlike this one.

See [the guide](/guide/setup.md), [this directory](./nested/index.md), and
[an anchor](/guide/setup.md#step-two).

An image lives at ![diagram](/assets/diagram.png).

Bare URLs such as http://example.com/a and https://example.org/b?x=1&y=2 appear
in prose, next to mailto:someone@example.com which is not a default scheme.

<a href=\"/raw/html.md\">raw html attribute</a>

```ts markers=\"warning:2\"
const ok = true;
const maybe = false;
```

```ts annotate=\"warning:1\"
const other = 1;
```
";

fn render(options: HtmlRendererOptions) -> String {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, SOURCE, ParserOptions::default())
        .parse()
        .expect("fixture parses");
    HtmlRenderer::with_options(options).render(&document)
}

/// Rewrites every string-valued field into an owned `Cow` holding the same
/// bytes, leaving all other settings alone.
fn into_owned_representation(options: &HtmlRendererOptions) -> HtmlRendererOptions {
    let mut owned = options.clone();
    owned.soft_break = Cow::Owned(options.soft_break.to_string());
    owned.hard_break = Cow::Owned(options.hard_break.to_string());
    owned.base_url = Cow::Owned(options.base_url.to_string());
    owned.source_path = Cow::Owned(options.source_path.to_string());
    owned.code_annotation_meta_key = Cow::Owned(options.code_annotation_meta_key.to_string());
    owned.autolink_patterns = Cow::Owned(
        options
            .autolink_patterns
            .iter()
            .map(|pattern| Cow::Owned(pattern.to_string()))
            .collect(),
    );
    assert!(matches!(owned.soft_break, Cow::Owned(_)));
    assert!(matches!(owned.autolink_patterns, Cow::Owned(_)));
    owned
}

/// The option shapes whose HTML must not depend on how the strings are stored.
fn combinations() -> Vec<(String, HtmlRendererOptions)> {
    let mut cases = vec![
        ("defaults".to_string(), HtmlRendererOptions::new()),
        ("commonmark".to_string(), HtmlRendererOptions::commonmark()),
        ("gfm".to_string(), HtmlRendererOptions::gfm()),
    ];

    for base in ["/", "", "/docs/", "/docs"] {
        for source_path in ["", "guide/index.md", "guide/setup.md"] {
            for xhtml in [false, true] {
                cases.push((
                    format!("links base={base:?} source={source_path:?} xhtml={xhtml}"),
                    HtmlRendererOptions {
                        convert_md_links: true,
                        base_url: base.into(),
                        source_path: source_path.into(),
                        xhtml,
                        hard_break: if xhtml { "<br />\n" } else { "<br>\n" }.into(),
                        ..HtmlRendererOptions::new()
                    },
                ));
            }
        }
    }

    for syntax in [
        CodeAnnotationSyntax::Attribute,
        CodeAnnotationSyntax::VitePress,
        CodeAnnotationSyntax::Both,
    ] {
        for key in ["annotate", "markers", ""] {
            cases.push((
                format!("annotations key={key:?} syntax={syntax:?}"),
                HtmlRendererOptions {
                    code_annotations: true,
                    code_annotation_meta_key: key.into(),
                    code_annotation_syntax: syntax,
                    ..HtmlRendererOptions::new()
                },
            ));
        }
    }

    for (label, patterns) in [
        ("default", HtmlRendererOptions::new().autolink_patterns),
        ("empty", Cow::Owned(Vec::new())),
        ("single", Cow::Owned(vec![Cow::Borrowed("mailto:")])),
        (
            "many",
            Cow::Owned(vec![
                Cow::Borrowed("http://"),
                Cow::Borrowed("https://"),
                Cow::Borrowed("ftp://"),
                Cow::Borrowed("mailto:"),
                Cow::Borrowed("tel:"),
            ]),
        ),
    ] {
        for autolink_urls in [false, true] {
            for autolink_target_blank in [false, true] {
                cases.push((
                    format!(
                        "autolink patterns={label} urls={autolink_urls} \
                         blank={autolink_target_blank}"
                    ),
                    HtmlRendererOptions {
                        autolink_urls,
                        autolink_target_blank,
                        autolink_patterns: patterns.clone(),
                        ..HtmlRendererOptions::new()
                    },
                ));
            }
        }
    }

    for sanitize in [false, true] {
        for disallow_raw_html in [false, true] {
            for source_spans in [false, true] {
                cases.push((
                    format!(
                        "output sanitize={sanitize} tagfilter={disallow_raw_html} \
                         spans={source_spans}"
                    ),
                    HtmlRendererOptions {
                        sanitize,
                        disallow_raw_html,
                        source_spans,
                        convert_md_links: true,
                        base_url: "/site/".into(),
                        source_path: "guide/index.md".into(),
                        ..HtmlRendererOptions::new()
                    },
                ));
            }
        }
    }

    cases
}

#[test]
fn borrowed_and_owned_strings_render_identical_html() {
    for (label, options) in combinations() {
        let owned = into_owned_representation(&options);
        let borrowed_html = render(options);
        let owned_html = render(owned);
        assert_eq!(
            borrowed_html, owned_html,
            "HTML differs between borrowed and owned option strings for: {label}"
        );
    }
}

#[test]
fn a_clone_renders_exactly_what_the_original_does() {
    for (label, options) in combinations() {
        let clone = options.clone();
        assert_eq!(
            render(options),
            render(clone),
            "HTML differs between an options value and its clone for: {label}"
        );
    }
}

#[test]
fn the_default_constructors_agree_with_explicit_defaults() {
    let allocator = Allocator::new();
    let document = Parser::with_options(&allocator, SOURCE, ParserOptions::default())
        .parse()
        .expect("fixture parses");

    let plain = HtmlRenderer::new().render(&document);
    let from_options = HtmlRenderer::with_options(HtmlRendererOptions::new()).render(&document);
    assert_eq!(
        plain, from_options,
        "`HtmlRenderer::new()` and `with_options(HtmlRendererOptions::new())` diverged"
    );

    // The documented defaults, written out by hand as owned strings.
    let spelled_out = HtmlRenderer::with_options(HtmlRendererOptions {
        soft_break: Cow::Owned(String::from("\n")),
        hard_break: Cow::Owned(String::from("<br>\n")),
        base_url: Cow::Owned(String::from("/")),
        source_path: Cow::Owned(String::new()),
        code_annotation_meta_key: Cow::Owned(String::from("annotate")),
        autolink_patterns: Cow::Owned(vec![
            Cow::Owned(String::from("http://")),
            Cow::Owned(String::from("https://")),
        ]),
        ..HtmlRendererOptions::new()
    })
    .render(&document);
    assert_eq!(
        plain, spelled_out,
        "the borrowed defaults do not match the documented values"
    );
}

#[test]
fn an_empty_base_url_is_not_the_default_base_url() {
    let with_root = render(HtmlRendererOptions {
        convert_md_links: true,
        base_url: "/".into(),
        ..HtmlRendererOptions::new()
    });
    let with_empty = render(HtmlRendererOptions {
        convert_md_links: true,
        base_url: "".into(),
        ..HtmlRendererOptions::new()
    });
    assert_ne!(
        with_root, with_empty,
        "an empty `base_url` was silently replaced by the default"
    );
    assert!(with_root.contains("href=\"/guide/setup/index.html\""));
    assert!(with_empty.contains("href=\"guide/setup/index.html\""));

    let with_owned_empty = render(HtmlRendererOptions {
        convert_md_links: true,
        base_url: Cow::Owned(String::new()),
        ..HtmlRendererOptions::new()
    });
    assert_eq!(with_empty, with_owned_empty);
}

#[test]
fn an_empty_source_path_is_not_treated_as_an_index_file() {
    let from_index = render(HtmlRendererOptions {
        convert_md_links: true,
        source_path: "guide/index.md".into(),
        ..HtmlRendererOptions::new()
    });
    let from_empty = render(HtmlRendererOptions {
        convert_md_links: true,
        source_path: "".into(),
        ..HtmlRendererOptions::new()
    });
    assert_ne!(
        from_index, from_empty,
        "an empty `source_path` behaved like an index file"
    );
    assert_eq!(
        from_empty,
        render(HtmlRendererOptions {
            convert_md_links: true,
            source_path: Cow::Owned(String::new()),
            ..HtmlRendererOptions::new()
        })
    );
}

#[test]
fn an_empty_autolink_pattern_list_disables_autolinking() {
    let defaults = render(HtmlRendererOptions {
        autolink_urls: true,
        ..HtmlRendererOptions::new()
    });
    assert!(
        defaults.contains("<a href=\"http://example.com/a\""),
        "the default patterns stopped autolinking"
    );

    for empty in [
        Cow::Borrowed(&[] as &[Cow<'static, str>]),
        Cow::Owned(Vec::new()),
    ] {
        let html = render(HtmlRendererOptions {
            autolink_urls: true,
            autolink_patterns: empty,
            ..HtmlRendererOptions::new()
        });
        assert!(
            !html.contains("<a href=\"http://example.com/a\""),
            "an empty pattern list still autolinked: {html}"
        );
        assert_ne!(defaults, html);
    }
}

#[test]
fn an_empty_annotation_meta_key_is_not_the_default_key() {
    let default_key = render(HtmlRendererOptions {
        code_annotations: true,
        ..HtmlRendererOptions::new()
    });
    let empty_key = render(HtmlRendererOptions {
        code_annotations: true,
        code_annotation_meta_key: "".into(),
        ..HtmlRendererOptions::new()
    });
    assert_ne!(
        default_key, empty_key,
        "an empty `code_annotation_meta_key` fell back to the default"
    );
    assert_eq!(
        empty_key,
        render(HtmlRendererOptions {
            code_annotations: true,
            code_annotation_meta_key: Cow::Owned(String::new()),
            ..HtmlRendererOptions::new()
        })
    );
}

#[test]
fn a_custom_hard_break_survives_both_representations() {
    let borrowed = render(HtmlRendererOptions {
        hard_break: "<br class=\"hard\">\n".into(),
        ..HtmlRendererOptions::new()
    });
    assert!(borrowed.contains("<br class=\"hard\">"));
    assert_eq!(
        borrowed,
        render(HtmlRendererOptions {
            hard_break: Cow::Owned(String::from("<br class=\"hard\">\n")),
            ..HtmlRendererOptions::new()
        })
    );
}
