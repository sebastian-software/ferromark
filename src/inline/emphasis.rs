//! Emphasis and strong emphasis resolution.
//!
//! Uses the "modulo-3" stack optimization from md4c to efficiently
//! match emphasis openers and closers according to CommonMark rules.

use super::marks::{flags, Mark};

/// Result of emphasis resolution for a mark pair.
#[derive(Debug, Clone, Copy)]
pub struct EmphasisMatch {
    /// Start position of opener delimiter(s).
    pub opener_start: u32,
    /// End position of opener delimiter(s).
    pub opener_end: u32,
    /// Start position of closer delimiter(s).
    pub closer_start: u32,
    /// End position of closer delimiter(s).
    pub closer_end: u32,
    /// Number of characters matched (1 for emphasis, 2 for strong).
    pub count: u32,
}

/// Resolve emphasis marks using modulo-3 stacks.
/// Returns a list of matched pairs.
pub fn resolve_emphasis(marks: &mut [Mark]) -> Vec<EmphasisMatch> {
    let mut matches = Vec::new();
    let mut resolver = EmphasisResolver::new();

    // Process marks left to right
    for i in 0..marks.len() {
        let mark = &marks[i];

        // Skip non-emphasis marks or those inside code spans
        if (mark.ch != b'*' && mark.ch != b'_') || mark.flags & flags::IN_CODE != 0 {
            continue;
        }

        if mark.can_close() {
            // Try to find a matching opener
            if let Some((opener_idx, match_count)) = resolver.find_opener(marks, i) {
                // Record positions BEFORE modifying marks
                let opener = &marks[opener_idx];
                let closer = &marks[i];

                // Opener delimiter is at the END of the opener mark (rightmost chars)
                let opener_delim_start = opener.end - match_count;
                let opener_delim_end = opener.end;

                // Closer delimiter is at the START of the closer mark (leftmost chars)
                let closer_delim_start = closer.pos;
                let closer_delim_end = closer.pos + match_count;

                matches.push(EmphasisMatch {
                    opener_start: opener_delim_start,
                    opener_end: opener_delim_end,
                    closer_start: closer_delim_start,
                    closer_end: closer_delim_end,
                    count: match_count,
                });

                // Consume characters from both marks
                let opener = &mut marks[opener_idx];
                let opener_remaining = opener.len() - match_count;
                if opener_remaining == 0 {
                    opener.resolve();
                } else {
                    // Shrink opener from the right
                    opener.end -= match_count;
                }

                let closer = &mut marks[i];
                let closer_remaining = closer.len() - match_count;
                if closer_remaining == 0 {
                    closer.resolve();
                } else {
                    // Shrink closer from the left
                    closer.pos += match_count;
                }

                // If closer still has characters and can open, push it
                if closer_remaining > 0 && closer.can_open() {
                    resolver.push_opener(marks, i);
                }
            } else if mark.can_open() {
                // Can't close but can open - push to stacks
                resolver.push_opener(marks, i);
            }
        } else if mark.can_open() {
            resolver.push_opener(marks, i);
        }
    }

    matches
}

/// Emphasis resolver with 6 stacks (2 chars x 3 modulo classes).
struct EmphasisResolver {
    /// Stacks indexed by: (is_underscore ? 3 : 0) + (run_length % 3)
    stacks: [Vec<usize>; 6],
}

impl EmphasisResolver {
    fn new() -> Self {
        Self {
            stacks: Default::default(),
        }
    }

    /// Get stack index for a mark.
    fn stack_index(ch: u8, run_len: u32) -> usize {
        let char_offset = if ch == b'_' { 3 } else { 0 };
        char_offset + (run_len as usize % 3)
    }

    /// Push an opener to the appropriate stack.
    fn push_opener(&mut self, marks: &[Mark], idx: usize) {
        let mark = &marks[idx];
        let stack_idx = Self::stack_index(mark.ch, mark.len());
        self.stacks[stack_idx].push(idx);
    }

    /// Find a matching opener for a closer.
    /// Returns (opener_index, match_count) if found.
    fn find_opener(&mut self, marks: &[Mark], closer_idx: usize) -> Option<(usize, u32)> {
        let closer = &marks[closer_idx];
        let closer_len = closer.len();

        // CommonMark "rule of three": if opener + closer lengths are multiples of 3,
        // they can only match if both are multiples of 3.
        // By using modulo-3 stacks, we only need to search compatible stacks.

        let base_idx = if closer.ch == b'_' { 3 } else { 0 };

        // Try to find an opener in compatible stacks
        // For strong emphasis, prefer matching 2 characters
        let _match_count = if closer_len >= 2 { 2 } else { 1 };

        // Calculate which stack(s) to search based on the rule of three
        let closer_mod = closer_len as usize % 3;

        // Search stacks that could produce a valid match
        for opener_mod in 0..3 {
            // Check rule of three compatibility
            // If (opener_len + closer_len) % 3 == 0, both must be multiples of 3
            // This is satisfied when opener_mod == closer_mod == 0, or when they sum to 3
            let sum_mod = (opener_mod + closer_mod) % 3;
            if sum_mod == 0 && (opener_mod != 0 || closer_mod != 0) {
                // Would violate rule of three
                continue;
            }

            let stack_idx = base_idx + opener_mod;
            if let Some(&opener_idx) = self.stacks[stack_idx].last() {
                let opener = &marks[opener_idx];

                // Must be same character
                if opener.ch != closer.ch {
                    continue;
                }

                // Determine how many to match
                let available = opener.len().min(closer_len);
                let actual_match = if available >= 2 { 2 } else { 1 };

                // Pop from stack and return match
                self.stacks[stack_idx].pop();

                // If opener has remaining characters, it might need to go back
                // (handled by caller)

                return Some((opener_idx, actual_match));
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inline::marks::{collect_marks, MarkBuffer};
    use crate::inline::code_span::resolve_code_spans;

    fn get_emphasis_matches(text: &[u8]) -> Vec<EmphasisMatch> {
        let mut buffer = MarkBuffer::new();
        collect_marks(text, &mut buffer);
        resolve_code_spans(buffer.marks_mut());
        resolve_emphasis(buffer.marks_mut())
    }

    #[test]
    fn test_simple_emphasis() {
        let matches = get_emphasis_matches(b"hello *world*");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].count, 1);
    }

    #[test]
    fn test_strong_emphasis() {
        let matches = get_emphasis_matches(b"hello **world**");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].count, 2);
    }

    #[test]
    fn test_underscore_emphasis() {
        let matches = get_emphasis_matches(b"hello _world_");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].count, 1);
    }

    #[test]
    fn test_nested_emphasis() {
        let matches = get_emphasis_matches(b"***bold and italic***");
        // Should produce multiple matches
        assert!(matches.len() >= 1);
    }

    #[test]
    fn test_no_emphasis_in_code() {
        let matches = get_emphasis_matches(b"`*not emphasis*`");
        // Asterisks inside code should not match
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_mismatched_delimiters() {
        // Asterisk and underscore don't match
        let matches = get_emphasis_matches(b"*hello_");
        assert_eq!(matches.len(), 0);
    }
}
