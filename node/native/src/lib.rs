use ferromark::{
    FencedCodeBlock, FencedCodeRenderer, Options as CoreOptions, RenderPolicy, TrustedHtml,
};
use napi::bindgen_prelude::{Error, FnArgs, Function, Result, Status};
use napi_derive::napi;

#[napi(object)]
pub struct Options {
    pub render_policy: Option<String>,
    pub allow_html: Option<bool>,
    pub allow_link_refs: Option<bool>,
    pub tables: Option<bool>,
    pub strikethrough: Option<bool>,
    pub highlight: Option<bool>,
    pub superscript: Option<bool>,
    pub subscript: Option<bool>,
    pub task_lists: Option<bool>,
    pub autolink_literals: Option<bool>,
    pub disallowed_raw_html: Option<bool>,
    pub footnotes: Option<bool>,
    pub front_matter: Option<bool>,
    pub heading_ids: Option<bool>,
    pub math: Option<bool>,
    pub callouts: Option<bool>,
    pub line_comments: Option<bool>,
}

impl Options {
    fn into_core(self) -> Result<CoreOptions> {
        let mut options = CoreOptions::default();

        if let Some(policy) = self.render_policy {
            options.render_policy = match policy.as_str() {
                "untrusted" => RenderPolicy::Untrusted,
                "trusted" => RenderPolicy::Trusted,
                _ => {
                    return Err(Error::new(
                        Status::InvalidArg,
                        "renderPolicy must be either 'untrusted' or 'trusted'",
                    ));
                }
            };
        }

        apply(&mut options.allow_html, self.allow_html);
        apply(&mut options.allow_link_refs, self.allow_link_refs);
        apply(&mut options.tables, self.tables);
        apply(&mut options.strikethrough, self.strikethrough);
        apply(&mut options.highlight, self.highlight);
        apply(&mut options.superscript, self.superscript);
        apply(&mut options.subscript, self.subscript);
        apply(&mut options.task_lists, self.task_lists);
        apply(&mut options.autolink_literals, self.autolink_literals);
        apply(&mut options.disallowed_raw_html, self.disallowed_raw_html);
        apply(&mut options.footnotes, self.footnotes);
        apply(&mut options.front_matter, self.front_matter);
        apply(&mut options.heading_ids, self.heading_ids);
        apply(&mut options.math, self.math);
        apply(&mut options.callouts, self.callouts);
        apply(&mut options.line_comments, self.line_comments);

        Ok(options)
    }
}

fn apply(target: &mut bool, value: Option<bool>) {
    if let Some(value) = value {
        *target = value;
    }
}

fn core_options(options: Option<Options>) -> Result<CoreOptions> {
    options.map_or_else(|| Ok(CoreOptions::default()), Options::into_core)
}

#[napi]
pub fn to_html(markdown: String, options: Option<Options>) -> Result<String> {
    Ok(ferromark::to_html_with_options(
        &markdown,
        &core_options(options)?,
    ))
}

struct CallbackRenderer<'scope> {
    callback: Function<'scope, FnArgs<(String, Option<String>)>, Option<String>>,
}

impl FencedCodeRenderer for CallbackRenderer<'_> {
    fn render(&mut self, block: FencedCodeBlock<'_>) -> Option<TrustedHtml> {
        self.callback
            .call(FnArgs::from((
                block.code.to_owned(),
                block.language.map(str::to_owned),
            )))
            .ok()
            .flatten()
            .map(TrustedHtml::from_trusted)
    }
}

#[napi]
pub fn to_html_with_renderer(
    markdown: String,
    options: Option<Options>,
    renderer: Function<FnArgs<(String, Option<String>)>, Option<String>>,
) -> Result<String> {
    let options = core_options(options)?;
    let mut renderer = CallbackRenderer { callback: renderer };
    Ok(ferromark::to_html_with_renderer(
        &markdown,
        &options,
        &mut renderer,
    ))
}
