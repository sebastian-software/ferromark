//! Pipe discovery and escape classification for GFM table rows.
//!
//! Splitting a row into cells and decoding `\|` inside a cell ask the same
//! two questions of the same bytes: where the `|` bytes are, and whether an
//! odd run of backslashes sits in front of each one. [`PipeCursor`] answers
//! both while walking a row forward, and [`EscapedPipes`] carries the escapes
//! the splitter passed over into the decoder, so a cell without one is never
//! scanned again and a cell with some is scanned only between its first and
//! its last.
//!
//! On aarch64 the cursor reads the row a sixteen-byte window at a time. Two
//! `vceqq_u8` compares and the nibble-narrowing movemask emulation the
//! crate's other vector scanners use give one mask bit per `|` and per `\`,
//! so every pipe of a window is found without a call and its escape state
//! follows from the backslash bits in front of it. A window without a pipe
//! hands the rest of the gap to `memchr`, and the next window starts on the
//! pipe it finds: pipes closer than a window apart cost no call at all, and
//! sparser ones at most the one `memchr` call per pipe the scalar scan made.
//! Nothing is read before the first pipe is asked for, and a row shorter
//! than a window is classified by a plain byte loop instead of a padded
//! copy.
//!
//! Everywhere else the cursor keeps the `memchr` probe and backwards walk
//! ([`is_escaped_table_pipe`]) the table parser always used, which is also
//! the reference the vector path is tested against.

use memchr::memchr;

/// A pipe in a table row, together with the escape state that both the row
/// splitter and the cell decoder need for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Pipe {
    pub(super) offset: usize,
    pub(super) escaped: bool,
}

/// Whether an odd run of backslashes ends immediately before `index`.
///
/// Only a non-backslash byte ends such a run, so the answer never changes
/// when the slice is cut at a `|` or at the whitespace a cell is trimmed of.
/// That is what lets the row splitter record escape positions in row
/// coordinates and the cell decoder use them on its own trimmed slice.
pub(super) fn is_escaped_table_pipe(bytes: &[u8], index: usize) -> bool {
    bytes[..index]
        .iter()
        .rev()
        .take_while(|&&byte| byte == b'\\')
        .count()
        % 2
        == 1
}

/// Where a cell's escaped pipes sit, as the row splitter passed over them.
///
/// Only the first and the last offset are kept, which fits in a register
/// pair: an empty record decodes the cell to its borrowed slice without any
/// scan, and a filled one bounds the decoder's walk to the stretch that holds
/// escapes. Empty is `first > last`.
#[derive(Clone, Copy)]
pub(super) struct EscapedPipes {
    first: u32,
    last: u32,
}

impl Default for EscapedPipes {
    fn default() -> Self {
        Self {
            first: u32::MAX,
            last: 0,
        }
    }
}

impl EscapedPipes {
    /// Notes an escaped pipe. Offsets must arrive in ascending order, which
    /// is how a forward row scan produces them.
    #[inline]
    pub(super) fn record(&mut self, offset: usize) {
        let offset = offset as u32;
        self.first = self.first.min(offset);
        self.last = offset;
    }

    /// The same record in the coordinates of a slice starting `by` bytes
    /// later. A `|` is never part of the whitespace a cell is trimmed of, so
    /// every recorded offset still lies inside the trimmed cell.
    #[inline]
    pub(super) fn shifted(self, by: usize) -> Self {
        if self.first > self.last {
            return self;
        }
        let by = by as u32;
        Self {
            first: self.first - by,
            last: self.last - by,
        }
    }

    /// The first and last escaped pipe, or `None` when the cell holds none.
    #[inline]
    pub(super) fn bounds(self) -> Option<(usize, usize)> {
        (self.first <= self.last).then_some((self.first as usize, self.last as usize))
    }

