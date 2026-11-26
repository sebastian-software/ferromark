//! Optional links for explicit GitHub issue, user, team, and commit references.

use std::borrow::Cow;
use std::error::Error;
use std::fmt::{self, Write as _};
use std::ops::Range;

use ferromark::ast::{Document, Span};

use crate::prose::{NodeList, is_in_raw_html, visit_document_prose};
use crate::{BoxError, RunReplacement, TransformContext, TransformPass, text_runs};

const MIN_COMMIT_LENGTH: usize = 7;
const MAX_COMMIT_LENGTH: usize = 40;
const MAX_OWNER_LENGTH: usize = 39;
const MAX_REPOSITORY_LENGTH: usize = 100;
const DENIED_BARE_HASHES: [&str; 5] = ["acceded", "deedeed", "defaced", "effaced", "fabaceae"];

const HAS_HASH: u8 = 1 << 0;
const HAS_AT: u8 = 1 << 1;
const HAS_SLASH: u8 = 1 << 2;
const HAS_GH: u8 = 1 << 3;
const HAS_HEX7: u8 = 1 << 4;
const ALL_CANDIDATES: u8 = HAS_HASH | HAS_AT | HAS_SLASH | HAS_GH | HAS_HEX7;

/// Error returned when a configured GitHub repository is not a safe `owner/repo` path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidGitHubRepository {
    value: String,
}

impl InvalidGitHubRepository {
    /// Returns the rejected repository configuration.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl fmt::Display for InvalidGitHubRepository {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid GitHub repository {:?}; expected an explicit owner/repository with safe ASCII path segments",
            self.value
        )
    }
}

impl Error for InvalidGitHubRepository {}

/// Links a qualified subset of GitHub references using an explicit repository.
///
/// Supported forms are local `#123` and `GH-123` issues, `owner/repo#123`,
/// `@user`, `@org/team`, 7–40 character hexadecimal commits and `start...end`
/// commit ranges. Link labels preserve the exact authored text. Existing links,
/// link destinations and titles, image metadata, code, math, raw HTML, MDX
/// expressions and renderer-recognized bare URLs stay unchanged. No GitHub API,
/// repository inference, callback, or filesystem access is used.
pub struct GitHubReferencesPass {
    owner: String,
    repository: String,
}

impl GitHubReferencesPass {
    /// Creates a pass for an explicit `owner/repository` pair.
    ///
    /// Owner and repository segments are validated before the pass can mutate a
    /// document. The repository is never inferred from package metadata or Git.
    pub fn new(repository: &str) -> Result<Self, InvalidGitHubRepository> {
        let Some((owner, name)) = validated_repository(repository) else {
            return Err(InvalidGitHubRepository {
                value: repository.to_owned(),
            });
        };

        Ok(Self {
            owner: owner.to_owned(),
            repository: name.to_owned(),
        })
    }
}

impl TransformPass for GitHubReferencesPass {
    fn name(&self) -> &'static str {
        "github-references"
    }

    fn apply<'arena>(
        &mut self,
        document: &mut Document<'arena>,
        context: &TransformContext<'arena>,
    ) -> Result<(), BoxError> {
        let owner = &self.owner;
        let repository = &self.repository;
        let mut visitor = |nodes: &mut NodeList<'arena>,
                           context: &TransformContext<'arena>,
                           raw_html_spans: &[Span]| {
            transform_text_runs(nodes, context, raw_html_spans, owner, repository)
        };
        visit_document_prose(document, context, false, &mut visitor)
    }
}

struct ReferencePayload<'arena> {
    label: &'arena str,
    url: &'arena str,
}

