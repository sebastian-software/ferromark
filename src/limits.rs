//! Enforced parser resource limits.
//!
//! Every constant in this module is consumed by a parser path and covered by
//! black-box tests in `tests/resource_limits_tests.rs`.

/// Maximum nesting depth for block containers (lists, blockquotes)
pub const MAX_BLOCK_NESTING: usize = 32;

/// Maximum number of marks collected during inline parsing
pub const MAX_INLINE_MARKS: usize = 4096;

/// Maximum backtick run length for code spans (prevents O(n^2) matching)
/// Longer runs are treated as literal text
pub const MAX_CODE_SPAN_BACKTICKS: usize = 32;

/// Maximum parentheses nesting in link destinations (CommonMark spec: 32)
pub const MAX_LINK_PAREN_DEPTH: usize = 32;

/// Maximum digits in ordered list marker (prevents big-integer parsing)
pub const MAX_LIST_MARKER_DIGITS: usize = 9;

/// Maximum table columns
pub const MAX_TABLE_COLUMNS: usize = 128;
