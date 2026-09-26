//! Explicit, language-aware typography for prose in a parsed document.

use std::error::Error;
use std::fmt;
use std::ops::Range;
use std::str::FromStr;

use ferromark::ast::{Document, Node, Span};

use crate::{BoxError, TransformContext, TransformPass, text_runs};

/// A supported language for the optional typography pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TypographyLanguage {
    /// English (`en`).
    English,
    /// Spanish (`es`).
    Spanish,
    /// French (`fr`).
    French,
    /// Portuguese (`pt`).
    Portuguese,
    /// German (`de`).
    German,
    /// Italian (`it`).
    Italian,
    /// Dutch (`nl`).
    Dutch,
    /// Polish (`pl`).
    Polish,
    /// Russian (`ru`).
    Russian,
    /// Ukrainian (`uk`).
    Ukrainian,
}

impl TypographyLanguage {
    /// Returns this language's lowercase ISO 639-1 code.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Spanish => "es",
            Self::French => "fr",
            Self::Portuguese => "pt",
            Self::German => "de",
            Self::Italian => "it",
            Self::Dutch => "nl",
            Self::Polish => "pl",
            Self::Russian => "ru",
            Self::Ukrainian => "uk",
        }
    }

    fn converts_apostrophes(self) -> bool {
        !matches!(self, Self::Russian | Self::Ukrainian)
    }

    fn converts_double_hyphens(self) -> bool {
        matches!(self, Self::English | Self::Russian)
    }

    fn converts_measurement_spaces(self) -> bool {
        !matches!(self, Self::Russian | Self::Ukrainian)
    }
}

impl FromStr for TypographyLanguage {
    type Err = UnsupportedTypographyLanguage;

    fn from_str(language: &str) -> Result<Self, Self::Err> {
        match language {
            "en" => Ok(Self::English),
            "es" => Ok(Self::Spanish),
            "fr" => Ok(Self::French),
            "pt" => Ok(Self::Portuguese),
            "de" => Ok(Self::German),
            "it" => Ok(Self::Italian),
            "nl" => Ok(Self::Dutch),
            "pl" => Ok(Self::Polish),
            "ru" => Ok(Self::Russian),
            "uk" => Ok(Self::Ukrainian),
            _ => Err(UnsupportedTypographyLanguage {
                language: language.to_owned(),
            }),
        }
    }
}

/// Error returned when a language code is not supported by the typography pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsupportedTypographyLanguage {
    language: String,
}

impl UnsupportedTypographyLanguage {
    /// Returns the unsupported language code.
    #[must_use]
    pub fn language(&self) -> &str {
        &self.language
    }
}

impl fmt::Display for UnsupportedTypographyLanguage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "unsupported typography language {:?}; supported languages are en, es, fr, pt, de, it, nl, pl, ru, and uk",
            self.language
        )
    }
}

impl Error for UnsupportedTypographyLanguage {}

/// Configuration for the optional typography pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypographyOptions {
    language: TypographyLanguage,
    dashes: bool,
    ellipses: bool,
}

impl TypographyOptions {
    /// Creates typography options for one explicit language.
    ///
    /// Dash and ellipsis conversion are enabled. The pass never detects a
    /// language or changes languages within a document.
    #[must_use]
    pub const fn new(language: TypographyLanguage) -> Self {
        Self {
            language,
            dashes: true,
            ellipses: true,
        }
    }

    /// Enables or disables dash conversion independently of other rules.
    #[must_use]
    pub const fn with_dashes(mut self, enabled: bool) -> Self {
        self.dashes = enabled;
        self
    }

    /// Enables or disables ellipsis conversion independently of other rules.
    #[must_use]
    pub const fn with_ellipses(mut self, enabled: bool) -> Self {
        self.ellipses = enabled;
        self
    }

    /// Returns the configured language.
    #[must_use]
    pub const fn language(self) -> TypographyLanguage {
        self.language
    }

    /// Returns whether dash conversion is enabled.
    #[must_use]
    pub const fn dashes(self) -> bool {
        self.dashes
    }

    /// Returns whether ellipsis conversion is enabled.
    #[must_use]
    pub const fn ellipses(self) -> bool {
        self.ellipses
    }
}

