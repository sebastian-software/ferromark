mod adapter {
    use ox_content_allocator::Allocator;
    use ox_content_parser::{Parser, ParserOptions};
    use ox_content_renderer::{HtmlRenderer, HtmlRendererOptions};
    use std::cell::RefCell;
    pub const NAME: &str = "ox-content";
    pub struct Renderer {
        parser_options: ParserOptions,
        html_options: HtmlRendererOptions,
        html: RefCell<HtmlRenderer>,
    }
    impl Renderer {
        pub fn new(flags: u32) -> Self {
            assert!(flags <= 7);
            let parser_options = ParserOptions {
                tables: flags & 1 != 0,
                strikethrough: flags & 2 != 0,
                task_lists: flags & 4 != 0,
                ..Default::default()
            };
            let html_options = HtmlRendererOptions {
                autolink_urls: false,
                autolink_target_blank: false,
                link_target_blank: false,
                ..Default::default()
            };
            let html = RefCell::new(HtmlRenderer::with_options(html_options.clone()));
            Self {
                parser_options,
                html_options,
                html,
            }
        }
        pub fn render(&self, input: &str) -> String {
            let arena = Allocator::new();
            let document = Parser::with_options(&arena, input, self.parser_options.clone())
                .parse()
                .unwrap();
            // render() resets per-document state and moves the owned output out.
            // render_borrowed(), AST reuse and incremental render APIs are not used.
            self.html.borrow_mut().render(&document)
        }
        pub fn options(&self) -> String {
            format!(
                "{:?}; {:?}; native generated heading IDs retained; fresh arena and owned output; renderer scratch reused",
                self.parser_options, self.html_options
            )
        }
    }
}
include!("../worker.rs");

#[cfg(test)]
mod tests {
    use super::adapter::Renderer;
    #[test]
    fn reused_renderer_does_not_reuse_document_output_or_heading_state() {
        let renderer = Renderer::new(0);
        for input in ["# Same\n\n# Same", "# Other", "", "# Same\n\n# Same"] {
            assert_eq!(renderer.render(input), Renderer::new(0).render(input));
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
            let bare = Renderer::new(flags).render("https://example.com/path");
            assert!(!bare.contains("<a "));
        }
    }
    #[test]
    fn native_heading_ids_are_retained_for_workload_review() {
        assert!(
            Renderer::new(0)
                .render("# Heading")
                .contains("id=\"heading\"")
        );
    }
}
