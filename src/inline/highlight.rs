//! Highlight/mark resolution (`==text==`).
//!
//! Matches equal-sign runs of exactly length 2 as opener/closer pairs.
//! Uses the same flanking rules as `*` emphasis (already computed in mark collection).

use super::marks::{Mark, flags};

/// A matched highlight pair.
#[derive(Debug, Clone, Copy)]
pub struct HighlightMatch {
    pub opener_start: u32,
    pub opener_end: u32,
    pub closer_start: u32,
    pub closer_end: u32,
}

/// Resolve highlight marks. Matches opener/closer `==` pairs greedily
/// (innermost first, left to right).
pub fn resolve_highlight_into(
    marks: &mut [Mark],
    text: &[u8],
    link_boundaries: &[(u32, u32)],
    link_dest_ranges: &[(u32, u32)],
    matches: &mut Vec<HighlightMatch>,
) {
    matches.clear();

    let mut openers: Vec<usize> = Vec::new();

    for i in 0..marks.len() {
        let mark = &marks[i];
        if mark.ch != b'=' || mark.flags & flags::IN_CODE != 0 || mark.len() != 2 {
            continue;
        }
        if pos_in_ranges(mark.pos, link_dest_ranges) {
            continue;
        }

        if mark.can_close() {
            let mut found = None;
            for j in (0..openers.len()).rev() {
                let opener_idx = openers[j];
                let opener = &marks[opener_idx];
                if opener.is_resolved() {
                    continue;
                }
                if !same_link_boundary(opener.pos, mark.pos, link_boundaries) {
                    continue;
                }
                if pos_in_ranges(opener.pos, link_dest_ranges) {
                    continue;
                }

                let content = &text[opener.end as usize..mark.pos as usize];
                if content.is_empty() || content.iter().all(|b| matches!(b, b' ' | b'\t' | b'\n')) {
                    continue;
                }

                found = Some(j);
                break;
            }

            if let Some(opener_stack_idx) = found {
                let opener_idx = openers[opener_stack_idx];
                let opener = &marks[opener_idx];
                let closer = &marks[i];

                matches.push(HighlightMatch {
                    opener_start: opener.pos,
                    opener_end: opener.end,
                    closer_start: closer.pos,
                    closer_end: closer.end,
                });

                marks[opener_idx].resolve();
                marks[i].resolve();
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
    let a_boundary = boundaries.iter().position(|&(s, e)| a >= s && a < e);
    let b_boundary = boundaries.iter().position(|&(s, e)| b >= s && b < e);
    a_boundary == b_boundary
}

fn pos_in_ranges(pos: u32, ranges: &[(u32, u32)]) -> bool {
    ranges.iter().any(|&(start, end)| pos >= start && pos < end)
}
