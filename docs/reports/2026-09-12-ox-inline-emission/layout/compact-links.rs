#![allow(dead_code)]
mod links { #[derive(Debug, Clone, Copy)]
pub enum AutolinkLiteralKind {
    /// `http://` or `https://` or `ftp://` URL.
    Url,
    /// `www.` link (needs `http://` prepended in href).
    Www,
    /// Email address.
    Email,
}
}
struct EmitPoint {
    pos: u32,
    kind: EmitKind,
    end: u32,
}

#[derive(Debug, Clone, Copy)]
enum EmitKind {
    CodeSpanStart,
    CodeSpanEnd,
    CodeContent(u32), // end position
    EmphasisStart,
    EmphasisEnd,
    StrongStart,
    StrongEnd,
    StrikethroughStart,
    StrikethroughEnd,
    SubscriptStart,
    SubscriptEnd,
    SuperscriptStart,
    SuperscriptEnd,
    HighlightStart,
    HighlightEnd,
    Escape(u8),
    HardBreak,
    SoftBreak,
    LinkStart { link_index: u32 },
    LinkStartRef {
        def_index: u32,
    },
    LinkEnd,
    ImageStart { link_index: u32 },
    ImageStartRef {
        def_index: u32,
    },
    ImageEnd,
    AutolinkUrl {
        content_start: u32,
        content_end: u32,
    },
    AutolinkEmail {
        content_start: u32,
        content_end: u32,
    },
    AutolinkLiteral {
        end: u32,
        kind: links::AutolinkLiteralKind,
    },
    HtmlRaw {
        end: u32,
    },
    FootnoteRef {
        def_index: u32,
    },
    InlineFootnote {
        content_start: u32,
        content_end: u32,
    },
    MathInline {
        content_start: u32,
        content_end: u32,
    },
    MathDisplay {
        content_start: u32,
        content_end: u32,
    },
}

fn main() {println!("{} {}",std::mem::size_of::<EmitPoint>(),std::mem::size_of::<EmitKind>());}
