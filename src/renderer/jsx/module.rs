//! Assembles an MDX module around a rendered JSX body.
//!
//! The module follows the MDX contract: authored ESM in document order, a
//! content function, and `MDXContent`. It still contains JSX and imports no
//! framework; the caller's JSX transform compiles it.

mod esm;
mod source_map;

use thiserror::Error;

use crate::ast::Span;
use crate::outline::OutlineEntry;

use self::esm::{DefaultExport, EsmBlock, Layout};
use super::line_index::{LineIndex, to_u32};
use super::{JsxCodeBlock, JsxModuleSource, JsxOutput, JsxSourceMapping};

/// Object that maps element and component names to implementations. Generated
/// Markdown elements are members of it, which is why module output fixes the
/// renderer's component prefix to this name.
pub(super) const COMPONENTS: &str = "_components";
const CONTENT: &str = "_createMdxContent";
const MISSING: &str = "_missingMdxReference";
const PROVIDER: &str = "_provideComponents";
const LAYOUT: &str = "MDXLayout";
/// Replaces `export default` in front of an expression.
const LAYOUT_DECLARATION: &str = "const MDXLayout = ";
const CONTENT_EXPORT: &str = "MDXContent";

/// Names the generated module declares. An authored declaration of one of
/// them is an error, which is what makes a naming pre-pass unnecessary.
const RESERVED: [&str; 6] = [
    COMPONENTS,
    CONTENT,
    MISSING,
    PROVIDER,
    LAYOUT,
    CONTENT_EXPORT,
];

/// Options for MDX module output.
#[derive(Debug, Clone)]
pub struct JsxModuleOptions {
    /// Module that exports `useMDXComponents`. When set, the module imports it
    /// and merges its components below `props.components`.
    pub provider_import_source: Option<String>,

    /// Export `MDXContent` as the default export.
    ///
    /// Set this to `false` to keep `MDXContent` a local binding, so the caller
    /// can append its own default export.
    ///
    /// Default: `true`.
    pub default_export: bool,

    /// Names the caller declares in code it adds to the module.
    ///
    /// A component reference to one of them uses that binding instead of the
    /// provider, and an authored declaration of the same name is an error.
    pub reserved_bindings: Vec<String>,

    /// Name of the source file, recorded as the source of the source map.
    pub filename: Option<String>,
}

impl Default for JsxModuleOptions {
    fn default() -> Self {
        Self {
            provider_import_source: None,
            default_export: true,
            reserved_bindings: Vec::new(),
            filename: None,
        }
    }
}

/// A version 3 source map from module code to the Markdown/MDX source.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JsxSourceMap {
    /// The one source file: [`JsxModuleOptions::filename`], or an empty name.
    pub sources: Vec<String>,
    /// The source text, parallel to `sources`.
    pub sources_content: Vec<String>,
    /// Base64 VLQ mappings with zero-based lines and UTF-16 columns.
    pub mappings: String,
}

impl JsxSourceMap {
    /// Serializes the map as source map JSON.
    #[must_use]
    pub fn to_json(&self) -> String {
        let mut json = String::with_capacity(
            self.mappings.len() + self.sources_content.iter().map(String::len).sum::<usize>() + 96,
        );
        json.push_str("{\"version\":3,\"sources\":");
        push_json_strings(&mut json, &self.sources);
        json.push_str(",\"sourcesContent\":");
        push_json_strings(&mut json, &self.sources_content);
        json.push_str(",\"names\":[],\"mappings\":");
        push_json_string(&mut json, &self.mappings);
        json.push('}');
        json
    }
}

/// Appends a JSON string, which is also a JavaScript string literal.
fn push_json_string(output: &mut String, value: &str) {
    match serde_json::to_string(value) {
        Ok(literal) => output.push_str(&literal),
        Err(_) => output.push_str("\"\""),
    }
}

fn push_json_strings(output: &mut String, values: &[String]) {
    output.push('[');
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        push_json_string(output, value);
    }
    output.push(']');
}

/// Structured output from MDX module rendering.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JsxModuleOutput {
    /// A complete ES module that still contains JSX.
    pub code: String,
    /// Source map for `code`.
    pub map: JsxSourceMap,
    /// Names the authored ESM exports, in document order. An authored default
    /// export becomes the layout and is not listed.
    pub exports: Vec<String>,
    /// Names the authored ESM binds at the top level, in document order:
    /// imports, exported declarations, and a named default export.
    pub bindings: Vec<String>,
    /// Headings emitted in the body; see [`JsxOutput::headings`].
    pub headings: Vec<OutlineEntry>,
    /// Source span of the omitted title heading, when configured.
    pub omitted_title_heading: Option<Span>,
    /// Code blocks encountered in source order.
    pub code_blocks: Vec<JsxCodeBlock>,
}

