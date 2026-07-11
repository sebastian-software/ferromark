use std::{borrow::Cow, fmt, str::FromStr};

const COMMONMARK_5K: &str = include_str!("../../../benches/fixtures/commonmark-5k.md");
const COMMONMARK_20K: &str = include_str!("../../../benches/fixtures/commonmark-20k.md");
const COMMONMARK_50K: &str = include_str!("../../../benches/fixtures/commonmark-50k.md");
const TABLES_5K: &str = include_str!("../../../benches/fixtures/tables-5k.md");

/// Named input corpus used by profiling and diagnostic runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Corpus {
    /// Existing approximately 5 KB CommonMark fixture.
    CommonMark5K,
    /// Existing approximately 20 KB CommonMark fixture.
    CommonMark20K,
    /// Existing approximately 50 KB CommonMark fixture.
    CommonMark50K,
    /// Repeated mixed fixture for scaling and buffer-growth measurements.
    Mixed250K,
    /// Plain prose with little markup.
    Simple,
    /// Fenced and inline-code-heavy input.
    Code,
    /// Safe absolute and relative links.
    SafeUrls,
    /// Many headings whose generated IDs all have distinct base slugs.
    UniqueHeadings,
    /// Many headings that intentionally collide on one generated base slug.
    RepeatedHeadings,
    /// Unsafe and obfuscated URL schemes.
    UnsafeUrls,
    /// Reference definitions and reference links.
    References,
    /// Table-heavy input.
    Tables,
    /// Nested lists and blockquotes.
    Containers,
    /// Nested unordered and ordered lists.
    Lists,
    /// Nested blockquotes with ordinary paragraph content.
    Blockquotes,
    /// Inline delimiter-heavy input.
    Delimiters,
    /// Raw HTML-heavy input.
    Html,
    /// Unicode text and HTML entities.
    UnicodeEntities,
    /// Shared extended-feature input.
    Extended,
}

impl Corpus {
    /// All corpus selectors accepted by the diagnostic runner.
    pub const ALL: [Self; 18] = [
        Self::CommonMark5K,
        Self::CommonMark20K,
        Self::CommonMark50K,
        Self::Mixed250K,
        Self::Simple,
        Self::Code,
        Self::SafeUrls,
        Self::UniqueHeadings,
        Self::RepeatedHeadings,
        Self::UnsafeUrls,
        Self::References,
        Self::Tables,
        Self::Containers,
        Self::Lists,
        Self::Blockquotes,
        Self::Delimiters,
        Self::Html,
        Self::UnicodeEntities,
    ];

