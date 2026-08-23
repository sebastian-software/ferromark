use rustc_hash::FxHashMap;

#[cfg(test)]
use std::cell::Cell;

const UNTERMINATED_EXPRESSION: usize = usize::MAX;

#[cfg(test)]
thread_local! {
    static CACHE_BUILD_COUNT: Cell<usize> = const { Cell::new(0) };
}

#[cfg(test)]
pub(crate) fn reset_cache_build_count() {
    CACHE_BUILD_COUNT.with(|count| count.set(0));
}

#[cfg(test)]
pub(crate) fn cache_build_count() -> usize {
    CACHE_BUILD_COUNT.with(Cell::get)
}

/// Cached expression ends for a single input buffer.
///
/// Entries are built from right to left. When a scan reaches a later opening
/// brace in normal expression syntax, its cached result is therefore already
/// available: a completed nested expression can be skipped as one unit, while
/// an unterminated nested expression proves that the current one cannot close.
/// Sparse documents store only opening braces; dense documents use a compact
/// direct index once that is smaller than a hash table.
pub(crate) struct ExpressionEnds {
    ends: ExpressionEndStorage,
    #[cfg(test)]
    scanned_bytes: usize,
    #[cfg(test)]
    lexical_failure_count: usize,
}

/// State needed to avoid rescanning the unterminated suffix of consecutive
/// comments. This is deliberately constant-size: expression-end reuse already
/// handles nested braces, and indexing every lexical delimiter would make
/// ordinary Markdown containing many quotes consume memory proportional to
/// every byte rather than MDX expression starts.
#[derive(Default)]
struct LexicalFailures {
    /// Earliest `/*` known to have no closing delimiter after its own opener.
    /// Expression ends are constructed right-to-left, so each earlier probe
    /// need only scan the previously unseen interval before this frontier.
    unterminated_block_comment_start: std::cell::Cell<Option<usize>>,
    /// Equivalent frontier for line comments which run through EOF.
    unterminated_line_comment_start: std::cell::Cell<Option<usize>>,
    /// Earliest unterminated lexical opener of each quoted form.  A later
    /// delimiter may still close an earlier string, so probes scan through
    /// the frontier once before proving the preceding suffix unterminated.
    unterminated_double_quote_start: std::cell::Cell<Option<usize>>,
    unterminated_single_quote_start: std::cell::Cell<Option<usize>>,
    unterminated_template_start: std::cell::Cell<Option<usize>>,
}

impl LexicalFailures {
    fn block_comment_end(&self, bytes: &[u8], start: usize, work: &mut ScanWork) -> Option<usize> {
        let mut pos = start + 2;
        let search_end = self
            .unterminated_block_comment_start
            .get()
            // A preceding comment can close on the `*` that overlaps this
            // later opener (`/*/`), so include one byte past the frontier.
            .map_or(bytes.len(), |frontier| {
                frontier.saturating_add(2).min(bytes.len())
            });

        while pos < search_end && pos + 1 < bytes.len() {
            work.visit();
            if bytes[pos] == b'*' && bytes[pos + 1] == b'/' {
                return Some(pos + 2);
            }
            pos += 1;
        }

        self.unterminated_block_comment_start.set(Some(
            self.unterminated_block_comment_start
                .get()
                .map_or(start, |old| old.min(start)),
        ));
        None
    }

    fn line_comment_end(&self, bytes: &[u8], start: usize, work: &mut ScanWork) -> usize {
        let mut pos = start + 2;
        let search_end = self
            .unterminated_line_comment_start
            .get()
            .unwrap_or(bytes.len());

        while pos < search_end {
            work.visit();
            if bytes[pos] == b'\n' {
                return pos + 1;
            }
            pos += 1;
        }

        self.unterminated_line_comment_start.set(Some(
            self.unterminated_line_comment_start
                .get()
                .map_or(start, |old| old.min(start)),
        ));
        bytes.len()
    }

    fn quote_search_end(&self, delimiter: u8, len: usize) -> usize {
        let frontier = match delimiter {
            b'"' => self.unterminated_double_quote_start.get(),
            b'\'' => self.unterminated_single_quote_start.get(),
            b'`' => self.unterminated_template_start.get(),
            _ => unreachable!("only quoted delimiters use a lexical frontier"),
        };
        // Include the frontier delimiter: it can close a preceding string.
        frontier.map_or(len, |start| start.saturating_add(1).min(len))
    }

