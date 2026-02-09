use super::expr::find_expression_end;

/// Information about a parsed JSX tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagInfo<'a> {
    /// Tag name (e.g. `"MyComponent"`, `"Foo.Bar"`, `""` for fragments).
    pub name: &'a str,
    /// Whether this is a self-closing tag (`<Foo />`).
    pub is_self_closing: bool,
    /// Whether this is a closing tag (`</Foo>`).
    pub is_closing: bool,
    /// Byte offset after the closing `>`.
    pub end_offset: usize,
}

/// Parse a JSX tag at the start of `input`.
///
/// `input` must begin with `<`. Returns `None` for invalid JSX
/// (e.g. `< 5`, `<123`, bare `<` without a valid tag).
///
/// Handles:
/// - Opening tags: `<Foo>`, `<Foo prop="val">`
/// - Closing tags: `</Foo>`
/// - Self-closing tags: `<Foo />`
/// - Fragments: `<>`, `</>`
/// - Member expressions: `<Foo.Bar>`
/// - Namespaces: `<svg:rect>`
/// - Attributes with string values, expression values (`{...}`), and bare attributes
/// - Multiline attributes (byte-based, not line-based)
pub fn parse_jsx_tag(input: &[u8]) -> Option<TagInfo<'_>> {
    let len = input.len();
    if len < 2 || input[0] != b'<' {
        return None;
    }

    let mut pos = 1;

    // Check for closing tag
    let is_closing = if pos < len && input[pos] == b'/' {
        pos += 1;
        true
    } else {
        false
    };

    // Check for fragment: `<>` or `</>`
    if pos < len && input[pos] == b'>' {
        return Some(TagInfo {
            name: "",
            is_self_closing: false,
            is_closing,
            end_offset: pos + 1,
        });
    }

    // Tag name must start with an ASCII letter (JSX components are uppercase,
    // HTML-like lowercase — both valid)
    if pos >= len || !input[pos].is_ascii_alphabetic() {
        return None;
    }

    let name_start = pos;
    // Consume identifier: [a-zA-Z0-9_-]
    while pos < len
        && (input[pos].is_ascii_alphanumeric() || input[pos] == b'_' || input[pos] == b'-')
    {
        pos += 1;
    }

    // Handle member expressions (Foo.Bar.Baz) and namespaces (svg:rect)
    while pos < len && (input[pos] == b'.' || input[pos] == b':') {
        pos += 1; // skip `.` or `:`
        // Must be followed by an identifier
        if pos >= len || !input[pos].is_ascii_alphabetic() {
            return None;
        }
        while pos < len
            && (input[pos].is_ascii_alphanumeric() || input[pos] == b'_' || input[pos] == b'-')
        {
            pos += 1;
        }
    }

    let name = std::str::from_utf8(&input[name_start..pos]).ok()?;

    // For closing tags, just expect `>`
    if is_closing {
        pos = skip_whitespace(input, pos);
        if pos < len && input[pos] == b'>' {
            return Some(TagInfo {
                name,
                is_self_closing: false,
                is_closing: true,
                end_offset: pos + 1,
            });
        }
        return None;
    }

    // Parse attributes until `>` or `/>`
    pos = skip_whitespace(input, pos);
    while pos < len && input[pos] != b'>' && !(input[pos] == b'/' && pos + 1 < len && input[pos + 1] == b'>') {
        // Attribute: either `{...spread}` or `name` or `name=value`
        if input[pos] == b'{' {
            // Spread attribute or expression
            let end = find_expression_end(&input[pos..])?;
            pos += end;
        } else if input[pos].is_ascii_alphabetic() || input[pos] == b'_' {
            // Attribute name
            while pos < len
                && (input[pos].is_ascii_alphanumeric()
                    || input[pos] == b'_'
                    || input[pos] == b'-'
                    || input[pos] == b':'
                    || input[pos] == b'.')
            {
                pos += 1;
            }
            pos = skip_whitespace(input, pos);
            // Check for `=`
            if pos < len && input[pos] == b'=' {
                pos += 1;
                pos = skip_whitespace(input, pos);
                // Value: string or expression
                if pos >= len {
                    return None;
                }
                match input[pos] {
                    b'"' => {
                        pos = skip_double_quoted(input, pos)?;
                    }
                    b'\'' => {
                        pos = skip_single_quoted(input, pos)?;
                    }
                    b'{' => {
                        let end = find_expression_end(&input[pos..])?;
                        pos += end;
                    }
                    _ => {
                        // Bare value (not standard JSX, but be lenient)
                        while pos < len
                            && !input[pos].is_ascii_whitespace()
                            && input[pos] != b'>'
                            && input[pos] != b'/'
                        {
                            pos += 1;
                        }
                    }
                }
            }
        } else {
            // Unexpected character in attribute position
            return None;
        }
        pos = skip_whitespace(input, pos);
    }

    if pos >= len {
        return None;
    }

    // Check for self-closing `/>`
    if input[pos] == b'/' && pos + 1 < len && input[pos + 1] == b'>' {
        return Some(TagInfo {
            name,
            is_self_closing: true,
            is_closing: false,
            end_offset: pos + 2,
        });
    }

    // Regular `>`
    if input[pos] == b'>' {
        return Some(TagInfo {
            name,
            is_self_closing: false,
            is_closing: false,
            end_offset: pos + 1,
        });
    }

    None
}

fn skip_whitespace(bytes: &[u8], mut pos: usize) -> usize {
    while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
        pos += 1;
    }
    pos
}

