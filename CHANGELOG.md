# Changelog

## [0.8.0](https://github.com/sebastian-software/ferromark/compare/ferromark-v0.7.0...ferromark-v0.8.0) (2026-09-04)


### ⚠ BREAKING CHANGES

* **api:** Options is now non-exhaustive; configure a preset by mutating public fields or with ferromark::options!. Add wildcard arms to matches on public enums.

### Features

* **api:** make options and public enums non-exhaustive ([#196](https://github.com/sebastian-software/ferromark/issues/196)) ([a1f4b6e](https://github.com/sebastian-software/ferromark/commit/a1f4b6ec6d4c007f58bc7d67909d90d4709f6454))
* **cli:** add documented command interface ([#211](https://github.com/sebastian-software/ferromark/issues/211)) ([dc76b70](https://github.com/sebastian-software/ferromark/commit/dc76b70c9ffeef9fe01083a9cb3d09a87e3b923c))
* **mdx:** improve diagnostic ergonomics ([#233](https://github.com/sebastian-software/ferromark/issues/233)) ([cda5ab8](https://github.com/sebastian-software/ferromark/commit/cda5ab8efa0440582d85398e4beb83c48e272ad5))
* **node:** publish per-platform native packages ([#217](https://github.com/sebastian-software/ferromark/issues/217)) ([3602c6e](https://github.com/sebastian-software/ferromark/commit/3602c6ea63221bab5b7ec12e9d25a6006c771b9e))
* **parser:** report resource limit fallbacks ([#215](https://github.com/sebastian-software/ferromark/issues/215)) ([c284c26](https://github.com/sebastian-software/ferromark/commit/c284c26c6966d9e3a325ca5b5dc66c904f6bd722))


### Bug Fixes

* align Node engine with CommonJS support ([#227](https://github.com/sebastian-software/ferromark/issues/227)) ([f37b798](https://github.com/sebastian-software/ferromark/commit/f37b798c1a6a329865cb8ff3909eac5eb454429e))
* bound reference-link resolution work ([#193](https://github.com/sebastian-software/ferromark/issues/193)) ([4695ecd](https://github.com/sebastian-software/ferromark/commit/4695ecd0306b8605ec82a0bc8ca5e61faf727a3c))
* **ci:** enforce workflow action SHA pins ([#195](https://github.com/sebastian-software/ferromark/issues/195)) ([520ac09](https://github.com/sebastian-software/ferromark/commit/520ac09f3538f0e12d44650ddd16d3fad82c29e5))
* **ci:** verify npm releases ([#201](https://github.com/sebastian-software/ferromark/issues/201)) ([33ce6d0](https://github.com/sebastian-software/ferromark/commit/33ce6d0af80aee7865cc92dae8e4171dd2b66508))
* complete crates.io trusted publishing ([617a29d](https://github.com/sebastian-software/ferromark/commit/617a29dbe8833e5bdbb207186cfe573051cfd18c))
* guarantee parser progress under fuzzing ([#197](https://github.com/sebastian-software/ferromark/issues/197)) ([2f7eb1a](https://github.com/sebastian-software/ferromark/commit/2f7eb1a989cdecb5b1b7be444573c50e4feff2f4))
* harden native addon loading ([#226](https://github.com/sebastian-software/ferromark/issues/226)) ([f8393bb](https://github.com/sebastian-software/ferromark/commit/f8393bb8831826bdcb23baf9dd4a71f9c796c366))
* **homepage:** patch transitive toml advisories ([8baa9f3](https://github.com/sebastian-software/ferromark/commit/8baa9f3f86c80d2c3f373506ddd70590d3816545))
* **homepage:** patch transitive TOML advisories blocking CI ([68d74c3](https://github.com/sebastian-software/ferromark/commit/68d74c353ddde7db3f82aadd15616d802c43f49e))
* ignore leading BOM in MDX documents ([#223](https://github.com/sebastian-software/ferromark/issues/223)) ([1e780e9](https://github.com/sebastian-software/ferromark/commit/1e780e948f82619f856a92da9d53ed1564169e8e))
* **mdx:** emit valid JSX components ([#203](https://github.com/sebastian-software/ferromark/issues/203)) ([649c3ab](https://github.com/sebastian-software/ferromark/commit/649c3ab9892fd08ce592820939a3de9d3d5fc691))
* **mdx:** preserve document state across segments ([c705b36](https://github.com/sebastian-software/ferromark/commit/c705b362e2bfc47fc5375a2f0ac8aca25542c00c))
* **mdx:** preserve prose after multiline ESM ([#200](https://github.com/sebastian-software/ferromark/issues/200)) ([84eff19](https://github.com/sebastian-software/ferromark/commit/84eff19b589edc1b1638961bfe5d1051fd75c51f))
* **mdx:** resolve references across segments ([#220](https://github.com/sebastian-software/ferromark/issues/220)) ([bdfe9b0](https://github.com/sebastian-software/ferromark/commit/bdfe9b04596045e602bb1a6d40f4649e654aac4d))
* **mdx:** scope front matter to document start ([#257](https://github.com/sebastian-software/ferromark/issues/257)) ([24b6628](https://github.com/sebastian-software/ferromark/commit/24b6628d3b3e181e0fa2d9d3bd5d6eafcd53282f))
* **mdx:** share document state and parse segments once ([23cf98d](https://github.com/sebastian-software/ferromark/commit/23cf98db5da162aa42cd81bf2e60378c3fb0412d))
* **node:** reject unknown option keys ([#216](https://github.com/sebastian-software/ferromark/issues/216)) ([924cce8](https://github.com/sebastian-software/ferromark/commit/924cce8b32ea977c56f6860d5105f3252448ed3e))
* **node:** unwind panics in native addon ([#192](https://github.com/sebastian-software/ferromark/issues/192)) ([c79a661](https://github.com/sebastian-software/ferromark/commit/c79a661df215885f5bd5038e26cc066407352ddb))
* preserve fenced code in MDX segmentation ([#191](https://github.com/sebastian-software/ferromark/issues/191)) ([16a4ce6](https://github.com/sebastian-software/ferromark/commit/16a4ce6de9c02b1805293972f7611088c6b7dedc))
* **profiling:** repair standalone profiling scripts ([4666393](https://github.com/sebastian-software/ferromark/commit/4666393783e1eca0a9fb1d09064dc2c5218b4c33))
* **profiling:** restore standalone sampling and process cleanup ([d05ddc2](https://github.com/sebastian-software/ferromark/commit/d05ddc2bbf7e0328bb333b2e8c12e8a8dcdbcf27))
* publish crate with trusted OIDC credentials ([a38e6ef](https://github.com/sebastian-software/ferromark/commit/a38e6ef8c74cc481745fb50b0a887423edaab253))
* reject oversized inputs ([#199](https://github.com/sebastian-software/ferromark/issues/199)) ([dabfe98](https://github.com/sebastian-software/ferromark/commit/dabfe98f668a2c3152667f676b0e06815226961b))
* reserve every emitted heading ID across the document ([4dfc019](https://github.com/sebastian-software/ferromark/commit/4dfc01930e0226a40ab4b807d4d708e419edabd0))
* reserve generated heading IDs ([1aef51f](https://github.com/sebastian-software/ferromark/commit/1aef51fc577f1b63f787965c38efa02dd2a20b8f))
* surface Node highlighter failures ([#225](https://github.com/sebastian-software/ferromark/issues/225)) ([054168d](https://github.com/sebastian-software/ferromark/commit/054168d5259174c355d19184051f5ad64f4070f4))


### Performance Improvements

* add reusable renderer sessions ([#218](https://github.com/sebastian-software/ferromark/issues/218)) ([948d631](https://github.com/sebastian-software/ferromark/commit/948d6316114459a98f98dd24225b3cd82acd0e99))
* avoid quadratic footnote ordinal initialization ([6b4e622](https://github.com/sebastian-software/ferromark/commit/6b4e622cfda9be942ee144635277176a725881e2))
* avoid reports outside Linux ([#228](https://github.com/sebastian-software/ferromark/issues/228)) ([a8591c3](https://github.com/sebastian-software/ferromark/commit/a8591c3e5031646b309e5ce2e064af5c639a3fe6))
* bound autolink candidate and suffix scans ([14e46d4](https://github.com/sebastian-software/ferromark/commit/14e46d43e9935fcfa6708e6dfda5a6d1500eb61d))
* bound inline boundary lookups ([#222](https://github.com/sebastian-software/ferromark/issues/222)) ([46abe56](https://github.com/sebastian-software/ferromark/commit/46abe56dafb30e4f393c9198544abafc66a67fec))
* index autolink exclusion ranges ([6918c21](https://github.com/sebastian-software/ferromark/commit/6918c21d6d748879c67056088d3d01ef7f205a46))
* **inline:** add x86 SIMD specials scan ([#206](https://github.com/sebastian-software/ferromark/issues/206)) ([e5083e8](https://github.com/sebastian-software/ferromark/commit/e5083e8d905ea2e332953323260982d5e57c9359))
* **inline:** index autolink exclusion ranges ([5a99aaa](https://github.com/sebastian-software/ferromark/commit/5a99aaa386180b681f7a7feb2db24f8e0db7a5a9))
* make autolink candidate and suffix scans linear ([13ed634](https://github.com/sebastian-software/ferromark/commit/13ed6343797df2bd7eabc31c52be1813bc8fb81c))
* **mdx:** cache unterminated expression scans ([#202](https://github.com/sebastian-software/ferromark/issues/202)) ([934a34e](https://github.com/sebastian-software/ferromark/commit/934a34ea4db7ed390c5f13397e8e6adc1d7041de))
* **mdx:** release compact segment buffers after rendering ([adf7272](https://github.com/sebastian-software/ferromark/commit/adf727227fea90c42f537d386f9e600411afe9cb))
* **node:** reduce N-API boundary copies ([#241](https://github.com/sebastian-software/ferromark/issues/241)) ([018fb03](https://github.com/sebastian-software/ferromark/commit/018fb03ed4d305f1f47cfd900e0dae7598b1c6d1))
* remove dead MDX tag stack ([#224](https://github.com/sebastian-software/ferromark/issues/224)) ([b2501a0](https://github.com/sebastian-software/ferromark/commit/b2501a04170ccca6cf4d4e76ed5b1c520faa123b))
* reuse document footnote numbering and render state ([021f2ea](https://github.com/sebastian-software/ferromark/commit/021f2ea169d263aa51a8d6feda4698c87921846a))

## [0.7.0](https://github.com/sebastian-software/ferromark/compare/ferromark-v0.6.0...ferromark-v0.7.0) (2026-07-27)


### ⚠ BREAKING CHANGES

* **msrv:** ferromark now requires Rust 1.88 or newer.
* **deps:** The ferromark npm package now requires Node.js 22 or newer.

### Miscellaneous Chores

* **deps:** standardize on pnpm 11 ([#130](https://github.com/sebastian-software/ferromark/issues/130)) ([26e3f52](https://github.com/sebastian-software/ferromark/commit/26e3f529bb23f68ef7f76f0a49e7e3a4c3b8e1bf))
* **msrv:** require Rust 1.88 ([#133](https://github.com/sebastian-software/ferromark/issues/133)) ([d4264a0](https://github.com/sebastian-software/ferromark/commit/d4264a00c1e82daa8227f03d4f760752eef14f0c))

## [0.6.0](https://github.com/sebastian-software/ferromark/compare/ferromark-v0.5.0...ferromark-v0.6.0) (2026-07-25)


### ⚠ BREAKING CHANGES

* Options no longer implements Copy (the new link_base_path field is heap-allocated) — clone it where a copy was relied upon. FencedCodeBlock gained the meta field and is now non_exhaustive.

### Features

* document metadata APIs and link base path for docs-tooling adoption ([#129](https://github.com/sebastian-software/ferromark/issues/129)) ([bbe3e2f](https://github.com/sebastian-software/ferromark/commit/bbe3e2fbae3b59fdc30e2dfbae40ff34dfaa5e1c))


### Performance Improvements

* **escape:** scan short text segments with the escape LUT instead of dual memchr passes ([#126](https://github.com/sebastian-software/ferromark/issues/126)) ([b8fb947](https://github.com/sebastian-software/ferromark/commit/b8fb94751562b452a7a7f40902b274635314f2a0))
* **escape:** vectorize the short escape scan with baseline SIMD ([#127](https://github.com/sebastian-software/ferromark/issues/127)) ([493b6d1](https://github.com/sebastian-software/ferromark/commit/493b6d123040a57e6836f5aca6c8a6a5819a91cb))
* **footnotes:** reuse parsers and avoid per-reference label allocation ([#119](https://github.com/sebastian-software/ferromark/issues/119)) ([2c13e10](https://github.com/sebastian-software/ferromark/commit/2c13e103bbb8be46378995de4a37a9426fde6191))
* **heading-ids:** store dedup slugs in an arena instead of per-heading map keys ([#125](https://github.com/sebastian-software/ferromark/issues/125)) ([32e23c4](https://github.com/sebastian-software/ferromark/commit/32e23c47a2dd0031ea57a67f5ae82d249e5cdf57))
* optimize table cell scanning and heading id generation ([#117](https://github.com/sebastian-software/ferromark/issues/117)) ([f97fc68](https://github.com/sebastian-software/ferromark/commit/f97fc68a8135f0b9e8e95a94ddee1db5ec8c9e77))

## [0.5.0](https://github.com/sebastian-software/ferromark/compare/ferromark-v0.4.0...ferromark-v0.5.0) (2026-07-25)


### ⚠ BREAKING CHANGES

* **inline-footnotes:** The inline-footnotes feature adds the inline_footnotes option, the InlineFootnote event, and a positional parse_with_options argument.

### Features

* add indented code block option ([#110](https://github.com/sebastian-software/ferromark/issues/110)) ([a63f684](https://github.com/sebastian-software/ferromark/commit/a63f68479b744752b69b421ccf9513e4f66679ee))
* add opt-in definition lists ([#113](https://github.com/sebastian-software/ferromark/issues/113)) ([baac345](https://github.com/sebastian-software/ferromark/commit/baac345cfe211fee3aaacd1074e604246abf714e))
* add opt-in merged table cells ([#115](https://github.com/sebastian-software/ferromark/issues/115)) ([1bf9576](https://github.com/sebastian-software/ferromark/commit/1bf9576d652c2da3cf0ba7da74af9cdca9c21194))
* add opt-in source-only line comments ([#111](https://github.com/sebastian-software/ferromark/issues/111)) ([fd10345](https://github.com/sebastian-software/ferromark/commit/fd10345b01c7f0fdc0fd1888e681c6e51e77e604))
* **inline-footnotes:** add opt-in inline footnotes ([#114](https://github.com/sebastian-software/ferromark/issues/114)) ([f2b9ad1](https://github.com/sebastian-software/ferromark/commit/f2b9ad16ee1ac28acd52c6b3550b84bf2bf4d964))
* **tables:** prototype relative column width hints ([#116](https://github.com/sebastian-software/ferromark/issues/116)) ([71590bd](https://github.com/sebastian-software/ferromark/commit/71590bd322801b2440f3cd51b3383df4116a2daf))


### Bug Fixes

* **deps:** harden md4c benchmark dependencies ([#102](https://github.com/sebastian-software/ferromark/issues/102)) ([70120c8](https://github.com/sebastian-software/ferromark/commit/70120c8b9527089abbc334c2d245b5002f71d18b))
* **deps:** patch homepage security advisories ([#101](https://github.com/sebastian-software/ferromark/issues/101)) ([2face32](https://github.com/sebastian-software/ferromark/commit/2face32665bb153a46c110efc9671972c86cccc2))

## [0.4.0](https://github.com/sebastian-software/ferromark/compare/ferromark-v0.3.3...ferromark-v0.4.0) (2026-07-24)


### ⚠ BREAKING CHANGES

* **options:** remove Profile and Options::from(Profile); use Options::minimal(), Options::commonmark(), Options::gfm(), or explicit flags.

### Features

* **mdx:** add inline JSX events ([#94](https://github.com/sebastian-software/ferromark/issues/94)) ([654459f](https://github.com/sebastian-software/ferromark/commit/654459f39c67d75b56dca8a71ef5d9dee9e8119b))
* **mdx:** add semantic event stream ([#95](https://github.com/sebastian-software/ferromark/issues/95)) ([f87f3a7](https://github.com/sebastian-software/ferromark/commit/f87f3a7f35d37fdf21570a5065d1b112aadc159d))
* **mdx:** add source-spanned segments ([c750c80](https://github.com/sebastian-software/ferromark/commit/c750c806587174d9f554860984ca1187b4e7f602))
* **mdx:** add strict diagnostics ([#93](https://github.com/sebastian-software/ferromark/issues/93)) ([8154348](https://github.com/sebastian-software/ferromark/commit/8154348496bc621381b6a1c8b6fc978a7cbfccb8))
* **mdx:** promote container-local flow events ([#96](https://github.com/sebastian-software/ferromark/issues/96)) ([b841ec7](https://github.com/sebastian-software/ferromark/commit/b841ec726d3518d01935f4febd6941e30819ad49))
* **options:** replace profiles with dialect presets ([#99](https://github.com/sebastian-software/ferromark/issues/99)) ([9249776](https://github.com/sebastian-software/ferromark/commit/9249776692da7e5f9462993fcefb70a81126eaea))


### Bug Fixes

* **ci:** use portable Linux CPU target ([f797c6a](https://github.com/sebastian-software/ferromark/commit/f797c6a2c16bf634b07705a5a8a2f7800d58d0ff))
* **homepage:** update audited dependencies ([#100](https://github.com/sebastian-software/ferromark/issues/100)) ([7f54712](https://github.com/sebastian-software/ferromark/commit/7f5471202100e3b0a7512720ef6d6899ece605b7))
* **mdx:** harden source segment ranges ([2d2e712](https://github.com/sebastian-software/ferromark/commit/2d2e712150b6b4c568c4fb604614cfc0e17aeba6))
* **parser:** honor disabled HTML in lazy containers ([#98](https://github.com/sebastian-software/ferromark/issues/98)) ([6b6dbf6](https://github.com/sebastian-software/ferromark/commit/6b6dbf606f7e09488e76af7d7655440c26867600))

## [0.3.3](https://github.com/sebastian-software/ferromark/compare/ferromark-v0.3.2...ferromark-v0.3.3) (2026-07-11)


### Bug Fixes

* **release:** pass all-targets directly to package check ([d82c4a8](https://github.com/sebastian-software/ferromark/commit/d82c4a8e6f56f3e4bd4a5c264a8fb38fccb17ae3))

## [0.3.2](https://github.com/sebastian-software/ferromark/compare/ferromark-v0.3.1...ferromark-v0.3.2) (2026-07-11)


### Bug Fixes

* **release:** run pnpm shim through Windows shell ([cf21fdd](https://github.com/sebastian-software/ferromark/commit/cf21fdd5f441a6af3ecac0aec93a746ee9ec0b30))

## [0.3.1](https://github.com/sebastian-software/ferromark/compare/ferromark-v0.3.0...ferromark-v0.3.1) (2026-07-11)


### Features

* **options:** add curated markdown profiles ([e991eeb](https://github.com/sebastian-software/ferromark/commit/e991eebdf73e96130ebbed68aaa549664627b0dc))


### Bug Fixes

* **bench:** isolate publication baseline artifacts ([4317f14](https://github.com/sebastian-software/ferromark/commit/4317f14cd6789cf48084b73819b19eaa81228e92))
* **bench:** keep publication lanes unique ([95aedd5](https://github.com/sebastian-software/ferromark/commit/95aedd52f1c1d4cf984de04bbc4254bd6caa58ea))


### Performance Improvements

* lazily allocate rare inline buffers ([5adc0d2](https://github.com/sebastian-software/ferromark/commit/5adc0d2b543e48c9e5cc0322ecb9bec3b3eadb7c))
* short-circuit container blank checks ([979de8b](https://github.com/sebastian-software/ferromark/commit/979de8b3aa4e0fabbaa4d0553ed53f387dcfafc2))
* skip absent inline resolver stages ([a52c3af](https://github.com/sebastian-software/ferromark/commit/a52c3af567924eefa57def419ce2e3cb4eaa5339))
* skip heading id state when disabled ([430be85](https://github.com/sebastian-software/ferromark/commit/430be85bc2e417c1537f460c895b897c94e1b200))

## [0.3.0](https://github.com/sebastian-software/ferromark/compare/ferromark-v0.2.0...ferromark-v0.3.0) (2026-07-10)


### ⚠ BREAKING CHANGES

* **render:** add a fenced-code rendering hook
* **features:** remove unused cargo features

### Features

* **homepage:** adopt Ardo 4 and shared config ([7000a71](https://github.com/sebastian-software/ferromark/commit/7000a714cd0beaa73a58d54a2baaf4c3c9fa5c55))
* **node:** add napi-rs distribution ([949d0ab](https://github.com/sebastian-software/ferromark/commit/949d0ab799352014f51dbc8c909f42ff87a5e366))
* **render:** add a fenced-code rendering hook ([d0aed38](https://github.com/sebastian-software/ferromark/commit/d0aed384c1473b1f5add97d400ca4ebf2547d320))
* ship the v0.3 integration release ([80c23e3](https://github.com/sebastian-software/ferromark/commit/80c23e360ee426113fd2a4815a4576dc995c095e))


### Bug Fixes

* **features:** remove unused cargo features ([97ea349](https://github.com/sebastian-software/ferromark/commit/97ea349a112229d0a25e6f32d09e9b88f4d72f6a))
* **release:** gate aligned registry publishes ([58d798b](https://github.com/sebastian-software/ferromark/commit/58d798b251ac3fc05a23be4004960718d00b8f89))


### Performance Improvements

* **render:** add fenced-code benchmark guardrails ([8a0407f](https://github.com/sebastian-software/ferromark/commit/8a0407f27a8ba49dad1f55bf3d3027b650241bfe))

## [0.2.0](https://github.com/sebastian-software/ferromark/compare/ferromark-v0.1.4...ferromark-v0.2.0) (2026-07-10)


### ⚠ BREAKING CHANGES

* **render:** Options now defaults to RenderPolicy::Untrusted, which escapes raw HTML and blocks unsafe URL schemes. Use RenderPolicy::Trusted for explicit passthrough rendering.
* **mdx:** MdxOutput::to_component now returns Result<String, ComponentNameError>.
* **api:** Range::slice_str, HtmlWriter::as_str, and HtmlWriter::into_string now return UTF-8 validation results. HtmlWriter::buffer_mut is no longer public.

### Bug Fixes

* **api:** validate UTF-8 string conversions ([b91acb6](https://github.com/sebastian-software/ferromark/commit/b91acb65c001a21b576c6baf685c80d6c81df0aa))
* **ci:** select stable for lint jobs ([11a4329](https://github.com/sebastian-software/ferromark/commit/11a4329588eada622e1e7b361805b5b4c7452aa8))
* **cursor:** enforce bounds in safe movement APIs ([70e018d](https://github.com/sebastian-software/ferromark/commit/70e018da2485b4cf8ec763f85c9e1319d82d6b63))
* **homepage:** clear high severity advisories ([f669bf2](https://github.com/sebastian-software/ferromark/commit/f669bf209e4974c9730a29af84cae04a9cd7f59b))
* **mdx:** validate component identifiers ([099ef33](https://github.com/sebastian-software/ferromark/commit/099ef3331913765f3b133e749427bb2f0e138191))
* **range:** reject oversized offsets ([5e08b88](https://github.com/sebastian-software/ferromark/commit/5e08b88fc12ec0bbd6e92ed25a51463c3cbe1249))
* **render:** close untrusted URL bypasses ([6092773](https://github.com/sebastian-software/ferromark/commit/6092773f768f694dd93da4af0b677498ce09a5a8))
* **render:** make untrusted output the default ([2a85fba](https://github.com/sebastian-software/ferromark/commit/2a85fba96624992a46a64da6bf9d61abdfcc7482))


### Performance Improvements

* **footnotes:** assign ordinals in constant time ([7ef0299](https://github.com/sebastian-software/ferromark/commit/7ef0299af2690427d8f4a7d9aa65ec8427a2ab75))

## [0.1.4](https://github.com/sebastian-software/ferromark/compare/ferromark-v0.1.3...ferromark-v0.1.4) (2026-07-10)


### Features

* add highlight mark extension ([a7cda59](https://github.com/sebastian-software/ferromark/commit/a7cda59ae7a1c9c73c746562db78c41240cfe20d))
* add highlight mark extension ([a52f632](https://github.com/sebastian-software/ferromark/commit/a52f63281b89bcd598655e2f45ea47a555925d28))
* add homepage with GitHub Pages deploy ([a34d23b](https://github.com/sebastian-software/ferromark/commit/a34d23b68144b98549ccdc1c48e8d93f92e5ce36))
* add subscript and superscript syntax ([d4e92dc](https://github.com/sebastian-software/ferromark/commit/d4e92dc9148b43b5ac8fc3292eb31feb8c58c1f5))
* add subscript and superscript syntax ([739b96a](https://github.com/sebastian-software/ferromark/commit/739b96a5f35aebaaef1aa09b70cbbef242738b2b))


### Bug Fixes

* address highlight review follow-ups ([3637a21](https://github.com/sebastian-software/ferromark/commit/3637a213f44f607c1195d7b74b6c72a179604fe6))
* replace O(n^2) heading ID dedup scan with hash map lookup ([301d493](https://github.com/sebastian-software/ferromark/commit/301d493af12a2a61112f25c980640a9fa41460ad))
* resolve heading ID perf regression, restore benchmark lead ([cbe7adf](https://github.com/sebastian-software/ferromark/commit/cbe7adf093d755331edac6ed0bf823b32a43172c))


### Performance Improvements

* gate highlight scanning behind option ([d9a8cea](https://github.com/sebastian-software/ferromark/commit/d9a8cea89fe10ecd999a4bdc0b09ad82f6456dfc))
* make heading ID generation allocation-free per heading ([057fcda](https://github.com/sebastian-software/ferromark/commit/057fcda974e9f836db53ef775312ac05ce37121b))
* specialize highlight scanner fast paths ([5b49bdb](https://github.com/sebastian-software/ferromark/commit/5b49bdb47fd355eebda331e4338e78b2e39ac564))

## [0.1.3](https://github.com/sebastian-software/ferromark/compare/ferromark-v0.1.2...ferromark-v0.1.3) (2026-02-09)


### Features

* **mdx:** add assembly layer with render() API ([2f9042a](https://github.com/sebastian-software/ferromark/commit/2f9042a9c92f21850af0ae7b193ae9b72f0aa4d9))
* **mdx:** add to_component() for JSX/TSX module output ([580463c](https://github.com/sebastian-software/ferromark/commit/580463cce7389fc956c42c5dfbc57434fa219b4b))

## [0.1.2](https://github.com/sebastian-software/ferromark/compare/ferromark-v0.1.1...ferromark-v0.1.2) (2026-02-09)


### Bug Fixes

* correct license to MIT and update copyright holder ([c3de5ae](https://github.com/sebastian-software/ferromark/commit/c3de5ae7242999bf000bd299e86cdb81da815593))
* resolve all clippy warnings and enable strict linting ([b4ec8e0](https://github.com/sebastian-software/ferromark/commit/b4ec8e04501574f6bbb067b145f1e8560e4cc5f2))

## [0.1.1](https://github.com/sebastian-software/ferromark/compare/ferromark-v0.1.0...ferromark-v0.1.1) (2026-02-09)


### Features

* achieve 100% in-scope CommonMark compliance ([7e9728a](https://github.com/sebastian-software/ferromark/commit/7e9728adc890ae260e0642bd136a21dde6bd18f3))
* add block-level event types ([c4e4def](https://github.com/sebastian-software/ferromark/commit/c4e4def6b9a5d869b8e540372c10db90f4d384e3))
* add callouts, math spans, heading IDs, front matter and cleanup ([00e4cac](https://github.com/sebastian-software/ferromark/commit/00e4caceff9f26958e206c6103aea7e1c465ff97))
* add CLI binary for md-fast ([ef77722](https://github.com/sebastian-software/ferromark/commit/ef777220242c888bb73d08cee073150b4b1fbb8a))
* add Criterion benchmark harness ([702eba4](https://github.com/sebastian-software/ferromark/commit/702eba40a64f5d49f4473a423944c42f66d11c76))
* add footnotes support and update README ([cdc36f5](https://github.com/sebastian-software/ferromark/commit/cdc36f570914fc247245d9da50ac5b84b99839c7))
* add HTML entity decoding for inline text content ([43c09ad](https://github.com/sebastian-software/ferromark/commit/43c09ad1f7150dcd246836f106bc92d095cd56ff))
* add HTML entity decoding for URLs and titles ([0d47b5e](https://github.com/sebastian-software/ferromark/commit/0d47b5e5ffccda8c85905d42359322fbf0425a66))
* add HTML escaping with memchr optimization ([7448caf](https://github.com/sebastian-software/ferromark/commit/7448caf5b364f1316fb1d4df41675faed4d7b566))
* add HtmlWriter with optimized buffer management ([88877a5](https://github.com/sebastian-software/ferromark/commit/88877a5ac7c532b3666870624e024a8e64aa5b7e))
* add lazy continuation and indented code blocks ([fe1e0e6](https://github.com/sebastian-software/ferromark/commit/fe1e0e6c2f04b023d16e0dc0627ae5beb7b76dfd))
* add parsing options for html and link refs ([808f7bd](https://github.com/sebastian-software/ferromark/commit/808f7bdc1348247d54ec8d25f0b8b710a4e9dfbb))
* add pointer-based Cursor for fast byte scanning ([25f423a](https://github.com/sebastian-software/ferromark/commit/25f423ab573fecbb51bf5c7e10e841d3de52ab0d))
* add Range and limits types ([015f418](https://github.com/sebastian-software/ferromark/commit/015f418cf5581ac79d39aa97bae81cc0647d8371))
* **bench:** add md4c integration ([2c4de21](https://github.com/sebastian-software/ferromark/commit/2c4de212b36006d3473e6df488a57e8385b80685))
* **block:** add CommonMark HTML block parsing ([3f3286c](https://github.com/sebastian-software/ferromark/commit/3f3286ce61b7ff2c2d00294905ff0ed33b9aec66))
* **blocks:** implement container blocks (blockquotes and lists) ([b72075d](https://github.com/sebastian-software/ferromark/commit/b72075dcabe977cbd07654ed0e9deae212ffb4d9))
* **emphasis:** add Unicode whitespace and punctuation detection ([ced960b](https://github.com/sebastian-software/ferromark/commit/ced960bfdaf7af227f0884934fba211e1de3caac))
* **escape:** add URL encoding for link destinations ([a93b618](https://github.com/sebastian-software/ferromark/commit/a93b618f613ca17097677c39510a29ff973f6b14))
* **images:** add title attribute support and improve rendering ([9986851](https://github.com/sebastian-software/ferromark/commit/998685146ac72260879248d532a062b2e2945fdf))
* implement all 5 GFM extensions for full spec compliance ([6aabd05](https://github.com/sebastian-software/ferromark/commit/6aabd05c21fca47fc7597a8e93dddd8459885bb0))
* implement block parser for Phase 1 ([35944b0](https://github.com/sebastian-software/ferromark/commit/35944b0e8ab940cfd3fcb8fc732556592fd0ce61))
* implement fenced code blocks ([5baac89](https://github.com/sebastian-software/ferromark/commit/5baac8994dea59e1620676723e0c7b75fda777f8))
* implement setext headings for CommonMark compliance ([1574125](https://github.com/sebastian-software/ferromark/commit/1574125619f8eb741e551e9c0b75d8989b9aec21))
* implement tab expansion for CommonMark compliance ([9d5453e](https://github.com/sebastian-software/ferromark/commit/9d5453e8de64b383a4bb507dde19e5516b9754ac))
* improve CommonMark compliance to 52.1% ([7b9e0c2](https://github.com/sebastian-software/ferromark/commit/7b9e0c2f9b5cab09f824280a29718cb05300c615))
* improve CommonMark compliance to 83.1% in-scope ([cbc191a](https://github.com/sebastian-software/ferromark/commit/cbc191a77ad57c0485358b4746171bd9dcf1862c))
* **inline:** add CommonMark raw HTML parsing ([34e6ed4](https://github.com/sebastian-software/ferromark/commit/34e6ed4bd703e900ad4724dc58fa10df99b51196))
* **inline:** implement hard line breaks ([b4beb5f](https://github.com/sebastian-software/ferromark/commit/b4beb5fbe8b090f4d013892dbbcc52a1115dae22))
* **inline:** implement inline parser with code spans and emphasis ([75aab16](https://github.com/sebastian-software/ferromark/commit/75aab168a186134078989509ed8d5c73cb95c639))
* **inline:** implement links, images, and autolinks ([db73a4d](https://github.com/sebastian-software/ferromark/commit/db73a4d20660414d6c19013e72fd052b6e410294))
* integrate block parser with public API ([c55d4ef](https://github.com/sebastian-software/ferromark/commit/c55d4efc14433a1a4515d47546bc7cfad73de1a5))
* **link:** implement link reference definitions ([72eb0b9](https://github.com/sebastian-software/ferromark/commit/72eb0b98395fbb95aa49e4ce0bb1475a6b83d91c))
* **lists:** implement tight vs loose list detection ([a6572f2](https://github.com/sebastian-software/ferromark/commit/a6572f2498df7f0403a0a8c84e3dee586c19737e))
* process backslash escapes in link URLs and titles ([1272bef](https://github.com/sebastian-software/ferromark/commit/1272bef8c1f20da027aee45109728fc7f5c4f375))


### Bug Fixes

* add SoftBreak events and fix ListEnd closing tags ([856722b](https://github.com/sebastian-software/ferromark/commit/856722bc50678437f759767093e77bf6fe3332b7))
* **bench:** link md4c entity and unsafe ffi ([eed866e](https://github.com/sebastian-software/ferromark/commit/eed866e85e934387a85f786d2c49c1b2bc1a7d37))
* **bench:** remove stale ref flag ([1600390](https://github.com/sebastian-software/ferromark/commit/16003909256a26421d001c521219570956d6640c))
* blank lines without &gt; markers close blockquotes ([1b72df0](https://github.com/sebastian-software/ferromark/commit/1b72df021900c0360d7ea018784026ebcf7dd82e))
* blank list items cannot interrupt paragraphs ([a0c07e4](https://github.com/sebastian-software/ferromark/commit/a0c07e4ac1e0ff2de620cbb1388992eca5211e21))
* **block:** correct HTML block tag list length ([4a98d32](https://github.com/sebastian-software/ferromark/commit/4a98d3246c232bfef089d81151d91014246a18c9))
* **blocks:** include newlines in fenced code block content ([faa6c94](https://github.com/sebastian-software/ferromark/commit/faa6c943054b59fa41916223634b477a425c66a9))
* **block:** tighten HTML parsing and fenced code handling ([4c5a82c](https://github.com/sebastian-software/ferromark/commit/4c5a82c43357355acd61bb7bf3bb275760951f7c))
* buffer blank lines in indented code blocks ([8a0c676](https://github.com/sebastian-software/ferromark/commit/8a0c676719be90a910af8dea7078e43781aa118e))
* calculate absolute content_indent for list items (Phase 1) ([fb52b24](https://github.com/sebastian-software/ferromark/commit/fb52b2478e58a6373c2a82c31f3f35faad44755d))
* close lists when indent &gt;= 4 prevents new items ([e73b435](https://github.com/sebastian-software/ferromark/commit/e73b43568c81e4c248e82397be7a42a4dfcd68c0))
* decode entities in fenced code block info strings ([10d3c31](https://github.com/sebastian-software/ferromark/commit/10d3c31113dd7ebd54944e866ea7203c1ebcb3f8))
* detect blank lines after container matching ([96bb06d](https://github.com/sebastian-software/ferromark/commit/96bb06d481681131bd3092fb9caeabcc75f23f43))
* detect indented code blocks within list items (Phase 5) ([25ad2a8](https://github.com/sebastian-software/ferromark/commit/25ad2a889f44b890dffcf101032fc3233de72e5b))
* don't recognize block starts at 4+ indent in lazy continuation ([167f1ee](https://github.com/sebastian-software/ferromark/commit/167f1ee35dd74b3ec42e8e42b5727d99a3a9b510))
* **emphasis:** apply rule of three only when delimiter can both open/close ([8e2952e](https://github.com/sebastian-software/ferromark/commit/8e2952ec8df9b0cf614bed36c240bc691f7e8372))
* **emphasis:** properly handle nested emphasis and partial consumption ([884074a](https://github.com/sebastian-software/ferromark/commit/884074ad0d6bef2a1b325d8b75064b237cc55b8d))
* enable GFM extensions for all parsers in comparison benchmarks ([d6318a6](https://github.com/sebastian-software/ferromark/commit/d6318a6f3dd93a61df911b871ad93856c6584d38))
* enable lazy continuation for list item paragraphs ([75d3a29](https://github.com/sebastian-software/ferromark/commit/75d3a29fa3ce066cb6acb4d89f58f858d778a1d9))
* **escape:** escape quotes as &quot; in text content ([11e2e61](https://github.com/sebastian-software/ferromark/commit/11e2e6118549d0133af617d89cfe536a159c1e97))
* **escape:** percent-encode double quotes in URLs ([07a8fc6](https://github.com/sebastian-software/ferromark/commit/07a8fc6de0601fd136af99b809c272da07e9fa86))
* fenced code blocks inside list items (Examples 263, 278) ([a6598b1](https://github.com/sebastian-software/ferromark/commit/a6598b1bde7fb72858958bca64cba168390098fe))
* **hardbreak:** improve hard line break compliance ([6353dd1](https://github.com/sebastian-software/ferromark/commit/6353dd1204b0cab951e2fbd38d1afd194cec5e59))
* **html:** tighten inline HTML parsing ([f4fe02f](https://github.com/sebastian-software/ferromark/commit/f4fe02f599541a5219d0cca6833e25f0f0085379))
* implement two-blank-line rule for list items ([0be1199](https://github.com/sebastian-software/ferromark/commit/0be1199ebe3de88aaaad03704499e7b54e6cd7f7))
* improve link parsing for nested brackets and link precedence ([8629bdc](https://github.com/sebastian-software/ferromark/commit/8629bdc5c1287ceb67bdba8c9e4a2e5c76a818ab))
* indented code block content handling ([a2919ad](https://github.com/sebastian-software/ferromark/commit/a2919adefade19a20d0c858d5cf0d4b325896c55))
* **inline:** enable multi-line emphasis via paragraph accumulation ([917deaa](https://github.com/sebastian-software/ferromark/commit/917deaae19d0038d579910c49dbd6e6a69d08dfb))
* **inline:** handle escaped image markers and HTML spans in link destinations ([9885b4c](https://github.com/sebastian-software/ferromark/commit/9885b4c84b61ca4f0a6b993ef2f07591991a0450))
* **lists:** require same marker for list continuation ([3f0d1d9](https://github.com/sebastian-software/ferromark/commit/3f0d1d9cc9a41b182e4b63232d1bfaee707bb275))
* multiple CommonMark compliance improvements ([3cdc9f0](https://github.com/sebastian-software/ferromark/commit/3cdc9f094ccf302af6b8520701c418098fda4d36))
* only apply same-list continuation when all parents matched ([76dfdfd](https://github.com/sebastian-software/ferromark/commit/76dfdfdbc850150d52f5866cbb496eb78a85b252))
* proper content_indent for blank list items with trailing spaces ([bd7f801](https://github.com/sebastian-software/ferromark/commit/bd7f801d469ce642fd249556a0fbba1fa0a2f1a0))
* proper nested list rendering and tight/loose detection (Phase 3) ([1ff4a3e](https://github.com/sebastian-software/ferromark/commit/1ff4a3ed31d3b9da940146a1f55aea5aca008715))
* recognize blank list items in same-list continuation ([691e160](https://github.com/sebastian-software/ferromark/commit/691e1607d5de1aa5ad894dcf5a810f884d821058))
* remove blank line breaking HTML table rendering on GitHub ([29e1ef6](https://github.com/sebastian-software/ferromark/commit/29e1ef6d83ad90aeb3dc8d5dd8f532eccbe5d320))
* **render:** render headings and info strings correctly ([6da858a](https://github.com/sebastian-software/ferromark/commit/6da858ac5a2f14d0d5360f88474675c0e2a00921))
* reset cursor position on failed blockquote match ([e514de5](https://github.com/sebastian-software/ferromark/commit/e514de55de92edf71f94d5c3f0eb7d897da147ec))
* two-blank-line rule keeps list open for more items ([b0f2883](https://github.com/sebastian-software/ferromark/commit/b0f28834487e3322595b7b72ccadd94581fb25cc))


### Performance Improvements

* add byte dispatch guards for block parser ([77f14c3](https://github.com/sebastian-software/ferromark/commit/77f14c34373ded642e4b6fdca2c25aa6013e2242))
* **block:** fast path simple lines ([ba1c057](https://github.com/sebastian-software/ferromark/commit/ba1c05718c0fa4f3725639d8297b7a521b52c829))
* **block:** preallocate link ref buffer ([b06dc29](https://github.com/sebastian-software/ferromark/commit/b06dc29f55564688fafbadced729cefed28f3f75))
* **block:** reduce cursor peeks ([9fa2479](https://github.com/sebastian-software/ferromark/commit/9fa2479a805161c80a101ba01c03e11468db6a6b))
* **block:** skip container matching when empty ([ef0f0ea](https://github.com/sebastian-software/ferromark/commit/ef0f0eafc9dea20a836884fd10a2187a15d00c14))
* defer link ref materialization until insert ([19acffc](https://github.com/sebastian-software/ferromark/commit/19acffcf2290bbf8c2258af216893811183c0288))
* drop ampersand from inline special precheck path ([b48ecfc](https://github.com/sebastian-software/ferromark/commit/b48ecfcabf8448fdcb7703ec529d227cc9996974))
* fast-path link title rendering in refs path ([5caf88e](https://github.com/sebastian-software/ferromark/commit/5caf88e6fbfebfe6c7d0ae1820d11d4bbdcac813))
* fast-path safe link destination URLs ([6a6ed26](https://github.com/sebastian-software/ferromark/commit/6a6ed26371f25cb94a24e13e2a6b1db24740126e))
* fast-path simple paragraph runs ([a8a2b41](https://github.com/sebastian-software/ferromark/commit/a8a2b41105e53fe18e4b65abd3439713954861f4))
* fix autolink detection regression with memchr-based scanning ([df5d8dd](https://github.com/sebastian-software/ferromark/commit/df5d8dd697c1cda27f6f5aa355c14a492996b6da))
* improve commonmark50k parser throughput ([ddd95d5](https://github.com/sebastian-software/ferromark/commit/ddd95d519b0b652ce3ab7b1800c0fd65a5d09bc9))
* **inline:** add ascii fast path ([b3c044c](https://github.com/sebastian-software/ferromark/commit/b3c044c8eb68e876c34f2d353ccab28ba24ef3ba))
* **inline:** add NEON scans for specials ([b276631](https://github.com/sebastian-software/ferromark/commit/b276631304a4fdb6b8f02dc40461f048960cde02))
* **inline:** cut emit hot-path checks ([a60bf54](https://github.com/sebastian-software/ferromark/commit/a60bf5417705477188f927ef85883169c0b415b6))
* **inline:** drop opener cleanup retain ([b573db8](https://github.com/sebastian-software/ferromark/commit/b573db897ff29563fb7a3359f7fd756604493aaa))
* **inline:** precompute span ranges for emit checks ([bfb1b9c](https://github.com/sebastian-software/ferromark/commit/bfb1b9ccaa189df57dcf868f7de13a1ad99dd992))
* **inline:** reduce emit allocations ([db2bc28](https://github.com/sebastian-software/ferromark/commit/db2bc288ecee7397c118553cb59d7945b5b4cc6c))
* **inline:** reserve mark buffer ([e5f6924](https://github.com/sebastian-software/ferromark/commit/e5f69242532d62dd861b03b94e61976c64649447))
* **inline:** reuse emphasis stacks ([4f481a8](https://github.com/sebastian-software/ferromark/commit/4f481a87047908608b13a873d3dcbc0ec26f61bd))
* **inline:** reuse event buffers in streaming emit ([5a0cd17](https://github.com/sebastian-software/ferromark/commit/5a0cd17e3d4a13c7363a7dbabf9fe94c66975207))
* **inline:** reuse link dest and range buffers ([d1dcb1b](https://github.com/sebastian-software/ferromark/commit/d1dcb1b4a0890e00fe5fb9a792bda32f7f5283c8))
* **inline:** reuse scratch buffers for links ([9489a36](https://github.com/sebastian-software/ferromark/commit/9489a36f10abcf44ddc044064c961fc4291a1fe6))
* **inline:** scan autolinks with memchr ([1a3c2a5](https://github.com/sebastian-software/ferromark/commit/1a3c2a5dfddcca3b3ff1b36d56b82b4578370395))
* **inline:** skip bracket collection without marks ([1315330](https://github.com/sebastian-software/ferromark/commit/1315330a5168447438e5bc46909f33a73a9ca699))
* **inline:** skip emphasis resolve when absent ([5607e01](https://github.com/sebastian-software/ferromark/commit/5607e0175aa58b4e46bcefabb55a5ac3c832fefb))
* **inline:** skip html/autolink and link resolution when unused ([cb8cf96](https://github.com/sebastian-software/ferromark/commit/cb8cf963a0a9ce49fcd821970ac717ae0cfc4639))
* **inline:** speed up mark scan ([6f8786a](https://github.com/sebastian-software/ferromark/commit/6f8786ad8281dba25b616c17b11dd6884a1a0d41))
* **inline:** stream event emission ([b1b9cdd](https://github.com/sebastian-software/ferromark/commit/b1b9cdd6d69f90e7152e35c5e5e87de23cd9b29a))
* **inline:** use unstable sort for emit points ([f788a90](https://github.com/sebastian-software/ferromark/commit/f788a902ad1773bb0d0ada55e72ec001a0a00c8f))
* **link-ref:** reuse label normalization buffer ([92d6a2e](https://github.com/sebastian-software/ferromark/commit/92d6a2e645cfba0efac77a8935f12d1286611d78))
* narrow link-ref extraction scan to paragraph start ([5a4696e](https://github.com/sebastian-software/ferromark/commit/5a4696eb3bcfc05813562e78c435bc3a029a68f7))
* optimize paragraph parsing and refresh benchmark docs ([456dd01](https://github.com/sebastian-software/ferromark/commit/456dd0191e3afd8014698643e4452e711925d80a))
* optimize table detection and make autolink literals opt-in ([b85d7a1](https://github.com/sebastian-software/ferromark/commit/b85d7a134a61ef684562a1843ef6c985551a26af))
* prealloc inline vectors and fast-path link ref defs ([570cd40](https://github.com/sebastian-software/ferromark/commit/570cd40cebecc4c54ae29f41e79ceba3bf9d02d6))
* preallocate block and inline event buffers ([4b30895](https://github.com/sebastian-software/ferromark/commit/4b30895857d7d95da2fb06950357fe91d6ae8ace))
* **render:** add escape fast path ([357ec07](https://github.com/sebastian-software/ferromark/commit/357ec07b81e8f20ca9fd536b99d5741754336c62))
* **render:** fast-path text without entities ([00b3ba4](https://github.com/sebastian-software/ferromark/commit/00b3ba49c5ebd185d8466263e5d0ac6d2656e5bb))
* **render:** reserve inline events ([3b019c3](https://github.com/sebastian-software/ferromark/commit/3b019c30e84af2e16407a0492eee419ecd67f4d4))
* reuse inline scratch buffers and pre-size parser state ([54e9597](https://github.com/sebastian-software/ferromark/commit/54e9597140e6162efa438b884bb6cc6f00867041))
* reuse link-ref parse buffer for refs extraction ([90b9fb2](https://github.com/sebastian-software/ferromark/commit/90b9fb28fda37e82198f204f2b12ce84d9fa1f47))
* reuse scratch label buffer for nested ref checks ([56ea971](https://github.com/sebastian-software/ferromark/commit/56ea97129116704838fce1ef0731e02392aefc6e))
* skip inline link resolver when no ]( candidate ([a0e5a2a](https://github.com/sebastian-software/ferromark/commit/a0e5a2a4e1a15e06f5c67f399c72c01b1dc7da0d))
* speed up blank-line detection ([b1d9dd4](https://github.com/sebastian-software/ferromark/commit/b1d9dd446589def9ba4daf0318b3c1e0785d3878))
* stream contiguous paragraph refs extraction ([6b14dc4](https://github.com/sebastian-software/ferromark/commit/6b14dc4c7a65bd36fbaa8fce98e111bd30952845))
* tighten inline-special precheck to avoid false positives ([d15523c](https://github.com/sebastian-software/ferromark/commit/d15523c8b3da965b6d67d944d5432cf11954e076))


### Reverts

* stream inline event emission ([9a721a5](https://github.com/sebastian-software/ferromark/commit/9a721a57c06f69e8ec572f1bc9bdc419eddf57f2))
