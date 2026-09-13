mod adapter {
    pub struct Renderer(satteri_pulldown_cmark::Options);
    impl Renderer {
        pub fn new(flags: u32) -> Self {
            assert_eq!(flags, 7);
            use satteri_pulldown_cmark::Options as O;
            Self(O::ENABLE_TABLES | O::ENABLE_STRIKETHROUGH | O::ENABLE_TASKLISTS)
        }
        pub fn render(&self, input: &str) -> String {
            let (arena, errors) = satteri_pulldown_cmark::parse(input, self.0);
            assert!(errors.is_empty());
            satteri_ast::mdast_to_html(&arena)
        }
        pub fn options(&self) -> String {
            format!(
                "{:?}; positions retained; native mdast_to_html; no MDX compilation",
                self.0
            )
        }
    }
}
