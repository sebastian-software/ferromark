# Changelog

## [2.0.0](https://github.com/sebastian-software/ferromark/compare/v2.0.0-rc.2...v2.0.0) (2026-09-19)


### ⚠ BREAKING CHANGES

* **api:** `Renderer`, `RenderError`, `RenderResult`, `ast::Position`, `ParseErrorKind::{UnexpectedToken, InvalidSyntax, UnexpectedEof}`, `ParserOptions.gfm` and `HtmlRendererOptions.highlight` are removed; `ParseErrorKind` is `#[non_exhaustive]`; `HtmlRendererOptions::gfm()` is now the convenience profile and the strict profile is `gfm_spec()`; `HtmlRendererOptions.soft_break` takes effect. See docs/migration-v2.md.

### Features

* **homepage:** put ferromark in the header lockup and quiet the family switcher ([98f630a](https://github.com/sebastian-software/ferromark/commit/98f630ae11919a0686ca57c4a3704e0250daa3a5))
* **homepage:** put ferromark in the header lockup and quiet the family switcher ([779f72a](https://github.com/sebastian-software/ferromark/commit/779f72aaa423618c2e15b0dd53cdd805935dc093))
* **homepage:** serve the documentation from ferromark.dev ([dc3c58f](https://github.com/sebastian-software/ferromark/commit/dc3c58faf2bb7d58919dc7b9f5302e885931cafc))
* **homepage:** serve the documentation from ferromark.dev ([0cb0bf1](https://github.com/sebastian-software/ferromark/commit/0cb0bf143480333e6cfe50f6c73efd560bd49320))


### Bug Fixes

* **node:** harden libc detection and verify registry dist-tags ([#367](https://github.com/sebastian-software/ferromark/issues/367)) ([7740a5d](https://github.com/sebastian-software/ferromark/commit/7740a5d21ab39c82ce53628aad184489b48329ec)), closes [#359](https://github.com/sebastian-software/ferromark/issues/359) [#361](https://github.com/sebastian-software/ferromark/issues/361) [#362](https://github.com/sebastian-software/ferromark/issues/362)
* **parser:** bound emphasis nesting with max_nesting_depth ([#372](https://github.com/sebastian-software/ferromark/issues/372)) ([e8439fd](https://github.com/sebastian-software/ferromark/commit/e8439fd81757678e7cb4e811cb40835d7dfe899d)), closes [#371](https://github.com/sebastian-software/ferromark/issues/371)
* **parser:** bound inline bracket nesting with max_nesting_depth ([#369](https://github.com/sebastian-software/ferromark/issues/369)) ([2887b2b](https://github.com/sebastian-software/ferromark/commit/2887b2bd256263fd1a11b579cd1a148dc5a5c2e9)), closes [#349](https://github.com/sebastian-software/ferromark/issues/349)
* **release:** hand npm publish local tarball paths ([5aa4384](https://github.com/sebastian-software/ferromark/commit/5aa43842bbea9a11342cc7974d5b0b596289d92d))
* **release:** hand npm publish local tarball paths ([3a3fa03](https://github.com/sebastian-software/ferromark/commit/3a3fa03a8ea1aa04950f34d07399ae1f0f267632))
* **renderer:** normalize base URL for Markdown links ([#376](https://github.com/sebastian-software/ferromark/issues/376)) ([012b1e4](https://github.com/sebastian-software/ferromark/commit/012b1e4d37828619bcb9bda1479bf5c720fb9008))


### Performance Improvements

* **parser:** make nested link probing linear ([#374](https://github.com/sebastian-software/ferromark/issues/374)) ([ca82ec2](https://github.com/sebastian-software/ferromark/commit/ca82ec2f4ae4d6154c008cd92c2ea32a8e51c4dc)), closes [#350](https://github.com/sebastian-software/ferromark/issues/350)
* **parser:** memoize missing MDX JSX closers ([#377](https://github.com/sebastian-software/ferromark/issues/377)) ([e1b3489](https://github.com/sebastian-software/ferromark/commit/e1b34892fdfa15254a9445bce2f0c9f858438a14))


### Miscellaneous Chores

* **release:** finalize the stable 2.0.0 release ([#373](https://github.com/sebastian-software/ferromark/issues/373)) ([e0fe363](https://github.com/sebastian-software/ferromark/commit/e0fe363636c9e11444f634b5f10a1edce5854481)), closes [#363](https://github.com/sebastian-software/ferromark/issues/363) [#351](https://github.com/sebastian-software/ferromark/issues/351) [#352](https://github.com/sebastian-software/ferromark/issues/352)


### Code Refactoring

* **api:** freeze the public API for 2.0.0 ([#370](https://github.com/sebastian-software/ferromark/issues/370)) ([bdcbf87](https://github.com/sebastian-software/ferromark/commit/bdcbf871d983de8323506f5ce98c82b5cdf60a0c)), closes [#353](https://github.com/sebastian-software/ferromark/issues/353) [#354](https://github.com/sebastian-software/ferromark/issues/354) [#355](https://github.com/sebastian-software/ferromark/issues/355) [#356](https://github.com/sebastian-software/ferromark/issues/356) [#357](https://github.com/sebastian-software/ferromark/issues/357) [#358](https://github.com/sebastian-software/ferromark/issues/358)

## [2.0.0-rc.2](https://github.com/sebastian-software/ferromark/compare/v2.0.0-rc.1...v2.0.0-rc.2) (2026-09-16)


### ⚠ BREAKING CHANGES

* **renderer:** `HtmlRendererOptions::soft_break`, `hard_break`, `base_url`, `source_path` and `code_annotation_meta_key` are now `Cow<'static, str>`, and `autolink_patterns` is `Cow<'static, [Cow<'static, str>]>`. Assignments need `.into()`: `base_url: "/docs/".to_string()` becomes `base_url: "/docs/".into()`, and `autolink_patterns: vec!["mailto:".to_string()]` becomes `autolink_patterns: vec!["mailto:".into()].into()`. A `&'static str` borrows, a `String` moves in, and any other value must be owned first (`value.to_string().into()`). Reads are unaffected because the fields deref to `str`. See docs/migration-v2.md and docs/decisions/2026-09-15-borrowed-renderer-options.md.

### Features

* **node:** add a profile-guided training driver ([7c684bc](https://github.com/sebastian-software/ferromark/commit/7c684bc23235c4e35be83533537ba7c7e5da3845))


### Performance Improvements

* accelerate parsing and rendering on Apple Silicon (rounds 2 and 3) ([71a051d](https://github.com/sebastian-software/ferromark/commit/71a051de4a26ef7b2ca62c7375991f48c3c09421))
* accelerate parsing and rendering on Apple Silicon (rounds 2 and 3) ([#323](https://github.com/sebastian-software/ferromark/issues/323)) ([71a051d](https://github.com/sebastian-software/ferromark/commit/71a051de4a26ef7b2ca62c7375991f48c3c09421))
* **parser:** drive the definition pre-pass from `]:` occurrences ([c8a118f](https://github.com/sebastian-software/ferromark/commit/c8a118f2f847c280f077eef3685c3e478d5e8887))
* **parser:** reuse the line the block dispatchers already scanned ([e3215b7](https://github.com/sebastian-software/ferromark/commit/e3215b7cd8196f45e88a187dac2162d1c079309d))
* **parser:** scan each list line once while walking items ([87556c3](https://github.com/sebastian-software/ferromark/commit/87556c31daebd35458109740fb7f1ce81a3591e4))
* **parser:** scan short inline slices without the vector search ([6e2442f](https://github.com/sebastian-software/ferromark/commit/6e2442fa424b56348d32cfd864f9e071948acd10))
* **parser:** step over the terminator each line walk already found ([7ff949b](https://github.com/sebastian-software/ferromark/commit/7ff949b25a9c535dc2641b94a7c68e6274833547))
* **parser:** trim the emphasis bookkeeping around pairing ([e2a3df7](https://github.com/sebastian-software/ferromark/commit/e2a3df7032abbf3d2e33b05899d5a43489b20275))
* **parser:** walk definition lists and table metadata line by line once ([72fcb46](https://github.com/sebastian-software/ferromark/commit/72fcb46e81ec70da3baed53b4c0da43b6c7e7de4))
* **renderer:** allocate heading scratch buffers on first use ([0958933](https://github.com/sebastian-software/ferromark/commit/095893390bd6bb6597b0fed794f6905fe7852de9))
* **renderer:** assemble plain fence markup from merged literals ([1dcf3ea](https://github.com/sebastian-software/ferromark/commit/1dcf3ea538c7d16856710e37edddb08f80f98ed1))
* **renderer:** borrow default renderer option strings ([62cdcd0](https://github.com/sebastian-software/ferromark/commit/62cdcd015b54abd770a4b056822282050b34441e))
* **renderer:** build the autolink first-byte index once per renderer ([ee662c5](https://github.com/sebastian-software/ferromark/commit/ee662c5eab01eb530ca80e4de44093b387edcf63))
* **renderer:** emit bare fence languages without the metadata tokenizer ([8059f0f](https://github.com/sebastian-software/ferromark/commit/8059f0fdb0adf0be5abba8eb627c0cf850c233f8))
* **renderer:** emit generated heading ids without escaping ([3ce3e43](https://github.com/sebastian-software/ferromark/commit/3ce3e432bdfd480951673aed0f4d18fbd2df284f))
* **renderer:** give short documents an output capacity floor ([bcc8ab1](https://github.com/sebastian-software/ferromark/commit/bcc8ab1396cd94106e4eac12dc6ff5260e00ff1a))
* **renderer:** keep the source-span attribute gate inline ([f1e6dcb](https://github.com/sebastian-software/ferromark/commit/f1e6dcb410321a943d38631da19cc0f8a1d93b25))
* **renderer:** reset only the footnote state the options can write ([33ef51b](https://github.com/sebastian-software/ferromark/commit/33ef51bb89ca8d1d0efcbd48cd4c77553ebc9bf4))
* **renderer:** resolve autolink patterns once per text node ([d3ec646](https://github.com/sebastian-software/ferromark/commit/d3ec646166b951a51fe41943be305e86804f1ec7))
* **renderer:** skip the setup scan when no option reads its result ([3819e0c](https://github.com/sebastian-software/ferromark/commit/3819e0cd9d0399fb2048b7e11a07b87bd5ea84b3))
* **renderer:** slugify single-text headings from the source ([fd69a21](https://github.com/sebastian-software/ferromark/commit/fd69a215b9f184db4e670186d5450cfa3b1b3bac))
* **renderer:** write ASCII slug bytes through a cursor ([6fef693](https://github.com/sebastian-software/ferromark/commit/6fef693f8efba87a63543e6dfd55a26fe84163ea))

## 2.0.0-rc.1

First release candidate for the new arena-allocated Markdown parser and HTML
renderer, based on the MIT-licensed OX-Content core. This is a breaking Rust API
change and a candidate for testing before the stable v2 release.

## Install

```sh
npm install ferromark@next
cargo add ferromark@=2.0.0-rc.1
```

For reproducible Node testing, install `ferromark@2.0.0-rc.1` explicitly.
The RC does not replace npm's stable `latest` tag.

The Rust distribution is a single `ferromark` crate with public allocator, AST,
parser and renderer modules. Node continues to install its native platform package.

## What to test

- Real documents through the Rust convenience functions or arena AST API.
- Node's rendering, reusable renderer, metadata extraction and highlighter APIs.
- Document-wide reference definitions and footnote scope, including containers.
- Optional marked text, inline notes, reference policy and CSS-addressable tables.

Read the [migration guide](https://github.com/sebastian-software/ferromark/blob/v2.0.0-rc.1/docs/migration-v2.md)
for removed options, changed defaults and the v2 API. The v1 CLI and MDX component
compiler APIs are not part of v2. MDX syntax capture does not execute JavaScript.

## Validation and known limits

The release requires the full platform CI, a verified Rust package build, and all
nine npm archives. Six native targets have runtime tests; the two musl targets
are built and inspected. Registry installations are checked before the GitHub
prerelease is created.

The measured structural reference-prepass overhead on two real documents is
tracked in [#320](https://github.com/sebastian-software/ferromark/issues/320) as a
follow-up optimization. The owner accepted it for this release. Performance
claims remain tied to the archived workloads, options and library pins.

Please include the input document, options, platform and version when reporting
RC issues. Upstream attribution and specification-fixture licenses are preserved.
