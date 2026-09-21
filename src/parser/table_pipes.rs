//! Pipe discovery and escape classification for GFM table rows.
//!
//! Splitting a row into cells and decoding `\|` inside a cell ask the same
//! two questions of the same bytes: where the `|` bytes are, and whether an
//! odd run of backslashes sits in front of each one. The row splitter used
//! to answer them with one `memchr` call per cell plus a backwards walk per
//! pipe, and the cell decoder then asked them again over every byte of every
//! cell it was handed. [`PipeCursor`] answers both once while walking a row
//! forward, and [`EscapedPipes`] carries what the splitter already saw into
//! the decoder.
//!
//! On aarch64 one block of sixteen bytes is classified by two `vceqq_u8`
//! compares and the nibble-narrowing movemask emulation the crate's other
//! vector scanners use, which leaves one nibble per byte in a `u64`. The
//! escape state of every pipe in the block then follows from the backslash
//! mask by bit arithmetic instead of a byte walk. Everywhere else the cursor
//! keeps the original `memchr` plus [`is_escaped_table_pipe`] scan, which is
//! also the reference the vector path is differentially tested against.

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
/// The splitter walks every pipe of a cell on its way to the cell's
/// terminator, so the decoder never has to look for them from scratch. Only
/// the first and the last offset are kept, which fits in a register pair: an
/// empty record decodes the cell to its borrowed slice without any scan at
/// all, and a filled one bounds the decoder's walk to the stretch that
/// actually holds escapes.
#[derive(Clone, Copy, Default)]
pub(super) struct EscapedPipes {
    bounds: Option<(u32, u32)>,
}

impl EscapedPipes {
    /// Notes an escaped pipe. Offsets must arrive in ascending order, which
    /// is how a forward row scan produces them.
    pub(super) fn record(&mut self, offset: usize) {
        let offset = offset as u32;
        self.bounds = Some(match self.bounds {
            Some((first, _)) => (first, offset),
            None => (offset, offset),
        });
    }

    /// The same record in the coordinates of a slice starting `by` bytes
    /// later. A `|` is never part of the whitespace a cell is trimmed of, so
    /// every recorded offset still lies inside the trimmed cell.
    pub(super) fn shifted(self, by: usize) -> Self {
        let by = by as u32;
        Self {
            bounds: self
                .bounds
                .map(|(first, last)| (first.saturating_sub(by), last.saturating_sub(by))),
        }
    }

    /// The first and last escaped pipe, or `None` when the cell holds none.
    pub(super) fn bounds(self) -> Option<(usize, usize)> {
        self.bounds
            .map(|(first, last)| (first as usize, last as usize))
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

/// A forward walk over the pipes of one table row.
///
/// Every pipe is reported once, in order, with its escape state already
/// decided. One pipe of pushback covers the merged-cell run scan, which has
/// to look at the pipe after a run before it can tell the run is over.
pub(super) struct PipeCursor<'a> {
    bytes: &'a [u8],
    pending: Option<Pipe>,
    scan: Scan,
}

impl<'a> PipeCursor<'a> {
    pub(super) fn new(bytes: &'a [u8], from: usize) -> Self {
        Self {
            bytes,
            pending: None,
            scan: Scan::new(bytes, from.min(bytes.len())),
        }
    }

    /// The next pipe at or after the cursor.
    pub(super) fn next_pipe(&mut self) -> Option<Pipe> {
        match self.pending.take() {
            Some(pipe) => Some(pipe),
            None => self.scan.next_pipe(self.bytes),
        }
    }

    /// The next pipe without consuming it.
    pub(super) fn peek_pipe(&mut self) -> Option<Pipe> {
        if self.pending.is_none() {
            self.pending = self.scan.next_pipe(self.bytes);
        }
        self.pending
    }

