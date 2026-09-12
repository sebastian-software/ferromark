# Native heap measurements

Generated from `raw.jsonl.gz`; all 30 measurements per engine/case agree exactly.

## Large-document focus: growing Ox arena

MiB uses 1,048,576 bytes. These are additional peak live requested Rust heap bytes, including owned HTML and excluding source storage. Negative change favors Ferromark.

| Case | Input MiB | Ferromark peak MiB | Ox growing peak MiB | Ferromark change |
|---|---:|---:|---:|---:|
| large/mixed-1mib | 1.000 | 6.98 | 14.27 | -51.1% |
| large/plain-1mib | 1.000 | 5.25 | 2.80 | +87.4% |
| large/mixed-8mib | 8.000 | 55.84 | 114.10 | -51.1% |
| large/plain-8mib | 8.000 | 42.00 | 22.40 | +87.5% |

## All matching cases: growing Ox arena

Incremental peak of simultaneously live, requested Rust heap bytes during a fresh Markdown→owned HTML operation. Source storage and pre-existing options are excluded; parser, renderer, scratch, arena and owned output are included. This is not process RSS or physical allocator consumption. Negative change means fewer peak bytes for Ferromark.

| Case | Input bytes | Ferromark peak bytes | Ox growing peak bytes | Ferromark change |
|---|---:|---:|---:|---:|
| ox-large-cm | 49898 | 357226 | 731305 | -51.2% |
| ox-large-tables | 49898 | 357226 | 731305 | -51.2% |
| ox-huge-tables | 1072848 | 7481162 | 15318645 | -51.2% |
| short | 17 | 3312 | 1841 | +79.9% |
| plain | 6300 | 15731 | 37413 | -58.0% |
| multiline | 7500 | 26363 | 39813 | -33.8% |
| inline | 8100 | 31354 | 270341 | -88.4% |
| lists | 5300 | 57874 | 275341 | -79.0% |
| code | 6200 | 23228 | 37213 | -37.6% |
| tables | 7500 | 80222 | 153085 | -47.6% |
| headings | 4200 | 69264 | 160869 | -56.9% |
| unique-headings | 4388 | 70972 | 161621 | -56.1% |
| long-prose | 65536 | 344096 | 184573 | +86.4% |
| late-marker | 65540 | 393813 | 184581 | +113.4% |
| long-code | 70712 | 160630 | 199021 | -19.3% |
| escape-dense | 2700 | 20412 | 23325 | -12.5% |
| commonmark/tiny | 18 | 3312 | 1845 | +79.5% |
| commonmark/short-100b | 103 | 4116 | 1979 | +108.0% |
| gfm_overlap/tiny | 18 | 3312 | 1845 | +79.5% |
| gfm_overlap/short-100b | 103 | 4116 | 1979 | +108.0% |
| tables/tables-plain | 4560 | 49448 | 79885 | -38.1% |
| tables/tables-commonmark-inline | 4800 | 64688 | 146365 | -55.8% |
| tables/tables-links | 6360 | 69252 | 148525 | -53.4% |
| commonmark/publication-5k | 5120 | 29352 | 68869 | -57.4% |
| guard/plain | 4560 | 49448 | 79885 | -38.1% |
| guard/mixed | 4800 | 64688 | 146365 | -55.8% |
| guard/links | 6360 | 69252 | 148525 | -53.4% |
| guard/mixed-document | 5632 | 34268 | 70389 | -51.3% |
| guard/one-table | 3056 | 33656 | 73869 | -54.4% |
| guard/long-cells | 53152 | 121160 | 241709 | -49.9% |
| guard/wide9 | 6621 | 71000 | 149569 | -52.5% |
| guard/wide16 | 11612 | 123414 | 230973 | -46.6% |
| guard/escapes-code | 6860 | 33208 | 267861 | -87.6% |
| guard/prose | 7050 | 24460 | 38913 | -37.1% |
| guard/links-prose | 5800 | 22836 | 134685 | -83.0% |
| guard/references | 876 | 7883 | 68685 | -88.5% |
| guard/empty | 0 | 2560 | 765 | +234.6% |
| guard/gfm-document | 5120 | 29352 | 68869 | -57.4% |
| guard/long-delimiters | 16426 | 38516 | 49489 | -22.2% |
| guard/deep-emphasis | 3073 | 173698 | 532641 | -67.4% |
| large/mixed-1mib | 1048898 | 7323174 | 14963545 | -51.1% |
| large/plain-1mib | 1048576 | 5505056 | 2937085 | +87.4% |
| large/mixed-8mib | 8388689 | 58547708 | 119640439 | -51.1% |
| large/plain-8mib | 8388608 | 44040224 | 23490813 | +87.5% |

## Excluded from the comparison

These retain raw diagnostics but differ in normalized HTML, so their memory values do not establish a fair comparison.

- guard/edge-cases

## Supplementary allocator diagnostics

