use super::Segment;
use super::expr::find_expression_end;
use super::jsx_tag::parse_jsx_tag;
use crate::{BlockEvent, BlockParser, CodeBlockKind};

/// Split MDX input into typed segments.
///
/// The splitter is a line-based state machine that categorises each region of
/// the input as one of: ESM (`import`/`export`), JSX block tag, expression, or
/// Markdown.  Only block-level constructs are detected — inline JSX inside
/// paragraphs is left for the Markdown parser.
///
/// The returned `Vec<Segment>` covers the entire input (no bytes are dropped).
pub fn split(input: &str) -> Vec<Segment<'_>> {
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut segments: Vec<Segment<'_>> = Vec::new();
    let mut pos = 0;
    let mut md_start: Option<usize> = None;
    let mut tag_stack: Vec<String> = Vec::new();
    // Track whether the previous line was non-blank markdown content.
    // ESM cannot interrupt a paragraph (requires blank line before it).
    let mut in_paragraph = false;
    let mut open_fence: Option<CodeFence> = None;
    let container_fences = container_fence_lines(bytes);

    while pos < len {
        let line_start = pos;

        if container_fences.contains_opener(line_start) {
            extend_markdown(&mut md_start, line_start);
            in_paragraph = true;
            pos = next_line(bytes, pos);
            continue;
        }

        // The line-oriented delimiter state below handles root-level fences.
        // For fences in Markdown containers (such as list items), use the
        // block parser's source-ranged code events to keep their contents
        // opaque without duplicating CommonMark's container rules here.
        if container_fences.contains_content(line_start) {
            extend_markdown(&mut md_start, line_start);
            in_paragraph = !is_blank_line(bytes, line_start);
            pos = next_line(bytes, pos);
            continue;
        }

        // A container fence closer ends the surrounding Markdown block.  It
        // must not be reinterpreted as a root fence merely because its list
        // indentation also happens to be valid root-fence indentation.
        if container_fences.contains_closer(line_start) {
            extend_markdown(&mut md_start, line_start);
            in_paragraph = false;
            pos = next_line(bytes, pos);
            continue;
        }

        if let Some(fence) = open_fence {
            extend_markdown(&mut md_start, line_start);
            if fence.is_closing(bytes, line_start) {
                open_fence = None;
                in_paragraph = false;
            } else {
                in_paragraph = true;
            }
            pos = next_line(bytes, pos);
            continue;
        }

        if let Some(fence) = opening_code_fence(bytes, line_start) {
            open_fence = Some(fence);
            extend_markdown(&mut md_start, line_start);
            in_paragraph = true;
            pos = next_line(bytes, pos);
            continue;
        }

        let first_non_ws = skip_whitespace_offset(bytes, pos);

        if first_non_ws >= len {
            // Remaining is whitespace-only — treat as markdown
            extend_markdown(&mut md_start, line_start);
            break;
        }

        let first = bytes[first_non_ws];

        // Detect blank lines (only newline after whitespace) — reset paragraph state
        if first == b'\n' || first == b'\r' {
            in_paragraph = false;
            extend_markdown(&mut md_start, line_start);
            pos = next_line(bytes, pos);
            continue;
        }

        // 1. Closing tag: `</`
        if first == b'<'
            && first_non_ws + 1 < len
            && bytes[first_non_ws + 1] == b'/'
            && let Some(tag_info) = parse_jsx_tag(&bytes[first_non_ws..])
            && tag_info.is_closing
        {
            let end = first_non_ws + tag_info.end_offset;
            // Flow JSX requires no trailing non-whitespace content on the line
            if has_trailing_content(bytes, end) {
                // Fall through to markdown
            } else {
                flush_markdown(input, &mut md_start, line_start, &mut segments);
                let seg_end = consume_trailing_newline(bytes, end);
                segments.push(Segment::JsxBlockClose(&input[line_start..seg_end]));
                if !tag_info.name.is_empty()
                    && let Some(top_pos) = tag_stack.iter().rposition(|n| n == tag_info.name)
                {
                    tag_stack.remove(top_pos);
                }
                pos = seg_end;
                in_paragraph = false;
                continue;
            }
        }
        // Fall through to markdown

        // 2. ESM: `import ` or `export ` at column 0, not interrupting a paragraph
        if pos == first_non_ws
            && !in_paragraph
            && let Some(esm_end) = try_esm(bytes, pos, &container_fences)
        {
            flush_markdown(input, &mut md_start, line_start, &mut segments);
            segments.push(Segment::Esm(&input[pos..esm_end]));
            pos = esm_end;
            in_paragraph = false;
            continue;
        }

        // 3. Expression: `{` as first non-whitespace
        if first == b'{'
            && let Some(expr_len) = find_expression_end(&bytes[first_non_ws..])
        {
            let end = first_non_ws + expr_len;
            // Flow expression requires no trailing non-whitespace content
            if !has_trailing_content(bytes, end) {
                flush_markdown(input, &mut md_start, line_start, &mut segments);
                let seg_end = consume_trailing_newline(bytes, end);
                segments.push(Segment::Expression(&input[line_start..seg_end]));
                pos = seg_end;
                in_paragraph = false;
                continue;
            }
            // Trailing content → treat as markdown
        }
        // Unterminated expression → treat as markdown

        // 4. JSX opening/self-closing tag: `<` followed by letter or `>`
        if first == b'<'
            && first_non_ws + 1 < len
            && (bytes[first_non_ws + 1].is_ascii_alphabetic() || bytes[first_non_ws + 1] == b'>')
            && let Some(tag_info) = parse_jsx_tag(&bytes[first_non_ws..])
        {
            let end = first_non_ws + tag_info.end_offset;
            // Flow JSX requires no trailing non-whitespace content on the line
            if !has_trailing_content(bytes, end) {
                flush_markdown(input, &mut md_start, line_start, &mut segments);
                let seg_end = consume_trailing_newline(bytes, end);
                let slice = &input[line_start..seg_end];
                if tag_info.is_self_closing {
                    segments.push(Segment::JsxBlockSelfClose(slice));
                } else {
                    if !tag_info.name.is_empty() {
                        tag_stack.push(tag_info.name.to_string());
                    }
                    segments.push(Segment::JsxBlockOpen(slice));
                }
                pos = seg_end;
                in_paragraph = false;
                continue;
            }
            // Trailing content → treat as markdown
        }
        // Invalid JSX → fall through to markdown

        // 5. Otherwise → Markdown
        extend_markdown(&mut md_start, line_start);
        in_paragraph = true;
        pos = next_line(bytes, pos);
    }

    // Flush any remaining markdown
    if let Some(start) = md_start
        && start < len
    {
        segments.push(Segment::Markdown(&input[start..len]));
    }

    segments
}

/// A CommonMark fenced code block delimiter.
#[derive(Clone, Copy)]
pub(crate) struct CodeFence {
    marker: u8,
    length: usize,
}

impl CodeFence {
    pub(crate) fn is_closing(self, bytes: &[u8], line_start: usize) -> bool {
        let mut pos = skip_closing_fence_indentation(bytes, line_start);
        let mut length = 0;
        while bytes.get(pos) == Some(&self.marker) {
            length += 1;
            pos += 1;
        }

        length >= self.length
            && bytes[pos..]
                .iter()
                .take_while(|byte| **byte != b'\n')
                .all(|byte| matches!(*byte, b' ' | b'\t'))
    }
}

/// Recognize a CommonMark fenced-code opening line.
///
/// Fences may be indented by up to three spaces. Backtick fence info strings
/// cannot contain backticks, while tilde fence info strings have no equivalent
/// restriction.
pub(crate) fn opening_code_fence(bytes: &[u8], line_start: usize) -> Option<CodeFence> {
    let mut pos = skip_fence_indentation(bytes, line_start);
    let marker = *bytes.get(pos)?;
    if !matches!(marker, b'`' | b'~') {
        return None;
    }

    let fence_start = pos;
    while bytes.get(pos) == Some(&marker) {
        pos += 1;
    }
    let length = pos - fence_start;
    if length < 3 {
        return None;
    }

    let info = &bytes[pos..];
    if marker == b'`'
        && info
            .iter()
            .take_while(|byte| **byte != b'\n')
            .any(|byte| *byte == b'`')
    {
        return None;
    }

    Some(CodeFence { marker, length })
}

/// Skip at most three leading spaces, leaving a fourth space in place so the
/// caller will reject indented code blocks as fenced-code delimiters.
fn skip_fence_indentation(bytes: &[u8], mut pos: usize) -> usize {
    for _ in 0..3 {
        if bytes.get(pos) != Some(&b' ') {
            break;
        }
        pos += 1;
    }
    pos
}

