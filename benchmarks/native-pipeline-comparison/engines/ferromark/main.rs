mod adapter {
    pub const NAME: &str = "ferromark";
    pub struct Renderer(ferromark::Options);
    impl Renderer {
        pub fn new(flags: u32) -> Self {
            assert!(flags <= 7);
            Self(ferromark::options!(ferromark::Options::commonmark();
                render_policy: ferromark::RenderPolicy::Trusted,
                tables: flags & 1 != 0, strikethrough: flags & 2 != 0, task_lists: flags & 4 != 0,
            ))
        }
        pub fn render(&self, input: &str) -> String {
            ferromark::to_html_with_options(input, &self.0)
        }
        pub fn options(&self) -> String {
            format!("{:?}", self.0)
        }
    }
}
include!("../worker.rs");