    /// Stable command-line and metadata name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CommonMark5K => "commonmark-5k",
            Self::CommonMark20K => "commonmark-20k",
            Self::CommonMark50K => "commonmark-50k",
            Self::Mixed250K => "mixed-250k",
            Self::Simple => "simple",
            Self::Code => "code",
            Self::SafeUrls => "safe-urls",
            Self::UniqueHeadings => "unique-headings",
            Self::RepeatedHeadings => "repeated-headings",
            Self::UnsafeUrls => "unsafe-urls",
            Self::References => "references",
            Self::Tables => "tables",
            Self::Containers => "containers",
            Self::Lists => "lists",
            Self::Blockquotes => "blockquotes",
            Self::Delimiters => "delimiters",
            Self::Html => "html",
            Self::UnicodeEntities => "unicode-entities",
            Self::Extended => "extended",
        }
    }

    /// Materialize the corpus outside any measurement window.
    pub fn materialize(self) -> CorpusData {
        let content = match self {
            Self::CommonMark5K => Cow::Borrowed(COMMONMARK_5K),
            Self::CommonMark20K => Cow::Borrowed(COMMONMARK_20K),
            Self::CommonMark50K => Cow::Borrowed(COMMONMARK_50K),
            Self::Mixed250K => Cow::Owned(repeat_to_at_least(COMMONMARK_50K, 250_000)),
            Self::Simple => Cow::Owned(repeat_to_at_least(
                "Ordinary prose stays deliberately uneventful so fixed parser and rendering costs remain visible.\n\n",
                20_000,
            )),
            Self::Code => Cow::Owned(repeat_to_at_least(
                "```rust\nfn measured(value: usize) -> usize { value + 1 }\n```\n\nUse `measured(41)` in this paragraph.\n\n",
                20_000,
            )),
            Self::SafeUrls => Cow::Owned(repeat_to_at_least(
                "[absolute](https://example.com/a%20path?q=one&v=two) [relative](/docs/start) <mailto:team@example.com>\n\n",
                20_000,
            )),
            Self::UniqueHeadings => Cow::Owned(unique_headings_corpus()),
            Self::RepeatedHeadings => Cow::Owned(repeat_to_at_least(
                "# Shared heading title with inline `code`\n\nParagraph content keeps ordinary block rendering present.\n\n",
                20_000,
            )),
            Self::UnsafeUrls => Cow::Owned(repeat_to_at_least(
                "[script](javascript:alert(1)) [entity](javas&#99;ript:alert(1)) [data](data:text/html,test)\n\n",
                20_000,
            )),
            Self::References => Cow::Owned(reference_corpus()),
            Self::Tables => Cow::Borrowed(TABLES_5K),
            Self::Containers => Cow::Owned(repeat_to_at_least(
                "> - first item\n>   - nested item\n>     1. ordered child\n>     2. second child\n>\n> continuation\n\n",
                20_000,
            )),
            Self::Lists => Cow::Owned(repeat_to_at_least(
                "- first item\n  - nested item\n    1. ordered child\n    2. second child\n  - another nested item\n- second top-level item\n\n",
                20_000,
            )),
            Self::Blockquotes => Cow::Owned(repeat_to_at_least(
                "> outer quote\n> > nested quote\n> > continued nested content\n>\n> continued outer content\n\n",
                20_000,
            )),
            Self::Delimiters => Cow::Owned(repeat_to_at_least(
                "***strong emphasis*** ~~strike~~ `code` [link](https://example.com) _a **b** c_ $x+y$\n\n",
                20_000,
            )),
            Self::Html => Cow::Owned(repeat_to_at_least(
                "<section data-kind=\"sample\"><strong>raw</strong> & content</section>\n\nInline <span title=\"x\">HTML</span>.\n\n",
                20_000,
            )),
            Self::UnicodeEntities => Cow::Owned(repeat_to_at_least(
                "Grüße aus München — 東京 — مرحبا — café &amp; tea &#x1F980; [é](https://example.com/über uns)\n\n",
                20_000,
            )),
            Self::Extended => Cow::Owned(repeat_to_at_least(
                "The result is ^squared^ with $a+b=c$ and a note.[^n]\n\n> [!NOTE]\n> Shared extended syntax.\n\n[^n]: Footnote text.\n\n",
                20_000,
            )),
        };
        CorpusData {
            corpus: self,
            content,
        }
    }
}

impl fmt::Display for Corpus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for Corpus {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .chain([Self::Extended])
            .find(|corpus| corpus.as_str() == value)
            .ok_or_else(|| format!("unknown corpus `{value}`"))
    }
}

/// Materialized corpus data owned outside the measurement window.
pub struct CorpusData {
    corpus: Corpus,
    content: Cow<'static, str>,
}

impl CorpusData {
    /// Corpus selector represented by this data.
    pub const fn corpus(&self) -> Corpus {
        self.corpus
    }

    /// Markdown input.
    pub fn input(&self) -> &str {
        &self.content
    }
}

fn repeat_to_at_least(section: &str, minimum_bytes: usize) -> String {
    let repeats = minimum_bytes.div_ceil(section.len());
    section.repeat(repeats)
}

fn reference_corpus() -> String {
    let mut output = String::with_capacity(24_000);
    for index in 0..256 {
        output.push_str("Read [the reference][ref-");
        output.push_str(&index.to_string());
        output.push_str("] and [another][ref-");
        output.push_str(&index.to_string());
        output.push_str("].\n\n");
    }
    for index in 0..256 {
        output.push_str("[ref-");
        output.push_str(&index.to_string());
        output.push_str("]: https://example.com/reference/");
        output.push_str(&index.to_string());
        output.push('\n');
    }
    output
}

fn unique_headings_corpus() -> String {
    let mut output = String::with_capacity(24_000);
    for index in 0..512 {
        output.push_str("# Unique heading ");
        output.push_str(&index.to_string());
        output.push_str(" with inline `code`\n\nParagraph content keeps ordinary block rendering present.\n\n");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaling_corpus_should_reach_minimum_size() {
        assert!(Corpus::Mixed250K.materialize().input().len() >= 250_000);
    }

    #[test]
    fn every_corpus_should_be_non_empty() {
        assert!(
            Corpus::ALL
                .into_iter()
                .chain([Corpus::Extended])
                .all(|corpus| !corpus.materialize().input().is_empty())
        );
    }

    #[test]
    fn heading_corpora_should_exercise_their_collision_modes() {
        let unique = Corpus::UniqueHeadings.materialize();
        let repeated = Corpus::RepeatedHeadings.materialize();

        assert!(unique.input().contains("Unique heading 0"));
        assert!(unique.input().contains("Unique heading 511"));
        assert!(repeated.input().matches("# Shared heading title").count() > 100);
    }
}