/// Match the block parser's closing-fence indentation scan, including tabs
/// that advance beyond its three-column limit.
fn skip_closing_fence_indentation(bytes: &[u8], mut pos: usize) -> usize {
    let mut columns = 0;
    while columns < 3 {
        match bytes.get(pos) {
            Some(b' ') => {
                columns += 1;
                pos += 1;
            }
            Some(b'\t') => {
                columns = (columns + 4) & !3;
                pos += 1;
            }
            _ => break,
        }
    }
    pos
}

/// If we're already accumulating markdown, do nothing.
/// Otherwise, mark `pos` as the start of a new markdown region.
fn extend_markdown(md_start: &mut Option<usize>, pos: usize) {
    if md_start.is_none() {
        *md_start = Some(pos);
    }
}

/// Flush accumulated markdown into `segments` and reset the accumulator.
fn flush_markdown<'a>(
    input: &'a str,
    md_start: &mut Option<usize>,
    current_pos: usize,
    segments: &mut Vec<Segment<'a>>,
) {
    if let Some(start) = md_start.take()
        && start < current_pos
    {
        segments.push(Segment::Markdown(&input[start..current_pos]));
    }
}

/// Advance past the current line (past `\n` or to EOF).
fn next_line(bytes: &[u8], mut pos: usize) -> usize {
    let len = bytes.len();
    while pos < len && bytes[pos] != b'\n' {
        pos += 1;
    }
    if pos < len {
        pos + 1 // skip `\n`
    } else {
        len
    }
}

/// Whether a physical line has only horizontal whitespace before its ending.
pub(crate) fn is_blank_line(bytes: &[u8], mut pos: usize) -> bool {
    while matches!(bytes.get(pos), Some(b' ' | b'\t')) {
        pos += 1;
    }
    matches!(bytes.get(pos), None | Some(b'\n' | b'\r'))
}

/// Return the offset of the first non-whitespace byte at or after `pos`.
fn skip_whitespace_offset(bytes: &[u8], mut pos: usize) -> usize {
    let len = bytes.len();
    while pos < len && (bytes[pos] == b' ' || bytes[pos] == b'\t') {
        pos += 1;
    }
    pos
}

/// Check if there is non-whitespace content between `pos` and the end of the
/// current line. Used to distinguish flow (block) constructs from inline ones:
/// `<x />` is flow, but `<x />.` has trailing content and is text/inline.
fn has_trailing_content(bytes: &[u8], mut pos: usize) -> bool {
    let len = bytes.len();
    while pos < len && (bytes[pos] == b' ' || bytes[pos] == b'\t') {
        pos += 1;
    }
    pos < len && bytes[pos] != b'\n' && bytes[pos] != b'\r'
}

/// Consume optional trailing whitespace + a single newline after a construct.
fn consume_trailing_newline(bytes: &[u8], mut pos: usize) -> usize {
    let len = bytes.len();
    // Skip trailing spaces/tabs
    while pos < len && (bytes[pos] == b' ' || bytes[pos] == b'\t') {
        pos += 1;
    }
    // Consume newline
    if pos < len && bytes[pos] == b'\r' {
        pos += 1;
    }
    if pos < len && bytes[pos] == b'\n' {
        pos += 1;
    }
    pos
}

