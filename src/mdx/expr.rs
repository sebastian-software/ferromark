const UNTERMINATED_EXPRESSION: usize = usize::MAX;

/// Cached expression ends for a single input buffer.
///
/// Entries are built from right to left. When a scan reaches a later opening
/// brace in normal expression syntax, its cached result is therefore already
/// available: a completed nested expression can be skipped as one unit, while
/// an unterminated nested expression proves that the current one cannot close.
pub(crate) struct ExpressionEnds {
    ends: Vec<usize>,
    #[cfg(test)]
    scanned_bytes: usize,
}

impl ExpressionEnds {
    pub(crate) fn new(bytes: &[u8]) -> Self {
        if !bytes.contains(&b'{') {
            return Self {
                ends: Vec::new(),
                #[cfg(test)]
                scanned_bytes: 0,
            };
        }

        let mut ends = vec![UNTERMINATED_EXPRESSION; bytes.len()];
        let mut work = ScanWork::default();

        for start in (0..bytes.len()).rev() {
            if bytes[start] == b'{' {
                ends[start] = find_expression_end_from(bytes, start, Some(&ends), &mut work)
                    .unwrap_or(UNTERMINATED_EXPRESSION);
            }
        }

        Self {
            ends,
            #[cfg(test)]
            scanned_bytes: bytes.len() + work.scanned_bytes,
        }
    }

    /// Return the absolute offset after the matching `}` for `start`.
    pub(crate) fn end_at(&self, start: usize) -> Option<usize> {
        self.ends
            .get(start)
            .copied()
            .filter(|end| *end != UNTERMINATED_EXPRESSION)
    }

    #[cfg(test)]
    fn scanned_bytes(&self) -> usize {
        self.scanned_bytes
    }
}

#[derive(Default)]
struct ScanWork {
    #[cfg(test)]
    scanned_bytes: usize,
}

impl ScanWork {
    #[inline]
    fn visit(&mut self) {
        #[cfg(test)]
        {
            self.scanned_bytes += 1;
        }
    }
}

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
    find_expression_end_from(bytes, 0, None, &mut ScanWork::default())
}

fn find_expression_end_from(
    bytes: &[u8],
    start: usize,
    cached_ends: Option<&[usize]>,
    work: &mut ScanWork,
) -> Option<usize> {
    debug_assert!(bytes.get(start) == Some(&b'{'));
    let len = bytes.len();
    let mut pos = start + 1; // skip opening `{`
    let mut depth: u32 = 1;

    while pos < len {
        work.visit();
        match bytes[pos] {
            b'{' => {
                if let Some(cached_ends) = cached_ends {
                    let end = cached_ends[pos];
                    if end == UNTERMINATED_EXPRESSION {
                        return None;
                    }
                    pos = end;
                } else {
                    depth += 1;
                    pos += 1;
                }
            }
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(pos + 1);
                }
                pos += 1;
            }
            b'"' => {
                pos = skip_double_quoted(bytes, pos, work)?;
            }
            b'\'' => {
                pos = skip_single_quoted(bytes, pos, work)?;
            }
            b'`' => {
                pos = skip_template_literal(bytes, pos, cached_ends, work)?;
            }
            b'/' if pos + 1 < len => match bytes[pos + 1] {
                b'/' => {
                    pos = skip_line_comment(bytes, pos, work);
                }
                b'*' => {
                    pos = skip_block_comment(bytes, pos, work)?;
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
fn skip_double_quoted(bytes: &[u8], start: usize, work: &mut ScanWork) -> Option<usize> {
    let len = bytes.len();
    let mut pos = start + 1;
    while pos < len {
        work.visit();
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
fn skip_single_quoted(bytes: &[u8], start: usize, work: &mut ScanWork) -> Option<usize> {
    let len = bytes.len();
    let mut pos = start + 1;
    while pos < len {
        work.visit();
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
fn skip_template_literal(
    bytes: &[u8],
    start: usize,
    cached_ends: Option<&[usize]>,
    work: &mut ScanWork,
) -> Option<usize> {
    let len = bytes.len();
    let mut pos = start + 1;
    while pos < len {
        work.visit();
        match bytes[pos] {
            b'\\' => pos += 2,
            b'`' => return Some(pos + 1),
            b'$' if pos + 1 < len && bytes[pos + 1] == b'{' => {
                // Nested expression inside template literal
                let expression_start = pos + 1;
                if let Some(cached_ends) = cached_ends {
                    let end = cached_ends[expression_start];
                    if end == UNTERMINATED_EXPRESSION {
                        return None;
                    }
                    pos = end;
                } else {
                    pos = find_expression_end_from(bytes, expression_start, None, work)?;
                }
            }
            _ => pos += 1,
        }
    }
    None
}

/// Skip a `// ...` line comment. Returns position after the newline (or EOF).
fn skip_line_comment(bytes: &[u8], start: usize, work: &mut ScanWork) -> usize {
    let len = bytes.len();
    let mut pos = start + 2;
    while pos < len {
        work.visit();
        if bytes[pos] == b'\n' {
            return pos + 1;
        }
        pos += 1;
    }
    len
}

/// Skip a `/* ... */` block comment. Returns position after `*/`, or `None` if unterminated.
fn skip_block_comment(bytes: &[u8], start: usize, work: &mut ScanWork) -> Option<usize> {
    let len = bytes.len();
    let mut pos = start + 2;
    while pos + 1 < len {
        work.visit();
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

    #[test]
    fn cached_ends_match_individual_expression_scans() {
        for input in [
            b"{outer {inner}}".as_slice(),
            b"{\"{inside string}\"}\n{later}".as_slice(),
            b"{`template ${value}`}\n{later}".as_slice(),
            b"{unterminated\n{later}".as_slice(),
            b"{\"unterminated\n{later}".as_slice(),
        ] {
            let ends = ExpressionEnds::new(input);
            for (start, byte) in input.iter().enumerate() {
                if *byte == b'{' {
                    assert_eq!(
                        ends.end_at(start).map(|end| end - start),
                        find_expression_end(&input[start..]),
                        "cache differs at byte {start} in {input:?}",
                    );
                }
            }
        }
    }

    #[test]
    fn cached_ends_bound_unterminated_expression_scan_work() {
        let input = "{\n".repeat(8_192);
        let ends = ExpressionEnds::new(input.as_bytes());

        assert!(
            (0..input.len())
                .step_by(2)
                .all(|start| ends.end_at(start).is_none()),
            "every opening brace is unterminated",
        );
        assert!(
            ends.scanned_bytes() <= input.len() * 3,
            "cached expression scans visited {} bytes for a {} byte input",
            ends.scanned_bytes(),
            input.len(),
        );
    }

    #[test]
    fn cached_ends_bound_unterminated_jsx_attribute_expression_scan_work() {
        let input = "<Component value={unterminated\n".repeat(8_192);
        let ends = ExpressionEnds::new(input.as_bytes());

        assert!(
            input
                .bytes()
                .enumerate()
                .filter(|(_, byte)| *byte == b'{')
                .all(|(start, _)| ends.end_at(start).is_none()),
            "every JSX attribute expression is unterminated",
        );
        assert!(
            ends.scanned_bytes() <= input.len() * 3,
            "cached JSX attribute scans visited {} bytes for a {} byte input",
            ends.scanned_bytes(),
            input.len(),
        );
    }
}