fn skip_double_quoted(bytes: &[u8], start: usize) -> Option<usize> {
    let len = bytes.len();
    let mut pos = start + 1;
    while pos < len {
        match bytes[pos] {
            b'\\' => pos += 2,
            b'"' => return Some(pos + 1),
            _ => pos += 1,
        }
    }
    None
}

fn skip_single_quoted(bytes: &[u8], start: usize) -> Option<usize> {
    let len = bytes.len();
    let mut pos = start + 1;
    while pos < len {
        match bytes[pos] {
            b'\\' => pos += 2,
            b'\'' => return Some(pos + 1),
            _ => pos += 1,
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_open_tag() {
        let info = parse_jsx_tag(b"<Foo>").unwrap();
        assert_eq!(info.name, "Foo");
        assert!(!info.is_closing);
        assert!(!info.is_self_closing);
        assert_eq!(info.end_offset, 5);
    }

    #[test]
    fn simple_close_tag() {
        let info = parse_jsx_tag(b"</Foo>").unwrap();
        assert_eq!(info.name, "Foo");
        assert!(info.is_closing);
        assert!(!info.is_self_closing);
        assert_eq!(info.end_offset, 6);
    }

    #[test]
    fn self_closing_tag() {
        let info = parse_jsx_tag(b"<Foo />").unwrap();
        assert_eq!(info.name, "Foo");
        assert!(info.is_self_closing);
        assert!(!info.is_closing);
        assert_eq!(info.end_offset, 7);
    }

    #[test]
    fn fragment_open() {
        let info = parse_jsx_tag(b"<>").unwrap();
        assert_eq!(info.name, "");
        assert!(!info.is_closing);
        assert!(!info.is_self_closing);
        assert_eq!(info.end_offset, 2);
    }

    #[test]
    fn fragment_close() {
        let info = parse_jsx_tag(b"</>").unwrap();
        assert_eq!(info.name, "");
        assert!(info.is_closing);
        assert_eq!(info.end_offset, 3);
    }

    #[test]
    fn member_expression() {
        let info = parse_jsx_tag(b"<Foo.Bar>").unwrap();
        assert_eq!(info.name, "Foo.Bar");
        assert!(!info.is_closing);
        assert_eq!(info.end_offset, 9);
    }

    #[test]
    fn namespace() {
        let info = parse_jsx_tag(b"<svg:rect>").unwrap();
        assert_eq!(info.name, "svg:rect");
        assert_eq!(info.end_offset, 10);
    }

    #[test]
    fn attribute_string() {
        let info = parse_jsx_tag(b"<Foo bar=\"baz\">").unwrap();
        assert_eq!(info.name, "Foo");
        assert_eq!(info.end_offset, 15);
    }

    #[test]
    fn attribute_single_quoted() {
        let info = parse_jsx_tag(b"<Foo bar='baz'>").unwrap();
        assert_eq!(info.name, "Foo");
        assert_eq!(info.end_offset, 15);
    }

    #[test]
    fn attribute_expression() {
        let info = parse_jsx_tag(b"<Foo bar={x}>").unwrap();
        assert_eq!(info.name, "Foo");
        assert_eq!(info.end_offset, 13);
    }

    #[test]
    fn spread_attribute() {
        let info = parse_jsx_tag(b"<Foo {...props}>").unwrap();
        assert_eq!(info.name, "Foo");
        assert_eq!(info.end_offset, 16);
    }

    #[test]
    fn bare_attribute() {
        let info = parse_jsx_tag(b"<Foo disabled>").unwrap();
        assert_eq!(info.name, "Foo");
        assert_eq!(info.end_offset, 14);
    }

    #[test]
    fn multiline_attributes() {
        let info = parse_jsx_tag(b"<Foo\n  bar=\"baz\"\n  qux={1}\n>").unwrap();
        assert_eq!(info.name, "Foo");
        assert_eq!(info.end_offset, 28);
    }

    #[test]
    fn self_closing_with_attrs() {
        let info = parse_jsx_tag(b"<Img src=\"x.png\" />").unwrap();
        assert_eq!(info.name, "Img");
        assert!(info.is_self_closing);
        assert_eq!(info.end_offset, 19);
    }

    #[test]
    fn invalid_less_than_number() {
        assert!(parse_jsx_tag(b"< 5").is_none());
    }

    #[test]
    fn invalid_bare_less_than() {
        assert!(parse_jsx_tag(b"<").is_none());
    }

    #[test]
    fn invalid_number_start() {
        assert!(parse_jsx_tag(b"<123>").is_none());
    }

    #[test]
    fn lowercase_html_tag() {
        let info = parse_jsx_tag(b"<div>").unwrap();
        assert_eq!(info.name, "div");
        assert_eq!(info.end_offset, 5);
    }

    #[test]
    fn tag_with_expression_containing_gt() {
        // Expression attribute with `>` inside should not terminate the tag
        let info = parse_jsx_tag(b"<Foo bar={a > b}>").unwrap();
        assert_eq!(info.name, "Foo");
        assert_eq!(info.end_offset, 17);
    }

    #[test]
    fn unterminated_tag() {
        assert!(parse_jsx_tag(b"<Foo bar=\"baz").is_none());
    }

    #[test]
    fn deep_member_expression() {
        let info = parse_jsx_tag(b"<A.B.C.D>").unwrap();
        assert_eq!(info.name, "A.B.C.D");
    }
}
