mod adapter {
    use rushdown::{
        parser::{self, Parser, ParserExtension},
        renderer::html,
        text::BasicReader,
    };
    pub const NAME: &str = "rushdown";
    pub struct Renderer {
        parser: Parser,
        html: html::Renderer<'static, String>,
        flags: u32,
    }
    impl Renderer {
        pub fn new(flags: u32) -> Self {
            assert!(flags <= 7);
            let parser = Parser::with_extensions(
                parser::Options::default(),
                parser::parser_extension(move |p| {
                    if flags & 1 != 0 {
                        parser::gfm_table().apply(p);
                    }
                    if flags & 2 != 0 {
                        parser::gfm_strikethrough().apply(p);
                    }
                    if flags & 4 != 0 {
                        parser::gfm_task_list_item().apply(p);
                    }
                }),
            );
            let html = html::Renderer::with_options(html::Options {
                allows_unsafe: true,
                ..Default::default()
            });
            Self {
                parser,
                html,
                flags,
            }
        }
        pub fn render(&self, input: &str) -> String {
            let mut reader = BasicReader::new(input);
            let (arena, document) = self.parser.parse(&mut reader);
            let mut output = String::new();
            self.html
                .render(&mut output, input, &arena, document)
                .unwrap();
            output
        }
        pub fn options(&self) -> String {
            format!(
                "CommonMark defaults; tables={}; strikethrough={}; task_lists={}; allows_unsafe=true; full html-entities; fresh arena/output; no cached AST",
                self.flags & 1 != 0,
                self.flags & 2 != 0,
                self.flags & 4 != 0
            )
        }
    }
}
include!("../worker.rs");

#[cfg(test)]
mod tests {
    use super::adapter::Renderer;
    #[test]
    fn native_stages_match_the_public_pipeline() {
        let pipeline = rushdown::new_markdown_to_html_string(
            Default::default(),
            rushdown::renderer::html::Options {
                allows_unsafe: true,
                ..Default::default()
            },
            rushdown::parser::empty_parser_extension(),
            rushdown::renderer::html::EmptyRendererExtension::new(),
        );
        for input in [
            "",
            "# Hello\n\n**Text** &amp; [link](/x)",
            "<script>x</script>\n",
        ] {
            let mut expected = String::new();
            pipeline(&mut expected, input).unwrap();
            assert_eq!(Renderer::new(0).render(input), expected);
        }
    }
    #[test]
    fn independent_extensions_and_trusted_rendering() {
        for flags in 0..8 {
            let html = Renderer::new(flags).render("| A |\n| --- |\n| B |\n\n~~old~~\n\n- [x] done\n- [ ] pending\n\nwww.example.com\n\n<script>x</script>\n");
            assert_eq!(html.contains("<table>"), flags & 1 != 0);
            assert_eq!(html.contains("<del>old</del>"), flags & 2 != 0);
            assert_eq!(
                html.matches("type=\"checkbox\"").count(),
                if flags & 4 != 0 { 2 } else { 0 }
            );
            assert!(!html.contains("href="));
            assert!(html.contains("<script>x</script>"));
            assert!(
                Renderer::new(flags)
                    .render("[x](javascript:alert)")
                    .contains("href=\"javascript:alert\"")
            );
        }
    }
}
