//! Link rewriting helpers for SSG-friendly output.
//!
//! Markdown links can be converted to generated `index.html` routes, while raw HTML
//! `href` and `src` attributes can receive the configured base URL. Keeping those rules
//! together prevents the Markdown and raw-HTML paths from drifting apart.

use super::super::html_attr::{html_attr_value_range, is_html_attr_name_byte};
use super::HtmlRenderer;
use crate::ast::Link;

impl HtmlRenderer {
    /// Shared anchor policy for ordinary and hook-enabled child traversal.
    pub(in crate::renderer::html::renderer) fn write_link_open(&mut self, link: &Link<'_>) {
        self.write("<a href=\"");
        let converted_url = if self.options.convert_md_links {
            self.convert_markdown_url(link.url)
        } else {
            None
        };
        let href = self.sanitized_url(converted_url.as_deref().unwrap_or(link.url), "#");
        self.write_url_escaped(href);
        self.write("\"");
        // Add target="_blank" for external links (http:// or https://)
        let automatic_target = self.options.link_target_blank
            && (href.starts_with("http://") || href.starts_with("https://"));
        if automatic_target {
            self.write(" target=\"_blank\" rel=\"noopener noreferrer\"");
        }
        if let Some(title) = link.title {
            self.write(" title=\"");
            self.write_escaped(title);
            self.write("\"");
        }
        if let Some(attributes) = &link.attributes {
            if let Some(id) = attributes.id {
                self.write_explicit_element_id(id);
            }
            if !attributes.classes.is_empty() {
                self.write(" class=\"");
                for (index, class_name) in attributes.classes.iter().enumerate() {
                    if index > 0 {
                        self.write(" ");
                    }
                    self.write_attribute_escaped(class_name);
                }
                self.write("\"");
            }
            match (automatic_target, link.title.is_some()) {
                (true, true) => self.write_authored_attributes(
                    &attributes.values,
                    &["href", "title", "id", "class", "target", "rel"],
                ),
                (true, false) => self.write_authored_attributes(
                    &attributes.values,
                    &["href", "id", "class", "target", "rel"],
                ),
                (false, true) => self.write_authored_attributes(
                    &attributes.values,
                    &["href", "title", "id", "class"],
                ),
                (false, false) => {
                    self.write_authored_attributes(&attributes.values, &["href", "id", "class"]);
                }
            }
        }
        self.write(">");
    }

    pub(in crate::renderer::html::renderer) fn convert_markdown_url(
        &self,
        url: &str,
    ) -> Option<String> {
        if let Some(converted) = self.convert_md_url(url) {
            return Some(converted);
        }

        self.apply_base_to_root_absolute_url(url)
    }

    pub(in crate::renderer::html::renderer) fn apply_base_to_root_absolute_url(
        &self,
        url: &str,
    ) -> Option<String> {
        if !self.options.convert_md_links || !url.starts_with('/') || url.starts_with("//") {
            return None;
        }

        let suffix_start = url.find(&['?', '#'][..]).unwrap_or(url.len());
        let (path, suffix) = url.split_at(suffix_start);
        let base = self.options.base_url().trim_end_matches('/');

        if base.is_empty() {
            None
        } else if path == "/" {
            Some(join3(base, "/", suffix))
        } else {
            Some(join3(base, path, suffix))
        }
    }

