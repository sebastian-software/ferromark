//! Reject active SVG and resource loads before either HTML or JSX can use it.

use cssparser::{ParseError, Parser, ParserInput, Token};
use roxmltree::Document;

use super::FerriguraError as Error;

const SVG: &str = "http://www.w3.org/2000/svg";
const XLINK: &str = "http://www.w3.org/1999/xlink";

pub fn sanitize(svg: &str) -> Result<String, Error> {
    // roxmltree rejects DTDs by default. Processing instructions have no place
    // in inline SVG either, including xml-stylesheet instructions.
    let document = Document::parse(svg)?;
    let root = document.root_element();
    if root.tag_name().name() != "svg" || root.tag_name().namespace() != Some(SVG) {
        return Err(reject("the root element"));
    }
    for node in document.descendants() {
        if node.is_pi() {
            return Err(reject("a processing instruction"));
        }
        if !node.is_element() {
            continue;
        }
        let tag = node.tag_name();
        if tag.namespace() != Some(SVG) || !element(tag.name()) {
            return Err(reject(["element '", tag.name(), "'"].concat()));
        }
        for attr in node.attributes() {
            let name = attr.name();
            if let Some(namespace) = attr.namespace()
                && !(namespace == XLINK && name == "href")
            {
                return Err(reject("an attribute namespace"));
            }
            if name == "href" {
                if !fragment(attr.value()) {
                    return Err(reject("a non-local href"));
                }
            } else if !attribute(name) {
                return Err(reject(["attribute '", name, "'"].concat()));
            }
            if matches!(
                name,
                "style"
                    | "fill"
                    | "stroke"
                    | "filter"
                    | "mask"
                    | "clip-path"
                    | "marker-start"
                    | "marker-mid"
                    | "marker-end"
                    | "cursor"
            ) {
                css(attr.value())?;
            }
        }
        if tag.name() == "style" {
            if node.children().any(|child| child.is_element()) {
                return Err(reject("non-text CSS content"));
            }
            // XML comments disappear during serialization. Tokenize the complete
            // concatenated text that the browser will receive, not each text node.
            let stylesheet: String = node
                .children()
                .filter(roxmltree::Node::is_text)
                .filter_map(|child| child.text())
                .collect();
            css(&stylesheet)?;
        }
    }
    // Re-serialize XML text and attributes rather than forwarding CDATA,
    // comments, namespace prefixes or parser-specific source spellings to HTML.
    let mut output = String::with_capacity(svg.len());
    let mut pending = Vec::from([(root, false)]);
    while let Some((node, closing)) = pending.pop() {
        if node.is_element() {
            let name = node.tag_name().name();
            if closing {
                output.push_str("</");
                output.push_str(name);
                output.push('>');
                continue;
            }
            output.push('<');
            output.push_str(name);
            if node == root {
                output.push_str(" xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\"");
            }
            for attr in node.attributes() {
                output.push(' ');
                if attr.namespace() == Some(XLINK) {
                    output.push_str("xlink:");
                }
                output.push_str(attr.name());
                output.push_str("=\"");
                escape(&mut output, attr.value(), true);
                output.push('"');
            }
            output.push('>');
            pending.push((node, true));
            pending.extend(node.children().rev().map(|child| (child, false)));
        } else if let Some(text) = node.text().filter(|_| node.is_text()) {
            escape(&mut output, text, false);
        }
    }
    Ok(output)
}

fn escape(output: &mut String, value: &str, attribute: bool) {
    for ch in value.chars() {
        match ch {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' if attribute => output.push_str("&quot;"),
            other => output.push(other),
        }
    }
}

fn reject(reason: impl Into<String>) -> Error {
    Error::UnsafeSvg(reason.into())
}

fn element(name: &str) -> bool {
    matches!(
        name,
        "svg"
            | "g"
            | "defs"
            | "style"
            | "title"
            | "desc"
            | "path"
            | "rect"
            | "circle"
            | "ellipse"
            | "line"
            | "polyline"
            | "polygon"
            | "text"
            | "tspan"
            | "textPath"
            | "marker"
            | "clipPath"
            | "mask"
            | "linearGradient"
            | "radialGradient"
            | "stop"
            | "filter"
            | "feGaussianBlur"
            | "feOffset"
            | "feMerge"
            | "feMergeNode"
            | "feColorMatrix"
            | "feFlood"
            | "feComposite"
            | "feBlend"
            | "feDropShadow"
            | "pattern"
            | "symbol"
            | "use"
            | "a"
            | "switch"
    )
}

