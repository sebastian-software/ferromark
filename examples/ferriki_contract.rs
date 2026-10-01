//! Executable fixture consumer. The compatibility lane also compiles this exact
//! source against the unpacked Cargo archive, without Node or private APIs.

use std::borrow::Cow;
use std::error::Error;
use std::io::{self, Write};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use ferromark::ferriki::{
    AssetDigest, AssetMetadata, AssetSource, DirectoryAssetSource, EmbeddedAssetSource, ErrorKind,
    Highlighter, ReleaseManifest, RenderOptions, StandardAssetCatalogs, TokenizeOptions,
    render_html,
};
use ferromark::{
    Allocator, CodeAnnotationSyntax, FerrikiHighlightHooks, HtmlRenderer, HtmlRendererOptions,
    Parser,
};
use serde_json::{Value, json};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

struct ObservedSource {
    source: DirectoryAssetSource,
    reads: Arc<AtomicUsize>,
}

impl AssetSource for ObservedSource {
    fn read(&self, digest: &AssetDigest) -> ferromark::ferriki::Result<Cow<'_, [u8]>> {
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.source.read(digest)
    }
}

struct CatalogFixture {
    languages: Vec<u8>,
    themes: Vec<u8>,
    release: ReleaseManifest,
}

impl CatalogFixture {
    fn load(root: &Path) -> Result<Self> {
        Ok(Self {
            languages: std::fs::read(root.join("languages/manifest.fkindex"))?,
            themes: std::fs::read(root.join("themes/manifest.fkindex"))?,
            release: ReleaseManifest::from_json(&std::fs::read_to_string(
                root.join("release-manifest.json"),
            )?)?,
        })
    }

    fn catalogs(&self, source: impl AssetSource + 'static) -> Result<StandardAssetCatalogs> {
        Ok(StandardAssetCatalogs::from_release_manifest(
            &self.languages,
            &self.themes,
            &self.release,
            source,
        )?)
    }
}

fn field<'a>(case: &'a Value, name: &str) -> Result<&'a str> {
    case[name]
        .as_str()
        .ok_or_else(|| format!("missing {name}: {case}").into())
}

fn token_case(highlighter: &mut Highlighter, case: &Value) -> Result<Value> {
    let (code, lang, theme) = (
        field(case, "code")?,
        field(case, "lang")?,
        field(case, "theme")?,
    );
    let result = highlighter.highlight_with_options(
        code,
        lang,
        theme,
        &TokenizeOptions::default().with_include_scopes(true),
    );
    match result {
        Ok(tokens) => {
            // UTF-8 offsets must select the original token content, even after
            // astral Unicode and CRLF. Neither comparison lane invents offsets.
            for token in tokens.tokens.iter().flatten() {
                assert_eq!(
                    code.get(token.offset..token.offset + token.content.len()),
                    Some(token.content.as_str())
                );
            }
            let typed = highlighter.highlight_with_options(
                code,
                lang,
                theme,
                &TokenizeOptions::default().with_include_token_type(true),
            )?;
            let html = render_html(&tokens, &RenderOptions::default());
            Ok(json!({"id": case["id"], "tokens": tokens, "typedTokens": typed, "html": html}))
        }
        Err(error) => Ok(json!({"id": case["id"], "error": format!("{:?}", error.kind())})),
    }
}

fn markdown_case(
    highlighter: &mut Highlighter,
    arena: &mut Allocator,
    renderer: &mut HtmlRenderer,
    case: &Value,
) -> Result<Value> {
    let mut errors = Vec::new();
    let mut record = |error: &ferromark::ferriki::Error| errors.push(format!("{:?}", error.kind()));
    let html = {
        let document = Parser::new(arena, field(case, "source")?).parse()?;
        let mut hooks = FerrikiHighlightHooks::new(highlighter, field(case, "theme")?)
            .with_error_handler(&mut record);
        renderer.render_with_hooks(&document, &mut hooks)
    };
    // The document borrow ended; reuse the same renderer/highlighter after an
    // allocator reset rather than retaining data from a previous document.
    arena.reset();
    Ok(json!({"id": case["id"], "html": html, "errors": errors}))
}