The presized arena is a diagnostic control, not the memory claim baseline. Requested bytes count all requests, including the full new size on reallocation. Peak bytes count only live logical sizes. Allocation counts include reallocations.

| Case | Configuration | Peak bytes | Cumulative requested bytes | Calls |
|---|---|---:|---:|---:|
| ox-large-cm | ferro | 357226 | 581131 | 39 |
| ox-large-cm | ox-grow | 731305 | 741686 | 22 |
| ox-large-cm | ox-presize | 1321161 | 1331542 | 20 |
| ox-large-tables | ferro | 357226 | 581131 | 39 |
| ox-large-tables | ox-grow | 731305 | 741686 | 22 |
| ox-large-tables | ox-presize | 1321161 | 1331542 | 20 |
| ox-huge-tables | ferro | 7481162 | 12306587 | 44 |
| ox-huge-tables | ox-grow | 15318645 | 15654442 | 27 |
| ox-huge-tables | ox-presize | 28171925 | 28507722 | 25 |
| short | ferro | 3312 | 3312 | 10 |
| short | ox-grow | 1841 | 1876 | 14 |
| short | ox-presize | 20801 | 20836 | 13 |
| plain | ferro | 15731 | 15731 | 4 |
| plain | ox-grow | 37413 | 37414 | 13 |
| plain | ox-presize | 66101 | 66102 | 12 |
| multiline | ferro | 26363 | 33851 | 9 |
| multiline | ox-grow | 39813 | 39814 | 13 |
| multiline | ox-presize | 76693 | 76694 | 12 |
| inline | ferro | 31354 | 41479 | 19 |
| inline | ox-grow | 270341 | 280722 | 22 |
| inline | ox-presize | 213045 | 223426 | 19 |
| lists | ferro | 57874 | 105635 | 22 |
| lists | ox-grow | 275341 | 285942 | 17 |
| lists | ox-presize | 156605 | 167206 | 14 |
| code | ferro | 23228 | 30978 | 4 |
| code | ox-grow | 37213 | 37214 | 13 |
| code | ox-presize | 65901 | 65902 | 12 |
| tables | ferro | 80222 | 142013 | 8 |
| tables | ox-grow | 153085 | 168086 | 16 |
| tables | ox-presize | 214557 | 229558 | 14 |
| headings | ferro | 69264 | 100990 | 12 |
| headings | ox-grow | 160869 | 169270 | 18 |
| headings | ox-presize | 144533 | 152934 | 15 |
| unique-headings | ferro | 70972 | 103979 | 12 |
| unique-headings | ox-grow | 161621 | 170398 | 18 |
| unique-headings | ox-presize | 145285 | 154062 | 15 |
| long-prose | ferro | 344096 | 345632 | 5 |
| long-prose | ox-grow | 184573 | 184574 | 12 |
| long-prose | ox-presize | 659709 | 659710 | 12 |
| late-marker | ferro | 393813 | 395349 | 11 |
| late-marker | ox-grow | 184581 | 184582 | 12 |
| late-marker | ox-presize | 659717 | 659718 | 12 |
| long-code | ferro | 160630 | 160630 | 3 |
| long-code | ox-grow | 199021 | 199022 | 12 |
| long-code | ox-presize | 711021 | 711022 | 12 |
| escape-dense | ferro | 20412 | 33225 | 6 |
| escape-dense | ox-grow | 23325 | 28726 | 14 |
| escape-dense | ox-presize | 35629 | 41030 | 13 |
| commonmark/tiny | ferro | 3312 | 3312 | 10 |
| commonmark/tiny | ox-grow | 1845 | 1882 | 14 |
| commonmark/tiny | ox-presize | 20805 | 20842 | 13 |
| commonmark/short-100b | ferro | 4116 | 4244 | 18 |
| commonmark/short-100b | ox-grow | 1979 | 2152 | 14 |
| commonmark/short-100b | ox-presize | 20939 | 21112 | 13 |
| gfm_overlap/tiny | ferro | 3312 | 3312 | 10 |
| gfm_overlap/tiny | ox-grow | 1845 | 1882 | 14 |
| gfm_overlap/tiny | ox-presize | 20805 | 20842 | 13 |
| gfm_overlap/short-100b | ferro | 4116 | 4244 | 18 |
| gfm_overlap/short-100b | ox-grow | 1979 | 2152 | 14 |
| gfm_overlap/short-100b | ox-presize | 20939 | 21112 | 13 |
| tables/tables-plain | ferro | 49448 | 87068 | 8 |
| tables/tables-plain | ox-grow | 79885 | 89006 | 16 |
| tables/tables-plain | ox-presize | 129069 | 138190 | 14 |
| tables/tables-commonmark-inline | ferro | 64688 | 116288 | 16 |
| tables/tables-commonmark-inline | ox-grow | 146365 | 155966 | 17 |
| tables/tables-commonmark-inline | ox-presize | 142317 | 151918 | 14 |
| tables/tables-links | ferro | 69252 | 121666 | 21 |
| tables/tables-links | ox-grow | 148525 | 171626 | 22 |
| tables/tables-links | ox-presize | 185421 | 208522 | 20 |
| commonmark/publication-5k | ferro | 29352 | 41772 | 45 |
| commonmark/publication-5k | ox-grow | 68869 | 73994 | 20 |
| commonmark/publication-5k | ox-presize | 146709 | 151834 | 19 |
| guard/plain | ferro | 49448 | 87068 | 8 |
| guard/plain | ox-grow | 79885 | 89006 | 16 |
| guard/plain | ox-presize | 129069 | 138190 | 14 |
| guard/mixed | ferro | 64688 | 116288 | 16 |
| guard/mixed | ox-grow | 146365 | 155966 | 17 |
| guard/mixed | ox-presize | 142317 | 151918 | 14 |
| guard/links | ferro | 69252 | 121666 | 21 |
| guard/links | ox-grow | 148525 | 171626 | 22 |
| guard/links | ox-presize | 185421 | 208522 | 20 |
| guard/mixed-document | ferro | 34268 | 49040 | 69 |
| guard/mixed-document | ox-grow | 70389 | 72654 | 24 |
| guard/mixed-document | ox-presize | 61733 | 63998 | 21 |
| guard/one-table | ferro | 33656 | 58868 | 8 |
| guard/one-table | ox-grow | 73869 | 79982 | 16 |
| guard/one-table | ox-presize | 86189 | 92302 | 14 |
| guard/long-cells | ferro | 121160 | 121160 | 4 |
| guard/long-cells | ox-grow | 241709 | 241710 | 13 |
| guard/long-cells | ox-presize | 532541 | 532542 | 12 |
| guard/wide9 | ferro | 71000 | 144924 | 110 |
| guard/wide9 | ox-grow | 149569 | 162812 | 16 |
| guard/wide9 | ox-presize | 186465 | 199708 | 14 |
| guard/wide16 | ferro | 123414 | 238521 | 110 |
| guard/wide16 | ox-grow | 230973 | 254198 | 16 |
| guard/wide16 | ox-presize | 329309 | 352534 | 14 |
| guard/escapes-code | ferro | 33208 | 49873 | 220 |
| guard/escapes-code | ox-grow | 267861 | 267862 | 16 |
| guard/escapes-code | ox-presize | 185989 | 185990 | 13 |
| guard/prose | ferro | 24460 | 31500 | 5 |
| guard/prose | ox-grow | 38913 | 38914 | 13 |
| guard/prose | ox-presize | 71697 | 71698 | 12 |
| guard/links-prose | ferro | 22836 | 30086 | 17 |
| guard/links-prose | ox-grow | 134685 | 145066 | 21 |
| guard/links-prose | ox-presize | 159293 | 169674 | 19 |
| guard/references | ferro | 7883 | 10002 | 34 |
| guard/references | ox-grow | 68685 | 75846 | 26 |
| guard/references | ox-presize | 65644 | 72342 | 21 |
| guard/empty | ferro | 2560 | 2560 | 2 |
| guard/empty | ox-grow | 765 | 766 | 11 |
| guard/empty | ox-presize | 20733 | 20734 | 11 |
| guard/gfm-document | ferro | 29352 | 41772 | 45 |
| guard/gfm-document | ox-grow | 68869 | 73994 | 20 |
| guard/gfm-document | ox-presize | 146709 | 151834 | 19 |
| guard/long-delimiters | ferro | 38516 | 38516 | 4 |
| guard/long-delimiters | ox-grow | 49489 | 49490 | 12 |
| guard/long-delimiters | ox-presize | 168273 | 168274 | 12 |
| guard/deep-emphasis | ferro | 173698 | 215363 | 23 |
| guard/deep-emphasis | ox-grow | 532641 | 538788 | 19 |
| guard/deep-emphasis | ox-presize | 442577 | 448724 | 16 |
| large/mixed-1mib | ferro | 7323174 | 12046805 | 44 |
| large/mixed-1mib | ox-grow | 14963545 | 15299342 | 27 |
| large/mixed-1mib | ox-presize | 27546489 | 27882286 | 25 |
| large/plain-1mib | ferro | 5505056 | 5506592 | 5 |
| large/plain-1mib | ox-grow | 2937085 | 2937086 | 12 |
| large/plain-1mib | ox-presize | 10490109 | 10490110 | 12 |
| large/mixed-8mib | ferro | 58547708 | 96300462 | 47 |
| large/mixed-8mib | ox-grow | 119640439 | 122327364 | 30 |
| large/mixed-8mib | ox-presize | 220279191 | 222966116 | 28 |
| large/plain-8mib | ferro | 44040224 | 44041760 | 5 |
| large/plain-8mib | ox-grow | 23490813 | 23490814 | 12 |
| large/plain-8mib | ox-presize | 83890429 | 83890430 | 12 |
