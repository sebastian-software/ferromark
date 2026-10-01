//! Fence metadata used by the native JSX code-block renderer.

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CodeBlockMetadata {
    pub language: Option<String>,
    pub title: Option<String>,
    pub label: Option<String>,
    pub line_number_start: Option<usize>,
    pub highlighted_lines: Vec<bool>,
}

pub(super) fn parse_code_block_metadata(
    language: Option<&str>,
    meta: Option<&str>,
    code: &str,
    show_line_numbers: bool,
) -> CodeBlockMetadata {
    let (language, inline_meta) = normalize_language(language);
    let mut tokens = Vec::new();
    if !inline_meta.is_empty() {
        tokens.extend(split_metadata(&inline_meta));
    }
    if let Some(meta) = meta.map(str::trim).filter(|meta| !meta.is_empty()) {
        tokens.extend(split_metadata(meta));
    }

    let mut title = None;
    let mut label = None;
    let mut line_number_start = show_line_numbers.then_some(1_usize);
    let mut highlight_ranges = Vec::new();
    for token in tokens {
        match token {
            MetadataToken::Brackets(value) => {
                let value = value.trim();
                if !value.is_empty() && label.is_none() {
                    label = Some(value.to_owned());
                    if title.is_none() {
                        title = Some(value.to_owned());
                    }
                }
            }
            MetadataToken::Braces(value) => highlight_ranges.push(value.to_owned()),
            MetadataToken::Raw(value) => {
                if let Some(value) = value.strip_prefix("title=") {
                    let value = unquote(value);
                    if !value.is_empty() {
                        title = Some(value.to_owned());
                    }
                } else if value == ":line-numbers" || value == "showLineNumbers" {
                    line_number_start = Some(1);
                } else if let Some(start) = value
                    .strip_prefix(":line-numbers=")
                    .and_then(parse_positive_number)
                {
                    line_number_start = Some(start);
                } else if value == ":no-line-numbers" || value == "noLineNumbers" {
                    line_number_start = None;
                }
            }
        }
    }

    let line_count = code.split('\n').count();
    let mut highlighted_lines = Vec::with_capacity(line_count);
    highlighted_lines.resize(line_count, false);
    for ranges in highlight_ranges {
        apply_line_ranges(&mut highlighted_lines, &ranges);
    }

    CodeBlockMetadata {
        language,
        title,
        label,
        line_number_start,
        highlighted_lines,
    }
}

fn normalize_language(language: Option<&str>) -> (Option<String>, String) {
    let Some(language) = language
        .map(str::trim)
        .filter(|language| !language.is_empty())
    else {
        return (None, String::new());
    };
    let end = language
        .char_indices()
        .find_map(|(index, character)| match character {
            '{' | '[' => Some(index),
            ':' if starts_metadata_suffix(&language[index..]) => Some(index),
            _ => None,
        })
        .unwrap_or(language.len());
    let name = language[..end].trim();
    let suffix = language[end..].trim();
    let normalized = (!name.is_empty()).then(|| name.to_owned());
    (normalized, suffix.to_owned())
}

fn starts_metadata_suffix(value: &str) -> bool {
    [":no-line-numbers", ":line-numbers", ":wrap", ":wrap-lines"]
        .iter()
        .any(|suffix| value.starts_with(suffix))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MetadataToken<'a> {
    Raw(&'a str),
    Braces(&'a str),
    Brackets(&'a str),
}

fn split_metadata(meta: &str) -> Vec<MetadataToken<'_>> {
    let bytes = meta.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if index == bytes.len() {
            break;
        }
        match bytes[index] {
            b'{' | b'[' => {
                let opening = bytes[index];
                let closing = if opening == b'{' { b'}' } else { b']' };
                let start = index + 1;
                index += 1;
                while index < bytes.len() && bytes[index] != closing {
                    index += 1;
                }
                let value = &meta[start..index];
                tokens.push(if opening == b'{' {
                    MetadataToken::Braces(value)
                } else {
                    MetadataToken::Brackets(value)
                });
                if index < bytes.len() {
                    index += 1;
                }
            }
            _ => {
                let start = index;
                let mut quote = None;
                while index < bytes.len() {
                    let byte = bytes[index];
                    if let Some(current) = quote {
                        if byte == current {
                            quote = None;
                        }
                        index += 1;
                        continue;
                    }
                    if byte == b'"' || byte == b'\'' {
                        quote = Some(byte);
                        index += 1;
                        continue;
                    }
                    if byte.is_ascii_whitespace() || byte == b'{' || byte == b'[' {
                        break;
                    }
                    index += 1;
                }
                tokens.push(MetadataToken::Raw(&meta[start..index]));
            }
        }
    }
    tokens
}

fn unquote(value: &str) -> &str {
    let value = value.trim();
    let Some(quote @ (b'"' | b'\'')) = value.as_bytes().first().copied() else {
        return value;
    };
    if value.as_bytes().last() == Some(&quote) && value.len() >= 2 {
        &value[1..value.len() - 1]
    } else {
        value
    }
}

fn parse_positive_number(value: &str) -> Option<usize> {
    value.parse::<usize>().ok().filter(|number| *number > 0)
}

fn apply_line_ranges(lines: &mut [bool], value: &str) {
    for part in value
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
    {
        if let Some((raw_start, raw_end)) = part.split_once('-') {
            let (Some(start), Some(end)) = (
                parse_positive_number(raw_start),
                parse_positive_number(raw_end),
            ) else {
                continue;
            };
            if end < start || start > lines.len() {
                continue;
            }
            let upper = end.min(lines.len());
            for selected in lines.iter_mut().take(upper).skip(start - 1) {
                *selected = true;
            }
            continue;
        }
        if let Some(number) = parse_positive_number(part)
            && let Some(selected) = lines.get_mut(number - 1)
        {
            *selected = true;
        }
    }
}