/// A pass that applies explicit language-aware typography to prose text.
///
/// Quotes pair across ordinary inline markup. Code, math, raw HTML, MDX
/// expressions, image metadata, link destinations and detected renderer
/// autolinks are protected. Paragraphs and other block containers have
/// independent quote context. Existing Unicode punctuation is preserved.
pub struct TypographyPass {
    options: TypographyOptions,
}

impl TypographyPass {
    /// Creates a typography pass with the given language and options.
    #[must_use]
    pub const fn new(options: TypographyOptions) -> Self {
        Self { options }
    }
}

impl TransformPass for TypographyPass {
    fn name(&self) -> &'static str {
        "typography"
    }

    fn apply<'arena>(
        &mut self,
        document: &mut Document<'arena>,
        context: &TransformContext<'arena>,
    ) -> Result<(), BoxError> {
        transform_block_children(&mut document.children, context, self.options);
        Ok(())
    }
}

enum InlineItem<'arena> {
    Text {
        value: &'arena str,
        protected: Vec<Range<usize>>,
        escaped: Vec<Range<usize>>,
    },
    Boundary,
    SoftBreak,
}

#[derive(Default)]
struct QuoteState {
    stack: Vec<QuoteFrame>,
    previous: Option<char>,
}

#[derive(Clone, Copy)]
struct QuoteFrame {
    source: char,
    primary: bool,
}

#[derive(Clone, Copy)]
struct QuoteMarks {
    open: &'static str,
    close: &'static str,
}

impl QuoteState {
    fn reset(&mut self) {
        self.stack.clear();
        self.previous = None;
    }

    fn observe(&mut self, value: &str) {
        if let Some(last) = value.chars().last() {
            self.previous = Some(last);
        }
    }

    fn replace_quote(
        &mut self,
        source: char,
        next: Option<char>,
        language: TypographyLanguage,
    ) -> Option<&'static str> {
        if self
            .stack
            .last()
            .is_some_and(|frame| frame.source == source)
            && self.previous.is_some_and(can_close_quote)
            && let Some(frame) = self.stack.pop()
        {
            return Some(quote_marks(language, frame.primary).close);
        }

        if can_open_quote(self.previous, next) {
            let primary = self
                .stack
                .last()
                .map_or(source == '"', |frame| !frame.primary);
            self.stack.push(QuoteFrame { source, primary });
            return Some(quote_marks(language, primary).open);
        }

        if self.previous.is_some_and(can_close_quote) {
            return Some(quote_marks(language, source == '"').close);
        }

        None
    }

    fn observe_authored_quote(
        &mut self,
        character: char,
        next: Option<char>,
        language: TypographyLanguage,
    ) {
        if let Some((opening, primary)) = authored_quote_event(character, language) {
            if opening && can_open_quote(self.previous, next) {
                self.stack.push(QuoteFrame {
                    source: if primary { '"' } else { '\'' },
                    primary,
                });
            } else if !opening
                && self.stack.last().is_some_and(|frame| {
                    frame.primary == primary && self.previous.is_some_and(can_close_quote)
                })
            {
                self.stack.pop();
            }
        }
    }
}

fn transform_block_children<'arena>(
    nodes: &mut [Node<'arena>],
    context: &TransformContext<'arena>,
    options: TypographyOptions,
) {
    let mut cursor = 0;
    while cursor < nodes.len() {
        if is_inline_node(&nodes[cursor]) {
            let start = cursor;
            cursor += 1;
            while cursor < nodes.len() && is_inline_node(&nodes[cursor]) {
                cursor += 1;
            }
            transform_inline_children(&mut nodes[start..cursor], context, options);
        } else {
            transform_block_node(&mut nodes[cursor], context, options);
            cursor += 1;
        }
    }
}

