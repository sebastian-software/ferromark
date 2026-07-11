use ferromark_pulldown_comparison::{
    Corpus, ParserKind, RunConfig, render_ferromark_config_into, render_pulldown_config_into,
};

#[test]
fn every_ferromark_configuration_should_render() {
    for configuration in RunConfig::ALL {
        let corpus = if configuration == RunConfig::ExtendedOverlap {
            Corpus::Extended
        } else {
            Corpus::CommonMark5K
        }
        .materialize();
        let mut output = Vec::new();
        render_ferromark_config_into(corpus.input(), configuration, &mut output);
        assert!(
            !output.is_empty(),
            "configuration `{configuration}` produced no output"
        );
    }
}

#[test]
fn every_pulldown_configuration_should_render() {
    for configuration in RunConfig::ALL
        .into_iter()
        .filter(|config| config.supports(ParserKind::PulldownCmark))
    {
        let corpus = if configuration == RunConfig::ExtendedOverlap {
            Corpus::Extended
        } else {
            Corpus::CommonMark5K
        }
        .materialize();
        let mut output = String::new();
        render_pulldown_config_into(corpus.input(), configuration, &mut output)
            .expect("supported pulldown configuration should render");
        assert!(
            !output.is_empty(),
            "configuration `{configuration}` produced no output"
        );
    }
}

#[test]
fn every_corpus_should_render_with_extended_secure() {
    for corpus in Corpus::ALL.into_iter().chain([Corpus::Extended]) {
        let data = corpus.materialize();
        let mut output = Vec::new();
        render_ferromark_config_into(data.input(), RunConfig::ExtendedSecure, &mut output);
        assert!(!output.is_empty(), "corpus `{corpus}` produced no output");
    }
}
