//! Convert the engine's XML SVG to intrinsic JSX without evaluating source text.

use std::fmt::Write;

use roxmltree::Document;

pub fn convert(svg: &str) -> Result<String, roxmltree::Error> {
    let document = Document::parse(svg)?;
    let root = document.root_element();
    let mut output = String::with_capacity(svg.len());
    // Traversal is iterative so deeply nested diagrams do not add Rust stack frames.
    let mut pending = vec![(root, false)];
    while let Some((node, closing)) = pending.pop() {
        match (node, closing) {
            (node, false) if node.is_element() => {
                output.push('<');
                output.push_str(node.tag_name().name());
                if node == root {
                    output.push_str(" xmlns=\"http://www.w3.org/2000/svg\"");
                    if root.lookup_namespace_uri(Some("xlink")).is_some() {
                        output.push_str(" xmlnsXlink=\"http://www.w3.org/1999/xlink\"");
                    }
                }
                for attribute in node.attributes() {
                    if attribute.name() == "style" && attribute.namespace().is_none() {
                        write_style(&mut output, attribute.value());
                    } else {
                        let name = attribute_name(attribute.name(), attribute.namespace());
                        write!(output, " {name}={{{}}}", string(attribute.value())).unwrap();
                    }
                }
                output.push('>');
                pending.push((node, true));
                pending.extend(node.children().rev().map(|child| (child, false)));
            }
            (node, true) if node.is_element() => {
                write!(output, "</{}>", node.tag_name().name()).unwrap();
            }
            (node, false) if node.is_text() => {
                write!(output, "{{{}}}", string(node.text().unwrap_or_default())).unwrap();
            }
            _ => {}
        }
    }
    output.push('\n');
    Ok(output)
}

fn string(value: &str) -> String {
    serde_json::to_string(value).expect("serializing a string cannot fail")
}

fn attribute_name(name: &str, namespace: Option<&str>) -> String {
    let prefix = match namespace {
        Some("http://www.w3.org/1999/xlink") => Some("xlink"),
        Some("http://www.w3.org/XML/1998/namespace") => Some("xml"),
        _ => None,
    };
    if let Some(prefix) = prefix {
        let mut chars = name.chars();
        return prefix.to_owned()
            + &chars
                .next()
                .map_or_else(String::new, |ch| ch.to_uppercase().to_string())
            + chars.as_str();
    }
    match name {
        "class" => "className".to_owned(),
        "for" => "htmlFor".to_owned(),
        "tabindex" => "tabIndex".to_owned(),
        name if name.starts_with("data-") || name.starts_with("aria-") => name.to_owned(),
        name => camel_case(name),
    }
}

fn camel_case(name: &str) -> String {
    let mut output = String::with_capacity(name.len());
    let mut upper = false;
    for ch in name.chars() {
        if ch == '-' {
            upper = true;
        } else if upper {
            output.extend(ch.to_uppercase());
            upper = false;
        } else {
            output.push(ch);
        }
    }
    output
}

fn write_style(output: &mut String, value: &str) {
    output.push_str(" style={{");
    let mut emitted = false;
    for declaration in declarations(value) {
        let Some((property, value)) = declaration.split_once(':') else {
            continue;
        };
        let (property, value) = (property.trim(), value.trim());
        if property.is_empty() || value.is_empty() {
            continue;
        }
        let name = if property.starts_with("--") {
            property.to_owned()
        } else if let Some(tail) = property.strip_prefix("-ms-") {
            format!("ms{}", camel_case(&format!("-{tail}")))
        } else {
            camel_case(property)
        };
        if emitted {
            output.push(',');
        }
        write!(output, "{}:{}", string(&name), string(value)).unwrap();
        emitted = true;
    }
    output.push_str("}}");
}

fn declarations(style: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut start = 0;
    let mut quote = None;
    let mut depth = 0usize;
    let mut escaped = false;
    let mut comment = false;
    let mut chars = style.char_indices().peekable();
    while let Some((index, ch)) = chars.next() {
        if comment {
            if ch == '*' && chars.peek().is_some_and(|(_, ch)| *ch == '/') {
                chars.next();
                comment = false;
            }
        } else if escaped {
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if let Some(active) = quote {
            if ch == active {
                quote = None;
            }
        } else {
            match ch {
                '/' if chars.peek().is_some_and(|(_, ch)| *ch == '*') => {
                    chars.next();
                    comment = true;
                }
                '\'' | '"' => quote = Some(ch),
                '(' | '[' | '{' => depth = depth.saturating_add(1),
                ')' | ']' | '}' => depth = depth.saturating_sub(1),
                ';' if depth == 0 => {
                    result.push(&style[start..index]);
                    start = index + 1;
                }
                _ => {}
            }
        }
    }
    result.push(&style[start..]);
    result
}

#[cfg(test)]
mod tests {
    use super::convert;

    #[test]
    fn preserves_svg_names_namespaces_styles_and_literal_source() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 10 10"><style>.a { fill: red; }</style><clipPath id="clip"/><use xlink:href="#clip" stroke-width="2" class="a" aria-label="a &amp; b" style="--color: red;fill:var(--color);background:url('data:a;b');-webkit-transform:scale(1);-ms-transform:scale(2)"/><text>{literal} &lt;safe&gt; 日本語</text></svg>"##;
        let jsx = convert(svg).unwrap();
        for expected in [
            "viewBox=",
            "clipPath",
            "xlinkHref={\"#clip\"}",
            "strokeWidth={\"2\"}",
            "className={\"a\"}",
            "aria-label={\"a & b\"}",
            "\"WebkitTransform\":\"scale(1)\"",
            "\"msTransform\":\"scale(2)\"",
            "url('data:a;b')",
            "{\".a { fill: red; }\"}",
            "{\"{literal} <safe> 日本語\"}",
        ] {
            assert!(jsx.contains(expected), "missing {expected} in {jsx}");
        }
        assert!(!jsx.contains("dangerouslySetInnerHTML"));
    }
}
