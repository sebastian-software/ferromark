//! A minimal gzip reader for the frozen measurement corpora (tests only).
//!
//! The corpora that the segment planner is proven against live in
//! `docs/reports/*/corpus.json.gz`. Reading them needs a DEFLATE decoder, and
//! the crate deliberately ships no compression dependency, so this is the
//! textbook `puff`-style decoder: correctness and small size over speed. It is
//! compiled only under `cfg(test)`.

#![allow(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

type Result<T> = std::result::Result<T, &'static str>;

/// Decompresses a gzip member and checks the trailer's uncompressed size.
pub fn gunzip(data: &[u8]) -> Result<Vec<u8>> {
    if data.len() < 18 || data[0] != 0x1f || data[1] != 0x8b {
        return Err("not a gzip stream");
    }
    if data[2] != 8 {
        return Err("unsupported compression method");
    }
    let flags = data[3];
    let mut at = 10usize;
    if flags & 0b0000_0100 != 0 {
        let extra = usize::from(u16::from_le_bytes([
            *data.get(at).ok_or("truncated header")?,
            *data.get(at + 1).ok_or("truncated header")?,
        ]));
        at += 2 + extra;
    }
    for flag in [0b0000_1000u8, 0b0001_0000] {
        if flags & flag != 0 {
            while *data.get(at).ok_or("truncated header")? != 0 {
                at += 1;
            }
            at += 1;
        }
    }
    if flags & 0b0000_0010 != 0 {
        at += 2;
    }
    let trailer = data.len() - 4;
    let size = u32::from_le_bytes([
        data[trailer],
        data[trailer + 1],
        data[trailer + 2],
        data[trailer + 3],
    ]) as usize;
    let out = inflate(data.get(at..trailer).ok_or("truncated member")?, size)?;
    if out.len() != size {
        return Err("uncompressed size does not match the trailer");
    }
    Ok(out)
}

struct Bits<'a> {
    data: &'a [u8],
    byte: usize,
    bit: u8,
}

impl Bits<'_> {
    fn bit(&mut self) -> Result<u32> {
        let byte = *self.data.get(self.byte).ok_or("out of input")?;
        let value = u32::from((byte >> self.bit) & 1);
        self.bit += 1;
        if self.bit == 8 {
            self.bit = 0;
            self.byte += 1;
        }
        Ok(value)
    }

    fn take(&mut self, count: u32) -> Result<u32> {
        let mut value = 0;
        for index in 0..count {
            value |= self.bit()? << index;
        }
        Ok(value)
    }

    fn align(&mut self) {
        if self.bit != 0 {
            self.bit = 0;
            self.byte += 1;
        }
    }
}

/// Canonical Huffman code described by its per-length symbol counts.
struct Huffman {
    counts: [u16; 16],
    symbols: Vec<u16>,
}

fn build(lengths: &[u8]) -> Huffman {
    let mut counts = [0u16; 16];
    for &length in lengths {
        counts[usize::from(length)] += 1;
    }
    counts[0] = 0;
    let mut offsets = [0u16; 16];
    for length in 1..16 {
        offsets[length] = offsets[length - 1] + counts[length - 1];
    }
    let mut symbols = vec![0u16; lengths.len()];
    for (symbol, &length) in lengths.iter().enumerate() {
        if length != 0 {
            let slot = usize::from(offsets[usize::from(length)]);
            symbols[slot] = symbol as u16;
            offsets[usize::from(length)] += 1;
        }
    }
    Huffman { counts, symbols }
}

fn decode(bits: &mut Bits<'_>, table: &Huffman) -> Result<u16> {
    let mut code = 0i32;
    let mut first = 0i32;
    let mut index = 0i32;
    for length in 1..16 {
        code |= bits.bit()? as i32;
        let count = i32::from(table.counts[length]);
        if code - first < count {
            return table
                .symbols
                .get((index + (code - first)) as usize)
                .copied()
                .ok_or("symbol out of range");
        }
        index += count;
        first = (first + count) << 1;
        code <<= 1;
    }
    Err("incomplete Huffman code")
}

