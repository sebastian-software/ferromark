    #[cfg(target_arch = "x86_64")]
    #[test]
    fn x86_escape_scanners_match_oracle_at_every_alignment() {
        fn check(input: &[u8]) {
            let text = input.iter().position(|&b| matches!(b, b'<' | b'>' | b'&' | b'"'));
            let attr = input.iter().position(|&b| matches!(b, b'<' | b'>' | b'&' | b'"' | b'\''));
            // SAFETY: SSE2 is guaranteed on x86-64.
            unsafe {
                assert_eq!(first_escape_sse2::<false>(input), text);
                assert_eq!(first_escape_sse2::<true>(input), attr);
            }
            if std::is_x86_feature_detected!("avx2") {
                // SAFETY: the runtime check establishes AVX2 support.
                unsafe {
                    assert_eq!(first_escape_avx2::<false>(input), text);
                    assert_eq!(first_escape_avx2::<true>(input), attr);
                }
            }
        }
        for len in [0, 1, 15, 16, 17, 31, 32, 33, 63, 64, 65, 127, 128, 129, 255, 256, 257] {
            for offset in 0..32 {
                // Escapes outside the slice must never appear in its result.
                let mut storage = vec![b'<'; offset + len + 32];
                let input = &mut storage[offset..offset + len];
                input.fill(b'x');
                check(input);
                if len != 0 {
                    for at in [0, len / 2, len - 1] {
                        for &byte in b"<>&\"'\0\x80\xff" {
                            input[at] = byte;
                            check(input);
                            input[at] = b'x';
                        }
                    }
                }
            }
        }
        let mut input = vec![b'x'; 161];
        for at in 0..161 {
            for byte in 0..=255 {
                input[at] = byte;
                check(&input);
            }
            input[at] = b'x';
        }
    }