fn transform_block_node<'arena>(
    node: &mut Node<'arena>,
    context: &TransformContext<'arena>,
    options: TypographyOptions,
) {
    match node {
        Node::Paragraph(node) => transform_inline_children(&mut node.children, context, options),
        Node::Heading(node) => transform_inline_children(&mut node.children, context, options),
        Node::BlockQuote(node) => transform_block_children(&mut node.children, context, options),
        Node::List(node) => {
            for item in &mut node.children {
                transform_block_children(&mut item.children, context, options);
            }
        }
        Node::ListItem(node) => transform_block_children(&mut node.children, context, options),
        Node::Table(node) => {
            if let Some(attributes) = &mut node.attributes {
                transform_inline_children(&mut attributes.caption, context, options);
            }
            for row in &mut node.children {
                for cell in &mut row.children {
                    transform_inline_children(&mut cell.children, context, options);
                }
            }
        }
        Node::DefinitionList(node) => {
            transform_block_children(&mut node.children, context, options);
        }
        Node::DefinitionListTerm(node) => {
            transform_inline_children(&mut node.children, context, options);
        }
        Node::DefinitionListDefinition(node) => {
            transform_block_children(&mut node.children, context, options);
        }
        Node::FootnoteDefinition(node) => {
            transform_block_children(&mut node.children, context, options);
        }
        Node::MdxJsxFlowElement(node) => {
            transform_block_children(&mut node.children, context, options);
        }
        Node::ThematicBreak(_)
        | Node::CodeBlock(_)
        | Node::MathBlock(_)
        | Node::Html(_)
        | Node::Text(_)
        | Node::Emphasis(_)
        | Node::Strong(_)
        | Node::InlineCode(_)
        | Node::InlineMath(_)
        | Node::Break(_)
        | Node::Link(_)
        | Node::Image(_)
        | Node::Highlight(_)
        | Node::Delete(_)
        | Node::Superscript(_)
        | Node::Subscript(_)
        | Node::FootnoteReference(_)
        | Node::Definition(_)
        | Node::MdxJsxTextElement(_)
        | Node::MdxjsEsm(_)
        | Node::MdxFlowExpression(_)
        | Node::MdxTextExpression(_) => {}
    }
}

fn is_inline_node(node: &Node<'_>) -> bool {
    matches!(
        node,
        Node::Text(_)
            | Node::Emphasis(_)
            | Node::Strong(_)
            | Node::InlineCode(_)
            | Node::InlineMath(_)
            | Node::Break(_)
            | Node::Link(_)
            | Node::Image(_)
            | Node::Highlight(_)
            | Node::Delete(_)
            | Node::Superscript(_)
            | Node::Subscript(_)
            | Node::FootnoteReference(_)
            | Node::Html(_)
            | Node::MdxJsxTextElement(_)
            | Node::MdxTextExpression(_)
    )
}

fn transform_inline_children<'arena>(
    nodes: &mut [Node<'arena>],
    context: &TransformContext<'arena>,
    options: TypographyOptions,
) {
    let mut items = Vec::new();
    collect_inline_items(nodes, context, &mut items);
    let next_after = next_char_after_items(&items);
    let mut state = QuoteState::default();
    let mut replacements = Vec::new();

    for (index, item) in items.iter().enumerate() {
        match item {
            InlineItem::Text {
                value,
                protected,
                escaped,
            } => replacements.push(transform_text(
                value,
                protected,
                escaped,
                next_after[index],
                &items,
                index,
                &mut state,
                options,
            )),
            InlineItem::Boundary => state.reset(),
            InlineItem::SoftBreak => state.previous = Some(' '),
        }
    }

    let mut replacement_index = 0;
    apply_inline_replacements(nodes, &replacements, &mut replacement_index, context);
}