    /// Records every escaped pipe of a standalone slice, for callers that
    /// hold a cell without having split its row.
    #[cfg(test)]
    pub(super) fn scan(bytes: &[u8]) -> Self {
        let mut escapes = Self::default();
        let mut cursor = PipeCursor::new(bytes, 0);
        while let Some(pipe) = cursor.next_pipe() {
            if pipe.escaped {
                escapes.record(pipe.offset);
            }
        }
        escapes
    }
}

/// A forward walk over the pipes of `bytes`, starting at a given offset.
///
/// Every pipe is reported once, in order, with its escape state already
/// decided from the whole slice before it. Cutting the slice after a pipe
/// therefore changes nothing about the pipes before the cut, which is how
/// the cell decoder stops at its last escape.
pub(super) struct PipeCursor<'a> {
    bytes: &'a [u8],
    scan: Scan,
}

impl<'a> PipeCursor<'a> {
    /// A cursor that reports the pipes at or after `from`. Nothing is read
    /// until the first pipe is asked for.
    #[inline]
    pub(super) fn new(bytes: &'a [u8], from: usize) -> Self {
        Self {
            bytes,
            scan: Scan::new(from.min(bytes.len())),
        }
    }

    /// The next pipe at or after the cursor.
    #[inline]
    pub(super) fn next_pipe(&mut self) -> Option<Pipe> {
        self.scan.next_pipe(self.bytes)
    }
}

/// Bytes classified per vector load.
#[cfg(target_arch = "aarch64")]
const LANES: usize = 16;

/// One bit per lane of a narrowed compare result: bit `4 * lane`.
#[cfg(target_arch = "aarch64")]
const LANE_BITS: u64 = 0x1111_1111_1111_1111;

/// Vector engine: the lane masks of the current window.
///
/// A window is the sixteen bytes from `base` (fewer at the row's end). Its
/// masks keep one bit per lane, bit `4 * lane`, so the lowest pipe is one
/// `trailing_zeros` away and the backslashes in front of it one
/// `leading_zeros`.
#[cfg(target_arch = "aarch64")]
struct Scan {
    /// Where the next window starts once this one is spent.
    next: usize,
    /// Offset of the window's lane 0.
    base: usize,
    /// Pipe lanes of the window that have not been reported yet.
    pipes: u64,
    /// Backslash lanes of the window.
    backslashes: u64,
}

#[cfg(target_arch = "aarch64")]
impl Scan {
    #[inline]
    fn new(from: usize) -> Self {
        Self {
            next: from,
            base: from,
            pipes: 0,
            backslashes: 0,
        }
    }

    #[inline]
    fn next_pipe(&mut self, bytes: &[u8]) -> Option<Pipe> {
        if self.pipes == 0 && !self.refill(bytes) {
            return None;
        }
        let bit = self.pipes.trailing_zeros();
        self.pipes &= self.pipes - 1;
        let offset = self.base + bit as usize / 4;
        Some(Pipe {
            offset,
            escaped: self.escaped(bytes, bit, offset),
        })
    }

    /// Whether the pipe at mask bit `bit` (at `offset`) carries a
    /// table-level escape.
    #[inline]
    fn escaped(&self, bytes: &[u8], bit: u32, offset: usize) -> bool {
        // The window's bytes before the pipe that are not backslashes.
        let stops = !self.backslashes & LANE_BITS & ((1u64 << bit) - 1);
        if stops == 0 {
            // The pipe opens the window, or only backslashes precede it
            // there: the run may reach into the bytes before the window, so
            // walk it. The walk is bounded by the run, and it happens at
            // most once per window.
            return is_escaped_table_pipe(bytes, offset);
        }
        // The run starts right after the last non-backslash lane, so its
        // length is the lane distance to that stop, minus one.
        let last_stop = u64::BITS - 1 - stops.leading_zeros();
        let run = (bit - last_stop) / 4 - 1;
        run % 2 == 1
    }