/// Result of scanning a potential ESM declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EsmScan {
    /// A complete declaration ending at this byte offset.
    Complete(usize),
    /// A declaration that needs more JavaScript but encountered a Markdown boundary.
    Incomplete(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Delimiter {
    Parenthesis,
    StatementParenthesis,
    Bracket,
    Brace,
    StatementBlock,
    TemplateExpression,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LexicalMode {
    Code,
    SingleQuote,
    DoubleQuote,
    Template,
    Regex,
    RegexFlags,
    LineComment,
    BlockComment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LineEnd {
    None,
    /// A completed regular-expression literal. This is distinct from `/`,
    /// which is a dangling division operator when it appears in code.
    Regex,
    Word {
        requires_following: bool,
        starts_control_condition: bool,
        allows_function_or_class: bool,
    },
    Punctuation(u8),
    StatementBoundary,
}

/// Whether a recognized statement introducer still expects a block body, or
/// may instead be followed by one braceless statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingStatementBlock {
    Required,
    Optional,
}

/// Lightweight JavaScript state needed to distinguish a continued declaration
/// from a following Markdown block. This deliberately recognises only lexical
/// structure and module-clause boundaries; it is not a JavaScript parser.
struct EsmContinuation {
    declaration_is_import: bool,
    declaration_is_export: bool,
    declaration_seen: bool,
    import_has_clause: bool,
    import_saw_from: bool,
    import_has_source: bool,
    export_from_candidate: bool,
    delimiters: Vec<Delimiter>,
    mode: LexicalMode,
    regex_char_class: bool,
    line_end: LineEnd,
    statement_block_pending: Option<PendingStatementBlock>,
    control_condition_pending: bool,
    malformed: bool,
}

impl EsmContinuation {
    fn new(bytes: &[u8], pos: usize) -> Self {
        Self {
            declaration_is_import: bytes[pos..].starts_with(b"import"),
            declaration_is_export: bytes[pos..].starts_with(b"export"),
            declaration_seen: false,
            import_has_clause: false,
            import_saw_from: false,
            import_has_source: false,
            export_from_candidate: false,
            delimiters: Vec::new(),
            mode: LexicalMode::Code,
            regex_char_class: false,
            line_end: LineEnd::None,
            statement_block_pending: None,
            control_condition_pending: false,
            malformed: false,
        }
    }

    fn scan_line(&mut self, bytes: &[u8], mut pos: usize, end: usize) {
        self.line_end = LineEnd::None;
        let mut word_start = None;

        while pos < end {
            let byte = bytes[pos];
            match self.mode {
                LexicalMode::Code => {
                    self.consume_optional_statement_body(bytes, pos);
                    if is_esm_word_byte(byte) {
                        word_start.get_or_insert(pos);
                        pos += 1;
                        continue;
                    }
                    self.finish_word(bytes, &mut word_start, pos);

                    match byte {
                        b'\'' => {
                            self.mark_string_start();
                            self.mode = LexicalMode::SingleQuote;
                        }
                        b'"' => {
                            self.mark_string_start();
                            self.mode = LexicalMode::DoubleQuote;
                        }
                        b'`' => {
                            self.line_end = LineEnd::Punctuation(byte);
                            self.mode = LexicalMode::Template;
                        }
                        b'/' if bytes.get(pos + 1) == Some(&b'/') => {
                            self.mode = LexicalMode::LineComment;
                            pos += 1;
                        }
                        b'/' if bytes.get(pos + 1) == Some(&b'*') => {
                            self.mode = LexicalMode::BlockComment;
                            pos += 1;
                        }
                        b'/' if self.can_start_regex() => {
                            self.mode = LexicalMode::Regex;
                            self.regex_char_class = false;
                        }
                        b'(' => self.open_parenthesis(byte),
                        b'[' => self.open(Delimiter::Bracket, byte),
                        b'{' => self.open_brace(byte),
                        b')' => self.close_parenthesis(byte),
                        b']' => self.close(Delimiter::Bracket, byte),
                        b'}' => self.close_brace(byte),
                        b' ' | b'\t' | b'\r' | b'\n' => {}
                        _ => {
                            if self.declaration_is_import
                                && self.declaration_seen
                                && self.delimiters.is_empty()
                                && matches!(byte, b'{' | b'*')
                            {
                                self.import_has_clause = true;
                            }
                            if self.declaration_is_export
                                && self.declaration_seen
                                && self.delimiters.is_empty()
                                && matches!(byte, b'{' | b'*')
                            {
                                self.export_from_candidate = true;
                            }
                            self.line_end = LineEnd::Punctuation(byte);
                        }
                    }
                }
                LexicalMode::SingleQuote => match byte {
                    b'\\' if pos + 1 < end => pos += 1,
                    b'\'' => {
                        self.mode = LexicalMode::Code;
                        self.line_end = LineEnd::Punctuation(byte);
                    }
                    _ => {}
                },
                LexicalMode::DoubleQuote => match byte {
                    b'\\' if pos + 1 < end => pos += 1,
                    b'"' => {
                        self.mode = LexicalMode::Code;
                        self.line_end = LineEnd::Punctuation(byte);
                    }
                    _ => {}
                },
                LexicalMode::Template => match byte {
                    b'\\' if pos + 1 < end => pos += 1,
                    b'`' => {
                        self.mode = LexicalMode::Code;
                        self.line_end = LineEnd::Punctuation(byte);
                    }
                    b'$' if bytes.get(pos + 1) == Some(&b'{') => {
                        self.delimiters.push(Delimiter::TemplateExpression);
                        self.mode = LexicalMode::Code;
                        self.line_end = LineEnd::Punctuation(b'{');
                        pos += 1;
                    }
                    _ => {}
                },
                LexicalMode::Regex => match byte {
                    b'\\' if pos + 1 < end => pos += 1,
                    b'[' => self.regex_char_class = true,
                    b']' => self.regex_char_class = false,
                    b'/' if !self.regex_char_class => {
                        self.mode = LexicalMode::RegexFlags;
                        self.line_end = LineEnd::Regex;
                    }
                    _ => {}
                },
                LexicalMode::RegexFlags => {
                    if !byte.is_ascii_alphabetic() {
                        self.mode = LexicalMode::Code;
                        continue;
                    }
                }
                LexicalMode::LineComment => {
                    if byte == b'\n' {
                        self.mode = LexicalMode::Code;
                    }
                }
                LexicalMode::BlockComment => {
                    if byte == b'*' && bytes.get(pos + 1) == Some(&b'/') {
                        self.mode = LexicalMode::Code;
                        pos += 1;
                    }
                }
            }
            pos += 1;
        }

        if self.mode == LexicalMode::RegexFlags {
            self.mode = LexicalMode::Code;
        }
        self.finish_word(bytes, &mut word_start, end);
    }

    fn finish_word(&mut self, bytes: &[u8], word_start: &mut Option<usize>, end: usize) {
        let Some(start) = word_start.take() else {
            return;
        };
        let word = &bytes[start..end];
        let is_from = word == b"from";
        let at_top_level = self.delimiters.is_empty();
        let starts_else_statement = self.is_statement_position() && word == b"else";
        let starts_do_statement = self.is_statement_position() && word == b"do";

        let starts_control_condition = self.is_statement_position()
            && matches!(
                word,
                b"if" | b"while" | b"for" | b"with" | b"switch" | b"catch"
            );
        if starts_control_condition {
            self.control_condition_pending = true;
        }

        if starts_else_statement || starts_do_statement {
            self.statement_block_pending = Some(PendingStatementBlock::Optional);
        } else if (self.is_statement_position() && matches!(word, b"try" | b"finally"))
            || (matches!(word, b"function" | b"class") && self.can_start_function_or_class())
        {
            self.statement_block_pending = Some(PendingStatementBlock::Required);
        }

        if !self.declaration_seen {
            self.declaration_seen = true;
        } else if self.declaration_is_import && at_top_level {
            if is_from {
                self.import_saw_from = true;
            } else {
                self.import_has_clause = true;
            }
        } else if self.declaration_is_export && at_top_level && is_from {
            self.export_from_candidate = false;
        }

        self.line_end = if starts_else_statement || starts_do_statement {
            // Both `else` and `do` are followed by a statement, which may be
            // braceless. Treat that following source position exactly like a
            // completed control condition while still retaining the pending
            // block marker for a `{` body.
            LineEnd::StatementBoundary
        } else {
            LineEnd::Word {
                requires_following: word_requires_following(word),
                starts_control_condition,
                allows_function_or_class: matches!(word, b"export" | b"default" | b"async"),
            }
        };
    }

    fn mark_string_start(&mut self) {
        if self.declaration_is_import
            && self.delimiters.is_empty()
            && (!self.import_has_clause || self.import_saw_from)
        {
            self.import_has_source = true;
        }
        self.line_end = LineEnd::Punctuation(b'\'');
    }

    fn open(&mut self, delimiter: Delimiter, byte: u8) {
        if self.declaration_is_import
            && self.declaration_seen
            && self.delimiters.is_empty()
            && matches!(delimiter, Delimiter::Brace)
        {
            self.import_has_clause = true;
        }
        if self.declaration_is_export
            && self.declaration_seen
            && self.delimiters.is_empty()
            && matches!(delimiter, Delimiter::Brace)
        {
            self.export_from_candidate = true;
        }
        self.delimiters.push(delimiter);
        self.line_end = LineEnd::Punctuation(byte);
    }

    fn open_parenthesis(&mut self, byte: u8) {
        let delimiter = if self.control_condition_pending
            || matches!(
                self.line_end,
                LineEnd::Word {
                    starts_control_condition: true,
                    ..
                }
            ) {
            self.control_condition_pending = false;
            Delimiter::StatementParenthesis
        } else {
            Delimiter::Parenthesis
        };
        self.open(delimiter, byte);
    }

    fn is_statement_position(&self) -> bool {
        matches!(
            self.delimiters.last(),
            None | Some(Delimiter::StatementBlock)
        ) && matches!(
            self.line_end,
            LineEnd::None | LineEnd::StatementBoundary | LineEnd::Punctuation(b'{' | b'}' | b';')
        )
    }

    fn can_start_function_or_class(&self) -> bool {
        if matches!(
            self.delimiters.last(),
            Some(Delimiter::Brace | Delimiter::Bracket)
        ) {
            return matches!(
                self.line_end,
                LineEnd::Punctuation(b':' | b'=' | b'(' | b'[' | b',')
            );
        }

        matches!(
            self.line_end,
            LineEnd::None
                | LineEnd::StatementBoundary
                | LineEnd::Word {
                    requires_following: true,
                    ..
                }
                | LineEnd::Word {
                    allows_function_or_class: true,
                    ..
                }
                | LineEnd::Punctuation(b'=' | b':' | b'(' | b'[' | b'{' | b',')
        )
    }

    fn open_brace(&mut self, byte: u8) {
        let delimiter = if self.statement_block_pending.is_some()
            || matches!(self.line_end, LineEnd::StatementBoundary)
        {
            self.statement_block_pending = None;
            Delimiter::StatementBlock
        } else {
            Delimiter::Brace
        };
        self.open(delimiter, byte);
    }

    fn consume_optional_statement_body(&mut self, bytes: &[u8], pos: usize) {
        if self.statement_block_pending != Some(PendingStatementBlock::Optional) {
            return;
        }

        let byte = bytes[pos];
        if matches!(byte, b' ' | b'\t' | b'\r' | b'\n' | b'{')
            || (byte == b'/' && matches!(bytes.get(pos + 1), Some(b'/' | b'*')))
        {
            return;
        }

        // A non-block token starts the one braceless body statement. Its
        // eventual semicolon or ASI boundary cannot introduce a delayed block,
        // so never let this marker affect a later object literal.
        self.statement_block_pending = None;
    }

    fn close_parenthesis(&mut self, byte: u8) {
        let Some(actual) = self.delimiters.pop() else {
            self.malformed = true;
            return;
        };
        if actual == Delimiter::StatementParenthesis {
            self.line_end = LineEnd::StatementBoundary;
        } else if actual == Delimiter::Parenthesis {
            self.line_end = LineEnd::Punctuation(byte);
        } else {
            self.malformed = true;
        }
    }

    fn close_brace(&mut self, byte: u8) {
        let Some(actual) = self.delimiters.pop() else {
            self.malformed = true;
            return;
        };
        if actual == Delimiter::TemplateExpression {
            self.mode = LexicalMode::Template;
            self.line_end = LineEnd::Punctuation(byte);
        } else if actual == Delimiter::StatementBlock {
            self.line_end = LineEnd::StatementBoundary;
        } else if actual == Delimiter::Brace {
            self.line_end = LineEnd::Punctuation(byte);
        } else {
            self.malformed = true;
        }
    }

    fn close(&mut self, expected: Delimiter, byte: u8) {
        let Some(actual) = self.delimiters.pop() else {
            self.malformed = true;
            return;
        };
        if actual == Delimiter::TemplateExpression && expected == Delimiter::Brace {
            self.mode = LexicalMode::Template;
        } else if actual != expected {
            self.malformed = true;
        }
        self.line_end = LineEnd::Punctuation(byte);
    }

    fn needs_explicit_continuation(&self) -> bool {
        !self.delimiters.is_empty()
            || !matches!(self.mode, LexicalMode::Code)
            || matches!(
                self.line_end,
                LineEnd::Word {
                    requires_following: true,
                    ..
                } | LineEnd::Punctuation(
                    b',' | b'.'
                        | b'='
                        | b'+'
                        | b'-'
                        | b'*'
                        | b'/'
                        | b'%'
                        | b'&'
                        | b'|'
                        | b'^'
                        | b'!'
                        | b'~'
                        | b'?'
                        | b':'
                        | b'<'
                        | b'>'
                )
            )
    }

    fn can_start_regex(&self) -> bool {
        matches!(
            self.line_end,
            LineEnd::None
                | LineEnd::Word {
                    requires_following: true,
                    ..
                }
                | LineEnd::StatementBoundary
                | LineEnd::Punctuation(
                    b'(' | b'['
                        | b'{'
                        | b','
                        | b':'
                        | b';'
                        | b'='
                        | b'!'
                        | b'?'
                        | b'&'
                        | b'|'
                        | b'^'
                        | b'~'
                        | b'+'
                        | b'-'
                        | b'*'
                        | b'%'
                        | b'<'
                        | b'>'
                )
        )
    }

    fn import_needs_source(&self) -> bool {
        self.declaration_is_import && !self.import_has_source
    }

    /// A completed regex expression can continue on the following line only
    /// through a lexically valid expression suffix chain. The lookahead is
    /// deliberately bounded to that physical line: this keeps ordinary
    /// Markdown from being accumulated while avoiding rescanning prior lines.
    fn regex_continues_with_expression(&self, bytes: &[u8], line_start: usize) -> bool {
        if self.line_end != LineEnd::Regex {
            return false;
        }

        let start = skip_whitespace_offset(bytes, line_start);
        has_regex_expression_suffix(bytes, start)
    }
}

fn is_esm_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$')
}

fn word_requires_following(word: &[u8]) -> bool {
    matches!(
        word,
        b"as"
            | b"async"
            | b"await"
            | b"class"
            | b"const"
            | b"default"
            | b"delete"
            | b"extends"
            | b"from"
            | b"function"
            | b"in"
            | b"instanceof"
            | b"let"
            | b"new"
            | b"of"
            | b"return"
            | b"throw"
            | b"typeof"
            | b"var"
            | b"void"
            | b"yield"
    )
}

/// Scan an ESM block once, continuing only across lexical or module-clause
/// boundaries that require another JavaScript line. The state is updated as
/// each byte is consumed, so long declarations are linear in their input size.
pub(crate) fn scan_esm(
    bytes: &[u8],
    pos: usize,
    container_fences: &ContainerFenceLines,
) -> Option<EsmScan> {
    if !is_esm_start(&bytes[pos..]) {
        return None;
    }

    let len = bytes.len();
    let mut state = EsmContinuation::new(bytes, pos);
    let mut line_start = pos;
    let mut end = next_line(bytes, line_start);
    loop {
        state.scan_line(bytes, line_start, end);

        if end >= len {
            return Some(
                if state.malformed
                    || state.needs_explicit_continuation()
                    || state.import_needs_source()
                {
                    EsmScan::Incomplete(end)
                } else {
                    EsmScan::Complete(end)
                },
            );
        }

        if opening_code_fence(bytes, end).is_some()
            || container_fences.contains_opener(end)
            || container_fences.contains_owner(end)
        {
            return Some(
                if state.malformed
                    || state.needs_explicit_continuation()
                    || state.import_needs_source()
                {
                    EsmScan::Incomplete(end)
                } else {
                    EsmScan::Complete(end)
                },
            );
        }

        if is_blank_line(bytes, end) {
            return Some(
                if state.malformed
                    || state.needs_explicit_continuation()
                    || state.import_needs_source()
                {
                    EsmScan::Incomplete(end)
                } else {
                    EsmScan::Complete(end)
                },
            );
        }

        if state.malformed {
            return Some(EsmScan::Incomplete(end));
        }

        if state.needs_explicit_continuation() {
            if is_markdown_block_start(bytes, end) || is_plain_prose_line(bytes, end) {
                return Some(EsmScan::Incomplete(end));
            }
            line_start = end;
            end = next_line(bytes, line_start);
            continue;
        }

        if state.regex_continues_with_expression(bytes, end) {
            line_start = end;
            end = next_line(bytes, line_start);
            continue;
        }

        if state.import_needs_source() {
            if is_import_clause_continuation(bytes, end) {
                line_start = end;
                end = next_line(bytes, line_start);
                continue;
            }
            return Some(EsmScan::Incomplete(end));
        }

        if state.export_from_candidate && starts_with_word(bytes, end, b"from") {
            line_start = end;
            end = next_line(bytes, line_start);
            continue;
        }

        return Some(EsmScan::Complete(end));
    }
}

fn is_markdown_block_start(bytes: &[u8], line_start: usize) -> bool {
    let first = skip_whitespace_offset(bytes, line_start);
    let Some(&byte) = bytes.get(first) else {
        return false;
    };
    match byte {
        b'#' | b'>' | b'!' | b'<' | b'{' => true,
        b'-' | b'+' | b'*' => matches!(bytes.get(first + 1), Some(b' ' | b'\t')),
        b'0'..=b'9' => {
            let mut pos = first;
            while bytes.get(pos).is_some_and(u8::is_ascii_digit) {
                pos += 1;
            }
            matches!(bytes.get(pos), Some(b'.' | b')'))
                && matches!(bytes.get(pos + 1), Some(b' ' | b'\t'))
        }
        _ => false,
    }
}

fn is_plain_prose_line(bytes: &[u8], line_start: usize) -> bool {
    let end = next_line(bytes, line_start);
    let mut pos = line_start;
    let mut first_word = None;
    let mut words = 0;

    while pos < end {
        while matches!(bytes.get(pos), Some(b' ' | b'\t' | b'\r' | b'\n')) {
            pos += 1;
        }
        let start = pos;
        while pos < end && !matches!(bytes[pos], b' ' | b'\t' | b'\r' | b'\n') {
            if !(bytes[pos].is_ascii_alphanumeric()
                || matches!(bytes[pos], b'_' | b'$' | b'.' | b'!' | b'?'))
            {
                return false;
            }
            pos += 1;
        }
        if start == pos {
            break;
        }
        if first_word.is_none() {
            first_word = Some(&bytes[start..pos]);
        }
        words += 1;
    }

    words > 1
        && first_word.is_some_and(|word| {
            word[0].is_ascii_alphabetic()
                && word != b"new"
                && word != b"async"
                && !word_requires_following(word)
        })
}

/// Return whether the rest of this physical line is a plausible continuation
/// of a completed regex expression.
///
/// Markdown and JavaScript intentionally overlap here: `[value](arg)`,
/// `(value, flags)`, and a template literal are all valid ECMAScript suffixes.
/// In those cases we follow ECMAScript continuation rules. We only stop for a
/// suffix that is lexically impossible, such as prose containing adjacent
/// identifiers inside an index or call. A semicolon or blank line before
/// Markdown makes the intended boundary explicit.
fn has_regex_expression_suffix(bytes: &[u8], start: usize) -> bool {
    try_regex_expression_suffix(bytes, start).unwrap_or(false)
}

fn try_regex_expression_suffix(bytes: &[u8], start: usize) -> Option<bool> {
    let end = next_line(bytes, start);
    let mut pos = start;
    let mut saw_suffix = false;

    loop {
        pos = skip_suffix_trivia(bytes, pos, end)?;
        match bytes.get(pos).copied() {
            Some(b'.') => {
                pos = scan_suffix_property(bytes, pos + 1, end)?;
                saw_suffix = true;
            }
            Some(b'?') if bytes.get(pos + 1) == Some(&b'.') => {
                pos += 2;
                pos = skip_suffix_trivia(bytes, pos, end)?;
                pos = match bytes.get(pos).copied() {
                    Some(b'[') => scan_suffix_group(bytes, pos, b'[', b']', end)?,
                    Some(b'(') => scan_suffix_group(bytes, pos, b'(', b')', end)?,
                    _ if suffix_identifier_starts_at(bytes, pos, end) => {
                        scan_suffix_identifier(bytes, pos, end)?
                    }
                    _ => return Some(false),
                };
                saw_suffix = true;
            }
            Some(b'[') => {
                pos = scan_suffix_group(bytes, pos, b'[', b']', end)?;
                saw_suffix = true;
            }
            Some(b'(') => {
                pos = scan_suffix_group(bytes, pos, b'(', b')', end)?;
                saw_suffix = true;
            }
            Some(b'`') => {
                pos = scan_suffix_template(bytes, pos, end)?;
                saw_suffix = true;
            }
            _ => break,
        }
    }

    if !saw_suffix {
        return Some(false);
    }

    pos = skip_suffix_trivia(bytes, pos, end)?;
    Some(pos >= end || matches!(bytes.get(pos), Some(b';')))
}

/// Skip whitespace and comments between suffix tokens without leaving the
/// current physical line. `None` denotes an unterminated block comment.
fn skip_suffix_trivia(bytes: &[u8], mut pos: usize, end: usize) -> Option<usize> {
    loop {
        while pos < end && matches!(bytes[pos], b' ' | b'\t' | b'\r' | b'\n') {
            pos += 1;
        }
        if bytes.get(pos..pos + 2) == Some(b"//") {
            return Some(end);
        }
        if bytes.get(pos..pos + 2) != Some(b"/*") {
            return Some(pos);
        }
        pos += 2;
        while pos + 1 < end && bytes.get(pos..pos + 2) != Some(b"*/") {
            pos += 1;
        }
        if bytes.get(pos..pos + 2) != Some(b"*/") {
            return None;
        }
        pos += 2;
    }
}

fn scan_suffix_property(bytes: &[u8], pos: usize, end: usize) -> Option<usize> {
    let pos = skip_suffix_trivia(bytes, pos, end)?;
    scan_suffix_identifier(bytes, pos, end)
}

fn suffix_identifier_starts_at(bytes: &[u8], pos: usize, end: usize) -> bool {
    scan_suffix_identifier_char(bytes, pos, end, true).is_some()
}

/// Scan an ECMAScript identifier token, including Unicode scalar values and
/// `\\uXXXX`/`\\u{X...}` escapes. The scanner deliberately validates only the
/// identifier primitive; declaration grammar remains outside this lightweight
/// continuation check.
fn scan_suffix_identifier(bytes: &[u8], mut pos: usize, end: usize) -> Option<usize> {
    pos = scan_suffix_identifier_char(bytes, pos, end, true)?;
    while let Some(next) = scan_suffix_identifier_char(bytes, pos, end, false) {
        pos = next;
    }
    Some(pos)
}

fn scan_suffix_identifier_char(
    bytes: &[u8],
    pos: usize,
    end: usize,
    is_start: bool,
) -> Option<usize> {
    let byte = *bytes.get(pos)?;
    if byte == b'\\' {
        return scan_suffix_identifier_escape(bytes, pos, end, is_start);
    }
    if byte.is_ascii() {
        let valid = if is_start {
            is_suffix_identifier_start_char(char::from(byte))
        } else {
            is_suffix_identifier_continue_char(char::from(byte))
        };
        return valid.then_some(pos + 1);
    }

    let character = std::str::from_utf8(bytes.get(pos..end)?)
        .ok()?
        .chars()
        .next()?;
    let valid = if is_start {
        is_suffix_identifier_start_char(character)
    } else {
        is_suffix_identifier_continue_char(character)
    };
    valid.then_some(pos + character.len_utf8())
}

#[inline]
fn is_suffix_identifier_start_char(character: char) -> bool {
    character == '$' || character == '_' || is_ecmascript_id_start(character)
}

#[inline]
fn is_suffix_identifier_continue_char(character: char) -> bool {
    character == '$'
        || character == '_'
        || matches!(character, '\u{200c}' | '\u{200d}')
        || unicode_ident::is_xid_continue(character)
        || is_ecmascript_id_start(character)
}

/// ECMAScript uses Unicode ID_Start/ID_Continue while `unicode-ident` exposes
/// the stricter normalization-closed XID properties. Keep XID as the fast
/// common path and include the finite ID_Start compatibility set it excludes.
#[inline]
fn is_ecmascript_id_start(character: char) -> bool {
    unicode_ident::is_xid_start(character)
        || matches!(
            character,
            '\u{037a}'
                | '\u{0e33}'
                | '\u{0eb3}'
                | '\u{2118}'
                | '\u{212e}'
                | '\u{309b}'
                | '\u{309c}'
                | '\u{fc5e}'
                ..='\u{fc63}'
                    | '\u{fdfa}'
                    | '\u{fdfb}'
                    | '\u{fe70}'
                    | '\u{fe72}'
                    | '\u{fe74}'
                    | '\u{fe76}'
                    | '\u{fe78}'
                    | '\u{fe7a}'
                    | '\u{fe7c}'
                    | '\u{fe7e}'
                    | '\u{ff9e}'
                    | '\u{ff9f}'
        )
}

fn scan_suffix_identifier_escape(
    bytes: &[u8],
    pos: usize,
    end: usize,
    is_start: bool,
) -> Option<usize> {
    if bytes.get(pos + 1) != Some(&b'u') {
        return None;
    }
    let mut cursor = pos + 2;
    let code_point = if bytes.get(cursor) == Some(&b'{') {
        cursor += 1;
        let digits_start = cursor;
        let mut value = 0_u32;
        while cursor < end && bytes[cursor] != b'}' {
            value = value.checked_mul(16)? + u32::from(hex_value(bytes[cursor])?);
            cursor += 1;
            if cursor - digits_start > 6 {
                return None;
            }
        }
        if cursor == digits_start || bytes.get(cursor) != Some(&b'}') {
            return None;
        }
        cursor += 1;
        value
    } else {
        let mut value = 0_u32;
        for _ in 0..4 {
            value = value.checked_mul(16)? + u32::from(hex_value(*bytes.get(cursor)?)?);
            cursor += 1;
        }
        value
    };
    let character = char::from_u32(code_point)?;
    let valid = if is_start {
        is_suffix_identifier_start_char(character)
    } else {
        is_suffix_identifier_continue_char(character)
    };
    valid.then_some(cursor)
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// Scan a bracketed/index or parenthesized/call suffix and reject obvious prose
/// (for example `Markdown prose`) while accepting balanced nested JavaScript
/// strings, comments, templates, and groups.
fn scan_suffix_group(
    bytes: &[u8],
    pos: usize,
    opening: u8,
    closing: u8,
    end: usize,
) -> Option<usize> {
    debug_assert_eq!(bytes.get(pos), Some(&opening));
    let mut stack = vec![closing];
    let mut pos = pos + 1;
    let mut previous_is_operand = false;

    while pos < end {
        match bytes[pos] {
            b' ' | b'\t' | b'\r' | b'\n' => pos += 1,
            b'/' if bytes.get(pos + 1) == Some(&b'/') => return None,
            b'/' if bytes.get(pos + 1) == Some(&b'*') => {
                pos = skip_suffix_trivia(bytes, pos, end)?;
            }
            b'\'' | b'"' => {
                if previous_is_operand {
                    return None;
                }
                pos = scan_suffix_string(bytes, pos, end)?;
                previous_is_operand = true;
            }
            b'`' => {
                if previous_is_operand {
                    return None;
                }
                pos = scan_suffix_template(bytes, pos, end)?;
                previous_is_operand = true;
            }
            b'/' if !previous_is_operand => {
                pos = scan_suffix_regex(bytes, pos, end)?;
                previous_is_operand = true;
            }
            _ if suffix_identifier_starts_at(bytes, pos, end) => {
                let word_start = pos;
                pos = scan_suffix_identifier(bytes, pos, end)?;
                let word = &bytes[word_start..pos];
                if word_requires_following(word) {
                    previous_is_operand = false;
                } else if previous_is_operand {
                    return None;
                } else {
                    previous_is_operand = true;
                }
            }
            b'0'..=b'9' => {
                if previous_is_operand {
                    return None;
                }
                pos = scan_suffix_number(bytes, pos, end)?;
                previous_is_operand = true;
            }
            b'(' => {
                stack.push(b')');
                pos += 1;
                previous_is_operand = false;
            }
            b'[' => {
                stack.push(b']');
                pos += 1;
                previous_is_operand = false;
            }
            b'{' => {
                stack.push(b'}');
                pos += 1;
                previous_is_operand = false;
            }
            b')' | b']' | b'}' => {
                if stack.pop()? != bytes[pos] {
                    return None;
                }
                pos += 1;
                previous_is_operand = true;
                if stack.is_empty() {
                    return Some(pos);
                }
            }
            // These are expression separators/operators. Full JavaScript
            // grammar validation happens downstream; this bounded lexical
            // check only needs to reject unambiguously prose-like adjacency.
            b'.' | b',' | b':' | b';' | b'=' | b'+' | b'-' | b'*' | b'/' | b'%' | b'&' | b'|'
            | b'^' | b'!' | b'~' | b'?' | b'<' | b'>' => {
                previous_is_operand = false;
                pos += 1;
            }
            _ => return None,
        }
    }

    None
}

/// Scan decimal (including a leading-dot fractional part when entered after a
/// punctuator), exponent, radix-prefixed, separator, and BigInt numeric forms.
fn scan_suffix_number(bytes: &[u8], mut pos: usize, end: usize) -> Option<usize> {
    debug_assert!(matches!(bytes.get(pos), Some(b'0'..=b'9')));
    if bytes.get(pos) == Some(&b'0') {
        let radix = match bytes.get(pos + 1) {
            Some(b'x' | b'X') => Some(16),
            Some(b'o' | b'O') => Some(8),
            Some(b'b' | b'B') => Some(2),
            _ => None,
        };
        if let Some(radix) = radix {
            pos += 2;
            pos = scan_suffix_digits(bytes, pos, end, radix)?;
            if bytes.get(pos) == Some(&b'n') {
                pos += 1;
            }
            return Some(pos);
        }
    }

    pos = scan_suffix_digits(bytes, pos, end, 10)?;
    let mut has_fraction_or_exponent = false;
    if bytes.get(pos) == Some(&b'.') {
        has_fraction_or_exponent = true;
        pos += 1;
        if matches!(bytes.get(pos), Some(b'0'..=b'9')) {
            pos = scan_suffix_digits(bytes, pos, end, 10)?;
        }
    }
    if matches!(bytes.get(pos), Some(b'e' | b'E')) {
        has_fraction_or_exponent = true;
        pos += 1;
        if matches!(bytes.get(pos), Some(b'+' | b'-')) {
            pos += 1;
        }
        pos = scan_suffix_digits(bytes, pos, end, 10)?;
    }
    if !has_fraction_or_exponent && bytes.get(pos) == Some(&b'n') {
        pos += 1;
    }
    Some(pos)
}

fn scan_suffix_digits(bytes: &[u8], mut pos: usize, end: usize, radix: u8) -> Option<usize> {
    let mut saw_digit = false;
    let mut previous_was_separator = false;
    while let Some(&byte) = bytes.get(pos).filter(|_| pos < end) {
        if digit_value(byte).is_some_and(|value| value < radix) {
            saw_digit = true;
            previous_was_separator = false;
            pos += 1;
        } else if byte == b'_' {
            if !saw_digit || previous_was_separator {
                return None;
            }
            previous_was_separator = true;
            pos += 1;
        } else {
            break;
        }
    }
    (saw_digit && !previous_was_separator).then_some(pos)
}

fn digit_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn scan_suffix_string(bytes: &[u8], mut pos: usize, end: usize) -> Option<usize> {
    let quote = *bytes.get(pos)?;
    pos += 1;
    while pos < end {
        match bytes[pos] {
            b'\\' => pos += 2,
            byte if byte == quote => return Some(pos + 1),
            _ => pos += 1,
        }
    }
    None
}

fn scan_suffix_regex(bytes: &[u8], mut pos: usize, end: usize) -> Option<usize> {
    pos += 1;
    let mut in_class = false;
    while pos < end {
        match bytes[pos] {
            b'\\' => pos += 2,
            b'[' => {
                in_class = true;
                pos += 1;
            }
            b']' => {
                in_class = false;
                pos += 1;
            }
            b'/' if !in_class => {
                pos += 1;
                while pos < end && bytes[pos].is_ascii_alphabetic() {
                    pos += 1;
                }
                return Some(pos);
            }
            _ => pos += 1,
        }
    }
    None
}

fn scan_suffix_template(bytes: &[u8], mut pos: usize, end: usize) -> Option<usize> {
    debug_assert_eq!(bytes.get(pos), Some(&b'`'));
    pos += 1;
    while pos < end {
        match bytes[pos] {
            b'\\' => pos += 2,
            b'`' => return Some(pos + 1),
            b'$' if bytes.get(pos + 1) == Some(&b'{') => {
                pos = scan_suffix_group(bytes, pos + 1, b'{', b'}', end)?;
            }
            _ => pos += 1,
        }
    }
    None
}

fn starts_with_word(bytes: &[u8], line_start: usize, word: &[u8]) -> bool {
    let start = skip_whitespace_offset(bytes, line_start);
    bytes.get(start..start + word.len()) == Some(word)
        && bytes
            .get(start + word.len())
            .is_none_or(|byte| !is_esm_word_byte(*byte))
}

fn is_import_clause_continuation(bytes: &[u8], line_start: usize) -> bool {
    let start = skip_whitespace_offset(bytes, line_start);
    matches!(bytes.get(start), Some(b'{' | b'*' | b'\'' | b'"'))
        || starts_with_word(bytes, line_start, b"from")
        || line_contains_from_clause(bytes, start)
}

fn line_contains_from_clause(bytes: &[u8], mut pos: usize) -> bool {
    let end = next_line(bytes, pos);
    while pos + 4 <= end {
        if bytes.get(pos..pos + 4) == Some(b"from")
            && (pos == 0 || !is_esm_word_byte(bytes[pos - 1]))
            && bytes
                .get(pos + 4)
                .is_none_or(|byte| !is_esm_word_byte(*byte))
        {
            return true;
        }
        pos += 1;
    }
    false
}

/// Try to parse a complete ESM block (`import`/`export`) starting at `pos`.
/// Incomplete declarations deliberately fall back to Markdown in permissive
/// mode; strict mode obtains their boundary through [`scan_esm`].
pub(crate) fn try_esm(
    bytes: &[u8],
    pos: usize,
    container_fences: &ContainerFenceLines,
) -> Option<usize> {
    match scan_esm(bytes, pos, container_fences)? {
        EsmScan::Complete(end) => Some(end),
        EsmScan::Incomplete(_) => None,
    }
}

/// Parser-derived physical line boundaries for fenced code in containers.
///
/// Root-level fences are tracked directly by [`split`]. CommonMark also lets
/// fences occur after container prefixes such as list markers, so reuse the
/// block parser's fenced-code events rather than copying its container rules.
pub(crate) struct ContainerFenceLines {
    openers: Vec<usize>,
    owners: Vec<usize>,
    content: Vec<usize>,
    closers: Vec<usize>,
    root_openers: Vec<usize>,
}

impl ContainerFenceLines {
    pub(crate) fn contains_opener(&self, line_start: usize) -> bool {
        self.openers.binary_search(&line_start).is_ok()
    }

    pub(crate) fn contains_owner(&self, line_start: usize) -> bool {
        self.owners.binary_search(&line_start).is_ok()
    }

    pub(crate) fn contains_content(&self, line_start: usize) -> bool {
        self.content.binary_search(&line_start).is_ok()
    }

    pub(crate) fn contains_closer(&self, line_start: usize) -> bool {
        self.closers.binary_search(&line_start).is_ok()
    }

    fn contains_root_opener(&self, line_start: usize) -> bool {
        self.root_openers.binary_search(&line_start).is_ok()
    }
}

struct ParsedFence {
    opener: usize,
    owner: Option<usize>,
    delimiter: CodeFence,
    content: Vec<usize>,
}

pub(crate) fn container_fence_lines(bytes: &[u8]) -> ContainerFenceLines {
    let mut parser = BlockParser::new(bytes);
    let mut events = Vec::new();
    let boundaries = parser.parse_with_fenced_code_boundaries(&mut events);

    let mut fences = ContainerFenceLines {
        openers: Vec::new(),
        owners: Vec::new(),
        content: Vec::new(),
        closers: Vec::new(),
        root_openers: Vec::new(),
    };

    // A container mismatch can close an existing fence and reparse the same
    // physical delimiter as a root-level opener. Record those parser-derived
    // root openers before assigning container closers below.
    let mut boundary_iter = boundaries.iter();
    let mut container_depth = 0usize;
    for event in &events {
        match event {
            BlockEvent::BlockQuoteStart { .. }
            | BlockEvent::ListItemStart { .. }
            | BlockEvent::DefinitionDescriptionStart { .. } => container_depth += 1,
            BlockEvent::BlockQuoteEnd
            | BlockEvent::ListItemEnd
            | BlockEvent::DefinitionDescriptionEnd => {
                container_depth = container_depth.saturating_sub(1);
            }
            BlockEvent::CodeBlockStart {
                kind: CodeBlockKind::Fenced { .. },
            } => {
                let boundary = boundary_iter
                    .next()
                    .expect("fenced-code boundary accompanies every fenced start");
                if container_depth == 0 {
                    fences
                        .root_openers
                        .push(line_start(bytes, boundary.delimiter.start_usize()));
                }
            }
            _ => {}
        }
    }
    fences.root_openers.sort_unstable();
    fences.root_openers.dedup();

    let mut boundaries = boundaries.into_iter();
    let mut current_fence = None;
    let mut container_depth = 0usize;
    for event in events {
        match event {
            BlockEvent::BlockQuoteStart { .. }
            | BlockEvent::ListItemStart { .. }
            | BlockEvent::DefinitionDescriptionStart { .. } => container_depth += 1,
            BlockEvent::BlockQuoteEnd
            | BlockEvent::ListItemEnd
            | BlockEvent::DefinitionDescriptionEnd => {
                container_depth = container_depth.saturating_sub(1);
            }
            BlockEvent::CodeBlockStart {
                kind: CodeBlockKind::Fenced { .. },
            } => {
                let boundary = boundaries
                    .next()
                    .expect("fenced-code boundary accompanies every fenced start");
                current_fence = (container_depth > 0).then(|| ParsedFence {
                    opener: line_start(bytes, boundary.delimiter.start_usize()),
                    owner: boundary.container_start,
                    delimiter: CodeFence {
                        marker: bytes[boundary.delimiter.start_usize()],
                        length: boundary.delimiter.len_usize(),
                    },
                    content: Vec::new(),
                });
            }
            BlockEvent::CodeBlockStart { .. } => current_fence = None,
            BlockEvent::Code(range) => {
                if let Some(fence) = &mut current_fence {
                    fence.content.push(line_start(bytes, range.start_usize()));
                }
            }
            BlockEvent::CodeBlockEnd => finish_fence(bytes, &mut fences, current_fence.take()),
            _ => {}
        }
    }

    finish_fence(bytes, &mut fences, current_fence);
    fences.openers.sort_unstable();
    fences.openers.dedup();
    fences.owners.sort_unstable();
    fences.owners.dedup();
    fences.content.sort_unstable();
    fences.content.dedup();
    fences.closers.sort_unstable();
    fences.closers.dedup();
    fences
}

fn finish_fence(bytes: &[u8], fences: &mut ContainerFenceLines, fence: Option<ParsedFence>) {
    let Some(fence) = fence else {
        return;
    };
    let opener = fence.opener;

    fences.openers.push(opener);
    if let Some(owner) = fence.owner {
        fences.owners.push(owner);
    }
    fences.content.extend(fence.content.iter().copied());
    let final_line = fence.content.last().copied().unwrap_or(opener);
    let closer = next_line(bytes, final_line);
    if closer < bytes.len()
        && !fences.contains_root_opener(closer)
        && is_container_fence_delimiter(bytes, closer, fence.delimiter)
    {
        fences.closers.push(closer);
    }
}

/// Whether `line_start` is a delimiter line after an optional container
/// prefix. The block parser emits `CodeBlockEnd` for both a real closing
/// delimiter and a container mismatch, so only the former may reset MDX's
/// paragraph state.
fn is_container_fence_delimiter(bytes: &[u8], mut pos: usize, fence: CodeFence) -> bool {
    skip_horizontal_whitespace(bytes, &mut pos);
    while bytes.get(pos) == Some(&b'>') {
        pos += 1;
        skip_horizontal_whitespace(bytes, &mut pos);
    }

    let fence_start = pos;
    while bytes.get(pos) == Some(&fence.marker) {
        pos += 1;
    }
    if pos - fence_start < fence.length {
        return false;
    }

    bytes[pos..]
        .iter()
        .take_while(|byte| **byte != b'\n')
        .all(|byte| matches!(*byte, b' ' | b'\t'))
}

fn skip_horizontal_whitespace(bytes: &[u8], pos: &mut usize) {
    while matches!(bytes.get(*pos), Some(b' ' | b'\t')) {
        *pos += 1;
    }
}

fn line_start(bytes: &[u8], mut pos: usize) -> usize {
    while pos > 0 && bytes[pos - 1] != b'\n' {
        pos -= 1;
    }
    pos
}

/// Check whether bytes start with a static `import` or `export` declaration.
///
/// This recognises the same declaration prefixes as [`try_esm`] and excludes
/// dynamic imports and `import.meta` access.
pub(crate) fn is_esm_start(rest: &[u8]) -> bool {
    let is_import = rest.starts_with(b"import ")
        || rest.starts_with(b"import\t")
        || rest.starts_with(b"import{")
        || (rest.len() >= 7 && rest.starts_with(b"import\""))
        || (rest.len() >= 7 && rest.starts_with(b"import'"));
    let is_export = rest.starts_with(b"export ")
        || rest.starts_with(b"export\t")
        || rest.starts_with(b"export{");

    if !is_import && !is_export {
        return false;
    }

    if is_import {
        let after_import = skip_whitespace_offset(rest, 6); // "import".len() == 6
        return after_import >= rest.len()
            || (rest[after_import] != b'(' && rest[after_import] != b'.');
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Basic ────────────────────────────────────────────────────────

    #[test]
    fn pure_markdown() {
        let input = "# Hello\n\nWorld\n";
        let segs = split(input);
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0], Segment::Markdown(input));
    }

    #[test]
    fn empty_input() {
        let segs = split("");
        assert!(segs.is_empty());
    }

    // ── ESM: valid forms ─────────────────────────────────────────────

    #[test]
    fn import_then_markdown() {
        let input = "import Foo from 'foo'\n\n# Hello\n";
        let segs = split(input);
        assert_eq!(segs[0], Segment::Esm("import Foo from 'foo'\n"));
        assert_eq!(segs[1], Segment::Markdown("\n# Hello\n"));
    }

    #[test]
    fn export_statement() {
        let input = "export const x = 1\n\nText\n";
        let segs = split(input);
        assert_eq!(segs[0], Segment::Esm("export const x = 1\n"));
    }

    #[test]
    fn multiple_imports() {
        let input = "import A from 'a'\nimport B from 'b'\n\n# Title\n";
        let segs = split(input);
        assert_eq!(segs[0], Segment::Esm("import A from 'a'\n"));
        assert_eq!(segs[1], Segment::Esm("import B from 'b'\n"));
    }

    #[test]
    fn import_default() {
        let segs = split("import a from \"b\"\n\nc\n");
        assert_eq!(segs[0], Segment::Esm("import a from \"b\"\n"));
    }

    #[test]
    fn import_namespace() {
        let segs = split("import * as a from \"b\"\n\nc\n");
        assert_eq!(segs[0], Segment::Esm("import * as a from \"b\"\n"));
    }

    #[test]
    fn import_destructured() {
        let segs = split("import {a} from \"b\"\n\nc\n");
        assert_eq!(segs[0], Segment::Esm("import {a} from \"b\"\n"));
    }

    #[test]
    fn import_side_effect() {
        let segs = split("import \"a\"\n\nc\n");
        assert_eq!(segs[0], Segment::Esm("import \"a\"\n"));
    }

    #[test]
    fn import_side_effect_single_quote() {
        let segs = split("import 'a'\n\nc\n");
        assert_eq!(segs[0], Segment::Esm("import 'a'\n"));
    }

    #[test]
    fn export_var() {
        let segs = split("export var a = 1\n\nb\n");
        assert_eq!(segs[0], Segment::Esm("export var a = 1\n"));
    }

    #[test]
    fn export_const() {
        let segs = split("export const a = \"\"\n\nb\n");
        assert_eq!(segs[0], Segment::Esm("export const a = \"\"\n"));
    }

    #[test]
    fn export_let() {
        let segs = split("export let a = \"\"\n\nb\n");
        assert_eq!(segs[0], Segment::Esm("export let a = \"\"\n"));
    }

    #[test]
    fn export_default() {
        let segs = split("export default a = 1\n\nb\n");
        assert_eq!(segs[0], Segment::Esm("export default a = 1\n"));
    }

    #[test]
    fn export_function() {
        let segs = split("export function a() {}\n\nb\n");
        assert_eq!(segs[0], Segment::Esm("export function a() {}\n"));
    }

    #[test]
    fn export_class() {
        let segs = split("export class a {}\n\nb\n");
        assert_eq!(segs[0], Segment::Esm("export class a {}\n"));
    }

    #[test]
    fn export_from() {
        let segs = split("export {a} from \"b\"\n\nc\n");
        assert_eq!(segs[0], Segment::Esm("export {a} from \"b\"\n"));
    }

    #[test]
    fn export_star_from() {
        let segs = split("export * from \"a\"\n\nb\n");
        assert_eq!(segs[0], Segment::Esm("export * from \"a\"\n"));
    }

    #[test]
    fn export_multiline() {
        let input = "export {\n  a\n} from \"b\"\n\nc\n";
        let segs = split(input);
        assert_eq!(segs[0], Segment::Esm("export {\n  a\n} from \"b\"\n"));
    }

    // ── ESM: NOT ESM (false positive protection) ─────────────────────

    #[test]
    fn not_esm_impossible() {
        // Word starting with "im" — not `import `
        let segs = split("impossible\n");
        assert_eq!(segs[0], Segment::Markdown("impossible\n"));
    }

    #[test]
    fn not_esm_exporting() {
        // Word starting with "export" — not `export `
        let segs = split("exporting\n");
        assert_eq!(segs[0], Segment::Markdown("exporting\n"));
    }

    #[test]
    fn not_esm_import_dot() {
        // `import.meta` is property access, not ESM
        let segs = split("import.meta.url\n");
        assert_eq!(segs[0], Segment::Markdown("import.meta.url\n"));
    }

    #[test]
    fn not_esm_dynamic_import() {
        // `import("a")` is dynamic import, not ESM
        let segs = split("import(\"a\")\n");
        assert_eq!(segs[0], Segment::Markdown("import(\"a\")\n"));
    }

    #[test]
    fn not_esm_dynamic_import_space() {
        // `import ('a')` is dynamic import with space
        let segs = split("import ('a')\n");
        assert_eq!(segs[0], Segment::Markdown("import ('a')\n"));
    }

    #[test]
    fn not_esm_indented() {
        // Indented import is not ESM
        let segs = split("  import a from \"b\"\n");
        assert_eq!(segs[0], Segment::Markdown("  import a from \"b\"\n"));
    }

    #[test]
    fn not_esm_interrupts_paragraph() {
        // ESM cannot interrupt a paragraph — needs blank line before
        let segs = split("a\nimport a from \"b\"\n");
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0], Segment::Markdown("a\nimport a from \"b\"\n"));
    }

    #[test]
    fn not_esm_interrupts_paragraph_export() {
        let segs = split("a\nexport default c\n");
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0], Segment::Markdown("a\nexport default c\n"));
    }

    #[test]
    fn esm_after_blank_line() {
        // After a blank line, ESM is valid again
        let segs = split("a\n\nimport a from \"b\"\n\nc\n");
        assert!(matches!(segs[0], Segment::Markdown(_)));
        assert_eq!(segs[1], Segment::Esm("import a from \"b\"\n"));
    }

    // ── JSX: flow ────────────────────────────────────────────────────

    #[test]
    fn jsx_open_close() {
        let input = "<Wrapper>\n\n## Heading\n\n</Wrapper>\n";
        let segs = split(input);
        assert_eq!(segs[0], Segment::JsxBlockOpen("<Wrapper>\n"));
        assert_eq!(segs[1], Segment::Markdown("\n## Heading\n\n"));
        assert_eq!(segs[2], Segment::JsxBlockClose("</Wrapper>\n"));
    }

    #[test]
    fn jsx_self_closing() {
        let input = "<Image src=\"foo.png\" />\n";
        let segs = split(input);
        assert_eq!(segs[0], Segment::JsxBlockSelfClose(input));
    }

    #[test]
    fn fragment_open_close() {
        let input = "<>\nHello\n</>\n";
        let segs = split(input);
        assert_eq!(segs[0], Segment::JsxBlockOpen("<>\n"));
        assert_eq!(segs[1], Segment::Markdown("Hello\n"));
        assert_eq!(segs[2], Segment::JsxBlockClose("</>\n"));
    }

    #[test]
    fn jsx_with_attributes() {
        let input = "<Button onClick={handleClick} variant=\"primary\">\nClick me\n</Button>\n";
        let segs = split(input);
        assert_eq!(
            segs[0],
            Segment::JsxBlockOpen("<Button onClick={handleClick} variant=\"primary\">\n")
        );
    }

    #[test]
    fn nested_jsx_components() {
        let input = "<Outer>\n<Inner />\n</Outer>\n";
        let segs = split(input);
        assert_eq!(segs[0], Segment::JsxBlockOpen("<Outer>\n"));
        assert_eq!(segs[1], Segment::JsxBlockSelfClose("<Inner />\n"));
        assert_eq!(segs[2], Segment::JsxBlockClose("</Outer>\n"));
    }

    #[test]
    fn jsx_self_closing_with_leading_spaces() {
        // Leading whitespace still counts as flow
        let segs = split("   <a />\n");
        assert_eq!(segs[0], Segment::JsxBlockSelfClose("   <a />\n"));
    }

    #[test]
    fn jsx_with_markdown_inside() {
        let input = "<a>\nb\n</a>\n";
        let segs = split(input);
        assert_eq!(segs[0], Segment::JsxBlockOpen("<a>\n"));
        assert_eq!(segs[1], Segment::Markdown("b\n"));
        assert_eq!(segs[2], Segment::JsxBlockClose("</a>\n"));
    }

    #[test]
    fn jsx_with_list_inside() {
        let input = "<a>\n- b\n</a>\n";
        let segs = split(input);
        assert_eq!(segs[0], Segment::JsxBlockOpen("<a>\n"));
        assert_eq!(segs[1], Segment::Markdown("- b\n"));
        assert_eq!(segs[2], Segment::JsxBlockClose("</a>\n"));
    }

    // ── JSX: NOT flow (inline/text) ──────────────────────────────────

    #[test]
    fn jsx_trailing_content_is_markdown() {
        // `<x />.` — trailing content makes entire line text/markdown
        let segs = split("<x />.\n");
        assert_eq!(segs[0], Segment::Markdown("<x />.\n"));
    }

    #[test]
    fn jsx_leading_text_is_markdown() {
        // `a <x />` — leading text makes it inline
        let segs = split("a <x />\n");
        assert_eq!(segs[0], Segment::Markdown("a <x />\n"));
    }

    #[test]
    fn close_tag_trailing_content_is_markdown() {
        let segs = split("</a>.\n");
        assert_eq!(segs[0], Segment::Markdown("</a>.\n"));
    }

    // ── Expression: flow ─────────────────────────────────────────────

    #[test]
    fn expression_block() {
        let input = "{variable}\n\nHello\n";
        let segs = split(input);
        assert_eq!(segs[0], Segment::Expression("{variable}\n"));
    }

    #[test]
    fn complex_expression() {
        let input = "{items.map(i => <li>{i}</li>)}\n";
        let segs = split(input);
        assert_eq!(segs[0], Segment::Expression(input));
    }

    #[test]
    fn empty_expression() {
        let segs = split("{}\n");
        assert_eq!(segs[0], Segment::Expression("{}\n"));
    }

    #[test]
    fn expression_with_leading_spaces() {
        let segs = split("  { a }\n");
        assert_eq!(segs[0], Segment::Expression("  { a }\n"));
    }

    #[test]
    fn expression_with_trailing_spaces() {
        let segs = split("{ a } \t\n");
        assert_eq!(segs[0], Segment::Expression("{ a } \t\n"));
    }

    #[test]
    fn expression_multiline() {
        let segs = split("{\n  1 + 1\n}\n");
        assert_eq!(segs[0], Segment::Expression("{\n  1 + 1\n}\n"));
    }

    #[test]
    fn expression_with_comment() {
        let segs = split("{/**/}\n");
        assert_eq!(segs[0], Segment::Expression("{/**/}\n"));
    }

    // ── Expression: NOT flow (inline/text) ───────────────────────────

    #[test]
    fn expression_trailing_content_is_markdown() {
        // `{ a } b` — trailing text makes it text/markdown
        let segs = split("{ a } b\n");
        assert_eq!(segs[0], Segment::Markdown("{ a } b\n"));
    }

    #[test]
    fn expression_in_paragraph_is_markdown() {
        let segs = split("a {b} c\n");
        assert_eq!(segs[0], Segment::Markdown("a {b} c\n"));
    }

    // ── Mixed document ───────────────────────────────────────────────

    #[test]
    fn mixed_document() {
        let input = "\
import A from 'a'

# Title

<Card>

Some **text**.

</Card>
";
        let segs = split(input);
        assert_eq!(segs[0], Segment::Esm("import A from 'a'\n"));
        assert_eq!(segs[1], Segment::Markdown("\n# Title\n\n"));
        assert_eq!(segs[2], Segment::JsxBlockOpen("<Card>\n"));
        assert_eq!(segs[3], Segment::Markdown("\nSome **text**.\n\n"));
        assert_eq!(segs[4], Segment::JsxBlockClose("</Card>\n"));
    }

    // ── Defensive ────────────────────────────────────────────────────

    #[test]
    fn invalid_jsx_becomes_markdown() {
        let segs = split("< 5\n");
        assert_eq!(segs[0], Segment::Markdown("< 5\n"));
    }

    #[test]
    fn unterminated_expression_becomes_markdown() {
        let segs = split("{unterminated\n");
        assert_eq!(segs[0], Segment::Markdown("{unterminated\n"));
    }
}
