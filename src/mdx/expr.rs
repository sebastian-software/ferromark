/// Find the end of a JSX expression starting at `{`.
///
/// `bytes` must begin with `{`. Returns the byte offset **after** the closing `}`,
/// or `None` if the expression is unterminated.
///
/// Tracks:
/// - Brace depth (`{` / `}`)
/// - Double-quoted strings (`"..."` with `\"` escapes)
/// - Single-quoted strings (`'...'` with `\'` escapes)
/// - Template literals (`` `...` `` with `${...}` nesting)
/// - Line comments (`// ...`)
/// - Block comments (`/* ... */`)
pub fn find_expression_end(bytes: &[u8]) -> Option<usize> {
    debug_assert!(bytes.first() == Some(&b'{'));
    let len = bytes.len();
    let mut pos = 1; // skip opening `{`
    let mut depth: u32 = 1;

    while pos < len {
        match bytes[pos] {
            b'{' => {
                depth += 1;
                pos += 1;
            }
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(pos + 1);
                }
                pos += 1;
            }
            b'"' => {
                pos = skip_double_quoted(bytes, pos)?;
            }
            b'\'' => {
                pos = skip_single_quoted(bytes, pos)?;
            }
            b'`' => {
                pos = skip_template_literal(bytes, pos)?;
            }
            b'/' if pos + 1 < len => match bytes[pos + 1] {
                b'/' => {
                    pos = skip_line_comment(bytes, pos);
                }
                b'*' => {
                    pos = skip_block_comment(bytes, pos)?;
                }
                _ => pos += 1,
            },
            _ => pos += 1,
        }
    }

    None // unterminated
}

/// Skip a `"..."` string. `pos` points at the opening `"`.
/// Returns position after the closing `"`, or `None` if unterminated.
fn skip_double_quoted(bytes: &[u8], start: usize) -> Option<usize> {
    let len = bytes.len();
    let mut pos = start + 1;
    while pos < len {
        match bytes[pos] {
            b'\\' => pos += 2, // skip escaped char
            b'"' => return Some(pos + 1),
            _ => pos += 1,
        }
    }
    None
}

/// Skip a `'...'` string. `pos` points at the opening `'`.
/// Returns position after the closing `'`, or `None` if unterminated.
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

/// Skip a `` `...` `` template literal, including nested `${...}`.
/// `pos` points at the opening `` ` ``.
/// Returns position after the closing `` ` ``, or `None` if unterminated.
fn skip_template_literal(bytes: &[u8], start: usize) -> Option<usize> {
    let len = bytes.len();
    let mut pos = start + 1;
    while pos < len {
        match bytes[pos] {
            b'\\' => pos += 2,
            b'`' => return Some(pos + 1),
            b'$' if pos + 1 < len && bytes[pos + 1] == b'{' => {
                // Nested expression inside template literal
                let end = find_expression_end(&bytes[pos + 1..])?;
                pos = pos + 1 + end;
            }
            _ => pos += 1,
        }
    }
    None
}

/// Skip a `// ...` line comment. Returns position after the newline (or EOF).
fn skip_line_comment(bytes: &[u8], start: usize) -> usize {
    let len = bytes.len();
    let mut pos = start + 2;
    while pos < len {
        if bytes[pos] == b'\n' {
            return pos + 1;
        }
        pos += 1;
    }
    len
}

/// Skip a `/* ... */` block comment. Returns position after `*/`, or `None` if unterminated.
fn skip_block_comment(bytes: &[u8], start: usize) -> Option<usize> {
    let len = bytes.len();
    let mut pos = start + 2;
    while pos + 1 < len {
        if bytes[pos] == b'*' && bytes[pos + 1] == b'/' {
            return Some(pos + 2);
        }
        pos += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_expression() {
        assert_eq!(find_expression_end(b"{x}"), Some(3));
    }

    #[test]
    fn nested_braces() {
        assert_eq!(find_expression_end(b"{a{b}c}"), Some(7));
    }

    #[test]
    fn double_quoted_string() {
        assert_eq!(find_expression_end(b"{\"}\"}"), Some(5));
    }

    #[test]
    fn single_quoted_string() {
        assert_eq!(find_expression_end(b"{'}'} rest"), Some(5));
    }

    #[test]
    fn template_literal() {
        assert_eq!(find_expression_end(b"{`}`}"), Some(5));
    }

    #[test]
    fn template_literal_with_nested_expr() {
        assert_eq!(find_expression_end(b"{`${a}`}"), Some(8));
    }

    #[test]
    fn line_comment() {
        assert_eq!(find_expression_end(b"{// }\n}"), Some(7));
    }

    #[test]
    fn block_comment() {
        assert_eq!(find_expression_end(b"{/* } */}"), Some(9));
    }

    #[test]
    fn unterminated() {
        assert_eq!(find_expression_end(b"{abc"), None);
    }

    #[test]
    fn unterminated_string() {
        assert_eq!(find_expression_end(b"{\"abc}"), None);
    }

    #[test]
    fn escaped_quote_in_string() {
        assert_eq!(find_expression_end(b"{\"a\\\"b\"}"), Some(8));
    }

    #[test]
    fn empty_expression() {
        assert_eq!(find_expression_end(b"{}"), Some(2));
    }

    #[test]
    fn complex_nested() {
        let input = b"{fn() { return { x: `${y}` }; }}";
        assert_eq!(find_expression_end(input), Some(input.len()));
    }

    #[test]
    fn slash_not_comment() {
        // A lone `/` inside an expression is not a comment start
        assert_eq!(find_expression_end(b"{a / b}"), Some(7));
    }
}
