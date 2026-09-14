# Feature timing matrix

Every percentage is `(enabled time / disabled time - 1) × 100`, using
the median of paired ratios. Positive values mean more time. The primary
stage is parse-only for parser options and render-only for renderer options.
Plain prose reveals idle checks; active probes repeat feature syntax. Their
different output is intentional and is not an optimization comparison.

Sizes name approximate input targets; exact byte counts, absolute times,
pair ranges, and round medians are in `summary.csv` and `summary.json`.
Treat small changes as noise unless confirmed by longer independent runs.

| Feature | Plain ~300 B | Plain ~4 KiB | Plain ~64 KiB | Active ~300 B | Active ~4 KiB | Active ~64 KiB | Active ~4 KiB parse + render, reuse |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `parser.footnotes` | +0.4% | +0.0% | -0.0% | +37.5% | +39.8% | +41.9% | +45.3% |
| `parser.task_lists` | -0.2% | -0.7% | +0.2% | -30.2% | -24.6% | -23.6% | -27.4% |
| `parser.tables` | -1.7% | +5.3% | +4.6% | +185.7% | +193.9% | +188.8% | +170.1% |
| `parser.merged_table_cells` | -0.1% | -0.2% | -0.2% | -1.3% | -0.4% | -2.0% | -0.9% |
| `parser.table_attributes` | -0.2% | +0.1% | -0.3% | +22.8% | +23.4% | +23.5% | +21.8% |
| `parser.line_comments` | +19.8% | +22.5% | +22.4% | +88.7% | +88.3% | +88.5% | +61.7% |
| `parser.front_matter` | +1.7% | -0.5% | +0.0% | -40.8% | -0.4% | +8.3% | -1.4% |
| `parser.strikethrough` | -0.0% | -0.4% | +0.1% | +86.1% | +95.3% | +91.1% | +42.3% |
| `parser.autolinks` | +32.6% | +43.6% | +44.9% | +890.3% | +1019.6% | +1040.2% | +942.9% |
| `parser.superscript` | +0.1% | +0.3% | +0.1% | +61.4% | +64.7% | +65.3% | +74.6% |
| `parser.subscript` | +0.0% | +0.2% | +0.1% | -3.5% | -2.8% | -3.1% | -5.6% |
| `parser.math` | +0.8% | +0.4% | +0.3% | -23.0% | -24.1% | -23.2% | +18.2% |
| `parser.definition_lists` | +168.2% | +217.6% | +221.9% | +280.6% | +284.6% | +275.4% | +235.8% |
| `parser.heading_attributes` | +0.4% | -0.0% | -0.3% | +64.4% | +68.4% | +71.0% | -4.7% |
| `parser.wiki_links` | -0.1% | +0.2% | +0.2% | -47.8% | -46.0% | -44.0% | -30.0% |
| `parser.cjk_emphasis` | -0.6% | +0.1% | -0.2% | +10.1% | +10.6% | +10.7% | +3.2% |
| `parser.mdx` | +0.7% | +0.2% | +0.5% | +39.9% | +43.1% | +44.5% | +274.1% |
| `renderer.xhtml` | -0.7% | +0.1% | -0.0% | -0.5% | -1.5% | -0.6% | -0.9% |
| `renderer.sanitize` | -0.7% | -0.2% | +1.0% | +191.9% | +195.7% | +200.9% | +47.2% |
| `renderer.disallow_raw_html` | -0.4% | +0.3% | +0.1% | +339.1% | +395.7% | +395.9% | +85.2% |
| `renderer.convert_md_links` | +0.0% | -0.5% | -0.0% | +212.2% | +221.3% | +218.2% | +66.6% |
| `renderer.code_fence_metadata` | +0.4% | -0.2% | +0.1% | +35.0% | +40.5% | +37.7% | +9.3% |
| `renderer.code_annotations` | -0.8% | +0.2% | +0.1% | +1206.4% | +1300.3% | +1321.5% | +325.0% |
| `renderer.code_annotation_default_line_numbers` | +0.1% | +0.7% | -0.2% | +89.9% | +89.5% | +89.8% | +66.8% |
| `renderer.autolink_urls` | +114.5% | +54.5% | +49.1% | +531.3% | +593.2% | +544.4% | +204.6% |
| `renderer.autolink_target_blank` | +0.2% | -0.3% | +0.1% | +0.8% | +1.0% | +0.9% | +0.6% |
| `renderer.link_target_blank` | +0.5% | +0.2% | +0.2% | +0.5% | +0.8% | +0.7% | +0.4% |
| `renderer.semantic_footnotes` | +0.1% | +0.4% | -0.3% | -3.6% | -10.0% | -5.5% | -0.9% |
| `renderer.heading_permalinks` | +0.2% | +0.1% | +0.1% | +22.3% | +21.5% | +20.9% | +10.8% |
| `renderer.source_spans` | +82.5% | +104.1% | +110.0% | +137.2% | +152.0% | +153.3% | +37.3% |
| `renderer.heading_ids` | -0.1% | +0.2% | -0.3% | +179.1% | +195.8% | +198.9% | +53.8% |
| `renderer.callouts` | -0.5% | +0.1% | -0.7% | +75.5% | +77.5% | +83.2% | +12.2% |
| `renderer.inline_toc` | +0.7% | +1.4% | -0.2% | +168.8% | +170.4% | +171.8% | +88.1% |
| `renderer.table_colgroup` | +0.4% | +0.8% | -0.7% | +35.4% | +37.0% | +34.8% | +4.9% |
| `renderer.table_column_names` | +0.3% | -0.3% | -0.1% | +259.5% | +260.8% | +266.3% | +47.0% |
| `renderer.highlight` | +0.5% | -0.0% | +0.6% | +1.4% | -0.6% | -0.6% | +0.1% |
| `renderer.soft_break` | -0.0% | -0.6% | -0.8% | +0.1% | -0.3% | -0.2% | -0.7% |

`highlight` and `soft_break` are currently ignored by the renderer;
their variation is a control, not evidence of implemented functionality.
`source_spans` changes even plain-prose HTML. All other plain-probe
equality results are recorded rather than assumed.
