//! Strikethrough resolution (`~~text~~`).
//!
//! Matches tilde runs of length 1 or 2 as opener/closer pairs.
//! Uses same flanking rules as `*` emphasis (already computed in mark collection).

use super::marks::{flags, Mark};

/// A matched strikethrough pair.
#[derive(Debug, Clone, Copy)]
pub struct StrikethroughMatch {
    pub opener_start: u32,
    pub opener_end: u32,
    pub closer_start: u32,
    pub closer_end: u32,
}

/// Resolve strikethrough marks. Matches opener/closer tilde runs greedily
/// (innermost first, left to right).
pub fn resolve_strikethrough_into(
    marks: &mut [Mark],
    link_boundaries: &[(u32, u32)],
    matches: &mut Vec<StrikethroughMatch>,
) {
    matches.clear();

    // Collect indices of tilde marks that can open
    let mut openers: Vec<usize> = Vec::new();

    for i in 0..marks.len() {
        let mark = &marks[i];
        if mark.ch != b'~' || mark.flags & flags::IN_CODE != 0 {
            continue;
        }

        let run_len = mark.len();
        // cmark-gfm only matches runs of exactly 1 or 2
        if run_len != 1 && run_len != 2 {
            continue;
        }

        if mark.can_close() {
            // Try to find a matching opener (most recent with same run length)
            let mut found = None;
            for j in (0..openers.len()).rev() {
                let opener_idx = openers[j];
                let opener = &marks[opener_idx];
                if opener.is_resolved() {
                    continue;
                }
                if opener.len() != run_len {
                    continue;
                }
                // Must be in same link boundary
                if !same_link_boundary(opener.pos, mark.pos, link_boundaries) {
                    continue;
                }
                found = Some(j);
                break;
            }

            if let Some(opener_stack_idx) = found {
                let opener_idx = openers[opener_stack_idx];
                let opener = &marks[opener_idx];
                let closer = &marks[i];

                matches.push(StrikethroughMatch {
                    opener_start: opener.pos,
                    opener_end: opener.end,
                    closer_start: closer.pos,
                    closer_end: closer.end,
                });

                // Remove openers between opener and closer (they can't match anymore)
                // and remove the matched opener
                marks[opener_idx].resolve();
                marks[i].resolve();
                // Remove the matched opener and any openers between it and closer
                openers.truncate(opener_stack_idx);
            } else if mark.can_open() {
                openers.push(i);
            }
        } else if mark.can_open() {
            openers.push(i);
        }
    }
}

fn same_link_boundary(a: u32, b: u32, boundaries: &[(u32, u32)]) -> bool {
    // Both must be in the same link boundary (or both outside any)
    let a_boundary = boundaries.iter().position(|&(s, e)| a >= s && a < e);
    let b_boundary = boundaries.iter().position(|&(s, e)| b >= s && b < e);
    a_boundary == b_boundary
}
