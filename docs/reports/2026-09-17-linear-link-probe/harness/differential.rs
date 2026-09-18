//! Differential corpus: renders many bracket-heavy shapes and prints one
//! line per source, so two builds can be diffed byte for byte.
use ferromark::allocator::Allocator;
use ferromark::parser::{Parser, ParserOptions};
use ferromark::renderer::{HtmlRenderer, HtmlRendererOptions};

const TOKENS: &[&str] = &[
    "a", " ", "[", "]", "![", "(u)", "(", ")", "*", "_", "`", "\\", "<", "&", "\n", "x",
    "[a]", "](u)", "[[", "]]", "[r]", "^[", "~", "=", "{", "$", "!", "<b>", "<a b>", "`c`",
    "[a](u)", "][r]", "[]", "\t", "|", "\"", "'", ":", "-", "e",
];

fn options(profile: usize) -> ParserOptions {
    let mut o = match profile {
        0 => ParserOptions::default(),
        1 => ParserOptions::gfm(),
        _ => {
            let mut all = ParserOptions::gfm();
            all.wiki_links = true;
            all.inline_footnotes = true;
            all.superscript = true;
            all.subscript = true;
            all.highlight = true;
            all.math = true;
            all.mdx = true;
            all.definition_lists = true;
            all
        }
    };
    o.max_nesting_depth = 0;
    o
}

fn render(source: &str, profile: usize) -> String {
    let allocator = Allocator::for_source_len(source.len());
    match Parser::with_options(&allocator, source, options(profile)).parse() {
        Ok(document) => {
            let html = HtmlRenderer::with_options(HtmlRendererOptions::new()).render(&document);
            format!("{html}\u{1}{:?}", document)
        }
        Err(error) => format!("ERR {error}"),
    }
}

pub fn run(seed: u64, count: usize, max_tokens: u64) {
    let mut state = seed;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let mut cases: Vec<String> = Vec::new();
    for _ in 0..count {
        let len = (next() % max_tokens) as usize + 1;
        let mut source = String::new();
        for _ in 0..len {
            source.push_str(TOKENS[(next() % TOKENS.len() as u64) as usize]);
        }
        cases.push(source);
    }
    // Hand-picked shapes around the in-place bracket path.
    for shape in [
        "[[a](u)](v)", "[[a](u)x](v)", "[a [b] c](v)", "[[a](u]x)]", "[x[y]](u)",
        "[[a](u)*b*](v)", "[*[a](u)]*", "[[a]](u)", "[[a][b]](u)", "[[[a]]]",
        "[[a](u)](v)(w)", "[[a](u)\n](v)", "[![a](i)](u)", "[[a](u)![b](i)](v)",
        "[[]](u)", "[[ ]](u)", "[[a] [b]](u)", "[a[b]c](u)", "[[a]b](u)",
        "[[a](u)] [b](v)", "[[a](u)][r]", "[[a]][r]", "[[a](u)]", "[[a](u)]x",
        "[[r]](u)", "[[a](u)](v)\n\n[r]: /r\n", "[[a]]\n\n[a]: /a\n",
        "[[a](u)][r]\n\n[r]: /r\n", "[[a][r]][r]\n\n[r]: /r\n",
        "[[a](u]x)](v)", "[[a](u`)`]", "[[a](<u>)](v)", "[[a](u \"t\")](v)",
    ] {
        cases.push(shape.to_owned());
        cases.push(format!("{shape} tail"));
        cases.push(format!("lead {shape}"));
    }
    for depth in [1usize, 2, 3, 4, 8, 17] {
        cases.push("[".repeat(depth) + "a" + &"](u)".repeat(depth));
        cases.push("[".repeat(depth) + "a" + &"]".repeat(depth));
        cases.push("[".repeat(depth) + "a" + &"][r]".repeat(depth) + "\n\n[r]: /r\n");
        cases.push("[![".repeat(depth) + "a" + &"](i)](u)".repeat(depth));
        cases.push("[[".repeat(depth) + "a" + &"]]".repeat(depth));
    }
    let mut hits = 0u64;
    for (index, source) in cases.iter().enumerate() {
        for profile in 0..3 {
            let rendered = render(source, profile);
            let mut hash = 0xcbf2_9ce4_8422_2325u64;
            for byte in rendered.as_bytes() {
                hash ^= u64::from(*byte);
                hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
            }
            hits = hits.wrapping_add(hash);
            println!("{index}\t{profile}\t{hash:016x}\t{}", source.escape_debug());
        }
    }
    eprintln!("cases={} checksum={hits:016x}", cases.len());
}
