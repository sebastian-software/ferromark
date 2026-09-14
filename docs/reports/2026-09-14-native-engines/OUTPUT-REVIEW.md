# Native comparison output review

Reviewed the archived 57-case corpus and all six native outputs. This report describes observed output groups; it does not treat any engine as a semantic oracle. The raw six-engine HTML remains in [`verification.json.gz`](verification.json.gz).

## Aggregate shape

- 57 workloads were verified; 14 satisfy `strict_all_six` (exact or the verifier's explicitly allowed serialization equivalence), and 43 are diagnostic mismatches.
- `v2` and `ox-content` are byte-identical on all 57 cases. That is an observed implementation relationship, not an assertion that either output is the correct rendering.
- Of the 43 diagnostic cases, 34 are `heading-id-only`: V1, md4c, pulldown-cmark, and Bun agree after canonicalization except that they omit heading `id` attributes emitted by V2/OX. These are fragment-link/output differences, so they are excluded from the strict aggregate.
- The remaining nine cases have at least one `other` relationship. Their observed differences are below.

The 34 heading-ID-only cases are: `comment-incident`; `legacy-contributing`, `legacy-node-ferromark-readme`, `legacy-docs-migration-0-2`, `legacy-docs-migration-0-3`, `legacy-docs-migration-0-4`, `legacy-docs-migration-0-8`, `legacy-docs-releasing`, `legacy-docs-readme-theme`, `legacy-docs-markdown-extensions`, `legacy-docs-mdx`, `legacy-docs-readme`, `legacy-docs-adr-readme-theme-composition`; `rust-book-ch03-04-comments`, `rust-book-appendix-02-operators`, `rust-book-ch00-00-introduction`, `rust-book-ch17-00-async-await`; `vue-docs-ways-of-using-vue`, `vue-docs-suspense`; `vite-docs-philosophy`, `vite-docs-performance`, `vite-docs-api-plugin`; `typescript-handbook-the-handbook`, `typescript-handbook-advanced-types`, `typescript-handbook-compiler-options`, `typescript-handbook-typescript-5-0`; and the encyclopedia pairs `wiki-rainbow-first-paragraph`, `wiki-rainbow-lead`, `wiki-tea-first-paragraph`, `wiki-tea-lead`, `wiki-chess-first-paragraph`, `wiki-chess-lead`, `wiki-volcano-first-paragraph`, `wiki-volcano-lead`.

## `other` cases

| Case | Engines in an `other` relationship | Observed rendering difference |
| --- | --- | --- |
| `comment-checklist` | md4c, Bun | Both add `class="task-list-item"` to each `<li>` and `class="task-list-item-checkbox"` to the checkbox. V2/OX/V1/pulldown emit no classes and include a leading space before the item text; md4c/Bun omit that space. Checked/disabled state is present in both forms. |
| `guard-angle-link` | V1 | The source is one inline link with an angle-bracket destination. V2/OX/md4c/pulldown/Bun emit one link. V1 emits that link and then a second autolink containing the full URL, so the URL appears twice. |
| `vue-docs-slots` | V1, md4c, pulldown-cmark, Bun | All four omit the V2/OX heading IDs. The fenced code info string `vue-html{2}` is preserved in their class (`language-vue-html{2}`), while V2/OX use `language-vue-html`; the raw source also contains Vue component markup and an explicit `{#...}` heading attribute. |
| `vue-docs-reactivity-in-depth` | V1, md4c, pulldown-cmark, Bun | All four omit heading IDs. The highlighted fence info `js{4,9,17,22}` is preserved as `language-js{4,9,17,22}`, while V2/OX use `language-js`. V1 additionally serializes the horizontal rule as `<hr />` in raw HTML. |
| `vite-docs-features` | V1 | V1 omits heading IDs and, for the explicit angle-bracket MDN destination in the `data:` heading, emits a second autolink containing the full URL after the intended code link. The other engines do not duplicate that URL link. |
| `wiki-rainbow-article-body` | V1, md4c | V1 omits IDs and misinterprets underscores in a URL destination (`German_Peasants'_War`) as emphasis, producing an extra `<em>War)` fragment and misplaced closing emphasis. md4c also omits IDs and percent-encodes an apostrophe in an href (`Snell's_law` → `Snell%27s_law`), while the V2/OX group retains the apostrophe. |
| `wiki-tea-article-body` | md4c | md4c percent-encodes the apostrophe in the href for `Pu'er_City` (`Pu%27er_City`); V2/OX retain it. The other engines are in the heading-ID-only/serialization-equivalent groups. |
| `wiki-chess-article-body` | V1, md4c | V1 omits IDs and misinterprets the underscore in `H.J.R._Murray` as emphasis, producing an extra `<em>Murray)` fragment. md4c also omits IDs and percent-encodes the apostrophe in `Scholar's_mate` (`%27`). |
| `wiki-volcano-article-body` | V1, md4c | V1 omits IDs and misinterprets the underscore in the `Plymouth,_Montserrat` destination as emphasis, producing an extra `<em>Montserrat)` fragment. md4c also omits IDs and percent-encodes the apostrophe in `Earth's_mantle` (`%27`). |

The V1 underscore cases are structural HTML differences, not merely entity spelling. The md4c URL cases are href serialization differences that the verifier deliberately does not collapse because changing apostrophe escaping can affect exact URL bytes and was not established as semantically interchangeable for this comparison.