fn transform_text_runs<'arena>(
    nodes: &mut NodeList<'arena>,
    context: &TransformContext<'arena>,
    raw_html_spans: &[Span],
    local_owner: &str,
    local_repository: &str,
) -> Result<(), BoxError> {
    let mut matches: Vec<RunReplacement<ReferencePayload<'arena>>> = Vec::new();
    let mut url_buf = String::new();
    for run in text_runs(nodes) {
        if is_in_raw_html(&run, raw_html_spans) {
            continue;
        }
        let value = run.value();
        let flags = scan_github_candidates(value.as_bytes());
        if flags == 0 {
            continue;
        }
        let protected = run.protected_url_ranges(context);
        let node_range = run.node_range();
        scan_references_into(
            &value,
            flags,
            &protected,
            local_owner,
            local_repository,
            &mut url_buf,
            |byte_range, url| {
                let label = match &value {
                    Cow::Borrowed(borrowed) => &borrowed[byte_range.clone()],
                    Cow::Owned(owned) => context.alloc_str(&owned[byte_range.clone()]),
                };
                matches.push(RunReplacement {
                    node_range: node_range.clone(),
                    byte_range,
                    payload: ReferencePayload {
                        label,
                        url: context.alloc_str(url),
                    },
                });
            },
        );
    }

    context.apply_text_run_replacements(nodes, matches, |payload, span| {
        context.text_link_node_allocated(payload.label, payload.url, span)
    });

    Ok(())
}

/// Fast single-pass pre-filter returning which reference kinds can possibly
/// appear in `bytes`. A GitHub reference requires `#`, `@`, `/`, `GH-`/`gh-`,
/// or a contiguous 7+ ASCII hex-digit run.
fn scan_github_candidates(bytes: &[u8]) -> u8 {
    let mut flags = 0u8;
    let mut hex_run = 0usize;
    for (i, &byte) in bytes.iter().enumerate() {
        match byte {
            b'#' => {
                flags |= HAS_HASH;
                hex_run = 0;
            }
            b'@' => {
                flags |= HAS_AT;
                hex_run = 0;
            }
            b'/' => {
                flags |= HAS_SLASH;
                hex_run = 0;
            }
            _ => {
                if (flags & HAS_GH) == 0
                    && matches!(byte, b'g' | b'G')
                    && bytes.get(i + 1).is_some_and(|b| matches!(b, b'h' | b'H'))
                    && bytes.get(i + 2) == Some(&b'-')
                {
                    flags |= HAS_GH;
                }
                if (flags & HAS_HEX7) == 0 {
                    if byte.is_ascii_hexdigit() {
                        hex_run += 1;
                        if hex_run >= MIN_COMMIT_LENGTH {
                            flags |= HAS_HEX7;
                        }
                    } else {
                        hex_run = 0;
                    }
                }
            }
        }
        if flags == ALL_CANDIDATES {
            break;
        }
    }
    flags
}

#[cfg(test)]
struct TextMatch {
    range: Range<usize>,
    url: String,
}

#[cfg(test)]
fn find_references(
    value: &str,
    protected: &[Range<usize>],
    local_owner: &str,
    local_repository: &str,
) -> Vec<TextMatch> {
    let flags = scan_github_candidates(value.as_bytes());
    if flags == 0 {
        return Vec::new();
    }
    let mut matches = Vec::new();
    let mut url_buf = String::new();
    scan_references_into(
        value,
        flags,
        protected,
        local_owner,
        local_repository,
        &mut url_buf,
        |range, url| {
            matches.push(TextMatch {
                range,
                url: url.to_owned(),
            });
        },
    );
    matches
}

fn next_trigger_offset(slice: &str, flags: u8) -> Option<usize> {
    match flags & (HAS_HASH | HAS_AT) {
        HAS_HASH => slice.find('#'),
        HAS_AT => slice.find('@'),
        _ => slice.find(['#', '@']),
    }
}

