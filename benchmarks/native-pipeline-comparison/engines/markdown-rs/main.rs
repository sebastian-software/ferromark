mod adapter {
    pub const NAME: &str = "markdown-rs";
    pub struct Renderer(markdown::Options);
    impl Renderer {
        pub fn new(flags: u32) -> Self {
            assert!(flags <= 7);
            Self(markdown::Options {
                parse: markdown::ParseOptions {
                    gfm_strikethrough_single_tilde: false,
                    constructs: markdown::Constructs {
                        gfm_table: flags & 1 != 0,
                        gfm_strikethrough: flags & 2 != 0,
                        gfm_task_list_item: flags & 4 != 0,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                compile: markdown::CompileOptions {
                    allow_dangerous_html: true,
                    allow_dangerous_protocol: true,
                    ..Default::default()
                },
            })
        }
        pub fn render(&self, input: &str) -> String {
            markdown::to_html_with_options(input, &self.0).unwrap()
        }
        pub fn options(&self) -> String {
            format!("{:?}", self.0)
        }
    }
}
include!("../worker.rs");

#[cfg(test)]
mod tests {
    use super::adapter::Renderer;
    #[test]
    fn core_matches_default_public_html_api_on_safe_inputs() {
        for input in [
            "",
            "# Hello\n\n**Text** &amp; [link](/x)",
            "```rust\nlet x = 1;\n```",
        ] {
            assert_eq!(Renderer::new(0).render(input), markdown::to_html(input));
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
