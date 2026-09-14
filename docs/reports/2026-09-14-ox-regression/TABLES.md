# Diagnostic tables

Time ratio against the stated control; **lower is faster**. Each aggregate
weights the 14 documents equally. These are descriptive local measurements.

## Stage screen

| Variant | Core | Fresh / OX | Reuse / OX | Init / OX | Parse / OX | Render / OX |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| v2 | `33c216b` | 1.0916 | 1.1203 | 1.4315 | 1.1543 | 1.0240 |

## Coarse history

| Variant | Core | Fresh / OX | Reuse / OX | Init / OX | Parse / OX | Render / OX |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| pre | `adf891a` | 0.9789 | 0.9835 | 0.9970 | 0.9772 | 1.0004 |
| fix1 | `4a1e55f` | 1.0026 | 1.0171 | 1.3553 | 1.0265 | 1.0035 |
| fix2 | `db987c8` | 1.0161 | 1.0237 | 1.3538 | 1.0309 | 1.0147 |
| features | `e0eba1d` | 1.0939 | 1.1195 | 1.4231 | 1.1672 | 1.0297 |
| current | `33c216b` | 1.0949 | 1.1161 | 1.4286 | 1.1546 | 1.0254 |

## Fine history and paragraph ablation

| Variant | Core | Fresh / OX | Reuse / OX | Init / OX | Parse / OX | Render / OX |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| fix2 | `db987c8` | 1.0138 | 1.0241 | 1.3544 | 1.0303 | 1.0149 |
| typo | `7010384` | 1.0170 | 1.0248 | 1.3553 | 1.0258 | 1.0238 |
| tables | `e01c611` | 1.0186 | 1.0272 | 1.3551 | 1.0288 | 1.0255 |
| columns | `46c761d` | 1.0187 | 1.0305 | 1.3539 | 1.0329 | 1.0228 |
| comments | `40047f7` | 1.0868 | 1.1132 | 1.3921 | 1.1544 | 1.0230 |
| features | `e0eba1d` | 1.0950 | 1.1217 | 1.4278 | 1.1648 | 1.0288 |
| current | `33c216b` | 1.0926 | 1.1139 | 1.4335 | 1.1537 | 1.0213 |
| paragraph | `33c216b` | 1.0381 | 1.0438 | 1.4419 | 1.0539 | 1.0205 |

## Ablations against their own controls

| Experiment | Mode | Candidate / control | Round 1 | Round 2 |
| --- | --- | ---: | ---: | ---: |
| normalization: skip/control | fresh | 0.9844 | 0.9835 | 0.9843 |
| normalization: skip/control | reuse | 0.9715 | 0.9715 | 0.9711 |
| normalization: skip/control | init | 0.7335 | 0.7348 | 0.7303 |
| normalization: skip/control | parse | 0.9668 | 0.9698 | 0.9664 |
| normalization: skip/control | render | 1.0069 | 1.0131 | 1.0020 |
| normalization: control/current | fresh | 0.9858 | 0.9859 | 0.9862 |
| normalization: control/current | reuse | 1.0015 | 1.0006 | 1.0009 |
| normalization: control/current | init | 1.0097 | 1.0099 | 1.0118 |
| normalization: control/current | parse | 0.9984 | 1.0003 | 0.9969 |
| normalization: control/current | render | 1.0012 | 1.0007 | 0.9981 |
| history-fine: paragraph/current | fresh | 0.9501 | 0.9512 | 0.9511 |
| history-fine: paragraph/current | reuse | 0.9371 | 0.9361 | 0.9378 |
| history-fine: paragraph/current | init | 1.0059 | 0.9999 | 1.0129 |
| history-fine: paragraph/current | parse | 0.9135 | 0.9139 | 0.9146 |
| history-fine: paragraph/current | render | 0.9992 | 0.9981 | 0.9982 |

## Selected absolute stage times

From the independent stage screen, nanoseconds per document.

| Input | Mode | OX, ns | v2, ns |
| --- | --- | ---: | ---: |
| comment-ack | fresh | 161.62 | 177.31 |
| comment-ack | reuse | 69.42 | 79.98 |
| comment-ack | init | 11.85 | 13.57 |
| comment-ack | parse | 40.93 | 50.84 |
| comment-ack | render | 28.76 | 29.20 |
| comment-review | fresh | 219.69 | 248.75 |
| comment-review | reuse | 129.57 | 151.55 |
| comment-review | init | 12.83 | 19.46 |
| comment-review | parse | 83.02 | 102.32 |
| comment-review | render | 48.90 | 49.59 |
| comment-table | fresh | 1018.84 | 1085.54 |
| comment-table | reuse | 929.93 | 1004.22 |
| comment-table | init | 14.09 | 21.40 |
| comment-table | parse | 726.65 | 778.36 |
| comment-table | render | 185.31 | 203.84 |
| wiki-chess-plain-prose | fresh | 28607.13 | 31274.61 |
| wiki-chess-plain-prose | reuse | 28591.70 | 31198.59 |
| wiki-chess-plain-prose | init | 3058.78 | 4085.10 |
| wiki-chess-plain-prose | parse | 18358.04 | 20870.68 |
| wiki-chess-plain-prose | render | 9117.83 | 9115.28 |

## Structure sizes

Native bytes from `size_of`, using the first node of `comment-ack`.

| Variant | Parser | Parser options | Renderer | Document | Node enum |
| --- | ---: | ---: | ---: | ---: | ---: |
| ox | 192 | 24 | 960 | 40 | 32 |
| pre | 192 | 24 | 960 | 40 | 32 |
| fix1 | 200 | 24 | 960 | 40 | 32 |
| fix2 | 200 | 24 | 968 | 40 | 32 |
| typo | 200 | 24 | 968 | 40 | 32 |
| tables | 200 | 24 | 968 | 40 | 32 |
| columns | 200 | 24 | 968 | 40 | 32 |
| comments | 224 | 32 | 968 | 40 | 32 |
| features | 232 | 32 | 968 | 48 | 32 |
| current | 256 | 32 | 968 | 48 | 32 |