fn asset_errors(fixture: &CatalogFixture, root: &Path) -> Result<Value> {
    let missing = Highlighter::builder()
        .with_assets(fixture.catalogs(EmbeddedAssetSource::default())?)
        .load_themes(["nord"])
        .build()
        .expect_err("missing payload");
    assert_eq!(missing.kind(), ErrorKind::AssetUnavailable);
    let mut old_manifest = fixture.languages.clone();
    old_manifest[0] = 2;
    let format = StandardAssetCatalogs::from_source(
        &old_manifest,
        &fixture.themes,
        [],
        EmbeddedAssetSource::default(),
    )
    .expect_err("old manifest format");
    assert_eq!(format.kind(), ErrorKind::AssetFormat);

    let asset = &fixture.release.assets["themes/nord.fktheme"];
    let payload = std::fs::read(root.join("themes/nord.fktheme"))?;
    let mut wrong_format = payload.clone();
    wrong_format[0] = 2;
    let wrong_digest = AssetDigest::of(&wrong_format);
    let metadata = fixture.release.assets.iter().map(|(path, entry)| {
        let digest = if path == "themes/nord.fktheme" {
            wrong_digest.clone()
        } else {
            entry.sha256.parse().unwrap()
        };
        (
            path.clone(),
            AssetMetadata::new(digest, entry.size, entry.format_version),
        )
    });
    let catalogs = StandardAssetCatalogs::from_source(
        &fixture.languages,
        &fixture.themes,
        metadata,
        EmbeddedAssetSource::new([(wrong_digest.clone(), Cow::Owned(wrong_format))]),
    )?;
    let payload_format = Highlighter::builder()
        .with_assets(catalogs)
        .load_themes(["nord"])
        .build()
        .expect_err("payload version mismatch with matching digest");
    assert_eq!(payload_format.kind(), ErrorKind::AssetFormat);

    let digest: AssetDigest = asset.sha256.parse()?;
    let mut corrupt = payload;
    corrupt[0] ^= 0xff;
    let integrity = Highlighter::builder()
        .with_assets(fixture.catalogs(EmbeddedAssetSource::new([(digest, Cow::Owned(corrupt))]))?)
        .load_themes(["nord"])
        .build()
        .expect_err("corrupt asset");
    assert_eq!(integrity.kind(), ErrorKind::AssetIntegrity);

    // A lazy asset failure reaches the actual Ferromark adapter and must render
    // escaped fallback while exposing the typed error to the observer.
    let mut failing = Highlighter::builder()
        .with_assets(fixture.catalogs(EmbeddedAssetSource::default())?)
        .build()?;
    let fallback = markdown_case(
        &mut failing,
        &mut Allocator::new(),
        &mut HtmlRenderer::new(),
        &json!({"id":"missing-assets", "theme":"nord", "source":"```rust\n<code> & text\n```"}),
    )?;
    assert!(
        fallback["html"]
            .as_str()
            .unwrap()
            .contains("&lt;code&gt; &amp; text")
    );
    assert_eq!(fallback["errors"], json!(["AssetUnavailable"]));
    Ok(
        json!({"missing": format!("{:?}", missing.kind()), "manifestFormat": format!("{:?}", format.kind()),
        "payloadFormat": format!("{:?}", payload_format.kind()), "integrity": format!("{:?}", integrity.kind()), "fallback":fallback}),
    )
}

fn run(root: &Path, cases_path: &Path) -> Result<Value> {
    let cases: Value = serde_json::from_slice(&std::fs::read(cases_path)?)?;
    let fixture = CatalogFixture::load(root)?;
    let reads = Arc::new(AtomicUsize::new(0));
    let mut highlighter = Highlighter::builder()
        .with_assets(fixture.catalogs(ObservedSource {
            source: DirectoryAssetSource::new(root.join("by-digest")),
            reads: Arc::clone(&reads),
        })?)
        .build()?;
    assert_eq!(
        reads.load(Ordering::Relaxed),
        0,
        "catalog creation must remain lazy"
    );
    highlighter.register_language_json(&cases["customLanguage"].to_string())?;
    highlighter.register_theme_json(&cases["customTheme"].to_string())?;
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        code_annotations: true,
        code_annotation_syntax: CodeAnnotationSyntax::VitePress,
        ..Default::default()
    });
    let mut arena = Allocator::new();
    let mut pass = || -> Result<Value> {
        let tokens = cases["tokens"]
            .as_array()
            .ok_or("missing token cases")?
            .iter()
            .map(|case| token_case(&mut highlighter, case))
            .collect::<Result<Vec<_>>>()?;
        let markdown = cases["markdown"]
            .as_array()
            .ok_or("missing Markdown cases")?
            .iter()
            .map(|case| markdown_case(&mut highlighter, &mut arena, &mut renderer, case))
            .collect::<Result<Vec<_>>>()?;
        Ok(json!({"tokens": tokens, "markdown": markdown}))
    };
    let first = pass()?;
    let loaded = reads.load(Ordering::Relaxed);
    assert!(loaded > 0, "standard assets must actually be read");
    let repeated = pass()?;
    assert_eq!(
        first, repeated,
        "reusing assets/renderer after allocator reset must preserve output"
    );
    assert_eq!(
        reads.load(Ordering::Relaxed),
        loaded,
        "repeated rendering must reuse decoded assets"
    );
    let mut result = first;
    result["assetErrors"] = asset_errors(&fixture, root)?;
    result["reuse"] = json!({"equal":true, "extraAssetReads":0});
    Ok(result)
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        return Err("usage: ferriki_contract <asset-root> <cases.json>".into());
    }
    let mut stdout = io::stdout().lock();
    serde_json::to_writer_pretty(&mut stdout, &run(Path::new(&args[1]), Path::new(&args[2]))?)?;
    writeln!(stdout)?;
    Ok(())
}