    pub(in crate::renderer::html::renderer) fn rewrite_html_root_urls<'a>(
        &self,
        html: &'a str,
    ) -> std::borrow::Cow<'a, str> {
        let bytes = html.as_bytes();
        let mut output: Option<String> = None;
        let mut copied = 0usize;
        let mut i = 0usize;
        let mut in_tag = false;
        let mut saw_self_closing_slash = false;
        let mut pending_raw_text_tag: Option<&str> = None;

        while i < bytes.len() {
            if !in_tag {
                let Some(rel) = memchr::memchr(b'<', &bytes[i..]) else {
                    break;
                };
                i += rel;
                // Skip HTML comments verbatim (including CommonMark empty forms
                // `<!-->` / `<!--->` and HTML5 `--!>` endings).
                if html[i..].starts_with("<!--") {
                    if let Some(next) = skip_html_comment(bytes, i) {
                        i = next;
                        continue;
                    }
                    break;
                }
                if html[i..].starts_with("<![CDATA[") {
                    if let Some(end) = memchr::memmem::find(&bytes[i + 9..], b"]]>") {
                        i = i + 9 + end + 3;
                        continue;
                    }
                    break;
                }
                if bytes.get(i + 1) == Some(&b'?') {
                    if let Some(end) = memchr::memmem::find(&bytes[i + 2..], b"?>") {
                        i = i + 2 + end + 2;
                        continue;
                    }
                    break;
                }
                if bytes.get(i + 1) == Some(&b'!') {
                    if let Some(end) = memchr::memchr(b'>', &bytes[i + 2..]) {
                        i = i + 2 + end + 1;
                        continue;
                    }
                    break;
                }
                if bytes.get(i + 1) == Some(&b'/') {
                    if bytes.get(i + 2).is_some_and(u8::is_ascii_alphabetic) {
                        if let Some(end) = memchr::memchr(b'>', &bytes[i + 3..]) {
                            i = i + 3 + end + 1;
                            continue;
                        }
                        break;
                    }
                    i += 2;
                    continue;
                }
                // Only an ASCII letter after `<` starts an HTML opening tag;
                // bare comparisons like `1 < 2` stay in text mode so later
                // apostrophes cannot be misread as attribute quotes.
                if !bytes.get(i + 1).is_some_and(u8::is_ascii_alphabetic) {
                    i += 1;
                    continue;
                }
                let tag_name_start = i + 1;
                let mut tag_name_end = tag_name_start + 1;
                while tag_name_end < bytes.len()
                    && !matches!(
                        bytes[tag_name_end],
                        b'>' | b'/' | b' ' | b'\t' | b'\n' | b'\r' | b'\x0c'
                    )
                {
                    tag_name_end += 1;
                }
                let tag_name = &html[tag_name_start..tag_name_end];
                pending_raw_text_tag = is_html_raw_text_tag(tag_name).then_some(tag_name);
                in_tag = true;
                saw_self_closing_slash = false;
                i = tag_name_end;
                continue;
            }

            match bytes[i] {
                b'>' => {
                    let self_closing = saw_self_closing_slash;
                    in_tag = false;
                    saw_self_closing_slash = false;
                    i += 1;
                    if let Some(raw_tag) = pending_raw_text_tag.take()
                        && !self_closing
                    {
                        if let Some(next) = skip_html_raw_text_body(bytes, i, raw_tag) {
                            i = next;
                        } else {
                            break;
                        }
                    }
                }
                b'/' => {
                    saw_self_closing_slash = true;
                    i += 1;
                }
                b'"' | b'\'' => {
                    saw_self_closing_slash = false;
                    let quote = bytes[i];
                    i += 1;
                    if let Some(rel) = memchr::memchr(quote, &bytes[i..]) {
                        i += rel + 1;
                    } else {
                        break;
                    }
                }
                b'=' => {
                    saw_self_closing_slash = false;
                    if let Some((value_start, value_end)) = html_attr_value_range(html, bytes, i) {
                        i = attr_advance(bytes, value_start, value_end);
                        continue;
                    }
                    i += 1;
                }
                byte if is_html_attr_name_byte(byte) => {
                    saw_self_closing_slash = false;
                    let name_start = i;
                    let mut name_end = i + 1;
                    while name_end < bytes.len() && is_html_attr_name_byte(bytes[name_end]) {
                        name_end += 1;
                    }

                    let name = &html[name_start..name_end];
                    if let Some((value_start, value_end)) =
                        html_attr_value_range(html, bytes, name_end)
                    {
                        if name.eq_ignore_ascii_case("href") || name.eq_ignore_ascii_case("src") {
                            let value = &html[value_start..value_end];
                            // Raw anchors link pages the same way Markdown links
                            // do; convert .md targets first, then fall back to
                            // rebasing root-absolute URLs.
                            if let Some(rewritten) = self.convert_markdown_url(value) {
                                let buf = output.get_or_insert_with(|| {
                                    String::with_capacity(html.len() + rewritten.len())
                                });
                                buf.push_str(&html[copied..value_start]);
                                buf.push_str(&rewritten);
                                copied = value_end;
                            }
                        }
                        i = attr_advance(bytes, value_start, value_end);
                        continue;
                    }

                    i = name_end;
                }
                _ => {
                    saw_self_closing_slash = false;
                    i += 1;
                }
            }
        }

        match output {
            Some(mut buf) => {
                buf.push_str(&html[copied..]);
                std::borrow::Cow::Owned(buf)
            }
            None => std::borrow::Cow::Borrowed(html),
        }
    }

    /// Converts a Markdown URL to an `.html` URL for SSG output.
    pub(in crate::renderer::html::renderer) fn convert_md_url(&self, url: &str) -> Option<String> {
        // A URL is not a filesystem path. Split the query and fragment off
        // before looking at the extension so `./guide.md?plain=1` is still
        // recognized as Markdown, and so whatever follows the path is carried
        // through the rewrite untouched.
        let suffix_start = url.find(['?', '#']).unwrap_or(url.len());
        let (path, suffix) = url.split_at(suffix_start);

        let markdown_extension_len = markdown_extension_len(path)?;

        if !self.options.convert_md_links {
            return None;
        }

        // Another origin's `.md` is not a page this build generates, so there
        // is no `index.html` route to point it at. Leaving it alone also keeps
        // it recognizable as external further down `render_link`, which is
        // what adds `target="_blank" rel="noopener noreferrer"`.
        if is_non_local_url(path) {
            return None;
        }

        // Remove the Markdown extension, including the leading dot.
        let path_without_ext = &path[..path.len() - markdown_extension_len - 1];

        // Check if the source file is an index file
        // index.md stays at the directory level, so relative paths work differently
        let source_is_index = self.is_source_index();

        // Convert path
        let converted = if path.starts_with('/') {
            // Absolute path: /getting-started.md -> {base}getting-started/index.html
            let path_without_slash = &path_without_ext[1..];
            if path_without_slash.is_empty() || path_without_slash == "index" {
                join_base_route(self.options.base_url(), "index.html")
            } else if let Some(dir) = path_without_slash.strip_suffix("/index") {
                // /lib/index.md names the lib/ directory page
                join_base_route(self.options.base_url(), &join2(dir, "/index.html"))
            } else {
                join_base_route(
                    self.options.base_url(),
                    &join2(path_without_slash, "/index.html"),
                )
            }
        } else if path.starts_with("./") {
            // Same-directory relative path
            let name = &path_without_ext[2..]; // Remove "./"
            if name == "index" {
                // ./index.md -> ./index.html (stay in same directory)
                "./index.html".to_string()
            } else if let Some(dir) = name.strip_suffix("/index") {
                // ./lib/index.md names the lib/ directory page
                if source_is_index {
                    join3("./", dir, "/index.html")
                } else {
                    join3("../", dir, "/index.html")
                }
            } else if source_is_index {
                // Source is index.md, so we're at directory level
                // ./types.md -> ./types/index.html
                join3("./", name, "/index.html")
            } else {
                // Source is not index.md (e.g., types.md -> types/index.html)
                // So we need to go up one level
                // ./types.md -> ../types/index.html
                join3("../", name, "/index.html")
            }
        } else if path.starts_with("../") {
            // Parent-relative path
            let rest = &path_without_ext[3..]; // Remove "../"
            if source_is_index {
                // Source is index.md at directory level
                // ../types.md -> ../types/index.html
                if let Some(dir) = rest
                    .strip_suffix("/index")
                    .or_else(|| (rest == "index").then_some(""))
                {
                    if dir.is_empty() {
                        "../index.html".to_string()
                    } else {
                        join3("../", dir, "/index.html")
                    }
                } else {
                    join3("../", rest, "/index.html")
                }
            } else {
                // Source is not index.md, need extra ../
                // ../types.md -> ../../types/index.html
                if let Some(dir) = rest
                    .strip_suffix("/index")
                    .or_else(|| (rest == "index").then_some(""))
                {
                    if dir.is_empty() {
                        "../../index.html".to_string()
                    } else {
                        join3("../../", dir, "/index.html")
                    }
                } else {
                    join3("../../", rest, "/index.html")
                }
            }
        } else {
            // Plain relative path: types.md
            if let Some(dir) = path_without_ext
                .strip_suffix("/index")
                .or_else(|| (path_without_ext == "index").then_some(""))
            {
                if dir.is_empty() {
                    "./index.html".to_string()
                } else if source_is_index {
                    join3("./", dir, "/index.html")
                } else {
                    join3("../", dir, "/index.html")
                }
            } else if source_is_index {
                // Source is index.md
                // types.md -> ./types/index.html
                join3("./", path_without_ext, "/index.html")
            } else {
                // Source is not index.md
                // types.md -> ../types/index.html
                join3("../", path_without_ext, "/index.html")
            }
        };

        // Reattach the query and/or fragment if there was one.
        Some(if suffix.is_empty() {
            converted
        } else {
            append_suffix(converted, suffix)
        })
    }

    /// Checks if the source file is an index file (index.md).
    pub(in crate::renderer::html::renderer) fn is_source_index(&self) -> bool {
        if self.options.source_path().is_empty() {
            return false;
        }
        let source = std::path::Path::new(self.options.source_path());
        source
            .file_stem()
            .is_some_and(|stem| stem.eq_ignore_ascii_case("index"))
    }
}

