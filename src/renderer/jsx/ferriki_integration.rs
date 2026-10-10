//! Optional Ferriki adapter for JSX code-block highlighting.

use ferriki::{
    Error, ErrorKind, FontStyle, HighlightThemeToken, HighlightThemeTokenStyle,
    HighlightTokensWithThemesResult, Highlighter, RenderOptions,
};

use super::{JsxCodeBlockInput, JsxHighlightedCodeBlock, JsxRenderHooks};

/// Connects a reusable Ferriki highlighter to native JSX rendering.
///
/// Build and preload the highlighter before rendering when lazy asset access is
/// not desired. Unknown languages and highlight failures use Ferromark's plain
/// code fallback. Errors other than an unknown language can be observed with
/// [`with_error_handler`](Self::with_error_handler).
pub struct FerrikiJsxHooks<'a> {
    highlighter: &'a mut Highlighter,
    light_theme: &'a str,
    dark_theme: Option<&'a str>,
    render_options: RenderOptions,
    on_error: Option<&'a mut dyn FnMut(&Error)>,
}

impl<'a> FerrikiJsxHooks<'a> {
    /// Borrows an initialized highlighter and selects one loaded theme.
    pub fn new(highlighter: &'a mut Highlighter, theme: &'a str) -> Self {
        Self {
            highlighter,
            light_theme: theme,
            dark_theme: None,
            render_options: RenderOptions::default(),
            on_error: None,
        }
    }

    /// Borrows an initialized highlighter and selects light and dark themes.
    pub fn with_light_dark_themes(
        highlighter: &'a mut Highlighter,
        light_theme: &'a str,
        dark_theme: &'a str,
    ) -> Self {
        Self {
            highlighter,
            light_theme,
            dark_theme: Some(dark_theme),
            render_options: RenderOptions::default(),
            on_error: None,
        }
    }

    /// Sets Ferriki's options for rendering highlighted line fragments.
    ///
    /// For single-theme output, Ferriki's whitespace and adjacent-token merge
    /// settings are applied. Dual-theme output uses Ferriki's aligned token API;
    /// these render settings remain limited to single-theme output.
    #[must_use]
    pub fn with_render_options(mut self, options: RenderOptions) -> Self {
        self.render_options = options;
        self
    }

    /// Observes Ferriki errors other than an unknown language before fallback.
    ///
    /// The callback runs synchronously during rendering and must not retain the
    /// error reference. A panic from the callback propagates to the caller.
    #[must_use]
    pub fn with_error_handler(mut self, handler: &'a mut dyn FnMut(&Error)) -> Self {
        self.on_error = Some(handler);
        self
    }

    fn report_error(&mut self, error: &Error) {
        if error.kind() != ErrorKind::UnknownLanguage
            && let Some(handler) = self.on_error.as_mut()
        {
            handler(error);
        }
    }
}

impl JsxRenderHooks for FerrikiJsxHooks<'_> {
    fn highlight_code_block(
        &mut self,
        input: JsxCodeBlockInput<'_>,
    ) -> Option<JsxHighlightedCodeBlock> {
        let language = normalize_language(input.language)?;
        let Some(dark_theme) = self.dark_theme else {
            return match self.highlighter.highlight_html_lines(
                input.code,
                &language,
                self.light_theme,
                &self.render_options,
            ) {
                Ok(highlighted) => Some(
                    JsxHighlightedCodeBlock::new(highlighted.lines)
                        .with_colors(highlighted.foreground, highlighted.background),
                ),
                Err(error) => {
                    self.report_error(&error);
                    None
                }
            };
        };

        let highlighted = match self.highlighter.highlight_with_themes(
            input.code,
            &language,
            &[("light", self.light_theme), ("dark", dark_theme)],
        ) {
            Ok(tokens) => tokens,
            Err(error) => {
                self.report_error(&error);
                return None;
            }
        };
        let Some(lines) = render_dual_theme_lines(&highlighted) else {
            let error = Error::new(
                ErrorKind::Internal,
                "Ferriki did not return the requested light and dark token styles.",
            );
            self.report_error(&error);
            return None;
        };
        // Ferriki returns metadata in request order, including repeated themes.
        let [light, dark] = highlighted.themes.as_slice() else {
            let error = Error::new(
                ErrorKind::Internal,
                "Ferriki did not return the requested light and dark theme metadata.",
            );
            self.report_error(&error);
            return None;
        };
        Some(JsxHighlightedCodeBlock::new(lines).with_light_dark_colors(
            light.foreground.clone(),
            light.background.clone(),
            dark.foreground.clone(),
            dark.background.clone(),
        ))
    }
}