fn scan_references_into(
    value: &str,
    flags: u8,
    protected: &[Range<usize>],
    local_owner: &str,
    local_repository: &str,
    url_buf: &mut String,
    mut on_match: impl FnMut(Range<usize>, &str),
) {
    let bytes = value.as_bytes();
    let mut cursor = 0usize;
    let mut protected_idx = 0usize;

    // Fast path: when only `#` and/or `@` candidates exist, jump directly
    // between those bytes instead of inspecting every character.
    if (flags & (HAS_SLASH | HAS_GH | HAS_HEX7)) == 0 {
        while let Some(rel) = next_trigger_offset(&value[cursor..], flags) {
            cursor += rel;
            while protected_idx < protected.len() && protected[protected_idx].end <= cursor {
                protected_idx += 1;
            }
            if let Some(range) = protected.get(protected_idx)
                && range.start <= cursor
                && cursor < range.end
            {
                cursor = range.end;
                continue;
            }

            if left_boundary(value, cursor)
                && let Some(end) =
                    match_reference_at(value, cursor, flags, local_owner, local_repository, url_buf)
                && !protected
                    .get(protected_idx)
                    .is_some_and(|range| cursor < range.end && range.start < end)
            {
                on_match(cursor..end, url_buf.as_str());
                cursor = end;
            } else {
                cursor += 1;
            }
        }
        return;
    }

    let mut can_start = true;
    while cursor < bytes.len() {
        while protected_idx < protected.len() && protected[protected_idx].end <= cursor {
            protected_idx += 1;
        }
        if let Some(range) = protected.get(protected_idx)
            && range.start <= cursor
            && cursor < range.end
        {
            cursor = range.end;
            can_start = left_boundary(value, cursor);
            continue;
        }

        let byte = bytes[cursor];
        if can_start
            && (matches!(byte, b'@' | b'#') || byte.is_ascii_alphanumeric())
            && let Some(end) =
                match_reference_at(value, cursor, flags, local_owner, local_repository, url_buf)
            && !protected
                .get(protected_idx)
                .is_some_and(|range| cursor < range.end && range.start < end)
        {
            on_match(cursor..end, url_buf.as_str());
            cursor = end;
            can_start = left_boundary(value, cursor);
            continue;
        }

        if byte < 0x80 {
            can_start = is_ascii_left_boundary(byte);
            cursor += 1;
        } else if let Some(ch) = value[cursor..].chars().next() {
            can_start = is_left_boundary_char(ch);
            cursor += ch.len_utf8();
        } else {
            break;
        }
    }
}

fn match_reference_at(
    value: &str,
    start: usize,
    flags: u8,
    local_owner: &str,
    local_repository: &str,
    url_buf: &mut String,
) -> Option<usize> {
    let bytes = value.as_bytes();
    let first = *bytes.get(start)?;

    if first == b'@' {
        if (flags & HAS_AT) != 0
            && let Some((end, username, team)) = parse_mention(bytes, start)
            && mention_right_boundary(value, end, team)
            && !matches_ignore_ascii_case(username, &["mention", "mentions"])
        {
            url_buf.clear();
            let _ = write!(url_buf, "https://github.com/{username}");
            return Some(end);
        }
        return None;
    }

    if first == b'#' {
        if (flags & HAS_HASH) != 0
            && let Some((end, number)) = parse_issue_number(bytes, start + 1)
            && right_boundary(value, end)
        {
            write_issue_url(url_buf, local_owner, local_repository, number);
            return Some(end);
        }
        return None;
    }

    if (flags & HAS_SLASH) != 0
        && (flags & (HAS_HASH | HAS_AT)) != 0
        && let Some((end, owner, repository, reference)) =
            parse_cross_repository_reference(bytes, start)
        && right_boundary(value, end)
    {
        match reference {
            RepositoryReference::Issue(number) => {
                write_issue_url(url_buf, owner, repository, number);
            }
            RepositoryReference::Commit(hash) => {
                write_commit_url(url_buf, owner, repository, hash);
            }
        }
        return Some(end);
    }

    if (flags & HAS_GH) != 0
        && matches!(first, b'g' | b'G')
        && is_gh_prefix(bytes, start)
        && let Some((end, number)) = parse_issue_number(bytes, start + 3)
        && right_boundary(value, end)
    {
        write_issue_url(url_buf, local_owner, local_repository, number);
        return Some(end);
    }

    if (flags & HAS_HEX7) != 0
        && first.is_ascii_hexdigit()
        && let Some((base_end, base)) = parse_commit_hash(bytes, start)
    {
        if bytes.get(base_end..base_end + 3) == Some(b"...") {
            if let Some((end, compare)) = parse_commit_hash(bytes, base_end + 3)
                && right_boundary(value, end)
            {
                url_buf.clear();
                let _ = write!(
                    url_buf,
                    "https://github.com/{local_owner}/{local_repository}/compare/{base}...{compare}"
                );
                return Some(end);
            }
            return None;
        }

        if right_boundary(value, base_end) && !matches_ignore_ascii_case(base, &DENIED_BARE_HASHES)
        {
            write_commit_url(url_buf, local_owner, local_repository, base);
            return Some(base_end);
        }
    }

    None
}

