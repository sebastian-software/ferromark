use ferro_byte_search::ByteSet;

fn check<const N: usize>(bytes: &[u8; N], input: &[u8]) {
    let set = ByteSet::new(bytes);
    let expected = input.iter().position(|byte| bytes.contains(byte));
    assert_eq!(set.find(input), expected, "set={bytes:?} input={input:?}");
    assert_eq!(set.contains_any(input), expected.is_some());
}

#[test]
fn all_bytes_at_vector_and_short_scan_boundaries() {
    for len in [0, 1, 7, 15, 16, 17, 31, 32, 33, 63, 64, 65, 127, 128, 129] {
        for byte in 0..=255 {
            for pos in 0..=len {
                let mut storage = vec![b'a'; len + 7];
                let input = &mut storage[7..];
                if pos < len {
                    input[pos] = byte;
                }
                check(b"&<>\"", input);
                check(b"*_`[]<\\\n~$=^", input);
                check(&[0, 128, 255], input);
                check(&[byte], input);
            }
        }
    }
}

#[test]
fn nul_padding_empty_sets_and_duplicates() {
    const EMPTY: ByteSet<0> = ByteSet::new(b"");
    assert_eq!(EMPTY.find(b"anything\0"), None);
    assert!(!EMPTY.contains_any(b"anything\0"));
    for len in 0..40 {
        check(b"\0", &vec![b'a'; len]);
        check(b"\0\0\0\0", &vec![b'a'; len]);
        check(b"aaaa", &vec![b'a'; len]);
        check(b"\0", &vec![0; len]);
    }
}

#[test]
fn overlapping_tail_returns_the_earliest_real_match() {
    for len in 17..66 {
        for first in 0..len {
            for second in first..len {
                let mut input = vec![b'a'; len];
                input[first] = b'&';
                input[second] = 0;
                check(&[b'&', 0], &input);
            }
        }
    }
}

#[test]
fn runtime_sets_and_arbitrary_binary_inputs() {
    let mut state = 0x1234_5678_u32;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        state as u8
    };
    for len in 0..1024 {
        let set: [u8; 16] = std::array::from_fn(|_| next());
        let input: Vec<u8> = (0..len).map(|_| next()).collect();
        check(&set, &input);
    }
    let every_byte: [u8; 256] = std::array::from_fn(|i| i as u8);
    check(&every_byte, b"\xff\0\x80abc");
}

#[test]
fn overlapping_nibbles_preserve_exact_membership() {
    let distinct_rows = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88];
    let shared_rows: [u8; 16] = std::array::from_fn(|i| (i as u8) << 4 | 3);
    for len in [1, 15, 16, 17, 31, 32, 33, 65] {
        for byte in 0..=255 {
            for pos in 0..len {
                let mut input = vec![0xfe; len];
                input[pos] = byte;
                check(&distinct_rows, &input);
                check(&shared_rows, &input);
            }
        }
    }
}
