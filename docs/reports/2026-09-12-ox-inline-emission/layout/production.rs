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
    // A combined opener/content point; the closing point remains separate.
    CodeSpan { index: u32 },
    CodeSpanStart,
    CodeContent(u32),
    CodeSpanEnd,
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
    LinkStartRef { def_index: u32 },
    LinkEnd,
    ImageStart { link_index: u32 },
    ImageStartRef { def_index: u32 },
    ImageEnd,
    AutolinkUrl { index: u32 },
    AutolinkEmail { index: u32 },
    AutolinkLiteral { kind: links::AutolinkLiteralKind },
    HtmlRaw { end: u32 },
    FootnoteRef { def_index: u32 },
    InlineFootnote { index: u32 },
    MathInline { index: u32 },
    MathDisplay { index: u32 },
}

fn main() {println!("{} {}",std::mem::size_of::<EmitPoint>(),std::mem::size_of::<EmitKind>());}