fn attribute(name: &str) -> bool {
    name.starts_with("aria-")
        || name.starts_with("data-")
        || matches!(
            name,
            "id" | "name"
                | "class"
                | "style"
                | "role"
                | "viewBox"
                | "width"
                | "height"
                | "x"
                | "y"
                | "x1"
                | "x2"
                | "y1"
                | "y2"
                | "cx"
                | "cy"
                | "r"
                | "rx"
                | "ry"
                | "dx"
                | "dy"
                | "d"
                | "points"
                | "transform"
                | "preserveAspectRatio"
                | "fill"
                | "fill-opacity"
                | "fill-rule"
                | "stroke"
                | "stroke-width"
                | "stroke-opacity"
                | "stroke-dasharray"
                | "stroke-dashoffset"
                | "stroke-linecap"
                | "stroke-linejoin"
                | "stroke-miterlimit"
                | "opacity"
                | "color"
                | "font-family"
                | "font-size"
                | "font-weight"
                | "font-style"
                | "text-anchor"
                | "text-decoration"
                | "textLength"
                | "lengthAdjust"
                | "startOffset"
                | "dominant-baseline"
                | "alignment-baseline"
                | "baseline-shift"
                | "letter-spacing"
                | "word-spacing"
                | "white-space"
                | "direction"
                | "unicode-bidi"
                | "writing-mode"
                | "marker-start"
                | "marker-mid"
                | "marker-end"
                | "markerWidth"
                | "markerHeight"
                | "markerUnits"
                | "orient"
                | "refX"
                | "refY"
                | "clip-path"
                | "clip-rule"
                | "clipPathUnits"
                | "mask"
                | "maskUnits"
                | "maskContentUnits"
                | "filter"
                | "filterUnits"
                | "primitiveUnits"
                | "stdDeviation"
                | "in"
                | "in2"
                | "result"
                | "type"
                | "values"
                | "operator"
                | "k1"
                | "k2"
                | "k3"
                | "k4"
                | "mode"
                | "flood-color"
                | "flood-opacity"
                | "gradientUnits"
                | "gradientTransform"
                | "spreadMethod"
                | "offset"
                | "stop-color"
                | "stop-opacity"
                | "fx"
                | "fy"
                | "fr"
                | "patternUnits"
                | "patternContentUnits"
                | "patternTransform"
                | "vector-effect"
                | "shape-rendering"
                | "text-rendering"
                | "paint-order"
                | "display"
                | "visibility"
                | "overflow"
                | "cursor"
                | "focusable"
                | "tabindex"
        )
}

fn fragment(value: &str) -> bool {
    value.strip_prefix('#').is_some_and(|id| {
        !id.is_empty()
            && id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b':'))
    })
}

fn local_resource(value: &str) -> bool {
    if fragment(value) {
        return true;
    }
    // Font data is embedded by Ferrigura, never fetched. Do not permit other
    // data MIME types (notably SVG/HTML), file URLs, relative paths or origins.
    let Some((mime, data)) = value
        .strip_prefix("data:font/")
        .and_then(|s| s.split_once(";base64,"))
    else {
        return false;
    };
    matches!(mime, "ttf" | "otf" | "woff" | "woff2")
        && !data.is_empty()
        && data
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'/' | b'='))
}

fn css(value: &str) -> Result<(), Error> {
    let mut input = ParserInput::new(value);
    tokens(&mut Parser::new(&mut input), 0)
        .map_err(|_| reject("CSS with active or non-local resources"))
}