enum RepositoryReference<'a> {
    Issue(&'a str),
    Commit(&'a str),
}

fn parse_cross_repository_reference(
    bytes: &[u8],
    start: usize,
) -> Option<(usize, &str, &str, RepositoryReference<'_>)> {
    let owner_end = parse_account(bytes, start)?;
    if bytes.get(owner_end) != Some(&b'/') {
        return None;
    }
    let repository_start = owner_end + 1;
    let repository_end = parse_repository_name(bytes, repository_start)?;
    let owner = std::str::from_utf8(&bytes[start..owner_end]).ok()?;
    let repository = std::str::from_utf8(&bytes[repository_start..repository_end]).ok()?;

    match bytes.get(repository_end) {
        Some(&b'#') => {
            let (end, number) = parse_issue_number(bytes, repository_end + 1)?;
            Some((end, owner, repository, RepositoryReference::Issue(number)))
        }
        Some(&b'@') => {
            let (end, hash) = parse_commit_hash(bytes, repository_end + 1)?;
            Some((end, owner, repository, RepositoryReference::Commit(hash)))
        }
        _ => None,
    }
}

fn parse_mention(bytes: &[u8], start: usize) -> Option<(usize, &str, bool)> {
    let user_start = start + 1;
    let user_end = parse_account(bytes, user_start)?;
    if bytes.get(user_end) == Some(&b'/') {
        let team_start = user_end + 1;
        let team_end = parse_account(bytes, team_start)?;
        let mention = std::str::from_utf8(&bytes[user_start..team_end]).ok()?;
        return Some((team_end, mention, true));
    }

    Some((
        user_end,
        std::str::from_utf8(&bytes[user_start..user_end]).ok()?,
        false,
    ))
}

fn parse_account(bytes: &[u8], start: usize) -> Option<usize> {
    if !bytes.get(start).is_some_and(u8::is_ascii_alphanumeric) {
        return None;
    }
    let mut cursor = start + 1;
    while bytes
        .get(cursor)
        .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'-')
    {
        cursor += 1;
    }
    if cursor - start > MAX_OWNER_LENGTH || bytes.get(cursor.wrapping_sub(1)) == Some(&b'-') {
        return None;
    }
    Some(cursor)
}

fn parse_repository_name(bytes: &[u8], start: usize) -> Option<usize> {
    let mut cursor = start;
    while bytes
        .get(cursor)
        .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(*byte, b'-' | b'_' | b'.'))
    {
        cursor += 1;
    }
    let length = cursor.checked_sub(start)?;
    if length == 0 || length > MAX_REPOSITORY_LENGTH {
        return None;
    }
    if &bytes[start..cursor] == b"." || &bytes[start..cursor] == b".." {
        return None;
    }
    Some(cursor)
}

