//! Static analysis of MDX module blocks.
//!
//! Oxc parses a block. This module reduces the syntax tree to the names a
//! block binds and exports, and to the text edits that turn an authored
//! default export into the layout binding. No JavaScript AST leaves it.

use oxc_ast::ast::{
    ExportDefaultDeclarationKind, ExportSpecifier, ModuleExportName, Statement, StringLiteral,
};
use oxc_ecmascript::BoundNames;
use oxc_span::GetSpan;

use super::{LAYOUT, LAYOUT_DECLARATION};

/// A name bound at the top level of a module block.
pub(super) struct Binding {
    pub name: String,
    /// Byte range inside the block.
    pub start: usize,
    pub end: usize,
}

/// Replaces a byte range of a block. The ranges of one block do not overlap.
pub(super) struct Edit {
    pub start: usize,
    pub end: usize,
    pub replacement: &'static str,
}

/// How an authored default export becomes the layout binding.
pub(super) enum Layout {
    /// The block binds this name; assembly adds `const MDXLayout = name;`.
    Local(String),
    /// An edit already turned the export into `const MDXLayout = …`.
    Inline,
    /// A re-export from another module; assembly adds this import statement.
    Import(String),
}

/// An authored default export and the start of its statement in the block.
pub(super) struct DefaultExport {
    pub layout: Layout,
    pub start: usize,
}

#[derive(Default)]
pub(super) struct EsmBlock {
    pub bindings: Vec<Binding>,
    pub exports: Vec<String>,
    pub edits: Vec<Edit>,
    pub defaults: Vec<DefaultExport>,
}

/// Parses one module block. The error is Oxc's first diagnostic message.
pub(super) fn analyze(value: &str) -> Result<EsmBlock, String> {
    let allocator = oxc_allocator::Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, value, oxc_span::SourceType::jsx()).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return Err(parsed.diagnostics.first().map_or_else(
            || "invalid import/export declaration".to_owned(),
            |error| error.message.as_ref().to_owned(),
        ));
    }
    let mut block = EsmBlock::default();
    for statement in &parsed.program.body {
        analyze_statement(statement, value, &mut block);
    }
    block.edits.sort_by_key(|edit| edit.start);
    Ok(block)
}