    fn mark_unterminated_quote(&self, delimiter: u8, start: usize) {
        let frontier = match delimiter {
            b'"' => &self.unterminated_double_quote_start,
            b'\'' => &self.unterminated_single_quote_start,
            b'`' => &self.unterminated_template_start,
            _ => unreachable!("only quoted delimiters use a lexical frontier"),
        };
        frontier.set(Some(frontier.get().map_or(start, |old| old.min(start))));
    }

    #[cfg(test)]
    fn entry_count(&self) -> usize {
        usize::from(self.unterminated_block_comment_start.get().is_some())
            + usize::from(self.unterminated_line_comment_start.get().is_some())
            + usize::from(self.unterminated_double_quote_start.get().is_some())
            + usize::from(self.unterminated_single_quote_start.get().is_some())
            + usize::from(self.unterminated_template_start.get().is_some())
    }
}

#[derive(Clone, Copy)]
struct CachedEnds<'a> {
    expressions: &'a ExpressionEndStorage,
    lexical_failures: &'a LexicalFailures,
}

enum ExpressionEndStorage {
    Sparse(FxHashMap<usize, usize>),
    Dense(Vec<usize>),
}

impl ExpressionEndStorage {
    fn new(input_len: usize, brace_count: usize) -> Self {
        // A direct index is more compact than a hash table once openings are
        // dense. Otherwise, retain only the positions that can be queried.
        if brace_count > input_len / 4 {
            Self::Dense(vec![UNTERMINATED_EXPRESSION; input_len])
        } else {
            Self::Sparse(FxHashMap::default())
        }
    }

    fn insert(&mut self, start: usize, end: usize) {
        match self {
            Self::Sparse(ends) => {
                ends.insert(start, end);
            }
            Self::Dense(ends) => ends[start] = end,
        }
    }

    fn get(&self, start: usize) -> usize {
        match self {
            Self::Sparse(ends) => ends[&start],
            Self::Dense(ends) => ends[start],
        }
    }

    #[cfg(test)]
    fn entry_count(&self) -> usize {
        match self {
            Self::Sparse(ends) => ends.len(),
            Self::Dense(ends) => ends.len(),
        }
    }
}

impl ExpressionEnds {
    pub(crate) fn new(bytes: &[u8]) -> Self {
        #[cfg(test)]
        CACHE_BUILD_COUNT.with(|count| count.set(count.get() + 1));

        let brace_count = bytes.iter().filter(|&&byte| byte == b'{').count();
        let mut ends = ExpressionEndStorage::new(bytes.len(), brace_count);
        let lexical_failures = LexicalFailures::default();
        let mut work = ScanWork::default();

        for start in (0..bytes.len()).rev() {
            if bytes[start] == b'{' {
                let end = find_expression_end_from(
                    bytes,
                    start,
                    Some(CachedEnds {
                        expressions: &ends,
                        lexical_failures: &lexical_failures,
                    }),
                    &mut work,
                )
                .unwrap_or(UNTERMINATED_EXPRESSION);
                ends.insert(start, end);
            }
        }

        Self {
            ends,
            #[cfg(test)]
            scanned_bytes: bytes.len() + work.scanned_bytes,
            #[cfg(test)]
            lexical_failure_count: lexical_failures.entry_count(),
        }
    }

    /// Return the absolute offset after the matching `}` for `start`.
    pub(crate) fn end_at(&self, start: usize) -> Option<usize> {
        let end = self.ends.get(start);
        (end != UNTERMINATED_EXPRESSION).then_some(end)
    }

    #[cfg(test)]
    fn scanned_bytes(&self) -> usize {
        self.scanned_bytes
    }

    #[cfg(test)]
    fn entry_count(&self) -> usize {
        self.ends.entry_count()
    }

