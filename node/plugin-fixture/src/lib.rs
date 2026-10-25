//! A separate plugin crate with no Node or engine dependencies.

use ferromark::{HtmlCodeBlockInput, HtmlRenderHooks, JsxCodeBlockInput, JsxRenderHooks};

/// Deterministic stand-in for a diagram engine, compiled only in isolated tests.
#[derive(Default)]
pub struct FixturePlugin {
    blocks: usize,
}

impl FixturePlugin {
    fn block(&mut self, code: &str, language: Option<&str>, jsx: bool) -> Option<String> {
        match language? {
            "native-omit" => Some(String::new()),
            "native-plugin" => {
                self.blocks += 1;
                // JSON string children keep JSX literal braces and markup inert.
                let text = if jsx {
                    format!("{{{}}}", serde_json::to_string(code).unwrap())
                } else {
                    code.replace('&', "&amp;")
                        .replace('<', "&lt;")
                        .replace('>', "&gt;")
                };
                Some(format!(
                    "<figure data-native=\"{}\">{text}</figure>\n",
                    self.blocks
                ))
            }
            _ => None,
        }
    }
}

impl HtmlRenderHooks for FixturePlugin {
    fn render_code_block(&mut self, input: HtmlCodeBlockInput<'_>) -> Option<String> {
        self.block(input.code, input.language, false)
    }
}

impl JsxRenderHooks for FixturePlugin {
    fn render_code_block(&mut self, input: JsxCodeBlockInput<'_>) -> Option<String> {
        self.block(input.code, input.language, true)
    }
}
