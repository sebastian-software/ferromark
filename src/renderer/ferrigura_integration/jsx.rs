//! Convert the engine's XML SVG to intrinsic JSX without evaluating source text.

use roxmltree::Document;

pub fn convert(svg: &str) -> Result<String, roxmltree::Error> {
    let document = Document::parse(svg)?;
    Ok(convert_document(&document, svg.len(), false))
}

pub fn convert_document(
    document: &Document<'_>,
    capacity_hint: usize,
    force_xlink_xmlns: bool,
) -> String {
    let root = document.root_element();
    let mut output = String::with_capacity(capacity_hint);
    // Traversal is iterative so deeply nested diagrams do not add Rust stack frames.
    let mut pending = Vec::from([(root, false)]);
    let mut style_prop = String::new();
    while let Some((node, closing)) = pending.pop() {
        match (node, closing) {
            (node, false) if node.is_element() => {
                output.push('<');
                output.push_str(node.tag_name().name());
                if node == root {
                    output.push_str(" xmlns=\"http://www.w3.org/2000/svg\"");
                    if force_xlink_xmlns || root.lookup_namespace_uri(Some("xlink")).is_some() {
                        output.push_str(" xmlnsXlink=\"http://www.w3.org/1999/xlink\"");
                    }
                }
                for attribute in node.attributes() {
                    if attribute.name() == "style" && attribute.namespace().is_none() {
                        write_style(&mut output, attribute.value(), &mut style_prop);
                    } else {
                        output.push(' ');
                        write_attribute_name(&mut output, attribute.name(), attribute.namespace());
                        output.push_str("={");
                        write_json_string(&mut output, attribute.value());
                        output.push('}');
                    }
                }
                output.push('>');
                pending.push((node, true));
                pending.extend(node.children().rev().map(|child| (child, false)));
            }
            (node, true) if node.is_element() => {
                output.push_str("</");
                output.push_str(node.tag_name().name());
                output.push('>');
            }
            (node, false) if node.is_text() => {
                output.push('{');
                write_json_string(&mut output, node.text().unwrap_or_default());
                output.push('}');
            }
            _ => {}
        }
    }
    output.push('\n');
    output
}

fn write_json_string(output: &mut String, value: &str) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    output.push('"');
    let bytes = value.as_bytes();
    let mut start = 0;
    for (idx, &byte) in bytes.iter().enumerate() {
        let escaped = match byte {
            b'"' => "\\\"",
            b'\\' => "\\\\",
            0x08 => "\\b",
            0x0c => "\\f",
            b'\n' => "\\n",
            b'\r' => "\\r",
            b'\t' => "\\t",
            0x00..=0x1f => {
                output.push_str(&value[start..idx]);
                output.push_str("\\u00");
                output.push(HEX[(byte >> 4) as usize] as char);
                output.push(HEX[(byte & 0x0f) as usize] as char);
                start = idx + 1;
                continue;
            }
            _ => continue,
        };
        output.push_str(&value[start..idx]);
        output.push_str(escaped);
        start = idx + 1;
    }
    output.push_str(&value[start..]);
    output.push('"');
}

fn write_attribute_name(output: &mut String, name: &str, namespace: Option<&str>) {
    let prefix = match namespace {
        Some("http://www.w3.org/1999/xlink") => Some("xlink"),
        Some("http://www.w3.org/XML/1998/namespace") => Some("xml"),
        _ => None,
    };
    if let Some(prefix) = prefix {
        output.push_str(prefix);
        let mut chars = name.chars();
        if let Some(first) = chars.next() {
            output.extend(first.to_uppercase());
            output.push_str(chars.as_str());
        }
        return;
    }
    match name {
        "class" => output.push_str("className"),
        "for" => output.push_str("htmlFor"),
        "tabindex" => output.push_str("tabIndex"),
        name if name.starts_with("data-") || name.starts_with("aria-") => output.push_str(name),
        name => write_camel_case(output, name),
    }
}

fn write_camel_case(output: &mut String, name: &str) {
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
}

