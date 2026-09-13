mod adapter {
    pub struct Renderer {
        engine: String,
        ferro: ferromark::Options,
        pulldown: pulldown_cmark::Options,
        comrak: comrak::Options<'static>,
    }
    impl Renderer {
        pub fn new(flags: u32) -> Self {
            assert_eq!(flags, 7);
            let mut comrak = comrak::Options::default();
            comrak.extension.table = true;
            comrak.extension.strikethrough = true;
            comrak.extension.tasklist = true;
            comrak.render.r#unsafe = true;
            Self {
                engine: std::env::args().nth(1).unwrap(),
                ferro: ferromark::options!(ferromark::Options::commonmark(); render_policy: ferromark::RenderPolicy::Trusted,
                    tables: true, strikethrough: true, task_lists: true),
                pulldown: pulldown_cmark::Options::ENABLE_TABLES
                    | pulldown_cmark::Options::ENABLE_STRIKETHROUGH
                    | pulldown_cmark::Options::ENABLE_TASKLISTS,
                comrak,
            }
        }
        pub fn render(&self, input: &str) -> String {
            match self.engine.as_str() {
                "ferromark" => ferromark::to_html_with_options(input, &self.ferro),
                "pulldown" => {
                    let mut html = String::new();
                    pulldown_cmark::html::push_html(
                        &mut html,
                        pulldown_cmark::Parser::new_ext(input, self.pulldown),
                    );
                    html
                }
                "comrak" => comrak::markdown_to_html(input, &self.comrak),
                _ => panic!("unknown engine"),
            }
        }
        pub fn options(&self) -> String {
            match self.engine.as_str() {
                "ferromark" => format!("{:?}", self.ferro),
                "pulldown" => format!("{:?}", self.pulldown),
                _ => format!("{:?}", self.comrak),
            }
        }
    }
}
