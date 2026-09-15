//! Positional and optional header-derived CSS classes for logical columns.

use std::fmt::Write as _;

use ferromark_ast::{Table, TableCell};
use rustc_hash::FxHashMap;

use super::super::heading::{collect_heading_text_into, slugify_heading_into};
use super::HtmlRenderer;

impl HtmlRenderer {
    pub(in crate::html::renderer) fn write_table_colgroup(&mut self, table: &Table<'_>) {
        if !self.options.table_colgroup {
            return;
        }
        self.write("<colgroup>\n");
        if self.options.table_column_names {
            self.write_named_table_columns(table);
        } else {
            for index in 0..table.align.len() {
                self.write_table_column(index, None);
            }
        }
        self.write("</colgroup>\n");
    }

    fn write_named_table_columns(&mut self, table: &Table<'_>) {
        // Table-local state prevents one table (or a reused renderer) from
        // renaming another table's columns. The positional-only path does not
        // construct a name registry or collect header text.
        let mut names = ColumnNames::default();
        let mut index = 0;
        if let Some(header) = table.children.first() {
            for cell in &header.children {
                if index == table.align.len() {
                    break;
                }
                let name = names.name_for(cell);
                let span = Self::normalized_table_colspan(cell).min(table.align.len() - index);
                for column in index..index + span {
                    self.write_table_column(column, name);
                }
                index += span;
            }
        }
        for column in index..table.align.len() {
            self.write_table_column(column, None);
        }
    }

    fn write_table_column(&mut self, index: usize, name: Option<&str>) {
        self.write("<col class=\"col-");
        self.write_display(index + 1);
        if let Some(name) = name {
            // A separate namespace keeps numeric headers from colliding with
            // positional classes, e.g. first-column text "2" and `col-2`.
            self.write(" col-name-");
            self.write_attribute_escaped(name);
        }
        self.write(if self.options.xhtml {
            "\" />\n"
        } else {
            "\">\n"
        });
    }
}

#[derive(Default)]
struct ColumnNames {
    text: String,
    base: String,
    name: String,
    // Every emitted name is reserved, including generated suffixes. Each base
    // remembers the next suffix to try so repeated headers do not rescan all
    // earlier duplicates. Natural "Price 1" and generated "Price" duplicates
    // cannot accidentally select the same logical column.
    next_suffix: FxHashMap<String, usize>,
}

impl ColumnNames {
    fn name_for(&mut self, cell: &TableCell<'_>) -> Option<&str> {
        self.text.clear();
        collect_heading_text_into(&cell.children, &mut self.text);
        if !self.text.chars().any(char::is_alphanumeric) {
            return None;
        }
        self.base.clear();
        slugify_heading_into(&self.text, &mut self.base);
        self.name.clear();
        self.name.push_str(&self.base);
        if let Some(&next) = self.next_suffix.get(&self.base) {
            let mut suffix = next;
            loop {
                self.name.truncate(self.base.len());
                let _ = write!(self.name, "-{suffix}");
                suffix += 1;
                if !self.next_suffix.contains_key(&self.name) {
                    break;
                }
            }
            self.next_suffix.insert(self.base.clone(), suffix);
        }
        self.next_suffix.insert(self.name.clone(), 1);
        Some(&self.name)
    }
}