fn tokens<'i>(parser: &mut Parser<'i, '_>, depth: u8) -> Result<(), ParseError<'i, String>> {
    if depth > 64 {
        return Err(parser.new_custom_error("unsafe CSS token".to_owned()));
    }
    while !parser.is_exhausted() {
        match parser.next()?.clone() {
            Token::UnquotedUrl(url) if !local_resource(&url) => {
                return Err(parser.new_custom_error("unsafe CSS token".to_owned()));
            }
            Token::Function(name) if name.eq_ignore_ascii_case("url") => {
                parser.parse_nested_block(|nested| {
                    let url = nested.expect_string()?;
                    if !local_resource(url) {
                        return Err(nested.new_custom_error("unsafe CSS token".to_owned()));
                    }
                    Ok(())
                })?;
            }
            Token::Function(name) => {
                if !matches!(
                    name.to_ascii_lowercase().as_str(),
                    "var"
                        | "not"
                        | "is"
                        | "where"
                        | "has"
                        | "nth-child"
                        | "nth-of-type"
                        | "calc"
                        | "min"
                        | "max"
                        | "clamp"
                        | "rgb"
                        | "rgba"
                        | "hsl"
                        | "hsla"
                        | "oklch"
                        | "oklab"
                        | "lab"
                        | "lch"
                        | "color"
                        | "color-mix"
                        | "light-dark"
                        | "scale"
                        | "scalex"
                        | "scaley"
                        | "translate"
                        | "translatex"
                        | "translatey"
                        | "matrix"
                        | "rotate"
                        | "skew"
                        | "skewx"
                        | "skewy"
                        | "blur"
                        | "drop-shadow"
                        | "cubic-bezier"
                        | "steps"
                        | "format"
                ) {
                    return Err(parser.new_custom_error(["CSS function ", &name].concat()));
                }
                parser.parse_nested_block(|nested| tokens(nested, depth + 1))?;
            }
            Token::ParenthesisBlock | Token::SquareBracketBlock | Token::CurlyBracketBlock => {
                parser.parse_nested_block(|nested| tokens(nested, depth + 1))?;
            }
            Token::AtKeyword(name)
                if !matches!(
                    name.to_ascii_lowercase().as_str(),
                    "media" | "supports" | "keyframes" | "font-face"
                ) =>
            {
                return Err(parser.new_custom_error("unsafe CSS token".to_owned()));
            }
            Token::BadUrl(_)
            | Token::BadString(_)
            | Token::CloseParenthesis
            | Token::CloseSquareBracket
            | Token::CloseCurlyBracket => {
                return Err(parser.new_custom_error("unsafe CSS token".to_owned()));
            }
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::sanitize;

    fn svg(body: &str) -> String {
        format!(r#"<svg xmlns="http://www.w3.org/2000/svg">{body}</svg>"#)
    }

    #[test]
    fn rejects_active_elements_attributes_and_non_local_resources() {
        for body in [
            "<script>alert(1)</script>",
            "<foreignObject><div>HTML</div></foreignObject>",
            "<image href='https://example.test/image.svg'/>",
            "<g onclick='alert(1)'/>",
            "<g onLoad='alert(1)'/>",
            "<use href='https://example.test/x.svg#id'/>",
            "<use href='//example.test/x.svg'/>",
            "<use href='other.svg#id'/>",
            "<use href='data:image/svg+xml;base64,PHN2Zy8+'/>",
            "<g xml:base='https://example.test/'/>",
            "<animate attributeName='href' values='javascript:alert(1)'/>",
            "<g style='fill:url(https://example.test/x)'/>",
            r#"<g style="fill:u\72l('https://example.test/x')"/>"#,
            "<style>@import 'https://example.test/x';</style>",
            r"<style>@\69mport 'https://example.test/x';</style>",
            "<style>g{fill:image-set('https://example.test/x')}</style>",
            "<style>g{fill:expression(alert(1))}</style>",
            "<style>g{fill:url(data:image/svg+xml;base64,AA==)}</style>",
            "<style>@namespace x 'https://example.test/';</style>",
            "<style><g>@import 'https://example.test/x';</g></style>",
        ] {
            assert!(sanitize(&svg(body)).is_err(), "accepted {body}");
        }
        for source in [
            "<svg xmlns='http://www.w3.org/1999/xhtml'/>",
            "<?xml-stylesheet href='https://example.test/style.css'?><svg/>",
            "<!DOCTYPE svg [<!ENTITY x 'unsafe'>]><svg>&x;</svg>",
        ] {
            assert!(sanitize(source).is_err(), "accepted {source}");
        }
    }

    #[test]
    fn keeps_local_paint_and_embedded_fonts_and_escapes_cdata_for_html() {
        let source = svg(
            r##"<defs><path id="shape"/></defs><use href="#shape" fill="url(#paint)"/><style>@font-face{src:url(data:font/ttf;base64,AA==)}@media(prefers-color-scheme:dark){g{fill:rgb(1,2,3)}}</style><text><![CDATA[<script> & {literal}]]></text>"##,
        );
        let output = sanitize(&source).unwrap();
        assert!(output.contains("url(#paint)"));
        assert!(output.contains("data:font/ttf;base64,AA=="));
        assert!(output.contains("&lt;script&gt; &amp; {literal}"));
        assert!(!output.contains("<![CDATA["));
        assert!(!output.contains("<script>"));
    }
}
