use std::{fmt, str::FromStr};

use ferromark::{Options, RenderPolicy};

use crate::{ParityConfig, ferromark_options, pulldown_options};

/// Parser implementation selected for a diagnostic run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserKind {
    /// Ferromark using an explicit parity or product configuration.
    Ferromark,
    /// pulldown-cmark using a semantically guarded parity configuration.
    PulldownCmark,
}

impl ParserKind {
    /// Stable command-line and metadata name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ferromark => "ferromark",
            Self::PulldownCmark => "pulldown-cmark",
        }
    }
}

impl fmt::Display for ParserKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for ParserKind {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "ferromark" => Ok(Self::Ferromark),
            "pulldown-cmark" | "pulldown" => Ok(Self::PulldownCmark),
            _ => Err(format!("unknown parser `{value}`")),
        }
    }
}

/// Exact configuration selected for one benchmark or diagnostic run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunConfig {
    /// Trusted CommonMark parity configuration.
    CommonMark,
    /// Trusted GFM-overlap parity configuration.
    GfmOverlap,
    /// Trusted extended-overlap parity configuration.
    ExtendedOverlap,
    /// Secure Ferromark minimal configuration.
    MinimalSecure,
    /// Trusted Ferromark minimal configuration.
    MinimalTrusted,
    /// Secure Ferromark default configuration.
    DefaultSecure,
    /// Secure default configuration with heading IDs explicitly disabled.
    DefaultSecureNoHeadingIds,
    /// Trusted Ferromark default configuration.
    DefaultTrusted,
    /// Secure Ferromark configuration with every extension enabled.
    AllExtensionsSecure,
    /// Trusted Ferromark configuration with every extension enabled.
    AllExtensionsTrusted,
}

impl RunConfig {
    /// All configurations accepted by the diagnostic runner.
    pub const ALL: [Self; 10] = [
        Self::CommonMark,
        Self::GfmOverlap,
        Self::ExtendedOverlap,
        Self::MinimalSecure,
        Self::MinimalTrusted,
        Self::DefaultSecure,
        Self::DefaultSecureNoHeadingIds,
        Self::DefaultTrusted,
        Self::AllExtensionsSecure,
        Self::AllExtensionsTrusted,
    ];

    /// Stable command-line and metadata name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CommonMark => "commonmark",
            Self::GfmOverlap => "gfm-overlap",
            Self::ExtendedOverlap => "extended-overlap",
            Self::MinimalSecure => "minimal-secure",
            Self::MinimalTrusted => "minimal-trusted",
            Self::DefaultSecure => "default-secure",
            Self::DefaultSecureNoHeadingIds => "default-secure-no-heading-ids",
            Self::DefaultTrusted => "default-trusted",
            Self::AllExtensionsSecure => "all-extensions-secure",
            Self::AllExtensionsTrusted => "all-extensions-trusted",
        }
    }

    /// Whether this configuration performs secure rendering checks.
    pub const fn is_secure(self) -> bool {
        matches!(
            self,
            Self::MinimalSecure
                | Self::DefaultSecure
                | Self::DefaultSecureNoHeadingIds
                | Self::AllExtensionsSecure
        )
    }

    /// Whether this configuration is valid for the selected parser.
    pub const fn supports(self, parser: ParserKind) -> bool {
        match parser {
            ParserKind::Ferromark => true,
            ParserKind::PulldownCmark => matches!(
                self,
                Self::CommonMark | Self::GfmOverlap | Self::ExtendedOverlap
            ),
        }
    }

    /// Explicit Ferromark options represented by this configuration.
    pub fn ferromark_options(self) -> Options {
        match self {
            Self::CommonMark => ferromark_options(ParityConfig::CommonMark),
            Self::GfmOverlap => ferromark_options(ParityConfig::GfmOverlap),
            Self::ExtendedOverlap => ferromark_options(ParityConfig::ExtendedOverlap),
            Self::MinimalSecure => Options::minimal(),
            Self::MinimalTrusted => trusted(Options::minimal()),
            Self::DefaultSecure => Options::default(),
            Self::DefaultSecureNoHeadingIds => Options {
                heading_ids: false,
                ..Options::default()
            },
            Self::DefaultTrusted => trusted(Options::default()),
            Self::AllExtensionsSecure => all_extensions(),
            Self::AllExtensionsTrusted => trusted(all_extensions()),
        }
    }

    /// Explicit pulldown-cmark options, if this is a parity configuration.
    pub fn pulldown_options(self) -> Option<pulldown_cmark::Options> {
        let parity = match self {
            Self::CommonMark => ParityConfig::CommonMark,
            Self::GfmOverlap => ParityConfig::GfmOverlap,
            Self::ExtendedOverlap => ParityConfig::ExtendedOverlap,
            _ => return None,
        };
        Some(pulldown_options(parity))
    }
}

impl fmt::Display for RunConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for RunConfig {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|config| config.as_str() == value)
            .ok_or_else(|| format!("unknown configuration `{value}`"))
    }
}

fn trusted(options: Options) -> Options {
    Options {
        render_policy: RenderPolicy::Trusted,
        ..options
    }
}

fn all_extensions() -> Options {
    Options {
        render_policy: RenderPolicy::Untrusted,
        allow_html: true,
        allow_link_refs: true,
        tables: true,
        strikethrough: true,
        highlight: true,
        superscript: true,
        subscript: true,
        task_lists: true,
        autolink_literals: true,
        disallowed_raw_html: true,
        footnotes: true,
        front_matter: true,
        heading_ids: true,
        math: true,
        callouts: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pulldown_should_reject_ferromark_only_configurations() {
        assert!(!RunConfig::AllExtensionsSecure.supports(ParserKind::PulldownCmark));
    }

    #[test]
    fn secure_configuration_should_keep_untrusted_policy() {
        assert_eq!(
            RunConfig::MinimalSecure.ferromark_options().render_policy,
            RenderPolicy::Untrusted
        );
    }

    #[test]
    fn trusted_configuration_should_select_trusted_policy() {
        assert_eq!(
            RunConfig::MinimalTrusted.ferromark_options().render_policy,
            RenderPolicy::Trusted
        );
    }

    #[test]
    fn heading_id_control_should_change_only_heading_ids() {
        let enabled = RunConfig::DefaultSecure.ferromark_options();
        let disabled = RunConfig::DefaultSecureNoHeadingIds.ferromark_options();

        assert!(enabled.heading_ids);
        assert!(!disabled.heading_ids);
        assert_eq!(
            Options {
                heading_ids: false,
                ..enabled
            },
            disabled
        );
    }
}