fn collect_inline_items<'arena>(
    nodes: &[Node<'arena>],
    context: &TransformContext<'arena>,
    items: &mut Vec<InlineItem<'arena>>,
) {
    let mut protected_by_node: Vec<Vec<Range<usize>>> =
        (0..nodes.len()).map(|_| Vec::new()).collect();

    for run in text_runs(nodes) {
        let node_range = run.node_range();
        let protected = run.protected_url_ranges(context);
        let mut byte_cursor = 0;

        for node_index in node_range.clone() {
            let Node::Text(text) = &nodes[node_index] else {
                continue;
            };
            let segment_end = byte_cursor + text.value.len();
            for range in &protected {
                let start = range.start.max(byte_cursor);
                let end = range.end.min(segment_end);
                if start < end {
                    protected_by_node[node_index].push(start - byte_cursor..end - byte_cursor);
                }
            }
            byte_cursor = segment_end;
        }
    }

    for (index, node) in nodes.iter().enumerate() {
        match node {
            Node::Text(text) => items.push(InlineItem::Text {
                value: text.value,
                protected: std::mem::take(&mut protected_by_node[index]),
                escaped: escaped_source_ranges(context.source(), text.span, text.value),
            }),
            Node::Emphasis(node) => collect_inline_items(&node.children, context, items),
            Node::Strong(node) => collect_inline_items(&node.children, context, items),
            Node::Highlight(node) => collect_inline_items(&node.children, context, items),
            Node::Delete(node) => collect_inline_items(&node.children, context, items),
            Node::Superscript(node) => collect_inline_items(&node.children, context, items),
            Node::Subscript(node) => collect_inline_items(&node.children, context, items),
            Node::MdxJsxTextElement(node) => collect_inline_items(&node.children, context, items),
            Node::Link(node) if is_url_label(node.url, &node.children) => {
                items.push(InlineItem::Boundary);
            }
            Node::Link(node) => collect_inline_items(&node.children, context, items),
            Node::Break(_) => items.push(InlineItem::SoftBreak),
            Node::InlineCode(_)
            | Node::InlineMath(_)
            | Node::Image(_)
            | Node::FootnoteReference(_)
            | Node::Html(_)
            | Node::MdxTextExpression(_)
            | Node::ThematicBreak(_)
            | Node::Paragraph(_)
            | Node::Heading(_)
            | Node::BlockQuote(_)
            | Node::List(_)
            | Node::ListItem(_)
            | Node::CodeBlock(_)
            | Node::MathBlock(_)
            | Node::Table(_)
            | Node::DefinitionList(_)
            | Node::DefinitionListTerm(_)
            | Node::DefinitionListDefinition(_)
            | Node::Definition(_)
            | Node::FootnoteDefinition(_)
            | Node::MdxJsxFlowElement(_)
            | Node::MdxjsEsm(_)
            | Node::MdxFlowExpression(_) => items.push(InlineItem::Boundary),
        }
    }
}

fn apply_inline_replacements<'arena>(
    nodes: &mut [Node<'arena>],
    replacements: &[Option<String>],
    replacement_index: &mut usize,
    context: &TransformContext<'arena>,
) {
    for node in nodes {
        match node {
            Node::Text(text) => {
                if let Some(Some(value)) = replacements.get(*replacement_index) {
                    context.replace_text_value(text, value);
                }
                *replacement_index += 1;
            }
            Node::Emphasis(node) => {
                apply_inline_replacements(
                    &mut node.children,
                    replacements,
                    replacement_index,
                    context,
                );
            }
            Node::Strong(node) => {
                apply_inline_replacements(
                    &mut node.children,
                    replacements,
                    replacement_index,
                    context,
                );
            }
            Node::Highlight(node) => {
                apply_inline_replacements(
                    &mut node.children,
                    replacements,
                    replacement_index,
                    context,
                );
            }
            Node::Delete(node) => {
                apply_inline_replacements(
                    &mut node.children,
                    replacements,
                    replacement_index,
                    context,
                );
            }
            Node::Superscript(node) => {
                apply_inline_replacements(
                    &mut node.children,
                    replacements,
                    replacement_index,
                    context,
                );
            }
            Node::Subscript(node) => {
                apply_inline_replacements(
                    &mut node.children,
                    replacements,
                    replacement_index,
                    context,
                );
            }
            Node::MdxJsxTextElement(node) => {
                apply_inline_replacements(
                    &mut node.children,
                    replacements,
                    replacement_index,
                    context,
                );
            }
            Node::Link(node) if is_url_label(node.url, &node.children) => {}
            Node::Link(node) => {
                apply_inline_replacements(
                    &mut node.children,
                    replacements,
                    replacement_index,
                    context,
                );
            }
            Node::ThematicBreak(_)
            | Node::Paragraph(_)
            | Node::Heading(_)
            | Node::BlockQuote(_)
            | Node::List(_)
            | Node::ListItem(_)
            | Node::CodeBlock(_)
            | Node::MathBlock(_)
            | Node::Html(_)
            | Node::InlineCode(_)
            | Node::InlineMath(_)
            | Node::Break(_)
            | Node::Image(_)
            | Node::Table(_)
            | Node::DefinitionList(_)
            | Node::DefinitionListTerm(_)
            | Node::DefinitionListDefinition(_)
            | Node::FootnoteReference(_)
            | Node::Definition(_)
            | Node::FootnoteDefinition(_)
            | Node::MdxJsxFlowElement(_)
            | Node::MdxjsEsm(_)
            | Node::MdxFlowExpression(_)
            | Node::MdxTextExpression(_) => {}
        }
    }
}