/// Why a document cannot become an MDX module.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum JsxModuleError {
    /// Module output owns the component prefix.
    #[error("module output sets the component prefix to `_components`; leave it unset")]
    ComponentPrefix,
    /// A module block is not valid JavaScript.
    #[error("invalid MDX module block at {span:?}: {message}")]
    InvalidEsm {
        /// Source range of the block.
        span: Span,
        /// The JavaScript parser's diagnostic.
        message: String,
    },
    /// An authored declaration uses a name the module needs.
    #[error("`{name}` at {span:?} is reserved in MDX module output; rename the declaration")]
    ReservedBinding {
        /// The declared name.
        name: String,
        /// Source range of the declared name.
        span: Span,
    },
    /// The document has more than one default export.
    #[error("an MDX module can have only one default export; found another at {span:?}")]
    DuplicateDefaultExport {
        /// Source range of the block with the second default export.
        span: Span,
    },
}

pub(super) fn assemble(
    output: JsxOutput,
    references: &[String],
    source: &str,
    options: &JsxModuleOptions,
) -> Result<JsxModuleOutput, JsxModuleError> {
    let blocks = analyze_blocks(&output.esm, source)?;
    let mut bindings = Vec::new();
    let mut exports = Vec::new();
    let mut layout = None;
    for (block, offset) in &blocks {
        for binding in &block.bindings {
            let reserved = RESERVED.contains(&binding.name.as_str())
                || options.reserved_bindings.contains(&binding.name);
            if reserved {
                return Err(JsxModuleError::ReservedBinding {
                    name: binding.name.clone(),
                    span: Span::new(to_u32(offset + binding.start), to_u32(offset + binding.end)),
                });
            }
            push_unique(&mut bindings, &binding.name);
        }
        for name in &block.exports {
            push_unique(&mut exports, name);
        }
        for DefaultExport {
            layout: found,
            start,
        } in &block.defaults
        {
            if layout.is_some() {
                let start = to_u32(offset + start);
                return Err(JsxModuleError::DuplicateDefaultExport {
                    span: Span::new(start, start),
                });
            }
            layout = Some(found);
        }
    }

    let provider = options.provider_import_source.as_deref();
    let scope = Scope {
        bindings: &bindings,
        reserved: &options.reserved_bindings,
    };
    let references = resolve_references(references, &scope);

    let mut writer = Writer::new(source, output.body.len() + source.len() / 2 + 1024);
    if let Some(provider) = provider {
        writer.push("import {useMDXComponents as ");
        writer.push(PROVIDER);
        writer.push("} from ");
        writer.push_js_string(provider);
        writer.push(";\n");
    }
    for ((block, offset), esm) in blocks.iter().zip(&output.esm) {
        writer.push_block(&esm.value, block, *offset);
    }
    match layout {
        Some(Layout::Local(name)) => {
            writer.push("const ");
            writer.push(LAYOUT);
            writer.push(" = ");
            writer.push(name);
            writer.push(";\n");
        }
        Some(Layout::Import(import)) => {
            writer.push(import);
            writer.push("\n");
        }
        Some(Layout::Inline) | None => {}
    }
    write_content_function(&mut writer, &output, &references, provider.is_some());
    write_content_export(&mut writer, layout.is_some(), provider.is_some(), options);
    if !references.checks.is_empty() {
        writer.push("function ");
        writer.push(MISSING);
        writer.push(
            "(id, component) {\n  throw new Error(\"Expected \" + (component ? \"component\" : \"object\") + \" `\" + id + \"` to be defined: import it, pass it in `components`, or provide it.\");\n}\n",
        );
    }

    Ok(JsxModuleOutput {
        code: writer.code,
        map: JsxSourceMap {
            sources: [options.filename.clone().unwrap_or_default()].into(),
            sources_content: [source.to_owned()].into(),
            mappings: source_map::encode(&writer.mappings),
        },
        exports,
        bindings,
        headings: output.headings,
        omitted_title_heading: output.omitted_title_heading,
        code_blocks: output.code_blocks,
    })
}

/// Parses every module block and locates its text in the source.
fn analyze_blocks(
    blocks: &[JsxModuleSource],
    source: &str,
) -> Result<Vec<(EsmBlock, usize)>, JsxModuleError> {
    blocks
        .iter()
        .map(|block| {
            let analyzed =
                esm::analyze(&block.value).map_err(|message| JsxModuleError::InvalidEsm {
                    span: block.span,
                    message,
                })?;
            // A strict MDX parse reports the exact range of the block. The
            // permissive parse includes surrounding whitespace in the span.
            let start = block.span.start as usize;
            let offset = source
                .get(start..block.span.end as usize)
                .and_then(|text| text.find(block.value.as_str()))
                .map_or(start, |relative| start + relative);
            Ok((analyzed, offset))
        })
        .collect()
}