fn join2(a: &str, b: &str) -> String {
    let mut out = String::with_capacity(a.len() + b.len());
    out.push_str(a);
    out.push_str(b);
    out
}

fn join3(a: &str, b: &str, c: &str) -> String {
    let mut out = String::with_capacity(a.len() + b.len() + c.len());
    out.push_str(a);
    out.push_str(b);
    out.push_str(c);
    out
}

/// Prefixes a generated route with the configured base.
///
/// Every caller passes a route derived from a root-absolute Markdown link, so
/// the result stays root-absolute. An empty base means "no prefix", not "make
/// the route relative": dropping the leading slash would resolve the link
/// against whatever directory the page is served from, and it disagreed with
/// [`HtmlRenderer::apply_base_to_root_absolute_url`], which leaves a
/// root-absolute non-Markdown URL alone under the same configuration. `base`
/// without a leading slash is still taken verbatim, which keeps a deliberately
/// relative base working.
fn join_base_route(base: &str, path: &str) -> String {
    let normalized = base.trim_end_matches('/');
    if normalized.is_empty() {
        join2("/", path)
    } else {
        join3(normalized, "/", path)
    }
}

fn append_suffix(mut converted: String, suffix: &str) -> String {
    converted.reserve(suffix.len());
    converted.push_str(suffix);
    converted
}

