# Changelog

## [3.2.0](https://github.com/sebastian-software/ferromark/compare/v3.1.1...v3.2.0) (2026-10-08)


### Features

* emit MDX modules from JsxCompiler ([#510](https://github.com/sebastian-software/ferromark/issues/510)) ([72339e4](https://github.com/sebastian-software/ferromark/commit/72339e4bbeeec2b9e119fa8c75505cd524079a9c))


### Bug Fixes

* **deps:** override vulnerable tinypool and source-map-js ([#511](https://github.com/sebastian-software/ferromark/issues/511)) ([88429ee](https://github.com/sebastian-software/ferromark/commit/88429ee291a1565242051a096fa2a08da00dfb5b))

## [3.1.1](https://github.com/sebastian-software/ferromark/compare/v3.1.0...v3.1.1) (2026-10-03)


### Bug Fixes

* **release:** skip exact npm versions on publication retries ([#508](https://github.com/sebastian-software/ferromark/issues/508)) ([f04aedb](https://github.com/sebastian-software/ferromark/commit/f04aedb86f3d6810145556b1861fecd5e90d9e50))
* **release:** tolerate missing release-please PR output ([#506](https://github.com/sebastian-software/ferromark/issues/506)) ([83c5c21](https://github.com/sebastian-software/ferromark/commit/83c5c219cd363d5f8b1af60b1a17f2ffd587cc18))

## [3.1.0](https://github.com/sebastian-software/ferromark/compare/v3.0.0...v3.1.0) (2026-10-03)


### Features

* add framework-neutral JSX and native Ferriki highlighting ([#502](https://github.com/sebastian-software/ferromark/issues/502)) ([f38aa31](https://github.com/sebastian-software/ferromark/commit/f38aa31069299aff0b1065824f04861db7161280))
* **homepage:** showcase Afterburner and refresh family design ([3cf130b](https://github.com/sebastian-software/ferromark/commit/3cf130b83805ee095d50bea388fbe2a19c729aba))

## [3.0.0](https://github.com/sebastian-software/ferromark/compare/v2.4.0...v3.0.0) (2026-10-01)


### ⚠ BREAKING CHANGES

* Image and Figure AST metadata now uses optional ElementAttributes. The new ParserOptions fields and Node::Figure variant in this feature require a major Rust API release.
* Image and Figure AST metadata now uses optional ElementAttributes. The new ParserOptions fields and Node::Figure variant in this feature require a major Rust API release.
* ParserOptions::wiki_links and the Node.js wikiLinks option are removed. Passing wikiLinks in Node.js throws an unknown-option TypeError.
* **parser:** Adding ParserOptions::guillemet_digraphs changes the exhaustive public ParserOptions struct and requires a major Rust API release.
* Adding the exhaustive Node::Insertion variant and ParserOptions::insertions field changes the public Rust API and requires a major release.
* **parser:** Image and Figure AST metadata now uses optional ElementAttributes. The new ParserOptions fields and Node::Figure variant in this feature require a major Rust API release.
* Image and Figure AST metadata now uses optional ElementAttributes. The new ParserOptions fields and Node::Figure variant in this feature require a major Rust API release.
* Image and Figure AST metadata now uses optional ElementAttributes. The new ParserOptions fields and Node::Figure variant in this feature require a major Rust API release.

### Features

* add Ferromark Flavored Markdown preset ([#480](https://github.com/sebastian-software/ferromark/issues/480)) ([7f0772d](https://github.com/sebastian-software/ferromark/commit/7f0772d63b1101e4e580c48d74f1c65a97d47f79))
* add image attributes and separate captions ([#471](https://github.com/sebastian-software/ferromark/issues/471)) ([f1c6c2b](https://github.com/sebastian-software/ferromark/commit/f1c6c2b8da51ef7bb7d93a3b45a329743b965fcd))
* add opt-in ++text++ syntax for inserted text ([#474](https://github.com/sebastian-software/ferromark/issues/474)) ([9d4eb3b](https://github.com/sebastian-software/ferromark/commit/9d4eb3b6e7a080d5f732980b3372bda4fd4270b1))
* add opt-in automatic abbreviation markup with dictionary overrides ([#475](https://github.com/sebastian-software/ferromark/issues/475)) ([3c93afe](https://github.com/sebastian-software/ferromark/commit/3c93afece8cac4160748b2362abde3961d029d02))
* add optional Ferriki highlighting integration ([5729e4a](https://github.com/sebastian-software/ferromark/commit/5729e4aff2e9e290a6ae14f32b7d5ee18de111fd))
* add shared attributes and bracketed spans ([#473](https://github.com/sebastian-software/ferromark/issues/473)) ([99da2b2](https://github.com/sebastian-software/ferromark/commit/99da2b21f31ff60ca01219f62898f8403a95546f))
* **benchmarks:** add balanced campaigns and update TanStack ([9af95fc](https://github.com/sebastian-software/ferromark/commit/9af95fc3175bc7c2a199682ba3e6a1c276bfdad9))
* **benchmarks:** add manual Blacksmith runner profiles ([4fbfecb](https://github.com/sebastian-software/ferromark/commit/4fbfecbd955760b50fb92b1a6bb622708f4317bf))
* **benchmarks:** add manual macOS measurement and publication workflow ([50e8be2](https://github.com/sebastian-software/ferromark/commit/50e8be2aa16379de31bad4725b11c34d055db81d))
* **benchmarks:** add manual workflow, skill and Blacksmith runners ([5a0221e](https://github.com/sebastian-software/ferromark/commit/5a0221e987257adc494b6fc94f3289864e24a334))
* **benchmarks:** compare markdown-rs and micromark on the homepage ([add4421](https://github.com/sebastian-software/ferromark/commit/add442103b7b1f3ac296bc51b024b3b850710165))
* **benchmarks:** compare prominent Node Markdown processors ([4019c6a](https://github.com/sebastian-software/ferromark/commit/4019c6a8eaf2916de1038a3a83787782ecb1d721))
* **benchmarks:** complete missing comparison values ([581251f](https://github.com/sebastian-software/ferromark/commit/581251ffe551c8b6908e53927fe63662f4280818))
* **benchmarks:** complete the 22-row native and Node comparison ([290459f](https://github.com/sebastian-software/ferromark/commit/290459f0bda2e889227eca732cc31a1b325d8b47))
* **benchmarks:** complete the native and Node candidate matrix ([5f9c74d](https://github.com/sebastian-software/ferromark/commit/5f9c74d9aa9ec2096f68505d5c04368ae0d1810f))
* **benchmarks:** expand Markdown ecosystem comparisons ([97acfa1](https://github.com/sebastian-software/ferromark/commit/97acfa1590d96b313d46421be0878fca60a8c9d4))
* **benchmarks:** focus comparisons and score all inputs ([efa5d62](https://github.com/sebastian-software/ferromark/commit/efa5d6299dade617b676a913e7e450770a17a133))
* **benchmarks:** focus the main comparison and add TanStack Markdown ([256e961](https://github.com/sebastian-software/ferromark/commit/256e961b073391e32a5256378eb527e325d19f14))
* **benchmarks:** measure missing native and Linux ecosystem pairs ([2908019](https://github.com/sebastian-software/ferromark/commit/290801961e2433d29d5a32ddb78c8384a0fcd336))
* **benchmarks:** score every input and annotate output differences ([df4d85e](https://github.com/sebastian-software/ferromark/commit/df4d85ef45928dae3af87f435f0623072f5a63fe))
* **homepage:** link projects and publish expanded Node comparisons ([2cbca34](https://github.com/sebastian-software/ferromark/commit/2cbca34ed6f0f1d3e45a9dfd25b2399754055735))
* integrate Ferriki highlighting behind an optional feature ([f8ebc39](https://github.com/sebastian-software/ferromark/commit/f8ebc39a5f2d811dfd7299557808f4501adc829e))
* **npm:** ship the macOS addon for Apple Silicon only ([#470](https://github.com/sebastian-software/ferromark/issues/470)) ([8e1bff0](https://github.com/sebastian-software/ferromark/commit/8e1bff040a73f8902d63f4a03a0b2433efa7f7e2))
* **parser:** add blockquote attributions ([#476](https://github.com/sebastian-software/ferromark/issues/476)) ([4d0d598](https://github.com/sebastian-software/ferromark/commit/4d0d598ac02f4a4ac041af61e576a8d8fb6c8e95))
* **parser:** support opt-in ASCII guillemet digraphs ([#472](https://github.com/sebastian-software/ferromark/issues/472)) ([b69e457](https://github.com/sebastian-software/ferromark/commit/b69e457a9bcdfd721511df2af12f41abcd5dbec3))
* remove wiki link syntax ([#482](https://github.com/sebastian-software/ferromark/issues/482)) ([f608d7b](https://github.com/sebastian-software/ferromark/commit/f608d7b02d7882161b10b83e4b7c168d1fe7a9f5))


### Bug Fixes

* **benchmarks:** allow the full comparison to finish on Blacksmith ([4369379](https://github.com/sebastian-software/ferromark/commit/43693790c059093e048f34adaabe25454814945a))
* **benchmarks:** derive Bun metadata from the release version ([acd3bcf](https://github.com/sebastian-software/ferromark/commit/acd3bcfd2ebcbb6dcec67c7c0419b22df4c2c651))
* **benchmarks:** freeze current native dependencies for locked replay ([b58ef83](https://github.com/sebastian-software/ferromark/commit/b58ef83f202ba56ab7867dc10be88b70fc05cf79))
* **benchmarks:** pin the observed Blacksmith Mac CPU model ([78a43af](https://github.com/sebastian-software/ferromark/commit/78a43aff8fa2e14bb73e7b3408f8a1c12a5db7b0))
* **benchmarks:** verify host provenance and preserve interrupted archives ([aa5bc63](https://github.com/sebastian-software/ferromark/commit/aa5bc636ac72bc170d0331ba2bb1455d34a9305e))
* **ci:** retain commit history for benchmark evidence checks ([f914ad4](https://github.com/sebastian-software/ferromark/commit/f914ad4304c768b88c68ff99cf51e8a0bf88273f))
* clarify highlighted theme color trust boundary ([a880bd4](https://github.com/sebastian-software/ferromark/commit/a880bd4ca6151f62bd5ebb88fbfd6b6281128e3d))
* clarify theme color trust boundary and derive traits ([ed1a024](https://github.com/sebastian-software/ferromark/commit/ed1a0242be0d9949a90afbd34a6c7cc319385aac))
* **homepage:** annotate output differences beside library names ([5d36d25](https://github.com/sebastian-software/ferromark/commit/5d36d259906f53fad1980bb3ea678a0ecf73be27))
* **homepage:** restore implementation labels and align table padding ([0263c21](https://github.com/sebastian-software/ferromark/commit/0263c214325a6307bf9d40d4c1f5e4ca7bc608c0))
* **homepage:** update brace-expansion security override ([caa6a3b](https://github.com/sebastian-software/ferromark/commit/caa6a3b5838ea9382c44d2ccb0d93921dfe205a7))
* **homepage:** update brace-expansion security override ([0ae17b3](https://github.com/sebastian-software/ferromark/commit/0ae17b3a6d468165435d87237bed2ab373fd34be))
* **homepage:** verify canonical comparison platform labels ([be8ac73](https://github.com/sebastian-software/ferromark/commit/be8ac733b8ecc1fb795bd92c3641d01f524314ce))


### Performance Improvements

* **renderer:** keep abbreviation checks off the default render path ([#481](https://github.com/sebastian-software/ferromark/issues/481)) ([e2ba904](https://github.com/sebastian-software/ferromark/commit/e2ba904980212d916f45c0760e364d4f93286cdb))


### Documentation

* introduce Ferromark Flavored Markdown guide ([#479](https://github.com/sebastian-software/ferromark/issues/479)) ([f7c2c65](https://github.com/sebastian-software/ferromark/commit/f7c2c65ad31395bfa9c115e76bd84fb084fcb505))

## [2.4.0](https://github.com/sebastian-software/ferromark/compare/v2.3.0...v2.4.0) (2026-09-26)


### Features

* **renderer:** add metadata-preserving highlighted-line hook ([#458](https://github.com/sebastian-software/ferromark/issues/458)) ([8b6b517](https://github.com/sebastian-software/ferromark/commit/8b6b5174f71615bf91f8596dc3921fcb8ad70118))
* **transforms:** add typography, GitHub, and emoji passes ([#455](https://github.com/sebastian-software/ferromark/issues/455)) ([40913a3](https://github.com/sebastian-software/ferromark/commit/40913a34524b7b538145bbece112d3427a31ab29))

## [2.3.0](https://github.com/sebastian-software/ferromark/compare/v2.2.0...v2.3.0) (2026-09-26)


### Features

* **core:** expose read-only document outline ([#448](https://github.com/sebastian-software/ferromark/issues/448)) ([d8073f1](https://github.com/sebastian-software/ferromark/commit/d8073f1348e5c92aa7587c43c082d3b55baad465))
* **homepage:** adopt ferramenta-family landing kit ([#446](https://github.com/sebastian-software/ferromark/issues/446)) ([cc5a411](https://github.com/sebastian-software/ferromark/commit/cc5a41102a6d4987a047dfa60508d9e11536964d))
* **transforms:** add optional native pass pipeline ([#452](https://github.com/sebastian-software/ferromark/issues/452)) ([b3a6d26](https://github.com/sebastian-software/ferromark/commit/b3a6d26b117680784e6afd1b94a226284bbf87fe))
* **transforms:** add table of contents builder ([#453](https://github.com/sebastian-software/ferromark/issues/453)) ([9567487](https://github.com/sebastian-software/ferromark/commit/9567487be70078c10a243c644e59ee87f8673761))


### Bug Fixes

* **homepage:** use release package version ([#454](https://github.com/sebastian-software/ferromark/issues/454)) ([39531fc](https://github.com/sebastian-software/ferromark/commit/39531fc9d2bba09eccc3d2d87e89b78677417585))
* **renderer:** keep heading and footnote IDs unique ([#444](https://github.com/sebastian-software/ferromark/issues/444)) ([01acef0](https://github.com/sebastian-software/ferromark/commit/01acef0760bb5e940461c5a05c9c2e1d5abbf407))

## [2.2.0](https://github.com/sebastian-software/ferromark/compare/v2.1.2...v2.2.0) (2026-09-25)


### Features

* **node:** accept UTF-8 bytes as Markdown input ([#441](https://github.com/sebastian-software/ferromark/issues/441)) ([7371efe](https://github.com/sebastian-software/ferromark/commit/7371efef2e1393d1f3e50edbe712df5cfe481401))


### Performance Improvements

* **node:** convert input strings in one pass ([#437](https://github.com/sebastian-software/ferromark/issues/437)) ([88e5e31](https://github.com/sebastian-software/ferromark/commit/88e5e31cfbb972f9355effb05f430e5952c61edf))
* **node:** pass options to the addon in a compact form ([#439](https://github.com/sebastian-software/ferromark/issues/439)) ([2f283d4](https://github.com/sebastian-software/ferromark/commit/2f283d4e4e7398c9fad64cfc77e45f672eac0900))
* **node:** render one-shot calls without options on a kept per-thread renderer ([#440](https://github.com/sebastian-software/ferromark/issues/440)) ([241944e](https://github.com/sebastian-software/ferromark/commit/241944ea35c11046abec0f2608d6d1a6361b1b41))

## [2.1.2](https://github.com/sebastian-software/ferromark/compare/v2.1.1...v2.1.2) (2026-09-24)


### Performance Improvements

* **parser:** scan ByteClass stop bytes with SSSE3 and AVX2 on x86-64 ([#426](https://github.com/sebastian-software/ferromark/issues/426)) ([f8fff4c](https://github.com/sebastian-software/ferromark/commit/f8fff4c905efdd145bf3c5e1fe0544ae1108015e))
* **parser:** scan line ends with SSE2 on x86-64 ([#431](https://github.com/sebastian-software/ferromark/issues/431)) ([c6a29e3](https://github.com/sebastian-software/ferromark/commit/c6a29e34460038055ecf7744cace59b0b60e8e70))
* **renderer:** classify escape bytes with inlined SSE2 on x86-64 ([#430](https://github.com/sebastian-software/ferromark/issues/430)) ([dfcfe98](https://github.com/sebastian-software/ferromark/commit/dfcfe983435e3f1291fce75c49ead3ed99ac0723))

## [2.1.1](https://github.com/sebastian-software/ferromark/compare/v2.1.0...v2.1.1) (2026-09-23)


### Performance Improvements

* **renderer:** compare a raw HTML tag only with the filtered names sharing its initial ([#418](https://github.com/sebastian-software/ferromark/issues/418)) ([4b7b259](https://github.com/sebastian-software/ferromark/commit/4b7b25913ae0b82cf07ce056497460eb0391f503))
* **renderer:** plan heading ids in reusable claim storage ([#422](https://github.com/sebastian-software/ferromark/issues/422)) ([77b2fcd](https://github.com/sebastian-software/ferromark/commit/77b2fcd19fc7b649dfda3fe01825dcde113d131c))

## [2.1.0](https://github.com/sebastian-software/ferromark/compare/v2.0.1...v2.1.0) (2026-09-23)


### Features

* prefix emitted heading IDs ([#410](https://github.com/sebastian-software/ferromark/issues/410)) ([1667e19](https://github.com/sebastian-software/ferromark/commit/1667e199fd0a76aa48b4dca2ab09c59c81f5ef4c))
* support heading level offsets ([#409](https://github.com/sebastian-software/ferromark/issues/409)) ([aceafd3](https://github.com/sebastian-software/ferromark/commit/aceafd32a14d4c6d1b20b46ec3c2c5b0db69d34e))


### Bug Fixes

* ensure unique heading IDs ([#408](https://github.com/sebastian-software/ferromark/issues/408)) ([e890ccb](https://github.com/sebastian-software/ferromark/commit/e890ccbd85018d7af1876d4f9ae4d41a0deadada))
* **parser:** trim only ASCII whitespace at block boundaries ([#414](https://github.com/sebastian-software/ferromark/issues/414)) ([7eda5a4](https://github.com/sebastian-software/ferromark/commit/7eda5a4bb06c859c1045ba81f0c1178f9e88b065))


### Performance Improvements

* **node:** keep the reusable renderer's output buffer between calls ([#412](https://github.com/sebastian-software/ferromark/issues/412)) ([1a4b606](https://github.com/sebastian-software/ferromark/commit/1a4b60646d1cb36431855bc0429ea493ea5af795))
* **parser:** find NUL bytes and the first ]: in one root scan ([#413](https://github.com/sebastian-software/ferromark/issues/413)) ([c12698b](https://github.com/sebastian-software/ferromark/commit/c12698bddc10a6573b1e75f9a0eadb33967e90e4))
* **renderer:** gate the GFM tag filter on the first name byte ([#411](https://github.com/sebastian-software/ferromark/issues/411)) ([6833228](https://github.com/sebastian-software/ferromark/commit/6833228649b301b7e6f8c8c271c3259206a4f458))

## [2.0.1](https://github.com/sebastian-software/ferromark/compare/v2.0.0...v2.0.1) (2026-09-22)


### Bug Fixes

* **renderer:** remove renderer-owned inline TOC ([#397](https://github.com/sebastian-software/ferromark/issues/397)) ([716a5c6](https://github.com/sebastian-software/ferromark/commit/716a5c6d3ecc7fcc042065350cbd4409dc1aa720))

## [2.0.0](https://github.com/sebastian-software/ferromark/compare/v2.0.0-rc.2...v2.0.0) (2026-09-21)


### ⚠ BREAKING CHANGES

* **api:** `Renderer`, `RenderError`, `RenderResult`, `ast::Position`, `ParseErrorKind::{UnexpectedToken, InvalidSyntax, UnexpectedEof}`, `ParserOptions.gfm` and `HtmlRendererOptions.highlight` are removed; `ParseErrorKind` is `#[non_exhaustive]`; `HtmlRendererOptions::gfm()` is now the convenience profile and the strict profile is `gfm_spec()`; `HtmlRendererOptions.soft_break` takes effect. See docs/migration-v2.md.

### Features

* **homepage:** put ferromark in the header lockup and quiet the family switcher ([98f630a](https://github.com/sebastian-software/ferromark/commit/98f630ae11919a0686ca57c4a3704e0250daa3a5))
* **homepage:** put ferromark in the header lockup and quiet the family switcher ([779f72a](https://github.com/sebastian-software/ferromark/commit/779f72aaa423618c2e15b0dd53cdd805935dc093))
* **homepage:** serve the documentation from ferromark.dev ([dc3c58f](https://github.com/sebastian-software/ferromark/commit/dc3c58faf2bb7d58919dc7b9f5302e885931cafc))
* **homepage:** serve the documentation from ferromark.dev ([0cb0bf1](https://github.com/sebastian-software/ferromark/commit/0cb0bf143480333e6cfe50f6c73efd560bd49320))
* **homepage:** tell the story of what sets Ferromark apart and the guardrails behind it ([eeef0e8](https://github.com/sebastian-software/ferromark/commit/eeef0e8f62fee8080fabc0f3bbe1bab3fd05b09a))
* **homepage:** tell the story of what sets Ferromark apart and the guardrails behind it ([fc93726](https://github.com/sebastian-software/ferromark/commit/fc9372690a1462994801547cbe103b430edf89b9))


### Bug Fixes

* **benchmarks:** read the runtime-profile sources from the tree at the revision ([9ec4e8c](https://github.com/sebastian-software/ferromark/commit/9ec4e8c75332e226a0be0aaeaca6a207bae773c8))
* **benchmarks:** read the runtime-profile sources from the tree at the revision ([d013b5d](https://github.com/sebastian-software/ferromark/commit/d013b5d8e924ad24fcc0a42bc9d8b3bbaae4e599))
* **node:** harden libc detection and verify registry dist-tags ([#367](https://github.com/sebastian-software/ferromark/issues/367)) ([7740a5d](https://github.com/sebastian-software/ferromark/commit/7740a5d21ab39c82ce53628aad184489b48329ec)), closes [#359](https://github.com/sebastian-software/ferromark/issues/359) [#361](https://github.com/sebastian-software/ferromark/issues/361) [#362](https://github.com/sebastian-software/ferromark/issues/362)
* **parser:** bound emphasis nesting with max_nesting_depth ([#372](https://github.com/sebastian-software/ferromark/issues/372)) ([e8439fd](https://github.com/sebastian-software/ferromark/commit/e8439fd81757678e7cb4e811cb40835d7dfe899d)), closes [#371](https://github.com/sebastian-software/ferromark/issues/371)
* **parser:** bound inline bracket nesting with max_nesting_depth ([#369](https://github.com/sebastian-software/ferromark/issues/369)) ([2887b2b](https://github.com/sebastian-software/ferromark/commit/2887b2bd256263fd1a11b579cd1a148dc5a5c2e9)), closes [#349](https://github.com/sebastian-software/ferromark/issues/349)
* **parser:** follow the spec for tabs after list markers, type-1 HTML closers and MDX flow lines ([#382](https://github.com/sebastian-software/ferromark/issues/382)) ([e532dcf](https://github.com/sebastian-software/ferromark/commit/e532dcf49af114fa04de4cd03bd1d7607d1cf4bf))
* **parser:** keep laziness to open paragraphs and bound container and inline costs ([#383](https://github.com/sebastian-software/ferromark/issues/383)) ([8d957e3](https://github.com/sebastian-software/ferromark/commit/8d957e308df87f04e253578830ec994b40ae2bdf))
* **parser:** key the JSX closer memo without std::String ([c84743c](https://github.com/sebastian-software/ferromark/commit/c84743cb5bc5ec5a75c7fda71e5013eba9cfd2f6))
* **parser:** key the JSX closer memo without std::String ([85660dd](https://github.com/sebastian-software/ferromark/commit/85660dd547ab729334adf4ec4b054a90c2c171d0))
* **release:** hand npm publish local tarball paths ([5aa4384](https://github.com/sebastian-software/ferromark/commit/5aa43842bbea9a11342cc7974d5b0b596289d92d))
* **release:** hand npm publish local tarball paths ([3a3fa03](https://github.com/sebastian-software/ferromark/commit/3a3fa03a8ea1aa04950f34d07399ae1f0f267632))
* **renderer:** bound code annotation ranges and keep root-absolute links with an empty base ([#380](https://github.com/sebastian-software/ferromark/issues/380)) ([1037fb5](https://github.com/sebastian-software/ferromark/commit/1037fb5d78929f83ed2d814187150c732ded64ff))
* **renderer:** normalize base URL for Markdown links ([#376](https://github.com/sebastian-software/ferromark/issues/376)) ([012b1e4](https://github.com/sebastian-software/ferromark/commit/012b1e4d37828619bcb9bda1479bf5c720fb9008))


### Performance Improvements

* **allocator:** lower the fresh arena floor to 2 KB ([be8d611](https://github.com/sebastian-software/ferromark/commit/be8d611297d4113cefd8d66eb9752eb2589cd4af))
* **allocator:** size a fresh arena from the input instead of a 16 KB floor ([6befcd4](https://github.com/sebastian-software/ferromark/commit/6befcd41a218668d7b3825af67e7f09898acea5f))
* **parser:** answer the closer probe from a forward window, not a hash table ([2a46cf5](https://github.com/sebastian-software/ferromark/commit/2a46cf5e34d1b57c3f534a977c5a1f0db1e19d59))
* **parser:** answer the closer question from a forward window ([6505b93](https://github.com/sebastian-software/ferromark/commit/6505b939d7634be598e0bd12517d76d737ca050d))
* **parser:** count inline nesting in a plain cell on the parser ([ac793b2](https://github.com/sebastian-software/ferromark/commit/ac793b2560dc024f55f2c42dfb17736faf494ac6))
* **parser:** keep the inline nesting depth in a plain cell ([39b1f0b](https://github.com/sebastian-software/ferromark/commit/39b1f0b75ab46651f4428c6beafca0a96ba189bc))
* **parser:** make math, MDX and definition-list scans linear ([#384](https://github.com/sebastian-software/ferromark/issues/384)) ([bffc89f](https://github.com/sebastian-software/ferromark/commit/bffc89f6034776da19ed24e40b3eb5ac33e50452))
* **parser:** make nested link probing linear ([#374](https://github.com/sebastian-software/ferromark/issues/374)) ([ca82ec2](https://github.com/sebastian-software/ferromark/commit/ca82ec2f4ae4d6154c008cd92c2ea32a8e51c4dc)), closes [#350](https://github.com/sebastian-software/ferromark/issues/350)
* **parser:** memoize missing MDX JSX closers ([#377](https://github.com/sebastian-software/ferromark/issues/377)) ([e1b3489](https://github.com/sebastian-software/ferromark/commit/e1b34892fdfa15254a9445bce2f0c9f858438a14))
* **parser:** repair the release-head regression and state measured figures on the homepage and README ([fe8987c](https://github.com/sebastian-software/ferromark/commit/fe8987c410dd7d21964169548a9c2c3ed8725dc9))
* **parser:** run the laziness tracker on demand and allocate memo tables on first use ([60602a5](https://github.com/sebastian-software/ferromark/commit/60602a5a78babff67d47b1dcf799d3ea1ca415f2))


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

### Install

```sh
npm install ferromark@next
cargo add ferromark@=2.0.0-rc.1
```

For reproducible Node testing, install `ferromark@2.0.0-rc.1` explicitly.
The RC does not replace npm's stable `latest` tag.

The Rust distribution is a single `ferromark` crate with public allocator, AST,
parser and renderer modules. Node continues to install its native platform package.

### What to test

- Real documents through the Rust convenience functions or arena AST API.
- Node's rendering, reusable renderer, metadata extraction and highlighter APIs.
- Document-wide reference definitions and footnote scope, including containers.
- Optional marked text, inline notes, reference policy and CSS-addressable tables.

Read the [migration guide](https://github.com/sebastian-software/ferromark/blob/v2.0.0-rc.1/docs/migration-v2.md)
for removed options, changed defaults and the v2 API. The v1 CLI and MDX component
compiler APIs are not part of v2. MDX syntax capture does not execute JavaScript.

### Validation and known limits

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