struct Scope<'a> {
    bindings: &'a [String],
    reserved: &'a [String],
}

impl Scope<'_> {
    fn has(&self, name: &str) -> bool {
        name == "props"
            || name == "this"
            || self.bindings.iter().any(|binding| binding == name)
            || self.reserved.iter().any(|binding| binding == name)
    }
}

#[derive(Default)]
struct References<'a> {
    /// Root identifiers the content function takes from the components object.
    provided: Vec<&'a str>,
    /// Member paths that must be defined, and whether each is a component
    /// rather than an object that holds one.
    checks: Vec<(&'a str, bool)>,
}

/// Decides which component references come from the components object.
///
/// A reference whose root identifier is bound in the module, by the author or
/// by the caller, uses that binding and needs no check.
fn resolve_references<'a>(references: &'a [String], scope: &Scope<'_>) -> References<'a> {
    let mut resolved = References::default();
    for reference in references {
        let root = reference.split('.').next().unwrap_or(reference);
        if scope.has(root) {
            continue;
        }
        if root != COMPONENTS && !resolved.provided.contains(&root) {
            resolved.provided.push(root);
        }
        // `a.b.C` needs `a` and `a.b` as objects and `a.b.C` as a component.
        let mut end = root.len();
        loop {
            let path = &reference[..end];
            let component = end == reference.len();
            if path != COMPONENTS && !resolved.checks.iter().any(|(known, _)| *known == path) {
                resolved.checks.push((path, component));
            }
            if component {
                break;
            }
            end = reference[end + 1..]
                .find('.')
                .map_or(reference.len(), |next| end + 1 + next);
        }
    }
    resolved
}

fn write_content_function(
    writer: &mut Writer<'_>,
    output: &JsxOutput,
    references: &References<'_>,
    provider: bool,
) {
    writer.push("function ");
    writer.push(CONTENT);
    writer.push("(props) {\n  const ");
    writer.push(COMPONENTS);
    writer.push(" = {\n");
    for element in &output.elements {
        writer.push("    ");
        writer.push_js_string(element);
        writer.push(": ");
        writer.push_js_string(element);
        writer.push(",\n");
    }
    if provider {
        writer.push("    ...");
        writer.push(PROVIDER);
        writer.push("(),\n");
    }
    writer.push("    ...props.components,\n  };\n");
    if !references.provided.is_empty() {
        writer.push("  const {");
        for (index, name) in references.provided.iter().enumerate() {
            if index > 0 {
                writer.push(", ");
            }
            writer.push(name);
        }
        writer.push("} = ");
        writer.push(COMPONENTS);
        writer.push(";\n");
    }
    for (path, component) in &references.checks {
        writer.push("  if (!");
        writer.push(path);
        writer.push(") ");
        writer.push(MISSING);
        writer.push("(");
        // A member of the components object is reported by its own name.
        let id = path
            .strip_prefix(COMPONENTS)
            .and_then(|rest| rest.strip_prefix('.'))
            .unwrap_or(path);
        writer.push_js_string(id);
        writer.push(if *component {
            ", true);\n"
        } else {
            ", false);\n"
        });
    }
    writer.push("  return ");
    writer.push_body(&output.body, &output.mappings);
    writer.push(";\n}\n");
}

fn write_content_export(
    writer: &mut Writer<'_>,
    authored_layout: bool,
    provider: bool,
    options: &JsxModuleOptions,
) {
    if options.default_export {
        writer.push("export default ");
    }
    writer.push("function ");
    writer.push(CONTENT_EXPORT);
    writer.push("(props = {}) {\n");
    if authored_layout {
        writer.push("  return <");
        writer.push(LAYOUT);
        writer.push(" {...props}><");
        writer.push(CONTENT);
        writer.push(" {...props} /></");
        writer.push(LAYOUT);
        writer.push(">;\n}\n");
        return;
    }
    // Without an authored layout, the `wrapper` component is the layout.
    writer.push("  const {wrapper: ");
    writer.push(LAYOUT);
    if provider {
        writer.push("} = {\n    ...");
        writer.push(PROVIDER);
        writer.push("(),\n    ...props.components,\n  };\n");
    } else {
        writer.push("} = props.components || {};\n");
    }
    writer.push("  return ");
    writer.push(LAYOUT);
    writer.push("\n    ? <");
    writer.push(LAYOUT);
    writer.push(" {...props}><");
    writer.push(CONTENT);
    writer.push(" {...props} /></");
    writer.push(LAYOUT);
    writer.push(">\n    : ");
    writer.push(CONTENT);
    writer.push("(props);\n}\n");
}

fn push_unique(names: &mut Vec<String>, name: &str) {
    if !names.iter().any(|existing| existing == name) {
        names.push(name.to_owned());
    }
}

