use ferromark::{
    FencedCodeBlock, FencedCodeRenderer, Options, TrustedHtml, to_html, to_html_into_with_renderer,
    to_html_with_renderer,
};

#[derive(Default)]
struct RecordingRenderer {
    calls: Vec<(Option<String>, String)>,
    output: Option<String>,
}

impl FencedCodeRenderer for RecordingRenderer {
    fn render(&mut self, block: FencedCodeBlock<'_>) -> Option<TrustedHtml> {
        self.calls
            .push((block.language.map(str::to_owned), block.code.to_owned()));
        self.output.clone().map(TrustedHtml::from_trusted)
    }
}

#[test]
fn renderer_receives_decoded_language_and_raw_code() {
    let mut renderer = RecordingRenderer {
        output: Some("<pre class=\"highlighted\">safe</pre>\n".to_owned()),
        ..RecordingRenderer::default()
    };

    let html = to_html_with_renderer(
        "```c\\+\\+&amp; extra metadata\n<tag>\n```",
        &Options::default(),
        &mut renderer,
    );

    assert_eq!(html, "<pre class=\"highlighted\">safe</pre>\n");
    assert_eq!(
        renderer.calls,
        vec![(Some("c++&".to_owned()), "<tag>\n".to_owned())]
    );
}

#[test]
fn renderer_receives_none_for_a_fence_without_info() {
    let mut renderer = RecordingRenderer::default();

    to_html_with_renderer("```\ncode\n```", &Options::default(), &mut renderer);

    assert_eq!(renderer.calls, vec![(None, "code\n".to_owned())]);
}

#[test]
fn renderer_reuses_and_clears_its_buffer_between_fences() {
    let mut renderer = RecordingRenderer::default();

    to_html_with_renderer(
        "```a\nfirst\n```\n\n```b\nsecond\n```",
        &Options::default(),
        &mut renderer,
    );

    assert_eq!(
        renderer.calls,
        vec![
            (Some("a".to_owned()), "first\n".to_owned()),
            (Some("b".to_owned()), "second\n".to_owned()),
        ]
    );
}

#[test]
fn none_uses_the_existing_escaped_fallback() {
    let markdown = "```rust title=demo\nlet value = <unsafe>;\n```";
    let mut renderer = RecordingRenderer::default();

    let with_renderer = to_html_with_renderer(markdown, &Options::default(), &mut renderer);

    assert_eq!(with_renderer, to_html(markdown));
    assert_eq!(
        with_renderer,
        "<pre><code class=\"language-rust\">let value = &lt;unsafe&gt;;\n</code></pre>\n"
    );
}

#[test]
fn indented_code_never_invokes_the_renderer() {
    let mut renderer = RecordingRenderer {
        output: Some("<p>should not be used</p>".to_owned()),
        ..RecordingRenderer::default()
    };

    let html = to_html_with_renderer("    <tag>\n", &Options::default(), &mut renderer);

    assert!(renderer.calls.is_empty());
    assert_eq!(html, "<pre><code>&lt;tag&gt;\n</code></pre>\n");
}

#[test]
fn renderer_is_an_explicit_boundary_under_untrusted_policy() {
    let mut renderer = RecordingRenderer {
        output: Some("<pre data-safe=\"true\"><code>escaped</code></pre>".to_owned()),
        ..RecordingRenderer::default()
    };

    let html = to_html_with_renderer("```html\n<script>\n```", &Options::default(), &mut renderer);

    assert_eq!(html, "<pre data-safe=\"true\"><code>escaped</code></pre>");
}

#[test]
fn buffer_api_reuses_the_caller_buffer() {
    let mut renderer = RecordingRenderer::default();
    let mut output = Vec::with_capacity(256);

    to_html_into_with_renderer(
        "```txt\nhello\n```",
        &mut output,
        &Options::default(),
        &mut renderer,
    );

    assert_eq!(
        String::from_utf8(output).unwrap(),
        "<pre><code class=\"language-txt\">hello\n</code></pre>\n"
    );
}

#[derive(Default)]
struct MetaRecordingRenderer {
    calls: Vec<(Option<String>, Option<String>)>,
}

impl FencedCodeRenderer for MetaRecordingRenderer {
    fn render(&mut self, block: FencedCodeBlock<'_>) -> Option<TrustedHtml> {
        self.calls.push((
            block.language.map(str::to_owned),
            block.meta.map(str::to_owned),
        ));
        None
    }
}

#[test]
fn renderer_receives_info_string_meta_after_the_language() {
    let mut renderer = MetaRecordingRenderer::default();

    to_html_with_renderer(
        "```ts {1-3} title=\"Example\"\ncode\n```",
        &Options::default(),
        &mut renderer,
    );

    assert_eq!(
        renderer.calls,
        vec![(
            Some("ts".to_owned()),
            Some("{1-3} title=\"Example\"".to_owned())
        )]
    );
}

#[test]
fn renderer_meta_is_none_without_trailing_info_text() {
    let mut renderer = MetaRecordingRenderer::default();

    to_html_with_renderer(
        "```rust\ncode\n```\n\n```rust   \ncode\n```\n\n```\ncode\n```",
        &Options::default(),
        &mut renderer,
    );

    assert_eq!(
        renderer.calls,
        vec![
            (Some("rust".to_owned()), None),
            (Some("rust".to_owned()), None),
            (None, None),
        ]
    );
}

#[test]
fn renderer_meta_decodes_escapes_and_entities() {
    let mut renderer = MetaRecordingRenderer::default();

    to_html_with_renderer(
        "```rust a\\+b &amp;c\ncode\n```",
        &Options::default(),
        &mut renderer,
    );

    assert_eq!(
        renderer.calls,
        vec![(Some("rust".to_owned()), Some("a+b &c".to_owned()))]
    );
}