fn parse_issue_number(bytes: &[u8], start: usize) -> Option<(usize, &str)> {
    if !bytes
        .get(start)
        .is_some_and(|byte| byte.is_ascii_digit() && *byte != b'0')
    {
        return None;
    }
    let mut cursor = start + 1;
    while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
        cursor += 1;
    }
    Some((cursor, std::str::from_utf8(&bytes[start..cursor]).ok()?))
}

fn parse_commit_hash(bytes: &[u8], start: usize) -> Option<(usize, &str)> {
    if !bytes.get(start).is_some_and(u8::is_ascii_hexdigit) {
        return None;
    }
    let mut cursor = start + 1;
    while bytes.get(cursor).is_some_and(u8::is_ascii_hexdigit) {
        cursor += 1;
    }
    let length = cursor - start;
    if !(MIN_COMMIT_LENGTH..=MAX_COMMIT_LENGTH).contains(&length) {
        return None;
    }
    Some((cursor, std::str::from_utf8(&bytes[start..cursor]).ok()?))
}

fn is_gh_prefix(bytes: &[u8], start: usize) -> bool {
    bytes
        .get(start..start + 3)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"gh-"))
}

const fn is_ascii_left_boundary(byte: u8) -> bool {
    !byte.is_ascii_alphanumeric() && !matches!(byte, b'_' | b'-' | b'.' | b'/' | b'@' | b'\\')
}

fn is_left_boundary_char(previous: char) -> bool {
    !previous.is_alphanumeric() && !matches!(previous, '_' | '-' | '.' | '/' | '@' | '\\')
}

fn left_boundary(value: &str, start: usize) -> bool {
    if start == 0 {
        return true;
    }
    let prev_byte = value.as_bytes()[start - 1];
    if prev_byte < 0x80 {
        return is_ascii_left_boundary(prev_byte);
    }
    value[..start]
        .chars()
        .next_back()
        .is_none_or(is_left_boundary_char)
}

fn right_boundary(value: &str, end: usize) -> bool {
    let Some(&next_byte) = value.as_bytes().get(end) else {
        return true;
    };
    if next_byte < 0x80 {
        return !next_byte.is_ascii_alphanumeric() && !matches!(next_byte, b'_' | b'-' | b'/');
    }
    value[end..]
        .chars()
        .next()
        .is_none_or(|next| !next.is_alphanumeric() && !matches!(next, '_' | '-' | '/'))
}

fn mention_right_boundary(value: &str, end: usize, team: bool) -> bool {
    if !right_boundary(value, end) {
        return false;
    }
    if team {
        return true;
    }

    let mut suffix = value[end..].chars();
    match suffix.next() {
        Some('@') => false,
        Some('.') => !suffix.next().is_some_and(char::is_alphanumeric),
        _ => true,
    }
}

fn matches_ignore_ascii_case(value: &str, candidates: &[&str]) -> bool {
    candidates
        .iter()
        .any(|candidate| value.eq_ignore_ascii_case(candidate))
}

fn validated_repository(value: &str) -> Option<(&str, &str)> {
    let (owner, repository) = value.split_once('/')?;
    if repository.contains('/') || owner.is_empty() || repository.is_empty() {
        return None;
    }
    let owner_bytes = owner.as_bytes();
    let repository_bytes = repository.as_bytes();
    let owner_end = parse_account(owner_bytes, 0)?;
    let repository_end = parse_repository_name(repository_bytes, 0)?;
    (owner_end == owner.len() && repository_end == repository.len()).then_some((owner, repository))
}

fn write_issue_url(url_buf: &mut String, owner: &str, repository: &str, number: &str) {
    url_buf.clear();
    let _ = write!(
        url_buf,
        "https://github.com/{owner}/{repository}/issues/{number}"
    );
}

fn write_commit_url(url_buf: &mut String, owner: &str, repository: &str, hash: &str) {
    url_buf.clear();
    let _ = write!(
        url_buf,
        "https://github.com/{owner}/{repository}/commit/{hash}"
    );
}