fn write_style(output: &mut String, value: &str, prop_buf: &mut String) {
    output.push_str(" style={{");
    let mut emitted = false;
    for_each_declaration(value, |declaration| {
        let Some((property, value)) = declaration.split_once(':') else {
            return;
        };
        let (property, value) = (property.trim(), value.trim());
        if property.is_empty() || value.is_empty() {
            return;
        }
        if emitted {
            output.push(',');
        }
        if property.starts_with("--") || !property.contains('-') {
            write_json_string(output, property);
        } else {
            prop_buf.clear();
            if let Some(tail) = property.strip_prefix("-ms-") {
                prop_buf.push_str("ms");
                let mut upper = true;
                for ch in tail.chars() {
                    if ch == '-' {
                        upper = true;
                    } else if upper {
                        prop_buf.extend(ch.to_uppercase());
                        upper = false;
                    } else {
                        prop_buf.push(ch);
                    }
                }
            } else {
                write_camel_case(prop_buf, property);
            }
            write_json_string(output, prop_buf);
        }
        output.push(':');
        write_json_string(output, value);
        emitted = true;
    });
    output.push_str("}}");
}

fn for_each_declaration<'a>(style: &'a str, mut on_decl: impl FnMut(&'a str)) {
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
                    on_decl(&style[start..index]);
                    start = index + 1;
                }
                _ => {}
            }
        }
    }
    on_decl(&style[start..]);
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

    #[test]
    fn write_json_string_matches_serde_json_for_ascii_and_unicode() {
        for byte in 0u8..=0x7f {
            let raw = [b'x', byte, b'y'];
            let s = std::str::from_utf8(&raw).unwrap();
            let mut got = String::new();
            super::write_json_string(&mut got, s);
            assert_eq!(got, serde_json::to_string(s).unwrap(), "byte {byte:#x}");
        }
        for sample in [
            "",
            "plain text",
            "\"quoted\" \\ backslash \n\r\t\x08\x0c\x00\x1f",
            "<script>&'\"/</script>",
            "日本語 😀 \u{2028}\u{2029}",
        ] {
            let mut got = String::new();
            super::write_json_string(&mut got, sample);
            assert_eq!(
                got,
                serde_json::to_string(sample).unwrap(),
                "sample {sample:?}"
            );
        }
    }

    #[test]
    fn validated_svg_matches_sanitize_roundtrip_across_xml_edge_cases() {
        for raw_svg in [
            r#"<svg xmlns="http://www.w3.org/2000/svg"><text><![CDATA[]]></text></svg>"#,
            r#"<svg xmlns="http://www.w3.org/2000/svg"><g style="x&#13;-&#10;y:z"/></svg>"#,
            r#"<svg xmlns="http://www.w3.org/2000/svg"><text>X&#13;&amp;Y</text></svg>"#,
            r#"<svg xmlns="http://www.w3.org/2000/svg"><text>X&#13;<!--drop-->&#10;Y<![CDATA[&1]]>Z</text></svg>"#,
            r#"<svg xmlns="http://www.w3.org/2000/svg"><text aria-label="a&#13;b&#9;c">plain</text></svg>"#,
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><path d="M0 0"/><text>hello &amp; world</text></svg>"#,
        ] {
            let sanitized = super::super::svg::sanitize(raw_svg).unwrap();
            let expected = convert(&sanitized).unwrap();

            let doc = roxmltree::Document::parse(raw_svg).unwrap();
            let direct_jsx_safe = super::super::svg::validate(&doc).unwrap();
            let actual = if direct_jsx_safe {
                super::convert_document(&doc, raw_svg.len(), true)
            } else {
                convert(&super::super::svg::serialize_html(&doc, raw_svg.len())).unwrap()
            };
            assert_eq!(actual, expected, "mismatch for {raw_svg}");
        }
    }

    #[test]
    fn mermaid_sequence_diagram_cr_and_entity_match_xml_roundtrip() {
        use crate::renderer::JsxRenderHooks;

        let mut hooks = super::super::FerriguraHooks::new(
            ferrigura::Renderer::new(&ferrigura::Value::Object(std::collections::BTreeMap::new())),
            "diagram",
        )
        .expect("valid id prefix");

        let jsx_lf = hooks
            .render_code_block(crate::renderer::JsxCodeBlockInput {
                code: "sequenceDiagram\nA->>B: X#13;Y\n",
                language: Some("mermaid"),
                meta: None,
                span: crate::ast::Span::new(0, 30),
            })
            .expect("mermaid JSX output");
        assert!(jsx_lf.contains(r#"{"X\nY"}"#), "{jsx_lf}");

        let jsx_amp = hooks
            .render_code_block(crate::renderer::JsxCodeBlockInput {
                code: "sequenceDiagram\nA->>B: X#13;&Y\n",
                language: Some("mermaid"),
                meta: None,
                span: crate::ast::Span::new(0, 31),
            })
            .expect("mermaid JSX output");
        assert!(jsx_amp.contains(r#"{"X\r&Y"}"#), "{jsx_amp}");
    }
}