fn is_url_label(url: &str, children: &[Node<'_>]) -> bool {
    matches!(children, [Node::Text(text)] if text.value == url)
}

fn next_char_after_items(items: &[InlineItem<'_>]) -> Vec<Option<char>> {
    let mut next_after = vec![None; items.len()];
    let mut next = None;

    for index in (0..items.len()).rev() {
        next_after[index] = next;
        match &items[index] {
            InlineItem::Text {
                value,
                protected,
                escaped,
            } => match first_context_char(value, protected, escaped) {
                ContextChar::Char(character) => next = Some(character),
                ContextChar::Empty => {}
                ContextChar::Boundary => next = None,
            },
            InlineItem::Boundary => next = None,
            InlineItem::SoftBreak => next = Some(' '),
        }
    }

    next_after
}

enum ContextChar {
    Char(char),
    Empty,
    Boundary,
}

fn first_context_char(
    value: &str,
    protected: &[Range<usize>],
    escaped: &[Range<usize>],
) -> ContextChar {
    if value.is_empty() {
        return ContextChar::Empty;
    }
    if is_blocked(0, protected, escaped) {
        return ContextChar::Boundary;
    }
    match value.chars().next() {
        Some(character) => ContextChar::Char(character),
        None => ContextChar::Empty,
    }
}

#[allow(clippy::too_many_arguments)]
fn transform_text(
    value: &str,
    protected: &[Range<usize>],
    escaped: &[Range<usize>],
    next_after: Option<char>,
    items: &[InlineItem<'_>],
    item_index: usize,
    state: &mut QuoteState,
    options: TypographyOptions,
) -> Option<String> {
    let mut result = String::with_capacity(value.len());
    let mut offset = 0;
    let mut protected_index = 0;
    let mut escaped_index = 0;
    let mut changed = false;

    while offset < value.len() {
        advance_range_index(protected, &mut protected_index, offset);
        advance_range_index(escaped, &mut escaped_index, offset);

        if let Some(range) = protected.get(protected_index)
            && range.start <= offset
            && offset < range.end
        {
            let end = range.end.min(value.len());
            state.reset();
            result.push_str(&value[offset..end]);
            offset = end;
            state.reset();
            continue;
        }

        let Some(character) = value[offset..].chars().next() else {
            break;
        };
        let character_end = offset + character.len_utf8();

        if let Some(range) = escaped.get(escaped_index)
            && range.start <= offset
            && offset < range.end
        {
            result.push_str(&value[offset..character_end]);
            state.observe(&value[offset..character_end]);
            offset = character_end;
            continue;
        }

        let next = next_context_char(
            value,
            character_end,
            protected,
            escaped,
            items,
            item_index,
            next_after,
        );

        if options.ellipses()
            && value[offset..].starts_with("...")
            && !is_blocked(offset + 1, protected, escaped)
            && !is_blocked(offset + 2, protected, escaped)
        {
            result.push('…');
            state.observe("…");
            offset += 3;
            changed = true;
            continue;
        }

        if options.dashes()
            && options.language().converts_double_hyphens()
            && value[offset..].starts_with("---")
            && dash_boundary(
                state.previous,
                next_context_char(
                    value,
                    offset + 3,
                    protected,
                    escaped,
                    items,
                    item_index,
                    next_after,
                ),
            )
        {
            result.push('—');
            state.observe("—");
            offset += 3;
            changed = true;
            continue;
        }

        if options.dashes()
            && options.language().converts_double_hyphens()
            && value[offset..].starts_with("--")
            && dash_boundary(
                state.previous,
                next_context_char(
                    value,
                    offset + 2,
                    protected,
                    escaped,
                    items,
                    item_index,
                    next_after,
                ),
            )
        {
            result.push('—');
            state.observe("—");
            offset += 2;
            changed = true;
            continue;
        }

        if character == ' '
            && options.language() == TypographyLanguage::French
            && is_french_punctuation(next)
        {
            result.push('\u{00a0}');
            state.observe("\u{00a0}");
            offset = character_end;
            changed = true;
            continue;
        }

        if character == ' '
            && options.language().converts_measurement_spaces()
            && state
                .previous
                .is_some_and(|previous| previous.is_ascii_digit())
            && has_unit_after(value, character_end, protected, escaped)
        {
            result.push('\u{00a0}');
            state.observe("\u{00a0}");
            offset = character_end;
            changed = true;
            continue;
        }

        let following_dash_length = dash_sequence_length(&value[character_end..]);
        if character == ' '
            && options.dashes()
            && options.language().converts_double_hyphens()
            && state
                .previous
                .is_some_and(|previous| !previous.is_whitespace())
            && following_dash_length > 0
            && next_context_char(
                value,
                character_end + following_dash_length,
                protected,
                escaped,
                items,
                item_index,
                next_after,
            )
            .is_none_or(|character| character.is_whitespace() || is_closing_punctuation(character))
        {
            result.push('\u{00a0}');
            state.observe("\u{00a0}");
            offset = character_end;
            changed = true;
            continue;
        }

        if character == '\''
            && state.previous.is_some_and(char::is_alphanumeric)
            && next.is_some_and(char::is_alphanumeric)
        {
            if options.language().converts_apostrophes() {
                result.push('’');
                state.observe("’");
                changed = true;
            } else {
                result.push(character);
                state.observe(&value[offset..character_end]);
            }
            offset = character_end;
            continue;
        }

        if (character == '"' || character == '\'')
            && let Some(replacement) = state.replace_quote(character, next, options.language())
        {
            result.push_str(replacement);
            state.observe(replacement);
            offset = character_end;
            changed = true;
            continue;
        }

        state.observe_authored_quote(character, next, options.language());
        result.push(character);
        state.observe(&value[offset..character_end]);
        offset = character_end;
    }

    changed.then_some(result)
}

fn advance_range_index(ranges: &[Range<usize>], index: &mut usize, offset: usize) {
    while ranges.get(*index).is_some_and(|range| range.end <= offset) {
        *index += 1;
    }
}

fn is_blocked(offset: usize, protected: &[Range<usize>], escaped: &[Range<usize>]) -> bool {
    protected
        .iter()
        .chain(escaped)
        .any(|range| range.start <= offset && offset < range.end)
}

fn next_context_char(
    value: &str,
    offset: usize,
    protected: &[Range<usize>],
    escaped: &[Range<usize>],
    items: &[InlineItem<'_>],
    item_index: usize,
    next_after: Option<char>,
) -> Option<char> {
    if offset < value.len() {
        if is_blocked(offset, protected, escaped) {
            return None;
        }
        return value[offset..].chars().next();
    }

    let mut index = item_index + 1;
    while let Some(item) = items.get(index) {
        match item {
            InlineItem::Text {
                value,
                protected,
                escaped,
            } => match first_context_char(value, protected, escaped) {
                ContextChar::Char(character) => return Some(character),
                ContextChar::Empty => index += 1,
                ContextChar::Boundary => return None,
            },
            InlineItem::Boundary => return None,
            InlineItem::SoftBreak => return Some(' '),
        }
    }
    next_after
}

fn can_open_quote(previous: Option<char>, next: Option<char>) -> bool {
    next.is_some_and(|character| !character.is_whitespace())
        && previous
            .is_none_or(|character| character.is_whitespace() || is_opening_punctuation(character))
}

fn can_close_quote(previous: char) -> bool {
    previous.is_alphanumeric() || is_closing_punctuation(previous)
}

fn is_opening_punctuation(character: char) -> bool {
    matches!(
        character,
        '(' | '[' | '{' | '<' | '«' | '„' | '‚' | '“' | '‘'
    )
}

fn is_closing_punctuation(character: char) -> bool {
    matches!(
        character,
        ')' | ']' | '}' | '>' | ',' | '.' | '!' | '?' | ';' | ':' | '»' | '”' | '’' | '“' | '‘'
    )
}

fn dash_boundary(previous: Option<char>, next: Option<char>) -> bool {
    previous.is_none_or(|character| character.is_whitespace() || is_opening_punctuation(character))
        && next
            .is_none_or(|character| character.is_whitespace() || is_closing_punctuation(character))
}

fn dash_sequence_length(value: &str) -> usize {
    if value.starts_with("---") {
        3
    } else if value.starts_with("--") {
        2
    } else {
        0
    }
}

fn quote_marks(language: TypographyLanguage, primary: bool) -> QuoteMarks {
    match (language, primary) {
        (TypographyLanguage::English, true) => QuoteMarks {
            open: "“",
            close: "”",
        },
        (TypographyLanguage::English, false) => QuoteMarks {
            open: "‘",
            close: "’",
        },
        (TypographyLanguage::Spanish, true) => QuoteMarks {
            open: "«",
            close: "»",
        },
        (TypographyLanguage::Spanish, false) => QuoteMarks {
            open: "“",
            close: "”",
        },
        (TypographyLanguage::French, true) => QuoteMarks {
            open: "«\u{202f}",
            close: "\u{202f}»",
        },
        (TypographyLanguage::French, false) => QuoteMarks {
            open: "“",
            close: "”",
        },
        (TypographyLanguage::Portuguese, true) => QuoteMarks {
            open: "«",
            close: "»",
        },
        (TypographyLanguage::Portuguese, false) => QuoteMarks {
            open: "“",
            close: "”",
        },
        (TypographyLanguage::German, true) => QuoteMarks {
            open: "„",
            close: "“",
        },
        (TypographyLanguage::German, false) => QuoteMarks {
            open: "‚",
            close: "‘",
        },
        (TypographyLanguage::Italian, true) => QuoteMarks {
            open: "«",
            close: "»",
        },
        (TypographyLanguage::Italian, false) => QuoteMarks {
            open: "“",
            close: "”",
        },
        (TypographyLanguage::Dutch, true) => QuoteMarks {
            open: "‘",
            close: "’",
        },
        (TypographyLanguage::Dutch, false) => QuoteMarks {
            open: "“",
            close: "”",
        },
        (TypographyLanguage::Polish, true) => QuoteMarks {
            open: "„",
            close: "”",
        },
        (TypographyLanguage::Polish, false) => QuoteMarks {
            open: "‚",
            close: "‘",
        },
        (TypographyLanguage::Russian, true) => QuoteMarks {
            open: "«",
            close: "»",
        },
        (TypographyLanguage::Russian, false) => QuoteMarks {
            open: "„",
            close: "“",
        },
        (TypographyLanguage::Ukrainian, true) => QuoteMarks {
            open: "«",
            close: "»",
        },
        (TypographyLanguage::Ukrainian, false) => QuoteMarks {
            open: "„",
            close: "“",
        },
    }
}

fn authored_quote_event(character: char, language: TypographyLanguage) -> Option<(bool, bool)> {
    let primary = quote_marks(language, true);
    let secondary = quote_marks(language, false);
    let primary_open = primary.open.chars().next()?;
    let primary_close = primary.close.chars().last()?;
    let secondary_open = secondary.open.chars().next()?;
    let secondary_close = secondary.close.chars().last()?;

    if character == primary_open {
        Some((true, true))
    } else if character == primary_close {
        Some((false, true))
    } else if character == secondary_open {
        Some((true, false))
    } else if character == secondary_close {
        Some((false, false))
    } else {
        None
    }
}

fn is_french_punctuation(character: Option<char>) -> bool {
    matches!(character, Some(';' | ':' | '!' | '?'))
}

fn has_unit_after(
    value: &str,
    offset: usize,
    protected: &[Range<usize>],
    escaped: &[Range<usize>],
) -> bool {
    const UNITS: &[&str] = &["km/h", "°C", "km", "cm", "mm", "kg", "mg", "m", "g", "%"];
    if is_blocked(offset, protected, escaped) {
        return false;
    }
    UNITS.iter().any(|unit| {
        let Some(end) = offset.checked_add(unit.len()) else {
            return false;
        };
        if value
            .get(offset..end)
            .is_none_or(|candidate| candidate != *unit)
            || protected
                .iter()
                .chain(escaped)
                .any(|range| range.start < end && range.end > offset)
        {
            return false;
        }
        value[end..]
            .chars()
            .next()
            .is_none_or(|character| !character.is_alphanumeric())
    })
}

fn escaped_source_ranges(source: &str, span: Span, value: &str) -> Vec<Range<usize>> {
    let Some(raw) = source.get(span.start as usize..span.end as usize) else {
        return Vec::new();
    };
    let mut raw_quotes = Vec::new();
    let mut raw_offset = 0;

    while raw_offset < raw.len() {
        if raw[raw_offset..].starts_with('\\')
            && let Some((character, width)) = next_char_at(raw, raw_offset + 1)
            && matches!(character, '\'' | '"')
            && character.is_ascii_punctuation()
        {
            raw_quotes.push((character, true));
            raw_offset += 1 + width;
            continue;
        }

        if raw[raw_offset..].starts_with('&')
            && let Some((character, end)) = entity_at(raw, raw_offset)
            && matches!(character, '\'' | '"')
        {
            raw_quotes.push((character, true));
            raw_offset = end;
            continue;
        }

        if let Some((character, width)) = next_char_at(raw, raw_offset) {
            if matches!(character, '\'' | '"') {
                raw_quotes.push((character, false));
            }
            raw_offset += width;
        } else {
            break;
        }
    }

    let mut text_quotes = Vec::new();
    for (offset, character) in value.char_indices() {
        if matches!(character, '\'' | '"') {
            text_quotes.push((offset, character));
        }
    }

    if raw_quotes.len() != text_quotes.len()
        || raw_quotes
            .iter()
            .zip(&text_quotes)
            .any(|((raw_character, _), (_, text_character))| raw_character != text_character)
    {
        return Vec::new();
    }

    raw_quotes
        .iter()
        .zip(text_quotes)
        .filter_map(|((_, escaped), (offset, character))| {
            escaped.then_some(offset..offset + character.len_utf8())
        })
        .collect()
}

fn next_char_at(value: &str, offset: usize) -> Option<(char, usize)> {
    let character = value.get(offset..)?.chars().next()?;
    Some((character, character.len_utf8()))
}

fn entity_at(value: &str, start: usize) -> Option<(char, usize)> {
    let semicolon = value[start..].find(';')? + start;
    let name = &value[start + 1..semicolon];
    let character = if let Some(numeric) = name.strip_prefix('#') {
        if let Some(hexadecimal) = numeric
            .strip_prefix('x')
            .or_else(|| numeric.strip_prefix('X'))
        {
            u32::from_str_radix(hexadecimal, 16)
                .ok()
                .and_then(char::from_u32)?
        } else {
            numeric.parse::<u32>().ok().and_then(char::from_u32)?
        }
    } else {
        match name {
            "quot" => '"',
            "apos" => '\'',
            _ => return None,
        }
    };
    Some((character, semicolon + 1))
}

#[cfg(test)]
mod tests {
    use super::{TypographyLanguage, TypographyOptions, UnsupportedTypographyLanguage};

    #[test]
    fn parses_only_the_ten_reviewed_language_codes() {
        for language in ["en", "es", "fr", "pt", "de", "it", "nl", "pl", "ru", "uk"] {
            assert_eq!(
                language
                    .parse::<TypographyLanguage>()
                    .map(TypographyLanguage::as_str),
                Ok(language)
            );
        }
        assert_eq!(
            "en-US".parse::<TypographyLanguage>(),
            Err(UnsupportedTypographyLanguage {
                language: "en-US".to_owned(),
            })
        );
    }

    #[test]
    fn dash_and_ellipsis_toggles_are_independent() {
        let options = TypographyOptions::new(TypographyLanguage::English)
            .with_dashes(false)
            .with_ellipses(true);
        assert!(!options.dashes());
        assert!(options.ellipses());
    }
}
