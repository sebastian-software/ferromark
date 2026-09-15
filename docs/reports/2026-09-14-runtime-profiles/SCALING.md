# Active-probe scaling

Enabled primary-stage time divided by original input bytes. These are
repeated syntax shapes at three sizes, not proofs of worst-case complexity.
Frontmatter has a fixed prefix and TOC a single marker; those shapes differ
from the features repeated throughout a document. Blank/escaped/near-miss
and adversarial inputs need separate studies before complexity claims.

| Feature | ~300 B ns/B | ~4 KiB ns/B | ~64 KiB ns/B | 64 KiB / 4 KiB time per byte |
| --- | ---: | ---: | ---: | ---: |
| `parser.footnotes` | 10.427 | 9.827 | 9.753 | 0.99× |
| `parser.task_lists` | 3.580 | 3.301 | 3.308 | 1.00× |
| `parser.tables` | 7.634 | 7.516 | 7.640 | 1.02× |
| `parser.merged_table_cells` | 8.408 | 8.326 | 8.268 | 0.99× |
| `parser.table_attributes` | 7.281 | 7.288 | 7.258 | 1.00× |
| `parser.line_comments` | 2.522 | 2.395 | 2.388 | 1.00× |
| `parser.front_matter` | 0.422 | 0.341 | 0.329 | 0.96× |
| `parser.strikethrough` | 4.695 | 4.690 | 4.754 | 1.01× |
| `parser.autolinks` | 5.498 | 5.365 | 5.423 | 1.01× |
| `parser.superscript` | 1.528 | 1.454 | 1.451 | 1.00× |
| `parser.subscript` | 1.660 | 1.551 | 1.589 | 1.02× |
| `parser.math` | 2.102 | 1.983 | 1.991 | 1.00× |
| `parser.definition_lists` | 12.895 | 12.677 | 12.802 | 1.01× |
| `parser.heading_attributes` | 3.251 | 3.107 | 3.169 | 1.02× |
| `parser.wiki_links` | 3.646 | 3.472 | 3.496 | 1.01× |
| `parser.cjk_emphasis` | 5.206 | 5.087 | 5.103 | 1.00× |
| `parser.mdx` | 4.906 | 4.839 | 4.818 | 1.00× |
| `renderer.xhtml` | 1.318 | 1.266 | 1.285 | 1.01× |
| `renderer.sanitize` | 2.156 | 2.126 | 2.150 | 1.01× |
| `renderer.disallow_raw_html` | 1.775 | 1.755 | 1.728 | 0.98× |
| `renderer.convert_md_links` | 4.234 | 4.234 | 4.229 | 1.00× |
| `renderer.code_fence_metadata` | 0.829 | 0.775 | 0.763 | 0.98× |
| `renderer.code_annotations` | 8.335 | 8.358 | 8.360 | 1.00× |
| `renderer.code_annotation_default_line_numbers` | 13.948 | 13.794 | 13.914 | 1.01× |
| `renderer.autolink_urls` | 2.297 | 2.154 | 2.209 | 1.03× |
| `renderer.autolink_target_blank` | 2.378 | 2.234 | 2.300 | 1.03× |
| `renderer.link_target_blank` | 1.404 | 1.382 | 1.405 | 1.02× |
| `renderer.semantic_footnotes` | 2.538 | 2.352 | 2.592 | 1.10× |
| `renderer.heading_permalinks` | 3.554 | 3.658 | 3.691 | 1.01× |
| `renderer.source_spans` | 3.133 | 3.238 | 3.343 | 1.03× |
| `renderer.heading_ids` | 2.941 | 2.993 | 3.035 | 1.01× |
| `renderer.callouts` | 3.019 | 2.962 | 3.052 | 1.03× |
| `renderer.inline_toc` | 7.826 | 7.989 | 8.154 | 1.02× |
| `renderer.table_colgroup` | 1.690 | 1.646 | 1.648 | 1.00× |
| `renderer.table_column_names` | 6.087 | 5.984 | 5.981 | 1.00× |
| `renderer.highlight` | 0.440 | 0.377 | 0.375 | 0.99× |
| `renderer.soft_break` | 0.488 | 0.444 | 0.488 | 1.10× |