#[cfg(test)]
mod tests {
    use super::{GitHubReferencesPass, find_references};

    fn references(source: &str) -> Vec<(String, String)> {
        find_references(source, &[], "ferromark", "fixtures")
            .into_iter()
            .map(|matched| (source[matched.range].to_owned(), matched.url))
            .collect()
    }

    #[test]
    fn validates_explicit_repository_path_segments() {
        assert!(GitHubReferencesPass::new("ferromark/fixtures").is_ok());
        for invalid in [
            "",
            "ferromark",
            "/fixtures",
            "ferromark/",
            "ferromark/a/b",
            "ferromark/..",
            "https://github.com/ferromark/fixtures",
            "ferromark/repo?query",
        ] {
            assert!(GitHubReferencesPass::new(invalid).is_err(), "{invalid:?}");
        }
    }

    #[test]
    fn links_the_qualified_reference_forms_without_changing_labels() {
        assert_eq!(
            references(
                "Fixes #123 and closes gH-7; see org/project#9 and @user, @org/team.\nCommit 1f2a4fb and range e2acebc...2aa9311."
            ),
            vec![
                (
                    "#123".to_owned(),
                    "https://github.com/ferromark/fixtures/issues/123".to_owned()
                ),
                (
                    "gH-7".to_owned(),
                    "https://github.com/ferromark/fixtures/issues/7".to_owned()
                ),
                (
                    "org/project#9".to_owned(),
                    "https://github.com/org/project/issues/9".to_owned()
                ),
                ("@user".to_owned(), "https://github.com/user".to_owned()),
                (
                    "@org/team".to_owned(),
                    "https://github.com/org/team".to_owned()
                ),
                (
                    "1f2a4fb".to_owned(),
                    "https://github.com/ferromark/fixtures/commit/1f2a4fb".to_owned()
                ),
                (
                    "e2acebc...2aa9311".to_owned(),
                    "https://github.com/ferromark/fixtures/compare/e2acebc...2aa9311".to_owned()
                ),
            ]
        );
    }

    #[test]
    fn excludes_emails_hash_like_words_urls_and_invalid_boundaries() {
        assert_eq!(
            references(
                "foo@bar.com; @mention; @mentions; x#123; #0; #123word; acceded; a1b2c3; https://example.com/x#123."
            ),
            Vec::new()
        );
        let protected_url = 0..30;
        assert_eq!(
            find_references(
                "See https://example.com/x#123.",
                std::slice::from_ref(&protected_url),
                "ferromark",
                "fixtures"
            )
            .len(),
            0
        );
    }

    #[test]
    fn preserves_large_issue_numbers_without_integer_conversion() {
        let number = "999999999999999999999999999999999999999999999999";
        assert_eq!(
            references(&format!("Fixes #{number}.")),
            vec![(
                format!("#{number}"),
                format!("https://github.com/ferromark/fixtures/issues/{number}")
            )]
        );
    }

    #[test]
    fn links_cross_repository_commits_and_the_supported_hash_lengths() {
        let long_hash = "0123456789abcdef0123456789abcdef01234567";
        let source = format!("org/my_repo@AbCd123 {long_hash}");
        assert_eq!(
            references(&source),
            vec![
                (
                    "org/my_repo@AbCd123".to_owned(),
                    "https://github.com/org/my_repo/commit/AbCd123".to_owned()
                ),
                (
                    long_hash.to_owned(),
                    format!("https://github.com/ferromark/fixtures/commit/{long_hash}")
                ),
            ]
        );
    }

    #[test]
    fn scans_past_multibyte_prose_before_ascii_references() {
        assert_eq!(
            references("Grüße: #123"),
            vec![(
                "#123".to_owned(),
                "https://github.com/ferromark/fixtures/issues/123".to_owned()
            )]
        );
    }
}
