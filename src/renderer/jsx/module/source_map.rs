//! The version 3 source map of an assembled module.

use crate::renderer::JsxSourceMapping;

/// Encodes mappings that are already sorted by generated position.
pub(super) fn encode(mappings: &[JsxSourceMapping]) -> String {
    let mut output = String::with_capacity(mappings.len() * 6);
    let mut generated_line = 0u32;
    let mut generated_column = 0i64;
    let mut source_line = 0i64;
    let mut source_column = 0i64;
    let mut first_in_line = true;
    for mapping in mappings {
        while generated_line < mapping.generated_line {
            output.push(';');
            generated_line += 1;
            generated_column = 0;
            first_in_line = true;
        }
        if !first_in_line {
            output.push(',');
        }
        first_in_line = false;
        push_vlq(
            &mut output,
            i64::from(mapping.generated_column) - generated_column,
        );
        // One source file: its index never changes, so the delta is zero.
        push_vlq(&mut output, 0);
        push_vlq(&mut output, i64::from(mapping.source_line) - source_line);
        push_vlq(
            &mut output,
            i64::from(mapping.source_column) - source_column,
        );
        generated_column = i64::from(mapping.generated_column);
        source_line = i64::from(mapping.source_line);
        source_column = i64::from(mapping.source_column);
    }
    output
}

fn push_vlq(output: &mut String, value: i64) {
    const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut rest = (value.unsigned_abs() << 1) | u64::from(value < 0);
    loop {
        let mut digit = (rest & 0b1_1111) as usize;
        rest >>= 5;
        if rest != 0 {
            digit |= 0b10_0000;
        }
        output.push(char::from(BASE64[digit]));
        if rest == 0 {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::encode;
    use crate::renderer::JsxSourceMapping;

    #[test]
    fn mappings_encode_as_relative_base64_vlq() {
        let mapping =
            |generated_line, generated_column, source_line, source_column| JsxSourceMapping {
                generated_line,
                generated_column,
                source_line,
                source_column,
            };
        let mappings = [
            mapping(0, 0, 0, 0),
            mapping(0, 16, 2, 5),
            mapping(2, 1, 1, 0),
        ];
        assert_eq!(encode(&mappings), "AAAA,gBAEK;;CADL");
    }
}