/// A zero-based line and UTF-16 column, advanced one character at a time.
#[derive(Clone, Copy, Default)]
struct Position {
    line: u32,
    column: u32,
    previous_cr: bool,
}

impl Position {
    fn advance(&mut self, ch: char) {
        match ch {
            '\r' => {
                self.line = self.line.saturating_add(1);
                self.column = 0;
                self.previous_cr = true;
            }
            '\n' if self.previous_cr => self.previous_cr = false,
            '\n' => {
                self.line = self.line.saturating_add(1);
                self.column = 0;
            }
            _ => {
                self.column = self.column.saturating_add(to_u32(ch.len_utf16()));
                self.previous_cr = false;
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CharClass {
    Space,
    Word,
    Punctuation,
}

impl CharClass {
    fn of(ch: char) -> Self {
        if ch.is_whitespace() {
            Self::Space
        } else if ch.is_alphanumeric() || matches!(ch, '_' | '$') || !ch.is_ascii() {
            Self::Word
        } else {
            Self::Punctuation
        }
    }
}

/// Module code under construction, with the position of its end and the
/// mappings collected so far in generated order.
struct Writer<'a> {
    code: String,
    position: Position,
    mappings: Vec<JsxSourceMapping>,
    index: LineIndex<'a>,
}

impl<'a> Writer<'a> {
    fn new(source: &'a str, capacity: usize) -> Self {
        Self {
            code: String::with_capacity(capacity),
            position: Position::default(),
            mappings: Vec::new(),
            index: LineIndex::new(source),
        }
    }

    /// Appends generated code that has no source position.
    fn push(&mut self, text: &str) {
        self.code.push_str(text);
        for ch in text.chars() {
            self.position.advance(ch);
        }
    }

    fn push_js_string(&mut self, value: &str) {
        let mut literal = String::with_capacity(value.len() + 2);
        push_json_string(&mut literal, value);
        self.push(&literal);
    }

    fn add_mapping(&mut self, generated: Position, source: Position) {
        let mapping = JsxSourceMapping {
            generated_line: generated.line,
            generated_column: generated.column,
            source_line: source.line,
            source_column: source.column,
        };
        // A later mapping for the same generated position is the more specific one.
        match self.mappings.last_mut() {
            Some(last)
                if last.generated_line == mapping.generated_line
                    && last.generated_column == mapping.generated_column =>
            {
                *last = mapping;
            }
            _ => self.mappings.push(mapping),
        }
    }

    fn source_position(&self, offset: usize) -> Position {
        let (line, column) = self.index.position(offset);
        Position {
            line,
            column,
            previous_cr: false,
        }
    }

    /// Appends generated code that replaces the source text at `offset`.
    fn push_replacement(&mut self, text: &str, offset: usize) {
        let source = self.source_position(offset);
        self.add_mapping(self.position, source);
        self.push(text);
    }

    /// Appends text copied from the source at `offset`. Each line and each
    /// word or punctuation run gets a mapping, so a position inside authored
    /// JavaScript resolves to its own column.
    fn push_source(&mut self, text: &str, offset: usize) {
        self.code.push_str(text);
        let mut source = self.source_position(offset);
        let mut previous = CharClass::Space;
        for (index, ch) in text.char_indices() {
            let class = CharClass::of(ch);
            if index == 0 || (class != previous && class != CharClass::Space) {
                self.add_mapping(self.position, source);
            }
            previous = class;
            self.position.advance(ch);
            source.advance(ch);
        }
    }

    /// Appends one module block with its default export rewritten.
    fn push_block(&mut self, value: &str, block: &EsmBlock, offset: usize) {
        let mut cursor = 0;
        for edit in &block.edits {
            if let Some(kept) = value.get(cursor..edit.start) {
                self.push_source(kept, offset + cursor);
            }
            if !edit.replacement.is_empty() {
                self.push_replacement(edit.replacement, offset + edit.start);
            }
            cursor = edit.end;
        }
        if let Some(rest) = value.get(cursor..) {
            self.push_source(rest, offset + cursor);
        }
        self.push("\n");
    }

    /// Appends the rendered body and moves its mappings to their position in
    /// the module.
    fn push_body(&mut self, body: &str, mappings: &[JsxSourceMapping]) {
        let start = self.position;
        for mapping in mappings {
            let generated = Position {
                line: start.line.saturating_add(mapping.generated_line),
                column: if mapping.generated_line == 0 {
                    start.column.saturating_add(mapping.generated_column)
                } else {
                    mapping.generated_column
                },
                previous_cr: false,
            };
            let source = Position {
                line: mapping.source_line,
                column: mapping.source_column,
                previous_cr: false,
            };
            self.add_mapping(generated, source);
        }
        self.push(body);
    }
}
