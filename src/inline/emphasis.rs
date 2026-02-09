//! Emphasis and strong emphasis resolution.
//!
//! Uses the "modulo-3" stack optimization from md4c to efficiently
//! match emphasis openers and closers according to CommonMark rules.

use super::marks::{Mark, flags};

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

/// Reusable emphasis stacks to avoid per-parse allocations.
#[derive(Default)]
pub struct EmphasisStacks {
    stacks: [Vec<OpenerEntry>; 6],
    order: usize,
}

impl EmphasisStacks {
    pub fn clear(&mut self) {
        for stack in &mut self.stacks {
            stack.clear();
        }
        self.order = 0;
    }

    pub fn reserve_for_marks(&mut self, marks_len: usize) {
        let target_per_stack = (marks_len / 6).max(8);
        for stack in &mut self.stacks {
            if stack.capacity() < target_per_stack {
                stack.reserve(target_per_stack - stack.capacity());
            }
        }
    }
}

/// Resolve emphasis marks using modulo-3 stacks.
/// Returns a list of matched pairs.
/// `link_boundaries` contains (start, text_end) pairs for each resolved link.
/// Emphasis delimiters cannot match if they cross a link boundary.
#[cfg(test)]
pub fn resolve_emphasis(marks: &mut [Mark], link_boundaries: &[(u32, u32)]) -> Vec<EmphasisMatch> {
    let mut stacks = EmphasisStacks::default();
    resolve_emphasis_with_stacks(marks, link_boundaries, &mut stacks)
}

#[cfg(test)]
pub fn resolve_emphasis_with_stacks(
    marks: &mut [Mark],
    link_boundaries: &[(u32, u32)],
    stacks: &mut EmphasisStacks,
) -> Vec<EmphasisMatch> {
    let mut matches = Vec::new();
    resolve_emphasis_with_stacks_into(marks, link_boundaries, stacks, &mut matches);
    matches
}

pub fn resolve_emphasis_with_stacks_into(
    marks: &mut [Mark],
    link_boundaries: &[(u32, u32)],
    stacks: &mut EmphasisStacks,
    matches: &mut Vec<EmphasisMatch>,
) {
    stacks.reserve_for_marks(marks.len());
    stacks.clear();
    matches.clear();
    let target_matches = (marks.len() / 4).max(8);
    if matches.capacity() < target_matches {
        matches.reserve(target_matches - matches.capacity());
    }
    let mut resolver = EmphasisResolver::new(link_boundaries, stacks);

    // Process marks left to right
    for i in 0..marks.len() {
        let mark = &marks[i];

        // Skip non-emphasis marks or those inside code spans
        if (mark.ch != b'*' && mark.ch != b'_') || mark.flags & flags::IN_CODE != 0 {
            continue;
        }

        if mark.can_close() {
            // Keep trying to close while we can find openers
            loop {
                let mark = &marks[i];
                if mark.is_resolved() || mark.len() == 0 {
                    break;
                }
                if !mark.can_close() {
                    break;
                }

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

                    // CommonMark: Remove all openers between opener and closer
                    // They can no longer form valid matches since we've closed past them
                    resolver.remove_openers_between(marks, opener_idx, i);

                    // Consume characters from both marks
                    let opener = &mut marks[opener_idx];
                    let opener_remaining = opener.len() - match_count;
                    if opener_remaining == 0 {
                        opener.resolve();
                    } else {
                        // Shrink opener from the right
                        opener.end -= match_count;
                        // Re-push opener to correct stack based on new length
                        resolver.push_opener(marks, opener_idx);
                    }

                    let closer = &mut marks[i];
                    let closer_remaining = closer.len() - match_count;
                    if closer_remaining == 0 {
                        closer.resolve();
                        break;
                    } else {
                        // Shrink closer from the left
                        closer.pos += match_count;
                        // Continue the loop to try matching more
                    }
                } else {
                    // No more openers to match, break out of the loop
                    break;
                }
            }

            // After closing, if closer still has characters and can open, push it
            let mark = &marks[i];
            if !mark.is_resolved() && mark.len() > 0 && mark.can_open() {
                resolver.push_opener(marks, i);
            }
        } else if mark.can_open() {
            resolver.push_opener(marks, i);
        }
    }
}

/// Entry in the opener stack with ordering info.
#[derive(Debug, Clone, Copy)]
struct OpenerEntry {
    /// Index into marks array.
    mark_idx: usize,
    /// Global push order for finding most recent opener.
    order: usize,
}

/// Emphasis resolver with 6 stacks (2 chars x 3 modulo classes).
struct EmphasisResolver<'a> {
    /// Stacks indexed by: (is_underscore ? 3 : 0) + (run_length % 3)
    stacks: &'a mut [Vec<OpenerEntry>; 6],
    /// Global order counter.
    order: &'a mut usize,
    /// Link boundaries (start, text_end) - emphasis can't cross these.
    link_boundaries: &'a [(u32, u32)],
}

impl<'a> EmphasisResolver<'a> {
    fn new(link_boundaries: &'a [(u32, u32)], stacks: &'a mut EmphasisStacks) -> Self {
        Self {
            stacks: &mut stacks.stacks,
            order: &mut stacks.order,
            link_boundaries,
        }
    }