fn analyze_statement(statement: &Statement<'_>, value: &str, block: &mut EsmBlock) {
    match statement {
        Statement::ImportDeclaration(declaration) => {
            declaration.bound_names(&mut |identifier| {
                block
                    .bindings
                    .push(binding(&identifier.name, identifier.span));
            });
        }
        Statement::ExportDeclaration(declaration) => {
            declaration.bound_names(&mut |identifier| {
                block
                    .bindings
                    .push(binding(&identifier.name, identifier.span));
                push_unique(&mut block.exports, &identifier.name);
            });
        }
        Statement::ExportNamedDeclaration(declaration) => {
            let statement_span = declaration.span;
            for (index, specifier) in declaration.specifiers.iter().enumerate() {
                if is_default(&specifier.exported) {
                    block.defaults.push(DefaultExport {
                        layout: Layout::Local(specifier.local.name().as_str().to_owned()),
                        start: statement_span.start as usize,
                    });
                    block.edits.push(remove_specifier(
                        &declaration.specifiers,
                        index,
                        statement_span,
                    ));
                } else {
                    push_unique(&mut block.exports, specifier.exported.name().as_str());
                }
            }
        }
        Statement::ExportFromDeclaration(declaration) => {
            let statement_span = declaration.span;
            for (index, specifier) in declaration.specifiers.iter().enumerate() {
                if is_default(&specifier.exported) {
                    block.defaults.push(DefaultExport {
                        layout: Layout::Import(import_layout(
                            &specifier.local,
                            &declaration.source,
                            value,
                        )),
                        start: statement_span.start as usize,
                    });
                    block.edits.push(remove_specifier(
                        &declaration.specifiers,
                        index,
                        statement_span,
                    ));
                } else {
                    push_unique(&mut block.exports, specifier.exported.name().as_str());
                }
            }
        }
        Statement::ExportAllDeclaration(declaration) => {
            let Some(exported) = &declaration.exported else {
                return;
            };
            if is_default(exported) {
                let mut import = String::from("import * as ");
                import.push_str(LAYOUT);
                import.push_str(" from ");
                import.push_str(raw(&declaration.source, value));
                import.push(';');
                block.defaults.push(DefaultExport {
                    layout: Layout::Import(import),
                    start: declaration.span.start as usize,
                });
                block.edits.push(Edit {
                    start: declaration.span.start as usize,
                    end: declaration.span.end as usize,
                    replacement: "",
                });
            } else {
                push_unique(&mut block.exports, exported.name().as_str());
            }
        }
        Statement::ExportDefaultDeclaration(declaration) => {
            let start = declaration.span.start as usize;
            let named = match &declaration.declaration {
                ExportDefaultDeclarationKind::FunctionDeclaration(function) => function.id.as_ref(),
                ExportDefaultDeclarationKind::ClassDeclaration(class) => class.id.as_ref(),
                _ => None,
            };
            let end = declaration.declaration.span().start as usize;
            if let Some(identifier) = named {
                // The declaration keeps its name as a module binding, which an
                // expression would not: `<Layout />` stays usable in the body.
                block
                    .bindings
                    .push(binding(&identifier.name, identifier.span));
                block.edits.push(Edit {
                    start,
                    end,
                    replacement: "",
                });
                block.defaults.push(DefaultExport {
                    layout: Layout::Local(identifier.name.as_str().to_owned()),
                    start,
                });
            } else {
                block.edits.push(Edit {
                    start,
                    end,
                    replacement: LAYOUT_DECLARATION,
                });
                block.defaults.push(DefaultExport {
                    layout: Layout::Inline,
                    start,
                });
            }
        }
        _ => {}
    }
}

fn binding(name: &str, span: oxc_span::Span) -> Binding {
    Binding {
        name: name.to_owned(),
        start: span.start as usize,
        end: span.end as usize,
    }
}

fn push_unique(names: &mut Vec<String>, name: &str) {
    if !names.iter().any(|existing| existing == name) {
        names.push(name.to_owned());
    }
}

fn is_default(name: &ModuleExportName<'_>) -> bool {
    name.name().as_str() == "default"
}

/// The source text of a string literal, with its authored quotes.
fn raw<'a>(literal: &StringLiteral<'_>, value: &'a str) -> &'a str {
    value
        .get(literal.span.start as usize..literal.span.end as usize)
        .unwrap_or("\"\"")
}

fn import_layout(local: &ModuleExportName<'_>, source: &StringLiteral<'_>, value: &str) -> String {
    let mut import = String::from("import ");
    if is_default(local) {
        import.push_str(LAYOUT);
    } else {
        import.push_str("{ ");
        import.push_str(raw_name(local, value));
        import.push_str(" as ");
        import.push_str(LAYOUT);
        import.push_str(" }");
    }
    import.push_str(" from ");
    import.push_str(raw(source, value));
    import.push(';');
    import
}

/// The source text of an export name, which keeps the quotes of a string name.
fn raw_name<'a>(name: &ModuleExportName<'_>, value: &'a str) -> &'a str {
    let span = name.span();
    value
        .get(span.start as usize..span.end as usize)
        .unwrap_or("default")
}

/// Removes one specifier together with the comma that separates it from a
/// neighbor. A statement with no other specifier is removed as a whole.
fn remove_specifier(
    specifiers: &[ExportSpecifier<'_>],
    index: usize,
    statement: oxc_span::Span,
) -> Edit {
    let (start, end) = if specifiers.len() == 1 {
        (statement.start, statement.end)
    } else if index + 1 < specifiers.len() {
        (
            specifiers[index].span.start,
            specifiers[index + 1].span.start,
        )
    } else {
        (specifiers[index - 1].span.end, specifiers[index].span.end)
    };
    Edit {
        start: start as usize,
        end: end as usize,
        replacement: "",
    }
}
