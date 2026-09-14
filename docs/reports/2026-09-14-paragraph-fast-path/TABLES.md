# Timing tables

Time ratios: **lower is faster**. Aggregates give each document equal weight.

## A-native: 14 agreeing documents

| Candidate | Mode | Candidate / baseline | Candidate / OX | Round 1 vs baseline | Round 2 vs baseline |
| --- | --- | ---: | ---: | ---: | ---: |
| A | fresh | 0.9526 | 1.0393 | 0.9519 | 0.9519 |
| A | reuse | 0.9498 | 1.0574 | 0.9506 | 0.9494 |
| A | init | 0.9995 | 1.4314 | 0.9982 | 1.0006 |
| A | parse | 0.9255 | 1.0691 | 0.9174 | 0.9265 |
| A | render | 1.0008 | 1.0227 | 0.9993 | 1.0005 |

## B-native: 14 agreeing documents

| Candidate | Mode | Candidate / baseline | Candidate / OX | Round 1 vs baseline | Round 2 vs baseline |
| --- | --- | ---: | ---: | ---: | ---: |
| A | fresh | 0.9514 | 1.0401 | 0.9524 | 0.9500 |
| A | reuse | 0.9484 | 1.0556 | 0.9312 | 0.9477 |
| A | init | 1.0009 | 1.4332 | 1.0006 | 1.0021 |
| A | parse | 0.9263 | 1.0698 | 0.9250 | 0.9255 |
| A | render | 1.0004 | 1.0248 | 1.0003 | 1.0007 |
| B | fresh | 0.9513 | 1.0400 | 0.9621 | 0.9491 |
| B | reuse | 0.9413 | 1.0477 | 0.9243 | 0.9421 |
| B | init | 1.0017 | 1.4343 | 1.0004 | 1.0030 |
| B | parse | 0.9170 | 1.0591 | 0.9157 | 0.9180 |
| B | render | 1.0021 | 1.0265 | 1.0013 | 1.0018 |

## A-features: runtime profiles

| Group | Documents | Fresh / baseline | Reuse / baseline | Parse / baseline |
| --- | ---: | ---: | ---: | ---: |
| Comments off, plain prose | 3 | 0.9412 | 0.9189 | 0.8796 |
| Comments on, no comments present | 3 | 0.9391 | 0.9125 | 0.8771 |
| Comments on, active comments | 3 | 1.0102 | 1.0083 | 1.0177 |
| Mixed CommonMark documents | 13 | 0.9712 | 0.9677 | 0.9582 |
| Mixed docs plus definition lists | 13 | 0.9879 | 0.9824 | 0.9835 |

## B-features: runtime profiles

| Group | Documents | Fresh / baseline | Reuse / baseline | Parse / baseline |
| --- | ---: | ---: | ---: | ---: |
| Comments off, plain prose | 3 | 0.9228 | 0.9084 | 0.8675 |
| Comments on, no comments present | 3 | 0.9288 | 0.9126 | 0.8730 |
| Comments on, active comments | 3 | 1.0127 | 1.0151 | 1.0232 |
| Mixed CommonMark documents | 13 | 0.9760 | 0.9629 | 0.9545 |
| Mixed docs plus definition lists | 13 | 0.9778 | 0.9739 | 0.9653 |

## accepted-features: runtime profiles

| Group | Documents | Fresh / baseline | Reuse / baseline | Parse / baseline |
| --- | ---: | ---: | ---: | ---: |
| Comments off, plain prose | 3 | 0.9332 | 0.9056 | 0.8654 |
| Comments on, no comments present | 3 | 0.9330 | 0.9079 | 0.8745 |
| Comments on, active comments | 3 | 1.0234 | 1.0200 | 1.0335 |
| Mixed CommonMark documents | 13 | 0.9794 | 0.9658 | 0.9550 |
| Mixed docs plus definition lists | 13 | 0.9812 | 0.9776 | 0.9707 |

## Prototype confirmation: all 57 native-comparison documents

| Mode | A / baseline | B / baseline | B / A |
| --- | ---: | ---: | ---: |
| fresh | 0.9728 | 0.9740 | 1.0013 |
| reuse | 0.9722 | 0.9720 | 0.9997 |
| parse | 0.9649 | 0.9602 | 0.9952 |

## Accepted source: all 57 inputs and the agreeing OX subset

| Mode | Candidate / baseline, all 57 | Candidate / baseline, agreeing 14 | Candidate / OX, agreeing 14 |
| --- | ---: | ---: | ---: |
| fresh | 0.9728 | 0.9522 | 1.0401 |
| reuse | 0.9715 | 0.9417 | 1.0495 |
| parse | 0.9626 | 0.9171 | 1.0579 |

## Same-binary flag cost on three plain-prose sizes

| Core | Fresh on/off | Reuse on/off | Parse on/off |
| --- | ---: | ---: | ---: |
| flag-baseline | 1.0000 | 1.0010 | 1.0039 |
| flag-B | 1.0077 | 1.0064 | 1.0044 |
| accepted-flag | 1.0033 | 1.0035 | 1.0091 |
