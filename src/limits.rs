//! DoS prevention constants.
//!
//! These limits prevent pathological inputs from causing
//! quadratic or worse time complexity.

/// Maximum nesting depth for block containers (lists, blockquotes)
pub const MAX_BLOCK_NESTING: usize = 32;

/// Maximum nesting depth for inline elements (emphasis, links)
pub const MAX_INLINE_NESTING: usize = 32;

/// Maximum bracket depth in link parsing [[[...]]]
pub const MAX_BRACKET_DEPTH: usize = 8;

/// Maximum delimiter stack size per type
pub const MAX_DELIMITER_STACK: usize = 64;

/// Maximum number of marks collected during inline parsing
pub const MAX_INLINE_MARKS: usize = 4096;

/// Maximum backtick run length for code spans (prevents O(n^2) matching)
/// Longer runs are treated as literal text
pub const MAX_CODE_SPAN_BACKTICKS: usize = 32;

/// Maximum parentheses nesting in link destinations (CommonMark spec: 32)
pub const MAX_LINK_PAREN_DEPTH: usize = 32;

/// Maximum digits in ordered list marker (prevents big-integer parsing)
pub const MAX_LIST_MARKER_DIGITS: usize = 9;

/// Link reference expansion limit (prevents recursive expansion DoS)
pub const MAX_LINK_REF_EXPANSIONS: usize = 100 * 1024;

/// Maximum table columns (if tables are implemented)
pub const MAX_TABLE_COLUMNS: usize = 128;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limits_are_reasonable() {
        // Ensure limits are within expected ranges
        const { assert!(MAX_BLOCK_NESTING >= 16) };
        const { assert!(MAX_BLOCK_NESTING <= 64) };
        const { assert!(MAX_INLINE_NESTING >= 16) };
        const { assert!(MAX_CODE_SPAN_BACKTICKS >= 16) };
    }
}