    /// Find which link boundary (if any) a position is inside.
    /// Returns Some(index) if inside a link, None if outside all links.
    fn link_boundary_for(&self, pos: u32) -> Option<usize> {
        for (i, &(start, end)) in self.link_boundaries.iter().enumerate() {
            if pos >= start && pos < end {
                return Some(i);
            }
        }
        None
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
        self.stacks[stack_idx].push(OpenerEntry {
            mark_idx: idx,
            order: *self.order,
        });
        *self.order += 1;
    }

    /// Find a matching opener for a closer.
    /// Returns (opener_index, match_count) if found.
    fn find_opener(&mut self, marks: &[Mark], closer_idx: usize) -> Option<(usize, u32)> {
        let closer = &marks[closer_idx];
        let closer_len = closer.len();
        let closer_can_open = closer.can_open();

        // Determine which link boundary the closer is in (if any)
        let closer_link = self.link_boundary_for(closer.pos);

        // CommonMark "rule of three": only applies when one of the delimiters
        // can BOTH open AND close. If neither can both open and close,
        // we can ignore the rule of three entirely.

        let base_idx = if closer.ch == b'_' { 3 } else { 0 };

        // Calculate closer's modulo-3 class (only needed if rule of three applies)
        let closer_mod = closer_len as usize % 3;

        // Find the most recent (highest order) opener across compatible stacks
        let mut best_opener: Option<(usize, OpenerEntry, u32)> = None; // (stack_idx, entry, match_count)

        for opener_mod in 0..3 {
            let stack_idx = base_idx + opener_mod;
            if let Some(&entry) = self.stacks[stack_idx].last() {
                let opener = &marks[entry.mark_idx];

                // Must be same character
                if opener.ch != closer.ch {
                    continue;
                }

                // Opener and closer must be in the same link boundary (or both outside)
                let opener_link = self.link_boundary_for(opener.pos);
                if opener_link != closer_link {
                    continue;
                }

                // Check rule of three: only applies if opener or closer can both open AND close
                let opener_can_close = opener.can_close();
                let rule_of_three_applies = closer_can_open || opener_can_close;

                if rule_of_three_applies {
                    // If (opener_len + closer_len) % 3 == 0, both must be multiples of 3
                    let sum_mod = (opener_mod + closer_mod) % 3;
                    if sum_mod == 0 && (opener_mod != 0 || closer_mod != 0) {
                        // Would violate rule of three
                        continue;
                    }
                }

                // Check if this is more recent than current best
                let dominated = match &best_opener {
                    Some((_, best_entry, _)) => entry.order < best_entry.order,
                    None => false,
                };
                if dominated {
                    continue;
                }

                // Determine how many to match
                let available = opener.len().min(closer_len);
                let actual_match = if available >= 2 { 2 } else { 1 };

                best_opener = Some((stack_idx, entry, actual_match));
            }
        }

        // Pop and return the best opener
        if let Some((stack_idx, entry, match_count)) = best_opener {
            self.stacks[stack_idx].pop();
            Some((entry.mark_idx, match_count))
        } else {
            None
        }
    }

    /// Remove all openers with mark indices between opener_idx and closer_idx.
    /// Per CommonMark spec: delimiters between an opener and closer can no longer
    /// form valid matches once we've closed past them.
    fn remove_openers_between(&mut self, _marks: &[Mark], opener_idx: usize, closer_idx: usize) {
        let _ = closer_idx;
        for stack in self.stacks.iter_mut() {
            while matches!(stack.last(), Some(entry) if entry.mark_idx > opener_idx) {
                stack.pop();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inline::code_span::resolve_code_spans;
    use crate::inline::marks::{MarkBuffer, collect_marks};

    fn get_emphasis_matches(text: &[u8]) -> Vec<EmphasisMatch> {
        let mut buffer = MarkBuffer::new();
        collect_marks(text, &mut buffer);
        resolve_code_spans(buffer.marks_mut(), text, &[]);
        resolve_emphasis(buffer.marks_mut(), &[]) // No link boundaries in basic tests
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
        assert!(!matches.is_empty());
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

    #[test]
    fn test_nested_strong_in_em() {
        // *foo **bar***
        // Position: 0123456789012
        // Expected: <em>foo <strong>bar</strong></em>
        let matches = get_emphasis_matches(b"*foo **bar***");

        eprintln!("Matches:");
        for m in &matches {
            let kind = if m.count == 2 { "strong" } else { "em" };
            eprintln!(
                "  {}: opener {}-{}, closer {}-{}",
                kind, m.opener_start, m.opener_end, m.closer_start, m.closer_end
            );
        }

        // Should have 2 matches: one strong, one em
        assert_eq!(matches.len(), 2);

        // The em opener should be at position 0
        let em_match = matches.iter().find(|m| m.count == 1).expect("em match");
        assert_eq!(em_match.opener_start, 0, "em opener should start at 0");
        assert_eq!(em_match.closer_start, 12, "em closer should start at 12");

        // The strong opener should be at position 5
        let strong_match = matches.iter().find(|m| m.count == 2).expect("strong match");
        assert_eq!(
            strong_match.opener_start, 5,
            "strong opener should start at 5"
        );
        assert_eq!(
            strong_match.closer_start, 10,
            "strong closer should start at 10"
        );
    }
}