fn normalize_language(raw: Option<&str>) -> Option<String> {
    let raw = raw?.trim();
    if raw.is_empty() {
        return None;
    }
    let end = raw
        .char_indices()
        .find_map(|(index, character)| {
            matches!(character, '{' | '[').then_some(index).or_else(|| {
                (character == ':' && starts_metadata_suffix(&raw[index..])).then_some(index)
            })
        })
        .unwrap_or(raw.len());
    let language = raw[..end].trim();
    (!language.is_empty()).then(|| language.to_ascii_lowercase())
}

fn starts_metadata_suffix(value: &str) -> bool {
    [":no-line-numbers", ":line-numbers", ":wrap", ":wrap-lines"]
        .iter()
        .any(|suffix| value.starts_with(suffix))
}

fn render_dual_theme_lines(highlighted: &HighlightTokensWithThemesResult) -> Option<Vec<String>> {
    highlighted
        .tokens
        .iter()
        .map(|tokens| render_dual_theme_line(tokens))
        .collect()
}

fn render_dual_theme_line(tokens: &[HighlightThemeToken]) -> Option<String> {
    let mut output = String::new();
    for token in tokens {
        if token.content.is_empty() {
            continue;
        }
        let style = dual_token_style(token.variants.get("light")?, token.variants.get("dark")?);
        if style.is_empty() {
            output.push_str(&escape_html(&token.content));
        } else {
            output.push_str("<span style=\"");
            output.push_str(&escape_attribute(&style));
            output.push_str("\">");
            output.push_str(&escape_html(&token.content));
            output.push_str("</span>");
        }
    }
    Some(output)
}

fn dual_token_style(light: &HighlightThemeTokenStyle, dark: &HighlightThemeTokenStyle) -> String {
    let mut declarations = Vec::new();
    let light_color = light.color.as_deref().filter(|value| !value.is_empty());
    let dark_color = dark.color.as_deref().filter(|value| !value.is_empty());
    if let Some(color) = light_color {
        declarations.push(css_declaration("color:", color));
    }
    if let Some(color) = light_color {
        declarations.push(css_declaration("--shiki-light:", color));
    }
    if let Some(color) = dark_color {
        declarations.push(css_declaration("--shiki-dark:", color));
    }

    append_font_style_declarations(
        &mut declarations,
        light.font_style.unwrap_or_default(),
        dark.font_style.unwrap_or_default(),
    );
    declarations.join(";")
}

fn append_font_style_declarations(
    declarations: &mut Vec<String>,
    light: FontStyle,
    dark: FontStyle,
) {
    append_font_property(
        declarations,
        [
            "font-style",
            "--shiki-light-font-style",
            "--shiki-dark-font-style",
            "italic",
            "normal",
        ],
        light.contains(FontStyle::ITALIC),
        dark.contains(FontStyle::ITALIC),
    );
    append_font_property(
        declarations,
        [
            "font-weight",
            "--shiki-light-font-weight",
            "--shiki-dark-font-weight",
            "bold",
            "normal",
        ],
        light.contains(FontStyle::BOLD),
        dark.contains(FontStyle::BOLD),
    );
    let light_decoration = decoration(light);
    let dark_decoration = decoration(dark);
    if light_decoration.is_some() || dark_decoration.is_some() {
        let light_decoration = light_decoration.unwrap_or("none");
        let dark_decoration = dark_decoration.unwrap_or("none");
        declarations.push(css_declaration("text-decoration:", light_decoration));
        declarations.push(css_declaration(
            "--shiki-light-text-decoration:",
            light_decoration,
        ));
        declarations.push(css_declaration(
            "--shiki-dark-text-decoration:",
            dark_decoration,
        ));
    }
}

