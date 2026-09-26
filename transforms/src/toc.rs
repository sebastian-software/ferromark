//! Caller-placed table-of-contents nodes built from a Ferromark outline.

use std::error::Error;
use std::fmt;

use ferromark::ast::{Link, List, ListItem, Node, Paragraph, Span, Text};
use ferromark::{Allocator, OutlineEntry};

/// Builds a nested unordered-list AST node from one document outline.
///
/// The returned node is allocated in `allocator`; every generated AST span is
/// [`Span::empty`]. The caller chooses where to insert the node and should
/// build it from the same document and heading-ID settings used by the
/// renderer. Transforms should finish before the outline is computed and this
/// node is added. The result has no shared state across committed or
/// provisional fragments.
///
/// An empty outline returns `Ok(None)`. If any entry has no heading ID, this
/// returns an error before allocating AST nodes. The helper does not parse a
/// marker, mutate a document, or render HTML.
pub fn build_table_of_contents<'arena>(
    allocator: &'arena Allocator,
    entries: &[OutlineEntry],
) -> Result<Option<Node<'arena>>, TableOfContentsError> {
    if entries.is_empty() {
        return Ok(None);
    }

    let ids = entries
        .iter()
        .enumerate()
        .map(|(entry_index, entry)| {
            entry
                .id
                .as_deref()
                .ok_or(TableOfContentsError { entry_index })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let items = build_items(entries);
    let list = build_list(allocator, entries, &ids, &items);
    Ok(Some(Node::List(allocator.boxed(list))))
}

/// Error returned when an outline entry cannot become a TOC link.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableOfContentsError {
    entry_index: usize,
}

impl TableOfContentsError {
    /// Returns the zero-based outline entry without a heading ID.
    #[must_use]
    pub const fn entry_index(&self) -> usize {
        self.entry_index
    }
}

impl fmt::Display for TableOfContentsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "outline entry {} has no heading ID; enable heading IDs when building the outline",
            self.entry_index
        )
    }
}

impl Error for TableOfContentsError {}

struct TocItem {
    entry_index: usize,
    children: Vec<TocItem>,
}

fn build_items(entries: &[OutlineEntry]) -> Vec<TocItem> {
    let mut roots = Vec::new();
    let mut path = Vec::<(u8, usize)>::new();

    for (entry_index, entry) in entries.iter().enumerate() {
        while path.last().is_some_and(|(level, _)| *level >= entry.level) {
            path.pop();
        }

        let siblings = children_at_path(&mut roots, &path);
        let item_index = siblings.len();
        siblings.push(TocItem {
            entry_index,
            children: Vec::new(),
        });
        path.push((entry.level, item_index));
    }

    roots
}

fn children_at_path<'items>(
    items: &'items mut Vec<TocItem>,
    path: &[(u8, usize)],
) -> &'items mut Vec<TocItem> {
    let Some(((_, item_index), remaining)) = path.split_first() else {
        return items;
    };
    children_at_path(&mut items[*item_index].children, remaining)
}

fn build_list<'arena>(
    allocator: &'arena Allocator,
    entries: &[OutlineEntry],
    ids: &[&str],
    items: &[TocItem],
) -> List<'arena> {
    let mut children = allocator.new_vec_with_capacity(items.len());

    for item in items {
        let entry = &entries[item.entry_index];
        let id = ids[item.entry_index];

        let mut url = String::from("#");
        url.push_str(id);
        let url = allocator.alloc_str(&url);

        let mut link_children = allocator.new_vec_with_capacity(1);
        link_children.push(Node::Text(Text {
            value: allocator.alloc_str(&entry.text),
            span: Span::empty(),
        }));
        let link = Node::Link(allocator.boxed(Link {
            url,
            title: None,
            children: link_children,
            span: Span::empty(),
        }));

        let mut paragraph_children = allocator.new_vec_with_capacity(1);
        paragraph_children.push(link);
        let paragraph = Node::Paragraph(allocator.boxed(Paragraph {
            children: paragraph_children,
            span: Span::empty(),
        }));

        let nested = !item.children.is_empty();
        let mut item_children = allocator.new_vec_with_capacity(if nested { 2 } else { 1 });
        item_children.push(paragraph);
        if nested {
            let nested_list = build_list(allocator, entries, ids, &item.children);
            item_children.push(Node::List(allocator.boxed(nested_list)));
        }

        children.push(ListItem {
            spread: false,
            checked: None,
            children: item_children,
            span: Span::empty(),
        });
    }

    List {
        ordered: false,
        start: None,
        spread: false,
        children,
        span: Span::empty(),
    }
}

#[cfg(test)]
mod tests {
    use ferromark::ast::Span;

    use super::{TableOfContentsError, build_items};
    use ferromark::OutlineEntry;

    fn entry(level: u8) -> OutlineEntry {
        OutlineEntry {
            level,
            text: String::new(),
            id: Some(String::new()),
            span: Span::empty(),
        }
    }

    #[test]
    fn skipped_levels_nest_under_the_nearest_shallower_heading() {
        let entries = [entry(1), entry(4), entry(2), entry(2), entry(1)];
        let items = build_items(&entries);

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].children.len(), 3);
        assert_eq!(items[0].children[0].children.len(), 0);
        assert_eq!(items[0].children[1].children.len(), 0);
        assert_eq!(items[0].children[2].children.len(), 0);
        assert_eq!(items[1].children.len(), 0);
    }

    #[test]
    fn missing_heading_id_error_reports_the_entry_index() {
        let error = TableOfContentsError { entry_index: 3 };
        assert_eq!(error.entry_index(), 3);
    }
}
