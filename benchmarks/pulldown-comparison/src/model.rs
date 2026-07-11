use std::{fmt, str::FromStr};

use ferromark::{Options, Profile, RenderPolicy};

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
    /// Secure Ferromark Essentials profile.
    EssentialsSecure,
    /// Trusted Ferromark Essentials profile.
    EssentialsTrusted,
    /// Secure Ferromark Extended profile.
    ExtendedSecure,
    /// Trusted Ferromark Extended profile.
    ExtendedTrusted,
    /// Secure Ferromark Full profile.
    FullSecure,
    /// Trusted Ferromark Full profile.
    FullTrusted,
}

impl RunConfig {
    /// All configurations accepted by the diagnostic runner.
    pub const ALL: [Self; 9] = [
        Self::CommonMark,
        Self::GfmOverlap,
        Self::ExtendedOverlap,
        Self::EssentialsSecure,
        Self::EssentialsTrusted,
        Self::ExtendedSecure,
        Self::ExtendedTrusted,
        Self::FullSecure,
        Self::FullTrusted,
    ];

    /// Stable command-line and metadata name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CommonMark => "commonmark",
            Self::GfmOverlap => "gfm-overlap",
            Self::ExtendedOverlap => "extended-overlap",
            Self::EssentialsSecure => "essentials-secure",
            Self::EssentialsTrusted => "essentials-trusted",
            Self::ExtendedSecure => "extended-secure",
            Self::ExtendedTrusted => "extended-trusted",
            Self::FullSecure => "full-secure",
            Self::FullTrusted => "full-trusted",
        }
    }

    /// Whether this configuration performs secure rendering checks.
    pub const fn is_secure(self) -> bool {
        matches!(
            self,
            Self::EssentialsSecure | Self::ExtendedSecure | Self::FullSecure
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
            Self::EssentialsSecure => Options::from(Profile::Essentials),
            Self::EssentialsTrusted => trusted(Profile::Essentials),
            Self::ExtendedSecure => Options::from(Profile::Extended),
            Self::ExtendedTrusted => trusted(Profile::Extended),
            Self::FullSecure => Options::from(Profile::Full),
            Self::FullTrusted => trusted(Profile::Full),
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

fn trusted(profile: Profile) -> Options {
    Options {
        render_policy: RenderPolicy::Trusted,
        ..Options::from(profile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pulldown_should_reject_product_profiles() {
        assert!(!RunConfig::FullSecure.supports(ParserKind::PulldownCmark));
    }

    #[test]
    fn secure_configuration_should_keep_untrusted_policy() {
        assert_eq!(
            RunConfig::EssentialsSecure
                .ferromark_options()
                .render_policy,
            RenderPolicy::Untrusted
        );
    }

    #[test]
    fn trusted_configuration_should_select_trusted_policy() {
        assert_eq!(
            RunConfig::EssentialsTrusted
                .ferromark_options()
                .render_policy,
            RenderPolicy::Trusted
        );
    }
}
