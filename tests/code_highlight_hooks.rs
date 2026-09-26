use ferromark::{
    Allocator, CodeAnnotationSyntax, CodeHighlightInput, HighlightedCodeBlock, HtmlRenderHooks,
    HtmlRenderer, HtmlRendererOptions, Parser,
};

type HighlightCall = (String, Option<String>, Option<String>, Option<String>);

#[derive(Default)]
struct MarkHighlighter {
    calls: Vec<HighlightCall>,
    invalid_line_count: bool,
}

impl HtmlRenderHooks for MarkHighlighter {
    fn highlight_code_block(
        &mut self,
        input: CodeHighlightInput<'_>,
    ) -> Option<HighlightedCodeBlock> {
        self.calls.push((
            input.code.to_owned(),
            input.language.map(str::to_owned),
            input.raw_language.map(str::to_owned),
            input.raw_meta.map(str::to_owned),
        ));
        if input.language == Some("unknown") {
            return None;
        }
        let mut lines = input
            .code
            .split('\n')
            .map(|line| format!("<mark>{}</mark>", escape(line)))
            .collect::<Vec<_>>();
        if self.invalid_line_count {
            lines.pop();
        }
        Some(HighlightedCodeBlock { lines })
    }
}

fn escape(source: &str) -> String {
    source
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[test]
fn plain_fenced_and_indented_blocks_keep_ferromark_wrappers() {
    let allocator = Allocator::new();
    let document = Parser::new(
        &allocator,
        "```rust\nlet x = \"<tag>\";\n```\n\n    & raw\n",
    )
    .parse()
    .unwrap();
    let mut renderer = HtmlRenderer::new();
    let mut hooks = MarkHighlighter::default();
    let html = renderer.render_with_hooks(&document, &mut hooks);

    assert!(html.contains("<pre><code class=\"language-rust\"><mark>let x = &quot;&lt;tag&gt;&quot;;</mark>\n<mark></mark></code></pre>"), "{html}");
    assert!(
        html.contains("<pre><code><mark>&amp; raw</mark>\n<mark></mark></code></pre>"),
        "{html}"
    );
    assert_eq!(hooks.calls.len(), 2);
    assert_eq!(hooks.calls[0].1.as_deref(), Some("rust"));
    assert_eq!(hooks.calls[1].1, None);
}

#[test]
fn metadata_and_annotation_lines_stay_owned_by_ferromark() {
    let allocator = Allocator::new();
    let document = Parser::new(
        &allocator,
        "```ts:line-numbers=7 :line-links=demo [a\"b.ts] {1}\n// [!code warning]\nconst x = '<tag>';\n```",
    )
    .parse()
    .unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        code_annotations: true,
        code_annotation_syntax: CodeAnnotationSyntax::VitePress,
        ..Default::default()
    });
    let mut hooks = MarkHighlighter::default();
    let html = renderer.render_with_hooks(&document, &mut hooks);

    assert!(html.contains("data-code-title=\"a&quot;b.ts\""), "{html}");
    assert!(html.contains("data-line-number-start=\"7\""), "{html}");
    assert!(html.contains("id=\"demo-L7\""), "{html}");
    assert!(html.contains("ox-code-line--warning"), "{html}");
    assert!(
        html.contains("<mark>const x = &#39;&lt;tag&gt;&#39;;</mark>"),
        "{html}"
    );
    assert_eq!(hooks.calls[0].1.as_deref(), Some("ts"));
    assert!(
        hooks.calls[0]
            .2
            .as_deref()
            .unwrap()
            .starts_with("ts:line-numbers")
    );
    assert_eq!(hooks.calls[0].0, "const x = '<tag>';\n");
    assert!(
        hooks.calls[0]
            .3
            .as_deref()
            .unwrap()
            .contains(":line-links=demo")
    );
}

#[test]
fn unknown_language_and_invalid_line_count_fall_back_to_escaped_code() {
    let allocator = Allocator::new();
    let document = Parser::new(
        &allocator,
        "```unknown\n<unsafe>\n```\n\n```rust\n<safe>\n```",
    )
    .parse()
    .unwrap();
    let mut renderer = HtmlRenderer::new();
    let mut hooks = MarkHighlighter {
        invalid_line_count: true,
        ..Default::default()
    };
    let html = renderer.render_with_hooks(&document, &mut hooks);

    assert!(html.contains("&lt;unsafe&gt;"), "{html}");
    assert!(html.contains("&lt;safe&gt;"), "{html}");
    assert!(!html.contains("<mark>"), "{html}");
}

#[test]
fn empty_unicode_and_crlf_blocks_can_reuse_one_highlighter() {
    let allocator = Allocator::new();
    let document = Parser::new(
        &allocator,
        "```rust\r\n😀 <x>\r\n```\r\n\r\n```rust\r\n```\r\n",
    )
    .parse()
    .unwrap();
    let mut renderer = HtmlRenderer::new();
    let mut hooks = MarkHighlighter::default();
    let first = renderer.render_with_hooks(&document, &mut hooks);
    let second = renderer.render_with_hooks(&document, &mut hooks);

    assert_eq!(first, second);
    assert!(first.contains("<mark>😀 &lt;x&gt;</mark>"), "{first}");
    assert!(
        first.contains("<pre><code class=\"language-rust\"><mark></mark></code></pre>"),
        "{first}"
    );
    assert_eq!(hooks.calls.len(), 4);
}
