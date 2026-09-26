//! Optional links for explicit GitHub issue, user, team, and commit references.

use std::error::Error;
use std::fmt;
use std::ops::Range;

use ferromark::ast::{Document, Span};

use crate::prose::{NodeList, is_in_raw_html, visit_document_prose};
use crate::{BoxError, TransformContext, TransformPass, text_runs};

const MIN_COMMIT_LENGTH: usize = 7;
const MAX_COMMIT_LENGTH: usize = 40;
const MAX_OWNER_LENGTH: usize = 39;
const MAX_REPOSITORY_LENGTH: usize = 100;
const DENIED_BARE_HASHES: [&str; 5] = ["acceded", "deedeed", "defaced", "effaced", "fabaceae"];

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

struct ReferenceMatch {
    node_range: Range<usize>,
    byte_range: Range<usize>,
    label: String,
    url: String,
}

fn transform_text_runs<'arena>(
    nodes: &mut NodeList<'arena>,
    context: &TransformContext<'arena>,
    raw_html_spans: &[Span],
    local_owner: &str,
    local_repository: &str,
) -> Result<(), BoxError> {
    let mut matches = Vec::new();
    for run in text_runs(nodes) {
        if is_in_raw_html(&run, raw_html_spans) {
            continue;
        }
        let value = run.value();
        let protected = run.protected_url_ranges(context);
        for matched in find_references(&value, &protected, local_owner, local_repository) {
            matches.push(ReferenceMatch {
                node_range: run.node_range(),
                label: value[matched.range.clone()].to_owned(),
                byte_range: matched.range,
                url: matched.url,
            });
        }
    }

    let mut active_run = None;
    let mut boundary = 0;
    for matched in matches.into_iter().rev() {
        let start = matched.node_range.start;
        if active_run != Some(start) {
            active_run = Some(start);
            boundary = matched.node_range.end;
        }
        let replacement_span = TransformContext::replace_text_range_with_node(
            nodes,
            start..boundary,
            matched.byte_range,
            |span| Some(context.text_link_node(&matched.label, &matched.url, span)),
        )?;
        boundary = nodes
            .iter()
            .position(
                |node| matches!(node, ferromark::ast::Node::Link(link) if link.span == replacement_span),
            )
            .ok_or_else(|| {
                std::io::Error::other("GitHub reference link node was not inserted")
            })?;
    }

    Ok(())
}

struct TextMatch {
    range: Range<usize>,
    url: String,
}

fn find_references(
    value: &str,
    protected: &[Range<usize>],
    local_owner: &str,
    local_repository: &str,
) -> Vec<TextMatch> {
    let bytes = value.as_bytes();
    let mut matches = Vec::new();
    let mut cursor = 0;

    while cursor < bytes.len() {
        if let Some(range) = protected
            .iter()
            .find(|range| range.start <= cursor && cursor < range.end)
        {
            cursor = range.end;
            continue;
        }

        if let Some((end, url)) = match_reference(value, cursor, local_owner, local_repository)
            && !protected
                .iter()
                .any(|range| cursor < range.end && range.start < end)
        {
            matches.push(TextMatch {
                range: cursor..end,
                url,
            });
            cursor = end;
        } else {
            cursor += value[cursor..].chars().next().map_or(1, char::len_utf8);
        }
    }

    matches
}

fn match_reference(
    value: &str,
    start: usize,
    local_owner: &str,
    local_repository: &str,
) -> Option<(usize, String)> {
    let bytes = value.as_bytes();
    if !left_boundary(value, start) {
        return None;
    }

    if let Some((end, owner, repository, reference)) =
        parse_cross_repository_reference(bytes, start)
        && right_boundary(value, end)
    {
        let url = match reference {
            RepositoryReference::Issue(number) => issue_url(owner, repository, number),
            RepositoryReference::Commit(hash) => commit_url(owner, repository, hash),
        };
        return Some((end, url));
    }

    if bytes.get(start) == Some(&b'@')
        && let Some((end, username, team)) = parse_mention(bytes, start)
        && left_boundary(value, start)
        && mention_right_boundary(value, end, team)
        && !matches_ignore_ascii_case(username, &["mention", "mentions"])
    {
        return Some((end, format!("https://github.com/{username}")));
    }

    if is_gh_prefix(bytes, start)
        && let Some((end, number)) = parse_issue_number(bytes, start + 3)
        && right_boundary(value, end)
    {
        return Some((end, issue_url(local_owner, local_repository, number)));
    }

    if bytes.get(start) == Some(&b'#')
        && let Some((end, number)) = parse_issue_number(bytes, start + 1)
        && right_boundary(value, end)
    {
        return Some((end, issue_url(local_owner, local_repository, number)));
    }

    if let Some((end, base, compare)) = parse_commit_range(bytes, start)
        && right_boundary(value, end)
    {
        return Some((
            end,
            format!(
                "https://github.com/{local_owner}/{local_repository}/compare/{base}...{compare}"
            ),
        ));
    }

    if let Some((end, hash)) = parse_commit_hash(bytes, start)
        && right_boundary(value, end)
        && !matches_ignore_ascii_case(hash, &DENIED_BARE_HASHES)
        && !bytes
            .get(end..)
            .is_some_and(|suffix| suffix.starts_with(b"..."))
    {
        return Some((end, commit_url(local_owner, local_repository, hash)));
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

fn parse_commit_range(bytes: &[u8], start: usize) -> Option<(usize, &str, &str)> {
    let (base_end, base) = parse_commit_hash(bytes, start)?;
    if bytes
        .get(base_end..base_end + 3)
        .is_none_or(|separator| separator != b"...")
    {
        return None;
    }
    let (end, compare) = parse_commit_hash(bytes, base_end + 3)?;
    Some((end, base, compare))
}

fn is_gh_prefix(bytes: &[u8], start: usize) -> bool {
    bytes
        .get(start..start + 3)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"gh-"))
}

fn left_boundary(value: &str, start: usize) -> bool {
    value[..start].chars().next_back().is_none_or(|previous| {
        !previous.is_alphanumeric() && !matches!(previous, '_' | '-' | '.' | '/' | '@' | '\\')
    })
}

fn right_boundary(value: &str, end: usize) -> bool {
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

fn issue_url(owner: &str, repository: &str, number: &str) -> String {
    format!("https://github.com/{owner}/{repository}/issues/{number}")
}

fn commit_url(owner: &str, repository: &str, hash: &str) -> String {
    format!("https://github.com/{owner}/{repository}/commit/{hash}")
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