/// Returns the length of the trailing Markdown extension of `path`, if it has
/// one.
///
/// This deliberately does not go through `std::path::Path`: a URL only ever
/// separates segments with `/`, and on Windows `Path` would also split on `\`
/// and read a drive letter, so the same href would convert differently
/// depending on the build host.
fn markdown_extension_len(path: &str) -> Option<usize> {
    let file_name = match path.rfind('/') {
        Some(slash) => &path[slash + 1..],
        None => path,
    };
    let dot = file_name.rfind('.')?;
    // A leading dot names a hidden file rather than an extension, matching
    // what `Path::extension` reports for `.md`.
    if dot == 0 {
        return None;
    }

    let extension = &file_name[dot + 1..];
    (extension.eq_ignore_ascii_case("md")
        || extension.eq_ignore_ascii_case("mdx")
        || extension.eq_ignore_ascii_case("markdown"))
    .then_some(extension.len())
}

/// Reports whether `path` addresses another origin, either through a URI
/// scheme (`https:`, `mailto:`) or as a protocol-relative URL (`//cdn.test`).
///
/// `path` must already have had its query and fragment removed, so a `:` or
/// `//` appearing only in those parts cannot be mistaken for a scheme.
fn is_non_local_url(path: &str) -> bool {
    if path.starts_with("//") {
        return true;
    }

    let mut chars = path.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() {
        return false;
    }

    for ch in chars {
        if ch == ':' {
            return true;
        }
        if !(ch.is_ascii_alphanumeric() || matches!(ch, '+' | '.' | '-')) {
            return false;
        }
    }
    false
}