    #[cfg(test)]
    fn lexical_entry_count(&self) -> usize {
        // A lexical failure frontier is deliberately the only lexical state;
        // it never grows with input delimiter density.
        self.lexical_failure_count
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
    cached_ends: Option<CachedEnds<'_>>,
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
                    let end = cached_ends.expressions.get(pos);
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
                pos = skip_double_quoted(bytes, pos, cached_ends, work)?;
            }
            b'\'' => {
                pos = skip_single_quoted(bytes, pos, cached_ends, work)?;
            }
            b'`' => {
                pos = skip_template_literal(bytes, pos, cached_ends, work)?;
            }
            b'/' if pos + 1 < len => match bytes[pos + 1] {
                b'/' => {
                    pos = skip_line_comment(bytes, pos, cached_ends, work);
                }
                b'*' => {
                    pos = skip_block_comment(bytes, pos, cached_ends, work)?;
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
fn skip_double_quoted(
    bytes: &[u8],
    start: usize,
    cached_ends: Option<CachedEnds<'_>>,
    work: &mut ScanWork,
) -> Option<usize> {
    let search_end = cached_ends.map_or(bytes.len(), |cached| {
        cached.lexical_failures.quote_search_end(b'"', bytes.len())
    });
    let mut pos = start + 1;
    while pos < search_end {
        work.visit();
        match bytes[pos] {
            b'\\' => pos += 2, // skip escaped char
            b'"' => return Some(pos + 1),
            _ => pos += 1,
        }
    }
    if let Some(cached_ends) = cached_ends {
        cached_ends
            .lexical_failures
            .mark_unterminated_quote(b'"', start);
    }
    None
}

/// Skip a `'...'` string. `pos` points at the opening `'`.
/// Returns position after the closing `'`, or `None` if unterminated.
fn skip_single_quoted(
    bytes: &[u8],
    start: usize,
    cached_ends: Option<CachedEnds<'_>>,
    work: &mut ScanWork,
) -> Option<usize> {
    let search_end = cached_ends.map_or(bytes.len(), |cached| {
        cached.lexical_failures.quote_search_end(b'\'', bytes.len())
    });
    let mut pos = start + 1;
    while pos < search_end {
        work.visit();
        match bytes[pos] {
            b'\\' => pos += 2,
            b'\'' => return Some(pos + 1),
            _ => pos += 1,
        }
    }
    if let Some(cached_ends) = cached_ends {
        cached_ends
            .lexical_failures
            .mark_unterminated_quote(b'\'', start);
    }
    None
}

/// Skip a `` `...` `` template literal, including nested `${...}`.
/// `pos` points at the opening `` ` ``.
/// Returns position after the closing `` ` ``, or `None` if unterminated.
fn skip_template_literal(
    bytes: &[u8],
    start: usize,
    cached_ends: Option<CachedEnds<'_>>,
    work: &mut ScanWork,
) -> Option<usize> {
    let search_end = cached_ends.map_or(bytes.len(), |cached| {
        cached.lexical_failures.quote_search_end(b'`', bytes.len())
    });
    let mut pos = start + 1;
    while pos < search_end {
        work.visit();
        match bytes[pos] {
            b'\\' => pos += 2,
            b'`' => return Some(pos + 1),
            b'$' if pos + 1 < search_end && bytes[pos + 1] == b'{' => {
                // Nested expression inside template literal
                let expression_start = pos + 1;
                if let Some(cached_ends) = cached_ends {
                    let end = cached_ends.expressions.get(expression_start);
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
    if let Some(cached_ends) = cached_ends {
        cached_ends
            .lexical_failures
            .mark_unterminated_quote(b'`', start);
    }
    None
}

/// Skip a `// ...` line comment. Returns position after the newline (or EOF).
fn skip_line_comment(
    bytes: &[u8],
    start: usize,
    cached_ends: Option<CachedEnds<'_>>,
    work: &mut ScanWork,
) -> usize {
    if let Some(cached_ends) = cached_ends {
        return cached_ends
            .lexical_failures
            .line_comment_end(bytes, start, work);
    }
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
fn skip_block_comment(
    bytes: &[u8],
    start: usize,
    cached_ends: Option<CachedEnds<'_>>,
    work: &mut ScanWork,
) -> Option<usize> {
    if let Some(cached_ends) = cached_ends {
        return cached_ends
            .lexical_failures
            .block_comment_end(bytes, start, work);
    }
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
            b"{/* unterminated\n{later}".as_slice(),
            b"{/*\n{/*/}".as_slice(),
            b"{// unterminated {later}".as_slice(),
            b"{'unterminated\n{later}".as_slice(),
            b"{`unterminated\n{later}".as_slice(),
            b"{\\\"\n{\\\"\n{later}".as_slice(),
            b"{\\'\n{\\'\n{later}".as_slice(),
            b"{\\`\n{\\`\n{later}".as_slice(),
            b"{/* comment */}\n{later}".as_slice(),
            b"{// comment\n}\n{later}".as_slice(),
            b"{`plain`}\n{later}".as_slice(),
            b"{`escaped \\${literal}`}\n{later}".as_slice(),
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

    #[test]
    fn cached_ends_bound_unterminated_lexical_scan_work() {
        let inputs = [
            "{/*\n".repeat(8_192),
            "{//".repeat(8_192),
            "{\"".repeat(8_192),
            "{'".repeat(8_192),
            "{`".repeat(8_192),
            "{/unterminated ".repeat(8_192),
        ];

        for (case, input) in inputs.into_iter().enumerate() {
            let ends = ExpressionEnds::new(input.as_bytes());
            assert!(
                input
                    .bytes()
                    .enumerate()
                    .filter(|(_, byte)| *byte == b'{')
                    .all(|(start, _)| ends.end_at(start).is_none()),
                "every opening brace is unterminated for case {case}: {input:?}",
            );
            assert!(
                ends.scanned_bytes() <= input.len() * 3,
                "case {case}: cached lexical scans visited {} bytes for a {} byte input",
                ends.scanned_bytes(),
                input.len(),
            );
        }
    }

    #[test]
    fn cached_ends_bound_escaped_quote_and_template_scan_work() {
        let inputs = [
            "{\\\"".repeat(8_192),
            "{\\'".repeat(8_192),
            "{\\`".repeat(8_192),
        ];

        for (case, input) in inputs.into_iter().enumerate() {
            let ends = ExpressionEnds::new(input.as_bytes());
            assert!(
                input
                    .bytes()
                    .enumerate()
                    .filter(|(_, byte)| *byte == b'{')
                    .all(|(start, _)| ends.end_at(start).is_none()),
                "every escaped lexical opener is unterminated for case {case}",
            );
            assert!(
                ends.scanned_bytes() <= input.len() * 3,
                "case {case}: cached escaped lexical scans visited {} bytes for a {} byte input",
                ends.scanned_bytes(),
                input.len(),
            );
            assert_eq!(ends.lexical_entry_count(), 1);
        }
    }

    #[test]
    fn cached_ends_bound_escaped_jsx_attribute_quote_scan_work() {
        // Roughly 640 KiB, matching the formerly quadratic JSX-attribute
        // shape without relying on wall-clock timing.
        let input = "<Component value={\\\"".repeat(32_768);
        let ends = ExpressionEnds::new(input.as_bytes());

        assert!(
            input
                .bytes()
                .enumerate()
                .filter(|(_, byte)| *byte == b'{')
                .all(|(start, _)| ends.end_at(start).is_none()),
        );
        assert!(
            ends.scanned_bytes() <= input.len() * 3,
            "cached escaped JSX scans visited {} bytes for a {} byte input",
            ends.scanned_bytes(),
            input.len(),
        );
        assert_eq!(ends.lexical_entry_count(), 1);
    }

    #[test]
    fn lexical_failures_use_constant_state() {
        for input in ["{/*\n".repeat(8_192), "{//".repeat(8_192)] {
            let ends = ExpressionEnds::new(input.as_bytes());
            assert_eq!(
                ends.lexical_entry_count(),
                1,
                "only the unterminated lexical frontier is retained",
            );
        }
    }

    #[test]
    fn sparse_lexical_delimiters_are_not_indexed() {
        // This is deliberately 8 MiB of ordinary Markdown punctuation. The
        // expression cache must not allocate one entry per quote/comment
        // delimiter merely because a document may contain an expression.
        let mut input = "\"'`///*\n".repeat(1_000_000);
        input.push_str("{value}");
        let ends = ExpressionEnds::new(input.as_bytes());

        assert_eq!(ends.entry_count(), 1);
        assert_eq!(ends.lexical_entry_count(), 0);
    }

    #[test]
    fn cached_ends_store_only_sparse_opening_braces() {
        let mut input = "plain Markdown without expressions\n".repeat(32_768);
        input.push_str("{value}\n");
        let ends = ExpressionEnds::new(input.as_bytes());

        assert_eq!(ends.entry_count(), 1);
        assert_eq!(
            ends.end_at(input.len() - "{value}\n".len()),
            Some(input.len() - 1)
        );
    }
}