fn append_font_property(
    declarations: &mut Vec<String>,
    property: [&str; 5],
    light_enabled: bool,
    dark_enabled: bool,
) {
    if light_enabled || dark_enabled {
        let light_value = if light_enabled {
            property[3]
        } else {
            property[4]
        };
        let dark_value = if dark_enabled {
            property[3]
        } else {
            property[4]
        };
        declarations.push(css_declaration_with_name(property[0], light_value));
        declarations.push(css_declaration_with_name(property[1], light_value));
        declarations.push(css_declaration_with_name(property[2], dark_value));
    }
}

fn css_declaration(prefix: &str, value: &str) -> String {
    let mut declaration = String::with_capacity(prefix.len().saturating_add(value.len()));
    declaration.push_str(prefix);
    declaration.push_str(value);
    declaration
}

fn css_declaration_with_name(name: &str, value: &str) -> String {
    let mut declaration = String::with_capacity(name.len().saturating_add(value.len() + 1));
    declaration.push_str(name);
    declaration.push(':');
    declaration.push_str(value);
    declaration
}

fn decoration(style: FontStyle) -> Option<&'static str> {
    let underline = style.contains(FontStyle::UNDERLINE);
    let strike = style.contains(FontStyle::STRIKETHROUGH);
    match (underline, strike) {
        (true, true) => Some("underline line-through"),
        (true, false) => Some("underline"),
        (false, true) => Some("line-through"),
        (false, false) => None,
    }
}

fn escape_html(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

fn escape_attribute(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::{HighlightThemeToken, render_dual_theme_line};

    #[test]
    fn dual_theme_fragments_keep_aligned_unicode_and_escape_styles() {
        let tokens: Vec<HighlightThemeToken> = serde_json::from_str(
            r##"[
              {"content":"a","offset":0,"variants":{"light":{"color":"#111111"},"dark":{"color":"#aaaaaa"}}},
              {"content":"😀","offset":1,"variants":{"light":{"color":"#111111"},"dark":{"color":"#bbbbbb"}}},
              {"content":"b","offset":5,"variants":{"light":{"color":"#222222"},"dark":{"color":"#bbbbbb"}}}
            ]"##,
        ).expect("aligned tokens should deserialize");
        assert_eq!(
            render_dual_theme_line(&tokens).unwrap(),
            concat!(
                "<span style=\"color:#111111;--shiki-light:#111111;--shiki-dark:#aaaaaa\">a</span>",
                "<span style=\"color:#111111;--shiki-light:#111111;--shiki-dark:#bbbbbb\">😀</span>",
                "<span style=\"color:#222222;--shiki-light:#222222;--shiki-dark:#bbbbbb\">b</span>",
            )
        );
        let tokens: Vec<HighlightThemeToken> = serde_json::from_str(
            r#"[{"content":"<&>","offset":0,"variants":{"light":{"color":"a&\"'<>"},"dark":{}}}]"#,
        )
        .unwrap();
        assert_eq!(
            render_dual_theme_line(&tokens).unwrap(),
            "<span style=\"color:a&amp;&quot;&#39;&lt;&gt;;--shiki-light:a&amp;&quot;&#39;&lt;&gt;\">&lt;&amp;&gt;</span>"
        );
        assert_eq!(render_dual_theme_line(&[]).unwrap(), "");
        let tokens: Vec<HighlightThemeToken> = serde_json::from_str(
            r#"[{"content":"<&>","offset":0,"variants":{"light":{},"dark":{}}}]"#,
        )
        .unwrap();
        assert_eq!(render_dual_theme_line(&tokens).unwrap(), "&lt;&amp;&gt;");
    }

    #[test]
    fn missing_theme_styles_reject_the_fragment() {
        let tokens: Vec<HighlightThemeToken> =
            serde_json::from_str(r#"[{"content":"a","offset":0,"variants":{"light":{}}}]"#)
                .unwrap();
        assert!(render_dual_theme_line(&tokens).is_none());
    }
}
