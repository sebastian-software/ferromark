//! Public API compatibility checks compiled as an external crate.

use ferromark::{
    Alignment, AutolinkLiteralKind, BlockEvent, CalloutType, CodeBlockKind, FootnoteDef,
    FootnoteStore, InlineEvent, ListKind, Options, RenderPolicy, TaskState, escape_attr_into,
    escape_text_into,
};

#[test]
fn downstream_uses_the_stable_root_facade() {
    let mut escaped = Vec::new();
    escape_text_into(&mut escaped, b"<text>");
    escape_attr_into(&mut escaped, b"\"");
    assert_eq!(escaped, b"&lt;text&gt;&quot;");

    let store = FootnoteStore::new();
    let definition: Option<&FootnoteDef> = store.get(0);
    assert!(definition.is_none());
}

#[test]
fn downstream_can_customize_non_exhaustive_options_by_mutating_a_preset() {
    let mut options = Options::default();
    options.heading_ids = false;

    assert!(!options.heading_ids);
}

#[test]
fn downstream_can_customize_non_exhaustive_options_with_the_options_macro() {
    let options = ferromark::options!(Options::default();
        heading_ids: false,
    );

    assert!(!options.heading_ids);
}

#[test]
fn downstream_event_matches_include_forward_compatible_fallbacks() {
    assert_eq!(render_policy_label(RenderPolicy::Untrusted), "untrusted");
    assert_eq!(callout_label(CalloutType::Note), "note");
    assert_eq!(alignment_label(Alignment::Left), "left");
    assert_eq!(code_block_label(CodeBlockKind::Indented), "indented");
    assert_eq!(block_event_label(BlockEvent::ParagraphStart), "paragraph");
    assert_eq!(list_label(ListKind::Unordered), "unordered");
    assert_eq!(task_label(TaskState::Checked), "checked");
    assert_eq!(inline_event_label(InlineEvent::SoftBreak), "soft-break");
    assert_eq!(autolink_label(AutolinkLiteralKind::Url), "url");
}

fn render_policy_label(policy: RenderPolicy) -> &'static str {
    match policy {
        RenderPolicy::Untrusted => "untrusted",
        _ => "other",
    }
}

fn callout_label(callout: CalloutType) -> &'static str {
    match callout {
        CalloutType::Note => "note",
        _ => "other",
    }
}

fn alignment_label(alignment: Alignment) -> &'static str {
    match alignment {
        Alignment::Left => "left",
        _ => "other",
    }
}

fn code_block_label(kind: CodeBlockKind) -> &'static str {
    match kind {
        CodeBlockKind::Indented => "indented",
        _ => "other",
    }
}

fn block_event_label(event: BlockEvent) -> &'static str {
    match event {
        BlockEvent::ParagraphStart => "paragraph",
        _ => "other",
    }
}

fn list_label(kind: ListKind) -> &'static str {
    match kind {
        ListKind::Unordered => "unordered",
        _ => "other",
    }
}

fn task_label(state: TaskState) -> &'static str {
    match state {
        TaskState::Checked => "checked",
        _ => "other",
    }
}

fn inline_event_label(event: InlineEvent) -> &'static str {
    match event {
        InlineEvent::SoftBreak => "soft-break",
        _ => "other",
    }
}

fn autolink_label(kind: AutolinkLiteralKind) -> &'static str {
    match kind {
        AutolinkLiteralKind::Url => "url",
        _ => "other",
    }
}

#[cfg(feature = "mdx")]
mod mdx {
    use ferromark::mdx::render::ComponentNameError;
    use ferromark::mdx::{MdxDiagnosticCode, MdxEvent, Segment};

    #[test]
    fn downstream_mdx_matches_include_forward_compatible_fallbacks() {
        assert_eq!(segment_label(Segment::Markdown("markdown")), "markdown");
        assert_eq!(
            diagnostic_label(MdxDiagnosticCode::InvalidJsxTag),
            "invalid-jsx"
        );
        assert_eq!(
            event_label(MdxEvent::Esm(ferromark::Range::new(0, 0))),
            "esm"
        );
        assert_eq!(error_label(ComponentNameError::Empty), "empty");
    }

    fn segment_label(segment: Segment<'_>) -> &'static str {
        match segment {
            Segment::Markdown(_) => "markdown",
            _ => "other",
        }
    }

    fn diagnostic_label(code: MdxDiagnosticCode) -> &'static str {
        match code {
            MdxDiagnosticCode::InvalidJsxTag => "invalid-jsx",
            _ => "other",
        }
    }

    fn event_label(event: MdxEvent) -> &'static str {
        match event {
            MdxEvent::Esm(_) => "esm",
            _ => "other",
        }
    }

    fn error_label(error: ComponentNameError) -> &'static str {
        match error {
            ComponentNameError::Empty => "empty",
            _ => "other",
        }
    }
}
