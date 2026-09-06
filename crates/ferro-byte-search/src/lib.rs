#![no_std]

//! Allocation-free search for small sets of bytes.
//!
//! ```
//! use ferro_byte_search::ByteSet;
//! const SPECIALS: ByteSet<4> = ByteSet::new(b"&<>\"");
//! assert_eq!(SPECIALS.find(b"text & more"), Some(5));
//! assert!(!SPECIALS.contains_any(b"ordinary text"));
//! ```
//!
//! Uses baseline SSE2 on x86-64 and NEON-enabled AArch64, with a scalar
//! fallback elsewhere. All public operations are safe and accept arbitrary
//! bytes, including NUL and invalid UTF-8. No runtime feature detection,
//! allocation, standard library, C/C++, or external SIMD library is required.

// One implementation is compiled both here and as a private Ferromark module.
// Ferromark can continue publishing without depending on this unpublished crate.
#[path = "../../../src/byte_search.rs"]
mod search;

pub use search::ByteSet;
