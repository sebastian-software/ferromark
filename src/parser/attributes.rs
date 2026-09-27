//! Shared ID and class grammar for opt-in Markdown attributes.

use crate::allocator::Vec;

use super::Parser;

impl<'a> Parser<'a> {
    /// Parse a nonempty brace body. Invalid tokens and repeated IDs make the
    /// entire block ordinary text rather than silently discarding metadata.
    pub(super) fn parse_id_classes(
        &self,
        tokens: &'a str,
    ) -> Option<(Option<&'a str>, Vec<'a, &'a str>)> {
        let mut id = None;
        let mut classes = self.allocator.new_vec();
        let mut found = false;
        for token in tokens.split_whitespace() {
            let (name, is_id) = if let Some(name) = token.strip_prefix('#') {
                (name, true)
            } else {
                (token.strip_prefix('.')?, false)
            };
            if name.is_empty()
                || name.chars().any(|ch| {
                    ch.is_control() || matches!(ch, '"' | '\'' | '<' | '>' | '=' | '{' | '}' | '\\')
                })
            {
                return None;
            }
            if is_id {
                if id.replace(name).is_some() {
                    return None;
                }
            } else {
                classes.push(name);
            }
            found = true;
        }
        found.then_some((id, classes))
    }

    /// A brace suffix starts at `end`, directly after an inline image.
    pub(super) fn parse_image_id_classes(
        &self,
        content: &'a str,
        end: usize,
    ) -> (Option<&'a str>, Vec<'a, &'a str>, usize) {
        if !self.options.image_attributes || content.as_bytes().get(end) != Some(&b'{') {
            return (None, self.allocator.new_vec(), end);
        }
        let Some(close) = content[end + 1..].find('}').map(|offset| end + 1 + offset) else {
            return (None, self.allocator.new_vec(), end);
        };
        let Some((id, classes)) = self.parse_id_classes(&content[end + 1..close]) else {
            return (None, self.allocator.new_vec(), end);
        };
        (id, classes, close + 1)
    }
}