    /// Makes the next window that holds a pipe current, or reports that no
    /// pipe is left.
    fn refill(&mut self, bytes: &[u8]) -> bool {
        let mut at = self.next;
        loop {
            let rest = bytes.len() - at;
            if rest == 0 {
                self.next = at;
                return false;
            }
            let (pipes, backslashes) = window(bytes, at);
            let spent = at + rest.min(LANES);
            if pipes != 0 {
                self.base = at;
                self.pipes = pipes;
                self.backslashes = backslashes;
                self.next = spent;
                return true;
            }
            // A window without a pipe: `memchr` skips the rest of the gap in
            // one call, and the next window starts on the pipe it finds.
            let Some(relative) = memchr(b'|', &bytes[spent..]) else {
                self.next = bytes.len();
                return false;
            };
            at = spent + relative;
        }
    }
}

/// Pipe and backslash lanes of the up to sixteen bytes from `at`, lane 0
/// being `at`. `at` must be inside `bytes`.
#[cfg(target_arch = "aarch64")]
#[inline]
fn window(bytes: &[u8], at: usize) -> (u64, u64) {
    let rest = &bytes[at..];
    if let Some(block) = rest.first_chunk::<LANES>() {
        classify(block)
    } else if let Some(block) = bytes.last_chunk::<LANES>() {
        // Fewer than sixteen bytes remain, but the row is longer than a
        // window: classify its last sixteen bytes and shift out the lanes
        // before `at`. The shift fills with zeros, so no lane past the end
        // is ever set.
        let (pipes, backslashes) = classify(block);
        let skipped = (LANES - rest.len()) * 4;
        (pipes >> skipped, backslashes >> skipped)
    } else {
        // The whole row is shorter than one window.
        let mut pipes = 0;
        let mut backslashes = 0;
        for (lane, &byte) in rest.iter().enumerate() {
            pipes |= u64::from(byte == b'|') << (lane * 4);
            backslashes |= u64::from(byte == b'\\') << (lane * 4);
        }
        (pipes, backslashes)
    }
}

/// Pipe and backslash lanes for one sixteen-byte block.
#[cfg(target_arch = "aarch64")]
#[allow(unsafe_code)]
#[inline]
fn classify(block: &[u8; LANES]) -> (u64, u64) {
    use std::arch::aarch64::*;
    // SAFETY: `block` is exactly one vector wide, so the load is in bounds.
    unsafe {
        // `vshrn_n_u16(_, 4)` narrows each pair of all-ones/all-zeros lanes
        // into one byte of nibbles, which is this crate's movemask
        // emulation: byte lane `i` becomes nibble `i` of the result.
        let narrow = |mask: uint8x16_t| {
            vget_lane_u64(
                vreinterpret_u64_u8(vshrn_n_u16(vreinterpretq_u16_u8(mask), 4)),
                0,
            ) & LANE_BITS
        };
        let bytes = vld1q_u8(block.as_ptr());
        (
            narrow(vceqq_u8(bytes, vdupq_n_u8(b'|'))),
            narrow(vceqq_u8(bytes, vdupq_n_u8(b'\\'))),
        )
    }
}

/// Portable engine: the `memchr` probe and backwards backslash walk the
/// table parser used before the vector path existed, kept both as the
/// fallback and as the behavior the vector path must reproduce.
#[cfg(not(target_arch = "aarch64"))]
struct Scan {
    next: usize,
}

#[cfg(not(target_arch = "aarch64"))]
impl Scan {
    #[inline]
    fn new(from: usize) -> Self {
        Self { next: from }
    }

    #[inline]
    fn next_pipe(&mut self, bytes: &[u8]) -> Option<Pipe> {
        let relative = memchr(b'|', &bytes[self.next..])?;
        let offset = self.next + relative;
        self.next = offset + 1;
        Some(Pipe {
            offset,
            escaped: is_escaped_table_pipe(bytes, offset),
        })
    }
}

#[cfg(test)]
mod tests;