/// Advances past an attribute value, including its closing quote when quoted.
fn attr_advance(bytes: &[u8], value_start: usize, value_end: usize) -> usize {
    let opening_quote = value_start
        .checked_sub(1)
        .and_then(|pos| bytes.get(pos).copied())
        .filter(|byte| matches!(byte, b'"' | b'\''));
    if opening_quote.is_some() && bytes.get(value_end) == opening_quote.as_ref() {
        value_end + 1
    } else {
        value_end
    }
}

/// Skips an HTML comment starting at `<!--`, handling `<!-->`, `<!--->`,
/// `-->`, and HTML5 `--!>` closers.
fn skip_html_comment(bytes: &[u8], start: usize) -> Option<usize> {
    if bytes.get(start + 4) == Some(&b'>') {
        return Some(start + 5);
    }
    if bytes.get(start + 4..start + 6) == Some(b"->") {
        return Some(start + 6);
    }
    let mut cursor = start + 4;
    while let Some(rel) = memchr::memmem::find(&bytes[cursor..], b"--") {
        let pos = cursor + rel;
        if bytes.get(pos + 2) == Some(&b'>') {
            return Some(pos + 3);
        }
        if bytes.get(pos + 2..pos + 4) == Some(b"!>") {
            return Some(pos + 4);
        }
        cursor = pos + 1;
    }
    None
}

fn is_html_raw_text_tag(tag_name: &str) -> bool {
    tag_name.eq_ignore_ascii_case("textarea")
        || tag_name.eq_ignore_ascii_case("title")
        || tag_name.eq_ignore_ascii_case("style")
        || tag_name.eq_ignore_ascii_case("script")
        || tag_name.eq_ignore_ascii_case("xmp")
}

/// Skips raw-text/RCDATA element bodies until their matching `</tag_name>` so
/// literal `<` and quotes inside them do not confuse tag scanning.
fn skip_html_raw_text_body(bytes: &[u8], start: usize, tag_name: &str) -> Option<usize> {
    let mut cursor = start;
    while let Some(rel) = memchr::memchr(b'<', &bytes[cursor..]) {
        let pos = cursor + rel;
        if bytes.get(pos + 1) == Some(&b'/') {
            let name_start = pos + 2;
            let name_end = name_start + tag_name.len();
            if bytes
                .get(name_start..name_end)
                .is_some_and(|slice| slice.eq_ignore_ascii_case(tag_name.as_bytes()))
                && bytes.get(name_end).is_some_and(|byte| {
                    matches!(byte, b'>' | b'/' | b' ' | b'\t' | b'\n' | b'\r' | b'\x0c')
                })
            {
                return memchr::memchr(b'>', &bytes[name_end..]).map(|gt| name_end + gt + 1);
            }
        }
        cursor = pos + 1;
    }
    None
}
