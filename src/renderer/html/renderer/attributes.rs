//! Renderer-only mapping from authored metadata keys to HTML attributes.

use crate::ast::Attribute;

use super::HtmlRenderer;

impl HtmlRenderer {
    pub(in crate::renderer::html::renderer) fn write_authored_attributes(
        &mut self,
        values: &[Attribute<'_>],
        reserved: &[&str],
    ) {
        for item in values {
            if reserved.contains(&item.name) {
                continue;
            }
            let name = item.name;
            let mapped =
                standard_name(name) || name.starts_with("aria-") || name.starts_with("data-");
            if !mapped
                && values
                    .iter()
                    .any(|other| other.name.strip_prefix("data-") == Some(name))
            {
                // An explicitly authored data-* key wins in either order.
                continue;
            }
            self.write(" ");
            if !mapped {
                self.write("data-");
            }
            self.write(name);
            self.write("=\"");
            self.write_attribute_escaped(item.value);
            self.write("\"");
        }
    }
}

/// Recognized HTML attribute names. Recognition is element-independent;
/// reserved Markdown-owned names are filtered by each caller separately.
fn standard_name(name: &str) -> bool {
    matches!(
        name,
        "abbr"
            | "accept"
            | "accept-charset"
            | "accesskey"
            | "action"
            | "align"
            | "alt"
            | "as"
            | "async"
            | "autocapitalize"
            | "autocomplete"
            | "autofocus"
            | "autoplay"
            | "capture"
            | "charset"
            | "checked"
            | "cite"
            | "class"
            | "cols"
            | "colspan"
            | "content"
            | "contenteditable"
            | "controls"
            | "coords"
            | "crossorigin"
            | "data"
            | "datetime"
            | "decoding"
            | "default"
            | "defer"
            | "dir"
            | "dirname"
            | "disabled"
            | "download"
            | "draggable"
            | "enctype"
            | "enterkeyhint"
            | "fetchpriority"
            | "for"
            | "form"
            | "formaction"
            | "headers"
            | "height"
            | "hidden"
            | "high"
            | "href"
            | "hreflang"
            | "http-equiv"
            | "id"
            | "inert"
            | "inputmode"
            | "integrity"
            | "ismap"
            | "kind"
            | "label"
            | "lang"
            | "list"
            | "loading"
            | "loop"
            | "low"
            | "max"
            | "maxlength"
            | "media"
            | "method"
            | "min"
            | "minlength"
            | "multiple"
            | "muted"
            | "name"
            | "nonce"
            | "open"
            | "optimum"
            | "pattern"
            | "placeholder"
            | "playsinline"
            | "poster"
            | "preload"
            | "readonly"
            | "referrerpolicy"
            | "rel"
            | "required"
            | "reversed"
            | "role"
            | "rows"
            | "rowspan"
            | "sandbox"
            | "scope"
            | "selected"
            | "shape"
            | "size"
            | "sizes"
            | "slot"
            | "span"
            | "spellcheck"
            | "src"
            | "srcdoc"
            | "srclang"
            | "srcset"
            | "start"
            | "step"
            | "style"
            | "tabindex"
            | "target"
            | "title"
            | "translate"
            | "type"
            | "usemap"
            | "value"
            | "width"
            | "wrap"
    )
}
