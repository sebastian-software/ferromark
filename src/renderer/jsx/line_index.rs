//! Byte offsets as the source positions JavaScript tooling expects.

/// Line starts of a source text, for converting byte offsets to a zero-based
/// line and UTF-16 column.
///
/// A line ends after `\n`, after `\r\n`, or after a lone `\r`. The index is
/// built once, so a lookup costs a binary search plus one line of text rather
/// than a scan from the start of the source.
pub(super) struct LineIndex<'a> {
    source: &'a str,
    starts: Vec<usize>,
}

impl<'a> LineIndex<'a> {
    pub(super) fn new(source: &'a str) -> Self {
        let bytes = source.as_bytes();
        let mut starts = Vec::with_capacity(bytes.len() / 40 + 1);
        starts.push(0);
        for index in memchr::memchr2_iter(b'\n', b'\r', bytes) {
            if bytes[index] == b'\r' && bytes.get(index + 1) == Some(&b'\n') {
                continue;
            }
            starts.push(index + 1);
        }
        Self { source, starts }
    }

    /// The line and UTF-16 column of a byte offset. An offset inside a
    /// character counts that character, and one past the end is the end.
    pub(super) fn position(&self, offset: usize) -> (u32, u32) {
        let mut offset = offset.min(self.source.len());
        while !self.source.is_char_boundary(offset) {
            offset += 1;
        }
        let line = self.starts.partition_point(|start| *start <= offset) - 1;
        let column = self.source[self.starts[line]..offset]
            .encode_utf16()
            .count();
        (to_u32(line), to_u32(column))
    }
}

pub(super) fn to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::LineIndex;

    /// The scan this index replaces: every position counted from the start.
    fn scanned(source: &str, offset: usize) -> (u32, u32) {
        let (mut line, mut column, mut previous_cr) = (0u32, 0u32, false);
        for (byte_offset, ch) in source.char_indices() {
            if byte_offset >= offset {
                break;
            }
            match ch {
                '\r' => {
                    line += 1;
                    column = 0;
                    previous_cr = true;
                }
                '\n' if previous_cr => previous_cr = false,
                '\n' => {
                    line += 1;
                    column = 0;
                    previous_cr = false;
                }
                _ => {
                    column += ch.len_utf16() as u32;
                    previous_cr = false;
                }
            }
        }
        (line, column)
    }

    #[test]
    fn positions_count_utf16_columns_and_every_line_ending() {
        let index = LineIndex::new("a😀b\r\nc\rd\ne");
        assert_eq!(index.position(0), (0, 0));
        // `😀` is four UTF-8 bytes and two UTF-16 code units.
        assert_eq!(index.position(5), (0, 3));
        assert_eq!(index.position(8), (1, 0));
        assert_eq!(index.position(10), (2, 0));
        assert_eq!(index.position(12), (3, 0));
        assert_eq!(index.position(99), (3, 1));
    }

    #[test]
    fn positions_match_a_scan_from_the_start_of_the_source() {
        for source in [
            "",
            "one line",
            "a😀b\r\nc\rd\ne\n",
            "\n\n# Ünï\r\n\r\ntext 🧪 {value}\n\r\rend",
        ] {
            let index = LineIndex::new(source);
            for offset in 0..=source.len() + 1 {
                // Between the two bytes of `\r\n` the scan has already left
                // the line; no node starts there.
                if source.as_bytes().get(offset) == Some(&b'\n')
                    && offset > 0
                    && source.as_bytes()[offset - 1] == b'\r'
                {
                    continue;
                }
                assert_eq!(
                    index.position(offset),
                    scanned(source, offset),
                    "{source:?} at {offset}"
                );
            }
        }
    }
}