const LENGTH_BASE: [u16; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];
const LENGTH_EXTRA: [u32; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
const DISTANCE_BASE: [u32; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DISTANCE_EXTRA: [u32; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];
const CODE_LENGTH_ORDER: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

fn inflate(data: &[u8], hint: usize) -> Result<Vec<u8>> {
    let mut bits = Bits {
        data,
        byte: 0,
        bit: 0,
    };
    let mut out = Vec::with_capacity(hint);
    loop {
        let last = bits.take(1)?;
        match bits.take(2)? {
            0 => {
                bits.align();
                let header = data
                    .get(bits.byte..bits.byte + 4)
                    .ok_or("truncated stored block")?;
                let len = usize::from(u16::from_le_bytes([header[0], header[1]]));
                bits.byte += 4;
                let end = bits.byte + len;
                out.extend_from_slice(data.get(bits.byte..end).ok_or("truncated stored block")?);
                bits.byte = end;
            }
            1 => {
                let mut lengths = [8u8; 288];
                lengths[144..256].fill(9);
                lengths[256..280].fill(7);
                inflate_block(&mut bits, &mut out, &build(&lengths), &build(&[5u8; 30]))?;
            }
            2 => {
                let (literals, distances) = dynamic_tables(&mut bits)?;
                inflate_block(&mut bits, &mut out, &literals, &distances)?;
            }
            _ => return Err("reserved block type"),
        }
        if last == 1 {
            return Ok(out);
        }
    }
}

fn dynamic_tables(bits: &mut Bits<'_>) -> Result<(Huffman, Huffman)> {
    let literal_count = bits.take(5)? as usize + 257;
    let distance_count = bits.take(5)? as usize + 1;
    let code_count = bits.take(4)? as usize + 4;
    let mut code_lengths = [0u8; 19];
    for &slot in CODE_LENGTH_ORDER.iter().take(code_count) {
        code_lengths[slot] = bits.take(3)? as u8;
    }
    let code_table = build(&code_lengths);

    let total = literal_count + distance_count;
    let mut lengths = vec![0u8; total];
    let mut at = 0usize;
    while at < total {
        let symbol = decode(bits, &code_table)?;
        let (value, repeat) = match symbol {
            0..=15 => (symbol as u8, 1),
            16 => {
                if at == 0 {
                    return Err("repeat with no previous length");
                }
                (lengths[at - 1], 3 + bits.take(2)? as usize)
            }
            17 => (0, 3 + bits.take(3)? as usize),
            18 => (0, 11 + bits.take(7)? as usize),
            _ => return Err("bad code-length symbol"),
        };
        if at + repeat > total {
            return Err("too many code lengths");
        }
        lengths[at..at + repeat].fill(value);
        at += repeat;
    }
    Ok((
        build(&lengths[..literal_count]),
        build(&lengths[literal_count..]),
    ))
}

fn inflate_block(
    bits: &mut Bits<'_>,
    out: &mut Vec<u8>,
    literals: &Huffman,
    distances: &Huffman,
) -> Result<()> {
    loop {
        let symbol = decode(bits, literals)?;
        match symbol {
            0..=255 => out.push(symbol as u8),
            256 => return Ok(()),
            _ => {
                let index = usize::from(symbol - 257);
                if index >= LENGTH_BASE.len() {
                    return Err("bad length symbol");
                }
                let length =
                    usize::from(LENGTH_BASE[index]) + bits.take(LENGTH_EXTRA[index])? as usize;
                let distance_symbol = usize::from(decode(bits, distances)?);
                if distance_symbol >= DISTANCE_BASE.len() {
                    return Err("bad distance symbol");
                }
                let distance = DISTANCE_BASE[distance_symbol] as usize
                    + bits.take(DISTANCE_EXTRA[distance_symbol])? as usize;
                if distance > out.len() {
                    return Err("distance before the start of output");
                }
                let from = out.len() - distance;
                for offset in 0..length {
                    let byte = out[from + offset];
                    out.push(byte);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::gunzip;

    #[test]
    fn rejects_input_that_is_not_gzip() {
        assert!(gunzip(b"not a gzip stream at all, really").is_err());
    }

    #[test]
    fn decodes_a_stored_block() {
        // A hand-built gzip member with one uncompressed DEFLATE block.
        let payload = b"ferromark";
        let mut stream = vec![0x1f, 0x8b, 0x08, 0x00, 0, 0, 0, 0, 0, 0xff];
        stream.push(0x01);
        stream.extend_from_slice(&(payload.len() as u16).to_le_bytes());
        stream.extend_from_slice(&(!(payload.len() as u16)).to_le_bytes());
        stream.extend_from_slice(payload);
        stream.extend_from_slice(&0u32.to_le_bytes());
        stream.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        assert_eq!(gunzip(&stream).expect("stored block"), payload);
    }
}