    /// Drops the peeked pipe. Called only right after [`Self::peek_pipe`]
    /// returned one that the caller decided to take.
    pub(super) fn take_peeked(&mut self) {
        self.pending = None;
    }
}

/// Bytes classified per vector load.
#[cfg(target_arch = "aarch64")]
const LANES: usize = 16;

/// The mask bits one byte lane occupies in a narrowed compare result.
#[cfg(target_arch = "aarch64")]
const LANE: u64 = 0xF;

/// Vector engine: the lane masks of one cached block, plus the backslash
/// parity that reaches that block from the bytes before it.
///
/// Pipes are usually spread over a row rather than packed into it, so the
/// scan does not classify every block on the way. `memchr` skips each
/// pipe-free stretch in one step and the scan re-seats on the block that
/// holds the next pipe; classifying that block then settles every pipe in
/// it, however many there are, without a backwards walk per pipe.
#[cfg(target_arch = "aarch64")]
struct Scan {
    /// Offset of the cached block.
    block: usize,
    /// Pipe lanes of the cached block that have not been yielded yet.
    pipes: u64,
    /// Backslash lanes of the cached block.
    backslashes: u64,
    /// Whether an odd run of backslashes ends immediately before `block`.
    carry_odd: bool,
}

#[cfg(target_arch = "aarch64")]
impl Scan {
    fn new(bytes: &[u8], from: usize) -> Self {
        let mut scan = Self {
            block: 0,
            pipes: 0,
            backslashes: 0,
            carry_odd: false,
        };
        scan.seek(bytes, from);
        scan
    }

    /// Caches the block holding `from` and drops the lanes before it.
    fn seek(&mut self, bytes: &[u8], from: usize) {
        self.block = from - from % LANES;
        // A seek can land inside a block, so the backslash run that reaches
        // the block start is walked once here, bounded by the run's own
        // length, instead of being carried block by block.
        self.carry_odd = is_escaped_table_pipe(bytes, self.block);
        let rest = &bytes[self.block..];
        let (pipes, backslashes) = if let Some(block) = rest.first_chunk::<LANES>() {
            classify_block(block)
        } else {
            // The row's last bytes, zero padded. Neither `|` nor `\` is
            // zero, so the padding adds no lane to either mask and the
            // masks stay exact without a separate tail mask.
            let mut tail = [0u8; LANES];
            tail[..rest.len()].copy_from_slice(rest);
            classify_block(&tail)
        };
        // Four mask bits per byte, so this drops the lanes before `from`.
        self.pipes = pipes & (u64::MAX << ((from - self.block) * 4));
        self.backslashes = backslashes;
    }

    fn next_pipe(&mut self, bytes: &[u8]) -> Option<Pipe> {
        if self.pipes == 0 {
            // The cached block is spent, and it held no pipe past the point
            // the scan reached, so the search resumes at the next block.
            let from = self.block + LANES;
            if from >= bytes.len() {
                return None;
            }
            let pipe = from + memchr(b'|', &bytes[from..])?;
            self.seek(bytes, pipe);
        }
        let lane = self.pipes.trailing_zeros() as usize / 4;
        self.pipes &= !(LANE << (lane * 4));
        Some(Pipe {
            offset: self.block + lane,
            escaped: self.lane_is_escaped(lane),
        })
    }

    /// Whether the pipe in `lane` carries a table-level escape, decided from
    /// the backslash mask alone.
    fn lane_is_escaped(&self, lane: usize) -> bool {
        let below = (1u64 << (lane * 4)) - 1;
        let gaps = !self.backslashes & below;
        if gaps == 0 {
            // Every byte before the pipe in this block is a backslash, so
            // the run continues into the bytes before the block.
            (lane % 2 == 1) != self.carry_odd
        } else {
            // Otherwise the run starts after the last non-backslash lane.
            (lane - 1 - last_lane(gaps)) % 2 == 1
        }
    }
}

/// The highest lane set in a narrowed compare result. `mask` must not be 0.
#[cfg(target_arch = "aarch64")]
fn last_lane(mask: u64) -> usize {
    (u64::BITS as usize - 1 - mask.leading_zeros() as usize) / 4
}

/// Pipe and backslash lane masks for one sixteen-byte block.
#[cfg(target_arch = "aarch64")]
#[allow(unsafe_code)]
#[inline]
fn classify_block(block: &[u8; LANES]) -> (u64, u64) {
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
            )
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
    fn new(_bytes: &[u8], from: usize) -> Self {
        Self { next: from }
    }

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
