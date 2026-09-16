//! Compare indented JSX children with standalone Markdown, including byte spans.
use ferromark::allocator::Allocator;
use ferromark::ast::{Node, Visit, walk_node};
use ferromark::parser::{Parser, ParserOptions};

struct Spans<'s> {
    source: &'s str,
    indent: &'s str,
    values: Vec<(String, String)>,
}

impl<'a> Visit<'a> for Spans<'_> {
    fn visit_node(&mut self, node: &Node<'a>) {
        let span = node.span();
        let text = self
            .source
            .get(span.start as usize..span.end as usize)
            .expect("span must be an in-bounds UTF-8 range");
        // Block spans include the original line indentation; inline spans
        // start at their token. Remove only the wrapper's added indentation.
        let text = if span.start == 0 || self.source.as_bytes()[span.start as usize - 1] == b'\n' {
            text.strip_prefix(self.indent).unwrap_or(text)
        } else {
            text
        };
        let kind = format!("{:?}", std::mem::discriminant(node));
        self.values
            .push((kind, text.replace(&format!("\n{}", self.indent), "\n")));
        walk_node(self, node);
    }
}

#[test]
fn jsx_indentation_preserves_nested_markdown_nodes_and_source_ranges() {
    let bodies = [
        "# Héading\n\n---\n\n> Quote **bold**\n\n- Item *em*\n\n```txt\ncode\n```\n",
        "*em* **strong** `code` $x$ [link](/url) ![alt](/img) ==mark== ~~old~~ x^2^ H~2~O  \nbreak\n",
        "| A | B |\n| --- | --- |\n| **x** | y |\n: Caption *text* {#table .wide}\n",
        "Term\n: First **meaning**\n\n  Continuation\n\n$$\nx + y\n$$\n",
        "<Inner title={value} {...props}>\n\n# Child\n\n</Inner>\n\nText <Badge title={value} {...props} /> {inline}\n\n{flow}\n\nexport const x = 1;\n",
        "[^note]: A **footnote**\n\nUse[^note].\n\n[ref]: /target\n\n[ref]\n",
    ];
    for body in bodies {
        for indent in ["  ", "\t"] {
            let options = ParserOptions {
                mdx: true,
                math: true,
                definition_lists: true,
                highlight: true,
                superscript: true,
                subscript: true,
                table_attributes: true,
                ..ParserOptions::gfm()
            };
            let allocator = Allocator::new();
            let plain_source = body.to_owned();
            let plain = Parser::with_options(&allocator, &plain_source, options.clone())
                .parse()
                .unwrap();
            let mut indented = String::new();
            for line in body.split_inclusive('\n') {
                indented.push_str(indent);
                indented.push_str(line);
            }
            let source = format!("<Outer>\n{indented}</Outer>\n");
            let wrapped = Parser::with_options(&allocator, &source, options)
                .parse()
                .unwrap();
            let [Node::MdxJsxFlowElement(outer)] = wrapped.children.as_slice() else {
                panic!("expected one JSX container: {wrapped:?}");
            };
            let mut expected = Spans {
                source: &plain_source,
                indent: "",
                values: Vec::new(),
            };
            let mut actual = Spans {
                source: &source,
                indent,
                values: Vec::new(),
            };
            for child in &plain.children {
                expected.visit_node(child);
            }
            for child in &outer.children {
                actual.visit_node(child);
            }
            assert_eq!(
                actual.values, expected.values,
                "indented JSX changed structure/spans: {source:?}"
            );
        }
    }
}
