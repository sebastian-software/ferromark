# Per-case tables

Ratios are baseline time over candidate time (median of the paired windows); higher is faster.
`rounds` lists the per-round medians. Baseline time is the median nanoseconds per document.

## A/A control, 16 documents

### fresh

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-checklist` | comments | 287 | 0.999 | 0.994 | 1.020 | 0.998 / 0.997 / 1.004 | 566 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 0.999 | 0.990 | 1.006 | 1.006 / 0.999 / 0.992 | 14,996 |
| `comment-review-long` | comments | 957 | 1.001 | 0.996 | 1.008 | 1.001 / 0.998 / 1.001 | 745 |
| `legacy-docs-releasing` | technical-docs | 4141 | 1.001 | 0.988 | 1.010 | 1.001 / 1.003 / 1.001 | 5,350 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.002 | 0.997 | 1.006 | 1.002 / 1.002 / 1.003 | 5,366 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.003 | 0.998 | 2.903 | 1.003 / 1.097 / 1.000 | 5,853 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 1.003 | 0.984 | 1.011 | 0.987 / 1.009 / 1.008 | 27,250 |
| `comment-ack` | comments | 37 | 1.005 | 0.993 | 1.010 | 1.005 / 1.009 / 0.999 | 146 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.005 | 0.995 | 1.009 | 1.005 / 1.002 / 1.007 | 2,452 |
| `comment-incident` | comments | 1124 | 1.006 | 0.978 | 1.014 | 1.010 / 1.010 / 1.003 | 1,380 |
| `comment-quote` | comments | 290 | 1.006 | 1.000 | 1.018 | 1.008 / 1.003 / 1.006 | 295 |
| `vite-docs-performance` | technical-docs | 8184 | 1.007 | 0.997 | 1.019 | 1.016 / 1.009 / 1.000 | 8,805 |
| `vite-docs-philosophy` | technical-docs | 3575 | 1.008 | 0.991 | 1.010 | 1.008 / 1.009 / 1.008 | 3,370 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.015 | 1.009 | 1.025 | 1.016 / 1.013 / 1.023 | 53,984 |
| `vite-docs-api-plugin` | reference | 31890 | 1.016 | 0.981 | 1.057 | 0.996 / 1.039 / 1.006 | 66,237 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.032 | 0.919 | 1.098 | 1.087 / 1.032 / 0.999 | 48,804 |

### reuse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 0.995 | 0.990 | 1.013 | 0.992 / 0.998 / 0.995 | 2,243 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 1.000 | 0.984 | 1.019 | 0.991 / 1.005 / 1.000 | 14,762 |
| `vite-docs-philosophy` | technical-docs | 3575 | 1.001 | 0.965 | 1.010 | 1.000 / 0.998 / 1.005 | 3,188 |
| `comment-quote` | comments | 290 | 1.001 | 0.987 | 1.006 | 0.999 / 1.005 / 0.995 | 213 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 1.001 | 0.988 | 1.034 | 1.009 / 1.001 / 0.992 | 27,149 |
| `comment-incident` | comments | 1124 | 1.001 | 0.995 | 1.007 | 0.997 / 1.001 / 1.006 | 1,221 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.001 | 0.998 | 1.012 | 1.001 / 1.002 / 1.000 | 5,517 |
| `vite-docs-performance` | technical-docs | 8184 | 1.002 | 1.000 | 1.546 | 1.008 / 1.006 / 1.001 | 8,483 |
| `comment-ack` | comments | 37 | 1.002 | 0.997 | 1.012 | 1.002 / 1.002 / 1.000 | 67 |
| `comment-review-long` | comments | 957 | 1.002 | 0.988 | 1.006 | 1.000 / 1.002 / 1.003 | 657 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.003 | 0.995 | 1.010 | 1.006 / 0.999 / 1.003 | 4,993 |
| `legacy-docs-releasing` | technical-docs | 4141 | 1.006 | 1.001 | 1.013 | 1.006 / 1.009 / 1.002 | 5,002 |
| `comment-checklist` | comments | 287 | 1.006 | 0.996 | 1.010 | 1.006 / 1.004 / 1.006 | 484 |
| `vite-docs-api-plugin` | reference | 31890 | 1.014 | 0.897 | 1.070 | 0.983 / 1.014 / 1.016 | 63,663 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.014 | 0.945 | 1.044 | 0.988 / 1.014 / 1.033 | 47,703 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.015 | 1.010 | 1.024 | 1.016 / 1.014 / 1.011 | 53,733 |

### parse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-review-long` | comments | 957 | 0.992 | 0.982 | 1.014 | 0.999 / 0.990 / 0.983 | 506 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 0.994 | 0.985 | 1.008 | 1.000 / 0.993 / 0.994 | 1,319 |
| `vite-docs-api-plugin` | reference | 31890 | 0.995 | 0.972 | 1.012 | 0.995 / 0.992 / 0.997 | 47,518 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.997 | 0.968 | 1.010 | 1.006 / 1.003 / 0.978 | 3,604 |
| `vite-docs-philosophy` | technical-docs | 3575 | 0.997 | 0.983 | 1.013 | 0.998 / 0.999 / 0.995 | 1,985 |
| `typescript-handbook-advanced-types` | reference | 36745 | 0.997 | 0.937 | 1.022 | 0.997 / 1.008 / 0.996 | 28,877 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 0.997 | 0.994 | 1.010 | 0.995 / 0.997 / 1.006 | 3,746 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.997 | 0.984 | 1.012 | 0.989 / 1.001 / 0.997 | 17,823 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 0.998 | 0.990 | 1.005 | 1.001 / 0.998 / 0.994 | 10,068 |
| `comment-ack` | comments | 37 | 1.001 | 0.990 | 1.007 | 1.005 / 0.998 / 1.000 | 50 |
| `legacy-docs-releasing` | technical-docs | 4141 | 1.001 | 0.994 | 1.003 | 1.001 / 1.002 / 1.000 | 3,386 |
| `comment-quote` | comments | 290 | 1.001 | 0.994 | 1.006 | 0.995 / 1.006 / 1.001 | 166 |
| `vite-docs-performance` | technical-docs | 8184 | 1.002 | 0.986 | 1.034 | 0.998 / 1.011 / 1.002 | 5,715 |
| `comment-incident` | comments | 1124 | 1.004 | 0.984 | 1.017 | 1.007 / 1.004 / 0.991 | 934 |
| `comment-checklist` | comments | 287 | 1.006 | 0.993 | 1.018 | 1.011 / 1.006 / 1.000 | 386 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.012 | 0.997 | 1.035 | 1.002 / 1.011 / 1.019 | 45,987 |

### render

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-quote` | comments | 290 | 0.998 | 0.989 | 1.006 | 0.996 / 0.998 / 1.002 | 49 |
| `comment-ack` | comments | 37 | 0.998 | 0.987 | 1.005 | 0.999 / 0.994 / 0.998 | 19 |
| `comment-review-long` | comments | 957 | 0.998 | 0.981 | 1.011 | 1.002 / 0.996 / 0.998 | 149 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.000 | 0.993 | 1.017 | 0.999 / 1.005 / 0.999 | 1,203 |
| `comment-checklist` | comments | 287 | 1.001 | 0.964 | 1.019 | 1.011 / 1.007 / 0.998 | 97 |
| `comment-incident` | comments | 1124 | 1.001 | 0.990 | 1.007 | 1.004 / 0.999 / 1.002 | 285 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 1.003 | 0.988 | 1.015 | 1.010 / 1.003 / 1.003 | 4,567 |
| `vite-docs-philosophy` | technical-docs | 3575 | 1.005 | 0.996 | 1.016 | 1.004 / 1.013 / 1.002 | 1,180 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 1.005 | 0.984 | 1.021 | 1.012 / 1.005 / 1.005 | 1,913 |
| `vite-docs-api-plugin` | reference | 31890 | 1.007 | 0.994 | 1.027 | 1.010 / 0.999 / 1.002 | 12,076 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 1.008 | 0.998 | 1.021 | 1.009 / 1.011 / 1.006 | 8,833 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.008 | 0.998 | 1.025 | 1.003 / 1.008 / 1.012 | 906 |
| `vite-docs-performance` | technical-docs | 8184 | 1.010 | 1.000 | 1.022 | 1.004 / 1.015 / 1.008 | 2,687 |
| `legacy-docs-releasing` | technical-docs | 4141 | 1.018 | 1.007 | 1.024 | 1.017 / 1.015 / 1.022 | 1,592 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.034 | 1.025 | 1.039 | 1.039 / 1.035 / 1.029 | 7,600 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.040 | 1.001 | 1.050 | 1.011 / 1.040 / 1.046 | 16,190 |

## HEAD bffc89f6, 57 broad documents

### fresh

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-checklist` | comments | 287 | 0.702 | 0.677 | 0.710 | 0.681 / 0.702 / 0.706 | 565 |
| `comment-quote` | comments | 290 | 0.767 | 0.764 | 0.770 | 0.767 / 0.768 / 0.765 | 294 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 0.768 | 0.762 | 0.774 | 0.767 / 0.766 / 0.772 | 5,364 |
| `comment-review-long` | comments | 957 | 0.773 | 0.768 | 0.775 | 0.774 / 0.772 / 0.772 | 747 |
| `vite-docs-api-plugin` | reference | 31890 | 0.811 | 0.779 | 0.857 | 0.799 / 0.833 / 0.834 | 65,060 |
| `comment-incident` | comments | 1124 | 0.819 | 0.804 | 0.823 | 0.819 / 0.821 / 0.819 | 1,373 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.850 | 0.842 | 0.862 | 0.850 / 0.858 / 0.849 | 5,754 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 0.852 | 0.845 | 0.862 | 0.855 / 0.849 / 0.850 | 14,919 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.864 | 0.832 | 0.873 | 0.861 / 0.867 / 0.864 | 27,147 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.878 | 0.871 | 0.885 | 0.880 / 0.875 / 0.877 | 5,248 |
| `comment-ack` | comments | 37 | 0.881 | 0.874 | 0.892 | 0.885 / 0.881 / 0.881 | 144 |
| `vite-docs-performance` | technical-docs | 8184 | 0.884 | 0.868 | 0.889 | 0.883 / 0.885 / 0.884 | 8,747 |
| `comment-question` | comments | 160 | 0.900 | 0.888 | 0.906 | 0.899 / 0.901 / 0.896 | 166 |
| `vite-docs-features` | technical-docs | 39739 | 0.904 | 0.883 | 0.969 | 0.917 / 0.903 / 0.904 | 59,718 |
| `vite-docs-philosophy` | technical-docs | 3575 | 0.907 | 0.894 | 0.926 | 0.913 / 0.905 / 0.907 | 3,375 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 0.921 | 0.900 | 0.973 | 0.921 / 0.925 / 0.914 | 62,687 |
| `legacy-contributing` | technical-docs | 9323 | 0.923 | 0.919 | 0.944 | 0.922 / 0.927 / 0.925 | 10,542 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 0.925 | 0.900 | 0.955 | 0.926 / 0.924 / 0.923 | 8,128 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 0.929 | 0.888 | 0.997 | 0.929 / 0.945 / 0.929 | 114,203 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 0.935 | 0.916 | 0.961 | 0.924 / 0.936 / 0.936 | 2,982 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 0.937 | 0.931 | 0.956 | 0.935 / 0.937 / 0.940 | 12,018 |
| `comment-reproduction` | comments | 298 | 0.938 | 0.929 | 0.946 | 0.936 / 0.938 / 0.937 | 305 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 0.939 | 0.917 | 0.969 | 0.949 / 0.938 / 0.930 | 22,081 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 0.942 | 0.933 | 0.954 | 0.951 / 0.942 / 0.942 | 1,656 |
| `comment-review` | comments | 282 | 0.945 | 0.916 | 0.965 | 0.946 / 0.950 / 0.926 | 230 |
| `guard-angle-link` | syntax-guard | 41 | 0.950 | 0.941 | 0.973 | 0.954 / 0.955 / 0.948 | 263 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 0.951 | 0.947 | 0.965 | 0.951 / 0.951 / 0.952 | 14,152 |
| `wiki-tea-lead` | encyclopedia | 6363 | 0.952 | 0.946 | 0.965 | 0.951 / 0.951 / 0.954 | 7,342 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 0.954 | 0.872 | 1.056 | 0.905 / 0.961 / 0.954 | 55,743 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 0.955 | 0.943 | 0.969 | 0.955 / 0.958 / 0.957 | 1,361 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 0.959 | 0.951 | 0.965 | 0.959 / 0.962 / 0.955 | 1,018 |
| `comment-inline-code` | comments | 285 | 0.959 | 0.938 | 0.980 | 0.959 / 0.961 / 0.957 | 293 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 0.964 | 0.958 | 0.971 | 0.963 / 0.967 / 0.970 | 1,561 |
| `comment-unicode` | comments | 327 | 0.965 | 0.955 | 0.971 | 0.965 / 0.965 / 0.966 | 450 |
| `wiki-chess-lead` | encyclopedia | 4125 | 0.966 | 0.946 | 0.987 | 0.961 / 0.968 / 0.975 | 3,912 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 0.967 | 0.955 | 0.973 | 0.964 / 0.967 / 0.969 | 2,232 |
| `vue-docs-suspense` | technical-docs | 8291 | 0.968 | 0.962 | 0.976 | 0.965 / 0.965 / 0.974 | 9,360 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 0.969 | 0.956 | 0.974 | 0.970 / 0.965 / 0.970 | 3,098 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 0.974 | 0.966 | 0.983 | 0.975 / 0.966 / 0.978 | 681 |
| `vue-docs-slots` | technical-docs | 24211 | 0.977 | 0.960 | 0.996 | 0.980 / 0.974 / 0.977 | 22,930 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 0.978 | 0.959 | 0.982 | 0.979 / 0.978 / 0.972 | 3,824 |
| `legacy-docs-mdx` | technical-docs | 7422 | 0.978 | 0.962 | 0.994 | 0.978 / 0.976 / 0.983 | 7,711 |
| `comment-links` | comments | 278 | 0.979 | 0.960 | 0.991 | 0.976 / 0.986 / 0.972 | 462 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 0.979 | 0.965 | 0.991 | 0.984 / 0.978 / 0.977 | 2,495 |
| `comment-table` | comments | 310 | 0.979 | 0.959 | 0.993 | 0.979 / 0.972 / 0.985 | 1,118 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 0.982 | 0.896 | 1.023 | 0.982 / 0.996 / 0.961 | 32,853 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 0.983 | 0.906 | 1.091 | 0.975 / 0.983 / 0.984 | 60,133 |
| `legacy-node-ferromark-readme` | readme | 9075 | 0.985 | 0.977 | 0.992 | 0.982 / 0.985 / 0.987 | 10,085 |
| `legacy-docs-readme` | readme | 1825 | 0.985 | 0.971 | 0.992 | 0.985 / 0.976 / 0.986 | 4,557 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 0.985 | 0.977 | 0.995 | 0.987 / 0.985 / 0.982 | 2,903 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 0.987 | 0.974 | 0.994 | 0.987 / 0.987 / 0.985 | 9,292 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 0.988 | 0.961 | 0.998 | 0.988 / 0.989 / 0.980 | 1,607 |
| `typescript-handbook-compiler-options` | reference | 54026 | 0.991 | 0.963 | 1.024 | 0.993 / 0.991 / 0.985 | 77,297 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 0.993 | 0.978 | 0.999 | 0.992 / 0.992 / 0.995 | 2,444 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 0.994 | 0.991 | 1.001 | 0.993 / 0.994 / 0.996 | 7,350 |
| `typescript-handbook-advanced-types` | reference | 36745 | 0.996 | 0.942 | 1.079 | 0.996 / 0.969 / 1.008 | 48,648 |
| `rust-book-appendix-02-operators` | reference | 22595 | 0.998 | 0.987 | 1.009 | 0.999 / 0.996 / 0.998 | 54,056 |

### reuse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-checklist` | comments | 287 | 0.665 | 0.658 | 0.671 | 0.664 / 0.670 / 0.665 | 478 |
| `comment-quote` | comments | 290 | 0.707 | 0.701 | 0.713 | 0.707 / 0.704 / 0.705 | 213 |
| `comment-review-long` | comments | 957 | 0.748 | 0.742 | 0.751 | 0.748 / 0.750 / 0.747 | 651 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 0.758 | 0.751 | 0.764 | 0.758 / 0.760 / 0.753 | 4,995 |
| `comment-incident` | comments | 1124 | 0.797 | 0.794 | 0.805 | 0.799 / 0.797 / 0.797 | 1,218 |
| `vite-docs-api-plugin` | reference | 31890 | 0.801 | 0.764 | 0.835 | 0.804 / 0.794 / 0.801 | 63,373 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.846 | 0.817 | 0.866 | 0.845 / 0.853 / 0.846 | 5,472 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 0.852 | 0.835 | 0.861 | 0.846 / 0.852 / 0.853 | 14,680 |
| `comment-ack` | comments | 37 | 0.852 | 0.829 | 0.862 | 0.852 / 0.853 / 0.851 | 67 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.860 | 0.850 | 0.874 | 0.862 / 0.860 / 0.860 | 26,781 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.871 | 0.850 | 0.886 | 0.871 / 0.873 / 0.870 | 4,938 |
| `vite-docs-performance` | technical-docs | 8184 | 0.879 | 0.853 | 0.905 | 0.876 / 0.881 / 0.885 | 8,448 |
| `comment-question` | comments | 160 | 0.896 | 0.892 | 0.906 | 0.897 / 0.898 / 0.895 | 88 |
| `vite-docs-philosophy` | technical-docs | 3575 | 0.904 | 0.897 | 0.914 | 0.904 / 0.905 / 0.904 | 3,176 |
| `vite-docs-features` | technical-docs | 39739 | 0.908 | 0.765 | 0.957 | 0.917 / 0.894 / 0.906 | 59,573 |
| `comment-reproduction` | comments | 298 | 0.909 | 0.903 | 0.913 | 0.912 / 0.907 / 0.909 | 224 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 0.917 | 0.892 | 0.991 | 0.933 / 0.924 / 0.908 | 62,415 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 0.918 | 0.907 | 0.967 | 0.917 / 0.929 / 0.914 | 113,457 |
| `legacy-contributing` | technical-docs | 9323 | 0.920 | 0.913 | 0.927 | 0.918 / 0.920 / 0.920 | 10,148 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 0.921 | 0.916 | 0.926 | 0.921 / 0.922 / 0.918 | 7,730 |
| `comment-review` | comments | 282 | 0.928 | 0.922 | 0.941 | 0.929 / 0.926 / 0.927 | 149 |
| `guard-angle-link` | syntax-guard | 41 | 0.932 | 0.919 | 0.937 | 0.932 / 0.933 / 0.935 | 172 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 0.935 | 0.931 | 0.947 | 0.936 / 0.935 / 0.933 | 11,721 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 0.940 | 0.926 | 0.966 | 0.940 / 0.938 / 0.940 | 1,504 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 0.941 | 0.928 | 0.950 | 0.934 / 0.942 / 0.941 | 21,505 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 0.944 | 0.939 | 0.957 | 0.944 / 0.944 / 0.946 | 2,810 |
| `comment-unicode` | comments | 327 | 0.947 | 0.942 | 0.954 | 0.944 / 0.947 / 0.947 | 359 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 0.949 | 0.937 | 0.961 | 0.954 / 0.942 / 0.947 | 1,189 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 0.951 | 0.922 | 0.994 | 0.955 / 0.951 / 0.945 | 54,585 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 0.951 | 0.948 | 0.958 | 0.953 / 0.951 / 0.950 | 13,839 |
| `wiki-tea-lead` | encyclopedia | 6363 | 0.952 | 0.943 | 0.965 | 0.955 / 0.950 / 0.953 | 7,060 |
| `comment-inline-code` | comments | 285 | 0.953 | 0.941 | 0.977 | 0.952 / 0.948 / 0.953 | 210 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 0.955 | 0.941 | 0.983 | 0.955 / 0.950 / 0.961 | 847 |
| `comment-links` | comments | 278 | 0.956 | 0.935 | 0.962 | 0.959 / 0.951 / 0.956 | 370 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 0.961 | 0.940 | 0.967 | 0.961 / 0.961 / 0.960 | 1,390 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 0.962 | 0.954 | 0.967 | 0.962 / 0.959 / 0.962 | 2,079 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 0.963 | 0.944 | 0.976 | 0.964 / 0.971 / 0.962 | 2,825 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 0.965 | 0.957 | 0.979 | 0.971 / 0.965 / 0.963 | 509 |
| `vue-docs-suspense` | technical-docs | 8291 | 0.967 | 0.956 | 0.972 | 0.969 / 0.964 / 0.966 | 9,005 |
| `wiki-chess-lead` | encyclopedia | 4125 | 0.968 | 0.959 | 0.978 | 0.965 / 0.971 / 0.966 | 3,608 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 0.973 | 0.969 | 0.978 | 0.976 / 0.971 / 0.975 | 3,636 |
| `vue-docs-slots` | technical-docs | 24211 | 0.974 | 0.963 | 0.991 | 0.980 / 0.970 / 0.974 | 22,629 |
| `comment-table` | comments | 310 | 0.977 | 0.973 | 0.990 | 0.979 / 0.977 / 0.977 | 1,036 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 0.979 | 0.947 | 0.988 | 0.980 / 0.979 / 0.978 | 2,698 |
| `legacy-docs-mdx` | technical-docs | 7422 | 0.980 | 0.972 | 1.012 | 0.974 / 0.981 / 0.982 | 7,416 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 0.982 | 0.975 | 0.989 | 0.984 / 0.982 / 0.979 | 1,423 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 0.982 | 0.967 | 0.991 | 0.983 / 0.975 / 0.981 | 2,335 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 0.983 | 0.937 | 0.989 | 0.986 / 0.987 / 0.976 | 2,255 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 0.986 | 0.974 | 1.007 | 0.979 / 0.989 / 0.986 | 8,946 |
| `legacy-node-ferromark-readme` | readme | 9075 | 0.987 | 0.977 | 0.993 | 0.984 / 0.988 / 0.989 | 9,820 |
| `legacy-docs-readme` | readme | 1825 | 0.987 | 0.978 | 0.994 | 0.981 / 0.988 / 0.988 | 4,404 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 0.988 | 0.876 | 1.039 | 0.983 / 0.988 / 0.997 | 59,674 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 0.989 | 0.926 | 1.043 | 0.997 / 0.990 / 0.967 | 32,370 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 0.993 | 0.973 | 1.002 | 0.994 / 0.976 / 0.994 | 6,916 |
| `typescript-handbook-compiler-options` | reference | 54026 | 0.995 | 0.986 | 1.001 | 0.994 / 0.996 / 0.999 | 76,641 |
| `rust-book-appendix-02-operators` | reference | 22595 | 0.999 | 0.968 | 1.009 | 0.993 / 1.001 / 0.998 | 53,797 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.024 | 0.955 | 1.092 | 1.030 / 1.004 / 1.024 | 47,589 |

### parse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-checklist` | comments | 287 | 0.616 | 0.611 | 0.622 | 0.614 / 0.619 / 0.614 | 385 |
| `comment-quote` | comments | 290 | 0.651 | 0.645 | 0.661 | 0.649 / 0.656 / 0.650 | 166 |
| `comment-review-long` | comments | 957 | 0.697 | 0.678 | 0.705 | 0.698 / 0.685 / 0.702 | 514 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 0.700 | 0.690 | 0.705 | 0.700 / 0.703 / 0.696 | 3,731 |
| `comment-incident` | comments | 1124 | 0.751 | 0.746 | 0.754 | 0.750 / 0.751 / 0.751 | 922 |
| `vite-docs-api-plugin` | reference | 31890 | 0.764 | 0.742 | 0.782 | 0.769 / 0.764 / 0.757 | 47,349 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.787 | 0.776 | 0.794 | 0.790 / 0.789 / 0.778 | 3,542 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 0.794 | 0.782 | 0.815 | 0.796 / 0.794 / 0.792 | 9,990 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.804 | 0.800 | 0.813 | 0.802 / 0.804 / 0.805 | 17,658 |
| `comment-ack` | comments | 37 | 0.822 | 0.818 | 0.825 | 0.823 / 0.821 / 0.822 | 50 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.826 | 0.820 | 0.834 | 0.823 / 0.826 / 0.828 | 3,383 |
| `vite-docs-performance` | technical-docs | 8184 | 0.833 | 0.825 | 0.842 | 0.832 / 0.835 / 0.833 | 5,706 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 0.851 | 0.799 | 0.874 | 0.839 / 0.860 / 0.861 | 35,444 |
| `vite-docs-philosophy` | technical-docs | 3575 | 0.858 | 0.854 | 0.864 | 0.858 / 0.859 / 0.857 | 1,974 |
| `comment-question` | comments | 160 | 0.864 | 0.856 | 0.869 | 0.860 / 0.865 / 0.865 | 65 |
| `vite-docs-features` | technical-docs | 39739 | 0.875 | 0.830 | 0.941 | 0.875 / 0.854 / 0.875 | 36,177 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 0.877 | 0.849 | 0.918 | 0.872 / 0.900 / 0.854 | 65,205 |
| `comment-reproduction` | comments | 298 | 0.879 | 0.874 | 0.889 | 0.880 / 0.878 / 0.879 | 162 |
| `legacy-contributing` | technical-docs | 9323 | 0.893 | 0.879 | 0.904 | 0.892 / 0.893 / 0.886 | 7,445 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 0.902 | 0.896 | 0.925 | 0.902 / 0.900 / 0.903 | 1,720 |
| `comment-review` | comments | 282 | 0.906 | 0.897 | 0.913 | 0.908 / 0.904 / 0.905 | 109 |
| `guard-angle-link` | syntax-guard | 41 | 0.907 | 0.871 | 0.912 | 0.909 / 0.907 / 0.902 | 142 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 0.907 | 0.892 | 0.917 | 0.907 / 0.910 / 0.907 | 13,941 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 0.910 | 0.898 | 0.914 | 0.904 / 0.905 / 0.911 | 7,910 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 0.910 | 0.893 | 0.911 | 0.910 / 0.909 / 0.909 | 6,168 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 0.916 | 0.905 | 0.923 | 0.916 / 0.913 / 0.916 | 957 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 0.917 | 0.898 | 0.928 | 0.923 / 0.918 / 0.914 | 31,907 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 0.925 | 0.912 | 0.947 | 0.928 / 0.925 / 0.925 | 750 |
| `comment-inline-code` | comments | 285 | 0.927 | 0.919 | 0.957 | 0.927 / 0.923 / 0.935 | 136 |
| `wiki-tea-lead` | encyclopedia | 6363 | 0.933 | 0.914 | 0.945 | 0.935 / 0.929 / 0.932 | 4,544 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 0.938 | 0.915 | 0.947 | 0.937 / 0.938 / 0.938 | 534 |
| `comment-unicode` | comments | 327 | 0.943 | 0.934 | 0.951 | 0.945 / 0.943 / 0.938 | 268 |
| `comment-links` | comments | 278 | 0.943 | 0.938 | 0.947 | 0.943 / 0.945 / 0.940 | 279 |
| `vue-docs-suspense` | technical-docs | 8291 | 0.944 | 0.933 | 0.948 | 0.941 / 0.945 / 0.944 | 4,970 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 0.944 | 0.888 | 1.020 | 0.941 / 0.944 / 0.944 | 33,687 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 0.944 | 0.940 | 0.948 | 0.946 / 0.944 / 0.942 | 874 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 0.947 | 0.924 | 0.977 | 0.947 / 0.948 / 0.942 | 1,763 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 0.947 | 0.941 | 0.950 | 0.949 / 0.947 / 0.947 | 11,472 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 0.949 | 0.939 | 0.957 | 0.949 / 0.950 / 0.948 | 350 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 0.949 | 0.939 | 0.959 | 0.947 / 0.951 / 0.950 | 1,293 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 0.950 | 0.938 | 0.976 | 0.949 / 0.950 / 0.960 | 19,537 |
| `wiki-chess-lead` | encyclopedia | 4125 | 0.952 | 0.937 | 0.963 | 0.941 / 0.953 / 0.955 | 2,282 |
| `vue-docs-slots` | technical-docs | 24211 | 0.964 | 0.956 | 0.971 | 0.968 / 0.967 / 0.960 | 12,885 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 0.964 | 0.953 | 0.975 | 0.964 / 0.959 / 0.964 | 1,320 |
| `legacy-docs-mdx` | technical-docs | 7422 | 0.966 | 0.956 | 0.973 | 0.962 / 0.970 / 0.966 | 4,895 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 0.968 | 0.956 | 0.977 | 0.967 / 0.968 / 0.971 | 1,577 |
| `comment-table` | comments | 310 | 0.974 | 0.958 | 0.981 | 0.974 / 0.976 / 0.971 | 843 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 0.975 | 0.961 | 0.979 | 0.975 / 0.968 / 0.976 | 1,603 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 0.975 | 0.970 | 0.986 | 0.977 / 0.973 / 0.978 | 5,736 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 0.975 | 0.964 | 0.984 | 0.974 / 0.976 / 0.975 | 2,453 |
| `typescript-handbook-advanced-types` | reference | 36745 | 0.977 | 0.949 | 1.012 | 0.980 / 0.977 / 0.975 | 28,855 |
| `legacy-node-ferromark-readme` | readme | 9075 | 0.979 | 0.973 | 0.991 | 0.978 / 0.979 / 0.979 | 6,201 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 0.980 | 0.964 | 1.002 | 0.980 / 0.975 / 0.989 | 930 |
| `legacy-docs-readme` | readme | 1825 | 0.980 | 0.978 | 0.987 | 0.979 / 0.980 / 0.982 | 3,329 |
| `typescript-handbook-compiler-options` | reference | 54026 | 0.988 | 0.959 | 1.007 | 0.987 / 0.988 / 0.997 | 19,532 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 0.989 | 0.977 | 0.998 | 0.989 / 0.989 / 0.991 | 4,377 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.000 | 0.980 | 1.011 | 1.002 / 1.001 / 0.997 | 45,672 |

### render

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-unicode` | comments | 327 | 0.980 | 0.941 | 0.987 | 0.982 / 0.979 / 0.981 | 93 |
| `wiki-tea-lead` | encyclopedia | 6363 | 0.981 | 0.966 | 0.995 | 0.983 / 0.973 / 0.982 | 2,498 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 0.983 | 0.973 | 1.000 | 0.988 / 0.981 / 0.980 | 509 |
| `comment-ack` | comments | 37 | 0.983 | 0.974 | 0.992 | 0.982 / 0.985 / 0.983 | 19 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 0.984 | 0.975 | 0.994 | 0.984 / 0.987 / 0.982 | 535 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 0.985 | 0.975 | 0.994 | 0.987 / 0.988 / 0.984 | 2,265 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 0.986 | 0.975 | 1.008 | 0.982 / 0.992 / 0.984 | 781 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 0.986 | 0.972 | 0.995 | 0.980 / 0.988 / 0.986 | 1,075 |
| `guard-angle-link` | syntax-guard | 41 | 0.988 | 0.973 | 1.001 | 0.986 / 0.985 / 0.990 | 31 |
| `comment-question` | comments | 160 | 0.988 | 0.980 | 1.000 | 0.986 / 0.993 / 0.988 | 24 |
| `wiki-chess-lead` | encyclopedia | 4125 | 0.988 | 0.978 | 1.012 | 0.986 / 0.989 / 0.988 | 1,315 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 0.988 | 0.973 | 0.993 | 0.988 / 0.988 / 0.983 | 313 |
| `vite-docs-api-plugin` | reference | 31890 | 0.989 | 0.979 | 1.004 | 0.989 / 0.985 / 0.997 | 12,054 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 0.989 | 0.982 | 0.996 | 0.988 / 0.986 / 0.991 | 439 |
| `vue-docs-suspense` | technical-docs | 8291 | 0.989 | 0.983 | 0.997 | 0.990 / 0.989 / 0.989 | 3,953 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.990 | 0.978 | 1.002 | 0.995 / 0.989 / 0.988 | 1,872 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 0.991 | 0.975 | 1.000 | 0.987 / 0.991 / 0.995 | 151 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 0.992 | 0.980 | 1.003 | 0.993 / 0.990 / 0.993 | 704 |
| `legacy-contributing` | technical-docs | 9323 | 0.992 | 0.976 | 1.036 | 0.991 / 0.993 / 0.989 | 2,669 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 0.992 | 0.976 | 1.009 | 0.998 / 0.988 / 0.990 | 3,747 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 0.993 | 0.982 | 1.001 | 0.993 / 0.993 / 0.993 | 1,493 |
| `comment-review` | comments | 282 | 0.993 | 0.985 | 1.000 | 0.993 / 0.993 / 0.993 | 43 |
| `vite-docs-philosophy` | technical-docs | 3575 | 0.993 | 0.988 | 1.001 | 0.995 / 0.993 / 0.990 | 1,172 |
| `comment-inline-code` | comments | 285 | 0.994 | 0.980 | 1.013 | 0.994 / 0.994 / 0.992 | 82 |
| `vue-docs-slots` | technical-docs | 24211 | 0.994 | 0.975 | 0.998 | 0.997 / 0.994 / 0.992 | 9,393 |
| `comment-review-long` | comments | 957 | 0.994 | 0.985 | 1.008 | 0.994 / 0.997 / 0.994 | 147 |
| `comment-checklist` | comments | 287 | 0.995 | 0.990 | 1.005 | 0.994 / 0.992 / 0.997 | 96 |
| `vite-docs-performance` | technical-docs | 8184 | 0.995 | 0.977 | 1.016 | 0.994 / 0.995 / 0.996 | 2,662 |
| `legacy-node-ferromark-readme` | readme | 9075 | 0.995 | 0.983 | 1.000 | 0.993 / 0.995 / 0.995 | 3,492 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 0.995 | 0.986 | 0.998 | 0.996 / 0.995 / 0.995 | 1,049 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 0.995 | 0.988 | 1.013 | 0.995 / 0.995 / 1.000 | 7,220 |
| `comment-table` | comments | 310 | 0.996 | 0.988 | 1.007 | 0.991 / 0.999 / 0.996 | 184 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 0.996 | 0.986 | 1.002 | 0.995 / 0.997 / 0.996 | 1,109 |
| `comment-links` | comments | 278 | 0.996 | 0.992 | 1.010 | 0.996 / 1.004 / 0.994 | 88 |
| `comment-reproduction` | comments | 298 | 0.997 | 0.984 | 1.006 | 0.995 / 0.996 / 0.997 | 64 |
| `legacy-docs-mdx` | technical-docs | 7422 | 0.998 | 0.989 | 1.010 | 0.994 / 0.998 / 1.005 | 2,450 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 0.998 | 0.996 | 1.003 | 0.998 / 0.999 / 0.998 | 1,165 |
| `comment-quote` | comments | 290 | 0.998 | 0.984 | 1.004 | 0.996 / 1.000 / 0.996 | 49 |
| `typescript-handbook-compiler-options` | reference | 54026 | 0.998 | 0.988 | 1.009 | 1.004 / 0.994 / 1.000 | 57,004 |
| `legacy-docs-readme` | readme | 1825 | 0.999 | 0.599 | 1.017 | 0.998 / 1.002 / 0.999 | 1,039 |
| `comment-incident` | comments | 1124 | 0.999 | 0.990 | 1.018 | 1.001 / 1.002 / 0.997 | 282 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 0.999 | 0.994 | 1.012 | 0.996 / 1.006 / 1.000 | 4,514 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 1.000 | 0.995 | 1.009 | 0.999 / 0.999 / 1.002 | 497 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 1.001 | 0.993 | 1.009 | 0.994 / 0.999 / 1.003 | 3,265 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 1.001 | 0.988 | 1.018 | 1.001 / 0.999 / 1.002 | 8,742 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 1.001 | 0.995 | 1.004 | 1.001 / 0.999 / 1.002 | 2,495 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.002 | 0.993 | 1.025 | 0.996 / 1.008 / 1.009 | 901 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.003 | 0.979 | 1.025 | 1.004 / 0.991 / 1.003 | 1,209 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.005 | 1.002 | 1.014 | 1.005 / 1.005 / 1.008 | 7,593 |
| `legacy-docs-releasing` | technical-docs | 4141 | 1.005 | 0.984 | 1.017 | 1.007 / 0.997 / 1.005 | 1,583 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.009 | 0.990 | 1.037 | 1.009 / 1.022 / 1.005 | 15,752 |
| `vite-docs-features` | technical-docs | 39739 | 1.018 | 0.993 | 1.052 | 1.014 / 1.032 / 1.003 | 17,453 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 1.031 | 0.999 | 1.090 | 1.036 / 1.017 / 1.031 | 11,347 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.050 | 0.998 | 1.090 | 1.050 / 1.062 / 1.034 | 18,519 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 1.071 | 1.006 | 1.181 | 1.066 / 1.071 / 1.097 | 18,776 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.075 | 1.027 | 1.109 | 1.047 / 1.107 / 1.075 | 22,156 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 1.117 | 1.041 | 1.246 | 1.119 / 1.090 / 1.130 | 35,546 |

## 8d957e30 (#383), 16 documents

### fresh

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-checklist` | comments | 287 | 0.706 | 0.697 | 0.714 | 0.702 / 0.712 / 0.706 | 559 |
| `comment-review-long` | comments | 957 | 0.775 | 0.760 | 0.790 | 0.775 / 0.789 / 0.760 | 737 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 0.788 | 0.787 | 0.792 | 0.790 / 0.787 / 0.788 | 5,283 |
| `comment-quote` | comments | 290 | 0.796 | 0.784 | 0.799 | 0.788 / 0.796 / 0.797 | 292 |
| `vite-docs-api-plugin` | reference | 31890 | 0.808 | 0.793 | 0.863 | 0.808 / 0.798 / 0.829 | 66,424 |
| `comment-incident` | comments | 1124 | 0.837 | 0.825 | 0.842 | 0.838 / 0.828 / 0.839 | 1,357 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 0.850 | 0.826 | 0.858 | 0.834 / 0.852 / 0.852 | 14,825 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.868 | 0.863 | 0.888 | 0.868 / 0.883 / 0.865 | 26,764 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.875 | 0.872 | 0.881 | 0.877 / 0.874 / 0.875 | 5,179 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.887 | 0.886 | 0.892 | 0.890 / 0.887 / 0.888 | 5,720 |
| `vite-docs-performance` | technical-docs | 8184 | 0.893 | 0.885 | 0.900 | 0.891 / 0.896 / 0.893 | 8,623 |
| `vite-docs-philosophy` | technical-docs | 3575 | 0.931 | 0.921 | 0.935 | 0.931 / 0.931 / 0.932 | 3,310 |
| `comment-ack` | comments | 37 | 0.971 | 0.966 | 0.976 | 0.976 / 0.971 / 0.967 | 144 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 0.991 | 0.976 | 1.000 | 0.979 / 0.991 / 0.992 | 2,480 |
| `rust-book-appendix-02-operators` | reference | 22595 | 0.996 | 0.991 | 1.002 | 0.994 / 0.996 / 0.998 | 53,373 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.003 | 0.976 | 1.149 | 0.977 / 1.004 / 0.999 | 47,185 |

### reuse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-checklist` | comments | 287 | 0.677 | 0.671 | 0.688 | 0.682 / 0.674 / 0.677 | 474 |
| `comment-quote` | comments | 290 | 0.736 | 0.731 | 0.742 | 0.735 / 0.735 / 0.741 | 211 |
| `comment-review-long` | comments | 957 | 0.762 | 0.760 | 0.773 | 0.763 / 0.762 / 0.761 | 646 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 0.774 | 0.757 | 0.783 | 0.780 / 0.774 / 0.774 | 4,919 |
| `vite-docs-api-plugin` | reference | 31890 | 0.799 | 0.774 | 0.886 | 0.787 / 0.799 / 0.829 | 62,211 |
| `comment-incident` | comments | 1124 | 0.809 | 0.800 | 0.811 | 0.807 / 0.808 / 0.811 | 1,205 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 0.847 | 0.842 | 0.854 | 0.846 / 0.847 / 0.849 | 14,492 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.867 | 0.861 | 0.885 | 0.867 / 0.867 / 0.866 | 26,410 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.871 | 0.861 | 0.886 | 0.874 / 0.873 / 0.868 | 4,931 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.882 | 0.876 | 0.886 | 0.882 / 0.882 / 0.882 | 5,424 |
| `vite-docs-performance` | technical-docs | 8184 | 0.887 | 0.878 | 0.890 | 0.887 / 0.885 / 0.889 | 8,330 |
| `comment-ack` | comments | 37 | 0.918 | 0.915 | 0.922 | 0.918 / 0.921 / 0.918 | 66 |
| `vite-docs-philosophy` | technical-docs | 3575 | 0.926 | 0.924 | 0.934 | 0.926 / 0.925 / 0.928 | 3,155 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 0.984 | 0.974 | 0.987 | 0.986 / 0.984 / 0.984 | 2,220 |
| `rust-book-appendix-02-operators` | reference | 22595 | 0.998 | 0.987 | 1.003 | 0.995 / 1.000 / 0.996 | 52,721 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.019 | 0.994 | 1.089 | 1.024 / 0.997 / 1.087 | 47,229 |

### parse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-checklist` | comments | 287 | 0.626 | 0.622 | 0.632 | 0.627 / 0.625 / 0.631 | 379 |
| `comment-quote` | comments | 290 | 0.685 | 0.682 | 0.688 | 0.686 / 0.685 / 0.685 | 163 |
| `comment-review-long` | comments | 957 | 0.715 | 0.710 | 0.722 | 0.716 / 0.713 / 0.716 | 498 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 0.720 | 0.716 | 0.724 | 0.720 / 0.719 / 0.721 | 3,697 |
| `vite-docs-api-plugin` | reference | 31890 | 0.754 | 0.746 | 0.788 | 0.754 / 0.771 / 0.754 | 46,535 |
| `comment-incident` | comments | 1124 | 0.766 | 0.760 | 0.767 | 0.767 / 0.761 / 0.766 | 913 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 0.796 | 0.790 | 0.807 | 0.793 / 0.801 / 0.795 | 9,921 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.809 | 0.800 | 0.811 | 0.808 / 0.811 / 0.809 | 17,503 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.827 | 0.821 | 0.838 | 0.833 / 0.822 / 0.827 | 3,340 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.830 | 0.822 | 0.834 | 0.834 / 0.830 / 0.828 | 3,533 |
| `vite-docs-performance` | technical-docs | 8184 | 0.845 | 0.831 | 0.863 | 0.845 / 0.844 / 0.846 | 5,639 |
| `vite-docs-philosophy` | technical-docs | 3575 | 0.883 | 0.864 | 0.890 | 0.864 / 0.884 / 0.883 | 1,956 |
| `comment-ack` | comments | 37 | 0.898 | 0.881 | 0.902 | 0.893 / 0.896 / 0.898 | 49 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 0.974 | 0.969 | 0.981 | 0.975 / 0.972 / 0.975 | 1,302 |
| `typescript-handbook-advanced-types` | reference | 36745 | 0.984 | 0.912 | 1.004 | 0.989 / 0.984 / 0.973 | 28,410 |
| `rust-book-appendix-02-operators` | reference | 22595 | 0.998 | 0.991 | 1.002 | 0.996 / 0.993 / 1.000 | 45,161 |

### render

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-ack` | comments | 37 | 0.982 | 0.975 | 0.985 | 0.983 / 0.982 / 0.983 | 19 |
| `comment-quote` | comments | 290 | 0.989 | 0.611 | 1.001 | 0.993 / 0.989 / 0.989 | 48 |
| `comment-checklist` | comments | 287 | 0.990 | 0.970 | 0.997 | 0.984 / 0.991 / 0.989 | 95 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.991 | 0.983 | 0.998 | 0.995 / 0.991 / 0.988 | 1,847 |
| `vite-docs-performance` | technical-docs | 8184 | 0.991 | 0.982 | 0.996 | 0.983 / 0.994 / 0.993 | 2,640 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 0.993 | 0.987 | 1.010 | 0.994 / 0.991 / 0.993 | 4,518 |
| `vite-docs-api-plugin` | reference | 31890 | 0.993 | 0.978 | 1.100 | 0.993 / 0.985 / 1.042 | 11,964 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 0.994 | 0.988 | 1.000 | 0.990 / 0.996 / 0.997 | 888 |
| `vite-docs-philosophy` | technical-docs | 3575 | 0.996 | 0.989 | 0.999 | 0.996 / 0.996 / 0.992 | 1,157 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.997 | 0.988 | 0.999 | 0.998 / 0.997 / 0.994 | 1,573 |
| `comment-review-long` | comments | 957 | 0.998 | 0.990 | 1.017 | 0.998 / 0.998 / 0.993 | 147 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 1.000 | 0.983 | 1.016 | 1.010 / 1.012 / 0.994 | 8,798 |
| `comment-incident` | comments | 1124 | 1.000 | 0.978 | 1.005 | 0.987 / 0.997 / 1.000 | 280 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.000 | 0.997 | 1.006 | 1.000 / 1.005 / 0.998 | 1,191 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.014 | 0.991 | 1.060 | 0.997 / 1.023 / 1.031 | 15,582 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.019 | 1.012 | 1.040 | 1.025 / 1.019 / 1.019 | 7,525 |

## HEAD bffc89f6, 16 documents

### fresh

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-checklist` | comments | 287 | 0.702 | 0.695 | 0.709 | 0.704 / 0.700 / 0.700 | 563 |
| `comment-quote` | comments | 290 | 0.762 | 0.753 | 0.769 | 0.755 / 0.761 / 0.765 | 294 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 0.767 | 0.761 | 0.771 | 0.764 / 0.768 / 0.766 | 5,343 |
| `comment-review-long` | comments | 957 | 0.773 | 0.736 | 0.779 | 0.773 / 0.768 / 0.777 | 746 |
| `vite-docs-api-plugin` | reference | 31890 | 0.812 | 0.754 | 0.836 | 0.807 / 0.823 / 0.812 | 65,332 |
| `comment-incident` | comments | 1124 | 0.822 | 0.792 | 0.832 | 0.806 / 0.821 / 0.829 | 1,385 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.849 | 0.843 | 0.862 | 0.847 / 0.861 / 0.847 | 5,797 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 0.850 | 0.844 | 0.858 | 0.849 / 0.849 / 0.857 | 14,905 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.862 | 0.837 | 0.866 | 0.861 / 0.866 / 0.862 | 26,913 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.880 | 0.871 | 0.884 | 0.882 / 0.880 / 0.874 | 5,239 |
| `vite-docs-performance` | technical-docs | 8184 | 0.880 | 0.872 | 0.888 | 0.880 / 0.885 / 0.879 | 8,665 |
| `comment-ack` | comments | 37 | 0.882 | 0.875 | 0.886 | 0.884 / 0.878 / 0.882 | 145 |
| `vite-docs-philosophy` | technical-docs | 3575 | 0.911 | 0.901 | 0.916 | 0.912 / 0.913 / 0.907 | 3,345 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 0.994 | 0.976 | 1.006 | 0.993 / 0.995 / 1.001 | 2,447 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.005 | 0.983 | 1.014 | 1.012 / 1.000 / 1.005 | 54,425 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.018 | 0.934 | 1.074 | 0.993 / 1.032 / 0.937 | 49,087 |

### reuse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-checklist` | comments | 287 | 0.669 | 0.660 | 0.673 | 0.669 / 0.669 / 0.670 | 482 |
| `comment-quote` | comments | 290 | 0.707 | 0.703 | 0.721 | 0.708 / 0.707 / 0.707 | 212 |
| `comment-review-long` | comments | 957 | 0.748 | 0.745 | 0.751 | 0.748 / 0.747 / 0.748 | 653 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 0.757 | 0.751 | 0.761 | 0.758 / 0.759 / 0.754 | 4,972 |
| `vite-docs-api-plugin` | reference | 31890 | 0.795 | 0.775 | 0.861 | 0.795 / 0.832 / 0.793 | 62,814 |
| `comment-incident` | comments | 1124 | 0.796 | 0.782 | 0.806 | 0.799 / 0.796 / 0.795 | 1,218 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 0.845 | 0.837 | 0.852 | 0.846 / 0.846 / 0.843 | 14,514 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.846 | 0.830 | 0.850 | 0.835 / 0.846 / 0.848 | 5,456 |
| `comment-ack` | comments | 37 | 0.854 | 0.834 | 0.862 | 0.844 / 0.857 / 0.854 | 66 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.860 | 0.835 | 0.902 | 0.877 / 0.857 / 0.860 | 26,678 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.874 | 0.861 | 0.904 | 0.882 / 0.872 / 0.874 | 4,989 |
| `vite-docs-performance` | technical-docs | 8184 | 0.881 | 0.874 | 0.884 | 0.881 / 0.878 / 0.881 | 8,436 |
| `vite-docs-philosophy` | technical-docs | 3575 | 0.906 | 0.898 | 0.916 | 0.910 / 0.906 / 0.903 | 3,162 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 0.988 | 0.971 | 0.998 | 0.988 / 0.991 / 0.986 | 2,248 |
| `rust-book-appendix-02-operators` | reference | 22595 | 0.998 | 0.984 | 1.006 | 0.998 / 0.999 / 0.990 | 53,420 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.018 | 0.983 | 1.134 | 1.000 / 1.055 / 1.018 | 49,393 |

### parse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-checklist` | comments | 287 | 0.621 | 0.616 | 0.640 | 0.627 / 0.618 / 0.621 | 385 |
| `comment-quote` | comments | 290 | 0.652 | 0.647 | 0.655 | 0.652 / 0.652 / 0.652 | 165 |
| `comment-review-long` | comments | 957 | 0.699 | 0.687 | 0.701 | 0.697 / 0.700 / 0.699 | 502 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 0.701 | 0.700 | 0.704 | 0.701 / 0.701 / 0.703 | 3,725 |
| `comment-incident` | comments | 1124 | 0.750 | 0.747 | 0.754 | 0.749 / 0.751 / 0.750 | 921 |
| `vite-docs-api-plugin` | reference | 31890 | 0.764 | 0.754 | 0.780 | 0.764 / 0.772 / 0.756 | 47,272 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.787 | 0.778 | 0.798 | 0.779 / 0.790 / 0.791 | 3,566 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 0.796 | 0.791 | 0.799 | 0.795 / 0.796 / 0.796 | 9,974 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.804 | 0.797 | 0.807 | 0.804 / 0.805 / 0.804 | 17,554 |
| `comment-ack` | comments | 37 | 0.822 | 0.814 | 0.823 | 0.822 / 0.823 / 0.821 | 50 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.827 | 0.821 | 0.835 | 0.828 / 0.824 / 0.828 | 3,370 |
| `vite-docs-performance` | technical-docs | 8184 | 0.829 | 0.827 | 0.839 | 0.829 / 0.829 / 0.828 | 5,686 |
| `vite-docs-philosophy` | technical-docs | 3575 | 0.856 | 0.844 | 0.863 | 0.851 / 0.857 / 0.856 | 1,965 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 0.968 | 0.953 | 0.979 | 0.974 / 0.968 / 0.968 | 1,313 |
| `typescript-handbook-advanced-types` | reference | 36745 | 0.975 | 0.943 | 1.001 | 0.989 / 0.975 / 0.969 | 28,398 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.000 | 0.936 | 1.011 | 0.999 / 1.000 / 1.003 | 45,397 |

### render

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-ack` | comments | 37 | 0.981 | 0.976 | 0.985 | 0.978 / 0.982 / 0.983 | 19 |
| `vite-docs-api-plugin` | reference | 31890 | 0.990 | 0.987 | 0.996 | 0.989 / 0.993 / 0.990 | 11,963 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.991 | 0.984 | 0.996 | 0.991 / 0.990 / 0.989 | 1,861 |
| `comment-checklist` | comments | 287 | 0.992 | 0.977 | 1.000 | 0.994 / 0.991 / 0.992 | 96 |
| `comment-quote` | comments | 290 | 0.994 | 0.979 | 1.004 | 0.991 / 0.994 / 0.996 | 49 |
| `vite-docs-philosophy` | technical-docs | 3575 | 0.995 | 0.991 | 0.999 | 0.994 / 0.992 / 0.996 | 1,167 |
| `vite-docs-performance` | technical-docs | 8184 | 0.995 | 0.981 | 1.004 | 0.992 / 0.997 / 0.999 | 2,667 |
| `comment-incident` | comments | 1124 | 0.995 | 0.977 | 1.003 | 1.002 / 0.987 / 0.995 | 284 |
| `comment-review-long` | comments | 957 | 0.996 | 0.993 | 1.888 | 0.996 / 0.997 / 0.993 | 148 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.998 | 0.989 | 1.005 | 0.990 / 0.999 / 0.999 | 8,712 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 0.998 | 0.966 | 1.018 | 0.993 / 0.999 / 1.000 | 4,525 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 0.999 | 0.982 | 1.011 | 1.000 / 0.999 / 0.990 | 899 |
| `legacy-docs-releasing` | technical-docs | 4141 | 1.001 | 0.998 | 1.008 | 1.006 / 1.001 / 1.002 | 1,581 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 1.003 | 0.989 | 1.010 | 0.997 / 1.000 / 1.009 | 1,204 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.007 | 0.726 | 1.012 | 1.003 / 1.009 / 1.007 | 7,590 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.032 | 0.987 | 1.052 | 1.017 / 1.032 / 1.044 | 15,945 |

## The fix, 16 documents

### fresh

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-checklist` | comments | 287 | 0.959 | 0.951 | 0.969 | 0.964 / 0.956 / 0.956 | 567 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.968 | 0.958 | 0.979 | 0.970 / 0.968 / 0.968 | 5,774 |
| `vite-docs-philosophy` | technical-docs | 3575 | 0.972 | 0.965 | 0.976 | 0.973 / 0.970 / 0.972 | 3,339 |
| `vite-docs-api-plugin` | reference | 31890 | 0.974 | 0.947 | 1.019 | 0.999 / 0.998 / 0.972 | 64,114 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 0.974 | 0.949 | 1.000 | 0.977 / 0.978 / 0.971 | 5,329 |
| `vite-docs-performance` | technical-docs | 8184 | 0.975 | 0.968 | 0.985 | 0.974 / 0.975 / 0.976 | 8,654 |
| `comment-review-long` | comments | 957 | 0.981 | 0.976 | 1.018 | 0.988 / 0.986 / 0.980 | 741 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.989 | 0.976 | 1.000 | 0.988 / 0.990 / 0.990 | 5,237 |
| `comment-incident` | comments | 1124 | 0.991 | 0.974 | 0.998 | 0.991 / 0.990 / 0.996 | 1,363 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.991 | 0.983 | 1.017 | 0.986 / 1.002 / 0.991 | 26,947 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 0.992 | 0.967 | 1.001 | 0.988 / 0.990 / 0.995 | 14,897 |
| `rust-book-appendix-02-operators` | reference | 22595 | 0.994 | 0.986 | 1.004 | 0.993 / 0.997 / 0.994 | 53,692 |
| `comment-ack` | comments | 37 | 0.995 | 0.992 | 1.005 | 0.998 / 0.995 / 0.993 | 145 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.003 | 0.997 | 1.015 | 1.002 / 1.007 / 1.003 | 2,431 |
| `comment-quote` | comments | 290 | 1.013 | 0.950 | 1.018 | 0.954 / 1.015 / 1.013 | 294 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.024 | 0.968 | 1.072 | 0.990 / 1.047 / 1.022 | 49,643 |

### reuse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `vite-docs-api-plugin` | reference | 31890 | 0.963 | 0.911 | 1.008 | 0.968 / 0.963 / 0.952 | 64,057 |
| `comment-checklist` | comments | 287 | 0.964 | 0.952 | 0.980 | 0.964 / 0.961 / 0.964 | 480 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.965 | 0.957 | 0.968 | 0.965 / 0.961 / 0.965 | 5,480 |
| `comment-review-long` | comments | 957 | 0.966 | 0.963 | 0.969 | 0.966 / 0.969 / 0.964 | 652 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 0.967 | 0.942 | 0.970 | 0.967 / 0.964 / 0.968 | 4,953 |
| `vite-docs-philosophy` | technical-docs | 3575 | 0.968 | 0.965 | 0.984 | 0.968 / 0.981 / 0.966 | 3,168 |
| `comment-ack` | comments | 37 | 0.970 | 0.964 | 0.978 | 0.971 / 0.965 / 0.970 | 66 |
| `vite-docs-performance` | technical-docs | 8184 | 0.977 | 0.950 | 0.993 | 0.992 / 0.974 / 0.977 | 8,445 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.989 | 0.981 | 1.008 | 0.992 / 0.988 / 0.989 | 4,995 |
| `comment-incident` | comments | 1124 | 0.991 | 0.967 | 0.996 | 0.991 / 0.994 / 0.990 | 1,216 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.994 | 0.982 | 1.004 | 0.995 / 0.994 / 0.988 | 26,621 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 0.995 | 0.968 | 1.003 | 0.996 / 0.993 / 0.995 | 14,610 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 0.995 | 0.988 | 1.006 | 0.995 / 0.992 / 1.001 | 2,241 |
| `rust-book-appendix-02-operators` | reference | 22595 | 0.999 | 0.981 | 1.009 | 1.000 / 0.983 / 0.999 | 53,830 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.014 | 0.937 | 1.031 | 1.014 / 1.004 / 1.028 | 47,570 |
| `comment-quote` | comments | 290 | 1.025 | 1.018 | 1.030 | 1.020 / 1.025 / 1.026 | 212 |

### parse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.947 | 0.937 | 0.953 | 0.945 / 0.952 / 0.949 | 3,574 |
| `vite-docs-api-plugin` | reference | 31890 | 0.951 | 0.932 | 0.982 | 0.948 / 0.968 / 0.951 | 47,222 |
| `comment-review-long` | comments | 957 | 0.952 | 0.939 | 0.956 | 0.949 / 0.952 / 0.954 | 502 |
| `vite-docs-philosophy` | technical-docs | 3575 | 0.952 | 0.945 | 0.960 | 0.952 / 0.955 / 0.952 | 1,981 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 0.954 | 0.948 | 0.971 | 0.954 / 0.967 / 0.952 | 3,722 |
| `comment-checklist` | comments | 287 | 0.957 | 0.944 | 0.963 | 0.957 / 0.951 / 0.958 | 384 |
| `vite-docs-performance` | technical-docs | 8184 | 0.959 | 0.950 | 0.964 | 0.954 / 0.963 / 0.959 | 5,673 |
| `comment-ack` | comments | 37 | 0.967 | 0.962 | 0.974 | 0.967 / 0.965 / 0.968 | 50 |
| `comment-incident` | comments | 1124 | 0.984 | 0.983 | 0.993 | 0.984 / 0.985 / 0.984 | 919 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 0.986 | 0.964 | 0.994 | 0.986 / 0.988 / 0.980 | 10,016 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 0.987 | 0.983 | 0.993 | 0.985 / 0.990 / 0.990 | 1,308 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.987 | 0.973 | 0.997 | 0.987 / 0.993 / 0.982 | 17,529 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.992 | 0.974 | 0.997 | 0.996 / 0.990 / 0.992 | 3,355 |
| `rust-book-appendix-02-operators` | reference | 22595 | 0.996 | 0.973 | 1.007 | 0.993 / 0.993 / 1.004 | 45,987 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.003 | 0.460 | 1.074 | 1.017 / 1.004 / 0.999 | 29,563 |
| `comment-quote` | comments | 290 | 1.027 | 1.023 | 1.035 | 1.028 / 1.027 / 1.023 | 165 |

### render

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-ack` | comments | 37 | 0.984 | 0.974 | 0.988 | 0.978 / 0.985 / 0.985 | 19 |
| `vite-docs-api-plugin` | reference | 31890 | 0.989 | 0.982 | 0.998 | 0.987 / 0.989 / 0.994 | 11,955 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 0.991 | 0.981 | 1.002 | 0.996 / 0.987 / 0.993 | 4,484 |
| `comment-review-long` | comments | 957 | 0.993 | 0.983 | 0.996 | 0.993 / 0.992 / 0.996 | 147 |
| `vite-docs-philosophy` | technical-docs | 3575 | 0.993 | 0.988 | 1.015 | 0.989 / 0.999 / 0.993 | 1,167 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 0.994 | 0.984 | 1.010 | 0.994 / 0.993 / 1.005 | 896 |
| `comment-quote` | comments | 290 | 0.994 | 0.980 | 1.011 | 0.995 / 0.989 / 0.994 | 49 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.996 | 0.985 | 1.001 | 0.997 / 0.997 / 0.991 | 1,864 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.996 | 0.994 | 1.002 | 0.997 / 0.994 / 0.995 | 1,573 |
| `comment-checklist` | comments | 287 | 0.996 | 0.981 | 1.022 | 1.001 / 0.984 / 0.996 | 96 |
| `vite-docs-performance` | technical-docs | 8184 | 0.998 | 0.978 | 1.007 | 0.996 / 0.998 / 1.000 | 2,677 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 0.999 | 0.991 | 1.003 | 1.001 / 1.001 / 0.997 | 1,193 |
| `comment-incident` | comments | 1124 | 1.000 | 0.995 | 1.019 | 1.004 / 1.001 / 0.996 | 282 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.006 | 0.999 | 1.009 | 1.006 / 1.006 / 1.001 | 7,536 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 1.008 | 0.995 | 1.035 | 1.013 / 1.010 / 0.999 | 8,764 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.037 | 0.997 | 1.092 | 1.050 / 1.004 / 1.068 | 16,053 |

## The fix, 57 broad documents

### fresh

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 0.956 | 0.937 | 0.965 | 0.958 / 0.956 / 0.956 | 1,666 |
| `wiki-tea-lead` | encyclopedia | 6363 | 0.960 | 0.940 | 0.974 | 0.956 / 0.965 / 0.955 | 7,327 |
| `comment-checklist` | comments | 287 | 0.961 | 0.950 | 0.977 | 0.954 / 0.961 / 0.966 | 563 |
| `vite-docs-api-plugin` | reference | 31890 | 0.962 | 0.906 | 1.028 | 0.980 / 0.985 / 0.960 | 63,762 |
| `comment-reproduction` | comments | 298 | 0.964 | 0.951 | 0.967 | 0.957 / 0.964 / 0.965 | 305 |
| `legacy-docs-readme` | readme | 1825 | 0.967 | 0.961 | 0.976 | 0.966 / 0.972 / 0.968 | 4,557 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 0.968 | 0.927 | 1.040 | 0.978 / 0.970 / 0.967 | 55,021 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.968 | 0.960 | 0.982 | 0.966 / 0.967 / 0.969 | 5,777 |
| `vite-docs-features` | technical-docs | 39739 | 0.968 | 0.937 | 1.052 | 0.960 / 0.962 / 0.983 | 59,314 |
| `wiki-chess-lead` | encyclopedia | 4125 | 0.969 | 0.949 | 0.977 | 0.968 / 0.971 / 0.971 | 3,868 |
| `vite-docs-performance` | technical-docs | 8184 | 0.969 | 0.934 | 0.985 | 0.964 / 0.979 / 0.969 | 8,744 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 0.970 | 0.909 | 1.057 | 0.981 / 0.949 / 0.970 | 112,981 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 0.970 | 0.954 | 0.983 | 0.974 / 0.964 / 0.970 | 2,236 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 0.970 | 0.961 | 0.978 | 0.970 / 0.974 / 0.970 | 676 |
| `vite-docs-philosophy` | technical-docs | 3575 | 0.971 | 0.958 | 0.999 | 0.974 / 0.968 / 0.971 | 3,344 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 0.971 | 0.958 | 0.977 | 0.969 / 0.968 / 0.972 | 1,362 |
| `comment-inline-code` | comments | 285 | 0.972 | 0.963 | 0.978 | 0.971 / 0.973 / 0.971 | 292 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 0.973 | 0.968 | 0.980 | 0.970 / 0.975 / 0.973 | 1,555 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 0.974 | 0.962 | 0.986 | 0.976 / 0.967 / 0.975 | 1,013 |
| `comment-links` | comments | 278 | 0.975 | 0.950 | 0.979 | 0.970 / 0.977 / 0.975 | 461 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 0.975 | 0.968 | 0.986 | 0.969 / 0.976 / 0.977 | 5,363 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 0.977 | 0.933 | 1.982 | 0.998 / 0.973 / 0.991 | 33,059 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 0.977 | 0.942 | 0.983 | 0.981 / 0.972 / 0.977 | 3,079 |
| `comment-unicode` | comments | 327 | 0.978 | 0.970 | 0.997 | 0.981 / 0.972 / 0.981 | 450 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 0.978 | 0.941 | 1.024 | 0.996 / 0.976 / 0.980 | 63,253 |
| `comment-table` | comments | 310 | 0.981 | 0.976 | 0.987 | 0.982 / 0.980 / 0.981 | 1,116 |
| `legacy-docs-mdx` | technical-docs | 7422 | 0.982 | 0.976 | 0.991 | 0.980 / 0.982 / 0.981 | 7,677 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 0.984 | 0.974 | 0.992 | 0.984 / 0.985 / 0.983 | 2,493 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 0.984 | 0.971 | 1.023 | 0.986 / 0.975 / 0.992 | 22,040 |
| `comment-review-long` | comments | 957 | 0.984 | 0.958 | 0.993 | 0.986 / 0.984 / 0.970 | 747 |
| `legacy-node-ferromark-readme` | readme | 9075 | 0.985 | 0.967 | 1.006 | 0.987 / 0.991 / 0.982 | 10,135 |
| `vue-docs-slots` | technical-docs | 24211 | 0.985 | 0.970 | 1.004 | 0.985 / 0.985 / 0.996 | 23,228 |
| `legacy-contributing` | technical-docs | 9323 | 0.986 | 0.979 | 1.003 | 0.985 / 0.986 / 0.987 | 10,540 |
| `guard-angle-link` | syntax-guard | 41 | 0.987 | 0.978 | 1.000 | 0.985 / 0.985 / 0.992 | 262 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 0.988 | 0.973 | 1.004 | 0.984 / 0.996 / 0.988 | 1,602 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 0.988 | 0.979 | 0.999 | 0.988 / 0.993 / 0.982 | 14,902 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.988 | 0.980 | 0.996 | 0.994 / 0.988 / 0.986 | 5,222 |
| `comment-incident` | comments | 1124 | 0.988 | 0.979 | 0.997 | 0.993 / 0.987 / 0.987 | 1,367 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 0.989 | 0.982 | 0.994 | 0.986 / 0.990 / 0.989 | 8,053 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 0.989 | 0.974 | 0.998 | 0.991 / 0.990 / 0.988 | 14,071 |
| `vue-docs-suspense` | technical-docs | 8291 | 0.989 | 0.955 | 1.003 | 0.994 / 0.989 / 0.983 | 9,307 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 0.990 | 0.973 | 1.010 | 0.982 / 0.994 / 0.989 | 3,804 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.993 | 0.983 | 1.018 | 0.995 / 0.993 / 0.990 | 27,031 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 0.994 | 0.980 | 1.012 | 0.996 / 0.988 / 0.997 | 12,050 |
| `typescript-handbook-advanced-types` | reference | 36745 | 0.994 | 0.958 | 1.049 | 1.043 / 0.982 / 0.978 | 48,088 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 0.994 | 0.971 | 1.003 | 0.992 / 0.995 / 0.995 | 7,330 |
| `comment-review` | comments | 282 | 0.995 | 0.968 | 1.008 | 0.988 / 0.997 / 0.997 | 229 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 0.995 | 0.986 | 1.005 | 0.996 / 0.995 / 0.994 | 9,247 |
| `comment-ack` | comments | 37 | 0.995 | 0.973 | 1.002 | 0.995 / 0.994 / 0.997 | 145 |
| `rust-book-appendix-02-operators` | reference | 22595 | 0.997 | 0.965 | 1.011 | 0.994 / 0.989 / 1.001 | 53,963 |
| `comment-question` | comments | 160 | 0.999 | 0.988 | 1.005 | 0.999 / 0.999 / 0.996 | 166 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 0.999 | 0.987 | 1.013 | 1.000 / 0.994 / 1.003 | 2,979 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 1.001 | 0.977 | 1.025 | 1.007 / 1.004 / 0.997 | 2,901 |
| `typescript-handbook-compiler-options` | reference | 54026 | 1.002 | 0.992 | 1.011 | 1.003 / 1.001 / 1.003 | 77,088 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 1.003 | 0.993 | 1.053 | 0.999 / 1.011 / 1.004 | 2,437 |
| `comment-quote` | comments | 290 | 1.015 | 0.997 | 1.028 | 1.015 / 1.015 / 1.017 | 294 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 1.016 | 0.867 | 1.139 | 1.081 / 1.016 / 0.996 | 60,073 |

### reuse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `guard-angle-link` | syntax-guard | 41 | 0.925 | 0.886 | 0.931 | 0.917 / 0.926 / 0.926 | 171 |
| `comment-reproduction` | comments | 298 | 0.947 | 0.941 | 0.963 | 0.952 / 0.950 / 0.944 | 223 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 0.947 | 0.932 | 0.960 | 0.943 / 0.951 / 0.949 | 506 |
| `comment-links` | comments | 278 | 0.953 | 0.933 | 0.962 | 0.951 / 0.953 / 0.954 | 368 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 0.953 | 0.947 | 0.969 | 0.952 / 0.957 / 0.954 | 1,496 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 0.955 | 0.942 | 0.962 | 0.954 / 0.950 / 0.959 | 1,197 |
| `wiki-tea-lead` | encyclopedia | 6363 | 0.957 | 0.948 | 0.965 | 0.958 / 0.954 / 0.958 | 7,041 |
| `comment-checklist` | comments | 287 | 0.960 | 0.947 | 0.971 | 0.960 / 0.962 / 0.957 | 477 |
| `vite-docs-api-plugin` | reference | 31890 | 0.962 | 0.913 | 1.043 | 0.962 / 0.961 / 0.986 | 63,710 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 0.963 | 0.950 | 0.966 | 0.964 / 0.957 / 0.961 | 1,386 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.963 | 0.957 | 0.975 | 0.962 / 0.966 / 0.964 | 5,478 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 0.964 | 0.949 | 0.989 | 0.964 / 0.960 / 0.969 | 4,959 |
| `comment-review-long` | comments | 957 | 0.965 | 0.952 | 0.997 | 0.963 / 0.971 / 0.965 | 657 |
| `wiki-chess-lead` | encyclopedia | 4125 | 0.965 | 0.958 | 0.978 | 0.966 / 0.965 / 0.965 | 3,581 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 0.966 | 0.958 | 0.976 | 0.970 / 0.973 / 0.964 | 2,068 |
| `comment-unicode` | comments | 327 | 0.967 | 0.963 | 0.979 | 0.968 / 0.966 / 0.965 | 359 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 0.968 | 0.959 | 0.973 | 0.970 / 0.965 / 0.969 | 852 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 0.968 | 0.917 | 1.023 | 0.943 / 0.968 / 0.975 | 32,471 |
| `legacy-docs-readme` | readme | 1825 | 0.969 | 0.955 | 0.978 | 0.969 / 0.968 / 0.969 | 4,424 |
| `vite-docs-philosophy` | technical-docs | 3575 | 0.970 | 0.964 | 0.977 | 0.972 / 0.966 / 0.970 | 3,164 |
| `comment-ack` | comments | 37 | 0.971 | 0.961 | 0.976 | 0.970 / 0.972 / 0.971 | 66 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 0.971 | 0.949 | 0.974 | 0.962 / 0.972 / 0.972 | 2,819 |
| `vite-docs-performance` | technical-docs | 8184 | 0.972 | 0.934 | 0.987 | 0.971 / 0.978 / 0.971 | 8,434 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 0.973 | 0.915 | 1.004 | 0.973 / 0.984 / 0.956 | 62,449 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 0.974 | 0.941 | 1.009 | 0.963 / 0.997 / 0.974 | 113,804 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 0.975 | 0.930 | 1.032 | 0.975 / 0.977 / 0.965 | 54,424 |
| `comment-review` | comments | 282 | 0.978 | 0.963 | 0.984 | 0.982 / 0.967 / 0.975 | 149 |
| `legacy-docs-mdx` | technical-docs | 7422 | 0.979 | 0.964 | 0.990 | 0.976 / 0.984 / 0.980 | 7,405 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 0.979 | 0.971 | 1.009 | 0.976 / 0.993 / 0.975 | 21,573 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 0.980 | 0.966 | 1.001 | 0.986 / 0.982 / 0.980 | 1,430 |
| `comment-question` | comments | 160 | 0.983 | 0.978 | 0.993 | 0.986 / 0.985 / 0.980 | 88 |
| `comment-table` | comments | 310 | 0.983 | 0.959 | 0.992 | 0.983 / 0.975 / 0.983 | 1,030 |
| `comment-inline-code` | comments | 285 | 0.983 | 0.974 | 1.011 | 0.986 / 0.983 / 0.983 | 212 |
| `vue-docs-slots` | technical-docs | 24211 | 0.984 | 0.967 | 1.006 | 0.984 / 0.982 / 0.987 | 22,614 |
| `vue-docs-suspense` | technical-docs | 8291 | 0.984 | 0.974 | 0.999 | 0.982 / 0.983 / 0.997 | 9,006 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 0.986 | 0.972 | 0.996 | 0.986 / 0.985 / 0.986 | 3,619 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 0.987 | 0.978 | 0.996 | 0.992 / 0.991 / 0.985 | 2,323 |
| `legacy-contributing` | technical-docs | 9323 | 0.988 | 0.972 | 1.003 | 0.986 / 0.990 / 0.986 | 10,248 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 0.988 | 0.968 | 1.007 | 1.000 / 0.987 / 0.988 | 14,564 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 0.988 | 0.973 | 0.994 | 0.990 / 0.988 / 0.985 | 2,702 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 0.989 | 0.982 | 0.994 | 0.987 / 0.991 / 0.987 | 7,676 |
| `comment-incident` | comments | 1124 | 0.989 | 0.972 | 0.996 | 0.986 / 0.990 / 0.989 | 1,215 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.989 | 0.976 | 0.997 | 0.993 / 0.989 / 0.986 | 4,974 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 0.989 | 0.982 | 0.999 | 0.992 / 0.989 / 0.988 | 13,791 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 0.991 | 0.978 | 1.032 | 0.991 / 0.990 / 0.993 | 11,659 |
| `legacy-node-ferromark-readme` | readme | 9075 | 0.991 | 0.975 | 1.000 | 0.991 / 0.991 / 0.994 | 9,835 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.993 | 0.951 | 1.025 | 0.993 / 0.986 / 0.995 | 26,617 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 0.993 | 0.987 | 1.015 | 0.994 / 0.998 / 0.989 | 2,236 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 0.993 | 0.986 | 1.012 | 0.992 / 0.997 / 0.993 | 2,794 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 0.993 | 0.982 | 1.005 | 0.993 / 0.993 / 0.994 | 6,929 |
| `rust-book-appendix-02-operators` | reference | 22595 | 0.994 | 0.987 | 1.006 | 0.994 / 0.995 / 0.991 | 53,451 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 1.000 | 0.985 | 1.009 | 0.995 / 1.000 / 1.007 | 8,967 |
| `typescript-handbook-compiler-options` | reference | 54026 | 1.001 | 0.986 | 1.013 | 1.000 / 1.001 / 1.003 | 76,387 |
| `vite-docs-features` | technical-docs | 39739 | 1.002 | 0.894 | 1.054 | 1.010 / 0.994 / 0.999 | 59,178 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 1.006 | 0.945 | 1.076 | 1.005 / 1.006 / 1.046 | 58,913 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.016 | 0.910 | 1.109 | 0.971 / 1.061 / 0.987 | 47,490 |
| `comment-quote` | comments | 290 | 1.020 | 1.018 | 1.032 | 1.022 / 1.020 / 1.020 | 211 |

### parse

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `guard-angle-link` | syntax-guard | 41 | 0.913 | 0.900 | 0.927 | 0.903 / 0.913 / 0.920 | 141 |
| `comment-reproduction` | comments | 298 | 0.924 | 0.914 | 0.937 | 0.924 / 0.923 / 0.927 | 162 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 0.930 | 0.879 | 0.970 | 0.920 / 0.936 / 0.930 | 32,072 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 0.930 | 0.909 | 0.964 | 0.937 / 0.914 / 0.930 | 35,252 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 0.932 | 0.924 | 0.948 | 0.931 / 0.935 / 0.930 | 956 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 0.934 | 0.919 | 0.952 | 0.930 / 0.935 / 0.934 | 744 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 0.934 | 0.876 | 0.973 | 0.921 / 0.937 / 0.948 | 66,306 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 0.938 | 0.933 | 0.946 | 0.935 / 0.941 / 0.938 | 348 |
| `comment-links` | comments | 278 | 0.941 | 0.917 | 0.967 | 0.931 / 0.941 / 0.966 | 278 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 0.944 | 0.925 | 0.959 | 0.944 / 0.941 / 0.944 | 19,674 |
| `wiki-tea-lead` | encyclopedia | 6363 | 0.945 | 0.934 | 0.954 | 0.945 / 0.949 / 0.938 | 4,533 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.945 | 0.930 | 0.958 | 0.949 / 0.943 / 0.943 | 3,581 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 0.945 | 0.936 | 0.956 | 0.946 / 0.951 / 0.943 | 872 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 0.946 | 0.930 | 0.956 | 0.946 / 0.945 / 0.946 | 534 |
| `vite-docs-api-plugin` | reference | 31890 | 0.947 | 0.912 | 0.974 | 0.947 / 0.956 / 0.941 | 46,831 |
| `vite-docs-philosophy` | technical-docs | 3575 | 0.948 | 0.943 | 0.958 | 0.946 / 0.951 / 0.950 | 1,960 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 0.950 | 0.937 | 0.954 | 0.952 / 0.944 / 0.950 | 1,289 |
| `comment-checklist` | comments | 287 | 0.952 | 0.946 | 0.958 | 0.948 / 0.955 / 0.952 | 382 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 0.952 | 0.943 | 0.959 | 0.954 / 0.950 / 0.953 | 1,739 |
| `wiki-chess-lead` | encyclopedia | 4125 | 0.952 | 0.937 | 0.967 | 0.953 / 0.951 / 0.952 | 2,275 |
| `legacy-docs-readme` | readme | 1825 | 0.954 | 0.946 | 0.961 | 0.954 / 0.957 / 0.948 | 3,328 |
| `comment-review-long` | comments | 957 | 0.956 | 0.947 | 0.964 | 0.952 / 0.956 / 0.956 | 502 |
| `vite-docs-features` | technical-docs | 39739 | 0.959 | 0.922 | 1.017 | 0.973 / 0.959 / 0.959 | 36,668 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 0.961 | 0.512 | 0.975 | 0.947 / 0.963 / 0.961 | 3,731 |
| `vite-docs-performance` | technical-docs | 8184 | 0.963 | 0.952 | 0.979 | 0.967 / 0.962 / 0.963 | 5,689 |
| `comment-review` | comments | 282 | 0.964 | 0.953 | 0.971 | 0.964 / 0.956 / 0.965 | 108 |
| `comment-ack` | comments | 37 | 0.966 | 0.945 | 0.979 | 0.965 / 0.970 / 0.965 | 50 |
| `comment-unicode` | comments | 327 | 0.966 | 0.960 | 0.974 | 0.969 / 0.965 / 0.967 | 268 |
| `legacy-docs-mdx` | technical-docs | 7422 | 0.970 | 0.959 | 0.978 | 0.968 / 0.974 / 0.970 | 4,879 |
| `vue-docs-slots` | technical-docs | 24211 | 0.975 | 0.959 | 0.984 | 0.975 / 0.970 / 0.980 | 12,964 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 0.977 | 0.970 | 1.013 | 0.983 / 0.976 / 0.982 | 13,962 |
| `legacy-node-ferromark-readme` | readme | 9075 | 0.979 | 0.972 | 0.993 | 0.982 / 0.977 / 0.978 | 6,189 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 0.979 | 0.911 | 1.035 | 0.961 / 0.979 / 0.984 | 33,820 |
| `comment-inline-code` | comments | 285 | 0.980 | 0.964 | 0.995 | 0.979 / 0.982 / 0.980 | 132 |
| `comment-table` | comments | 310 | 0.980 | 0.964 | 0.997 | 0.980 / 0.970 / 0.985 | 847 |
| `vue-docs-suspense` | technical-docs | 8291 | 0.981 | 0.961 | 0.989 | 0.981 / 0.981 / 0.980 | 4,936 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 0.981 | 0.968 | 0.990 | 0.977 / 0.982 / 0.981 | 926 |
| `comment-question` | comments | 160 | 0.982 | 0.968 | 0.986 | 0.982 / 0.978 / 0.982 | 65 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 0.984 | 0.955 | 1.001 | 0.984 / 0.982 / 0.992 | 2,463 |
| `comment-incident` | comments | 1124 | 0.985 | 0.975 | 1.043 | 0.984 / 0.985 / 0.982 | 919 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 0.985 | 0.976 | 0.992 | 0.985 / 0.984 / 0.986 | 1,597 |
| `legacy-contributing` | technical-docs | 9323 | 0.985 | 0.978 | 0.997 | 0.991 / 0.985 / 0.984 | 7,481 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 0.987 | 0.981 | 0.998 | 0.987 / 0.986 / 0.987 | 7,928 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 0.987 | 0.974 | 1.001 | 0.995 / 0.988 / 0.986 | 10,033 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 0.988 | 0.962 | 1.012 | 0.988 / 0.988 / 0.984 | 11,411 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 0.988 | 0.967 | 0.992 | 0.984 / 0.990 / 0.988 | 17,547 |
| `typescript-handbook-advanced-types` | reference | 36745 | 0.989 | 0.960 | 1.021 | 0.999 / 0.998 / 0.985 | 29,133 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 0.990 | 0.983 | 1.003 | 0.998 / 0.990 / 0.986 | 1,309 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.990 | 0.978 | 1.000 | 0.990 / 0.992 / 0.990 | 3,339 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 0.990 | 0.984 | 1.026 | 0.990 / 0.988 / 0.992 | 6,128 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 0.991 | 0.981 | 1.002 | 0.992 / 0.995 / 0.988 | 1,577 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 0.991 | 0.983 | 1.004 | 0.989 / 0.991 / 0.993 | 1,708 |
| `typescript-handbook-compiler-options` | reference | 54026 | 0.997 | 0.985 | 1.008 | 0.997 / 0.997 / 0.998 | 19,443 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 0.998 | 0.991 | 1.024 | 0.995 / 0.997 / 1.002 | 4,391 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.000 | 0.988 | 1.007 | 1.000 / 1.002 / 0.994 | 45,724 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 1.001 | 0.993 | 1.010 | 1.001 / 0.998 / 1.002 | 5,744 |
| `comment-quote` | comments | 290 | 1.028 | 1.013 | 1.040 | 1.028 / 1.028 / 1.030 | 165 |

### render

| Case | Category | Bytes | Ratio | Min | Max | Rounds | Baseline ns/doc |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: |
| `comment-unicode` | comments | 327 | 0.979 | 0.966 | 1.012 | 0.983 / 0.981 / 0.979 | 93 |
| `comment-ack` | comments | 37 | 0.982 | 0.974 | 0.993 | 0.983 / 0.982 / 0.982 | 19 |
| `wiki-tea-lead` | encyclopedia | 6363 | 0.983 | 0.980 | 0.991 | 0.981 / 0.983 / 0.988 | 2,502 |
| `wiki-tea-first-paragraph` | encyclopedia | 1126 | 0.983 | 0.965 | 1.001 | 0.970 / 0.984 / 0.983 | 532 |
| `wiki-rainbow-lead` | encyclopedia | 1859 | 0.983 | 0.974 | 0.996 | 0.983 / 0.983 / 0.985 | 507 |
| `comment-inline-code` | comments | 285 | 0.985 | 0.973 | 1.003 | 0.987 / 0.987 / 0.981 | 81 |
| `rust-book-ch00-00-introduction` | technical-docs | 10839 | 0.987 | 0.979 | 0.994 | 0.992 / 0.984 / 0.987 | 2,247 |
| `wiki-volcano-first-paragraph` | encyclopedia | 2644 | 0.988 | 0.978 | 1.003 | 0.989 / 0.991 / 0.987 | 774 |
| `wiki-volcano-lead` | encyclopedia | 4232 | 0.988 | 0.960 | 1.004 | 0.991 / 0.990 / 0.987 | 1,068 |
| `wiki-rainbow-first-paragraph` | encyclopedia | 859 | 0.989 | 0.975 | 1.007 | 0.989 / 0.983 / 0.987 | 311 |
| `comment-question` | comments | 160 | 0.989 | 0.980 | 0.995 | 0.989 / 0.987 / 0.989 | 24 |
| `vite-docs-api-plugin` | reference | 31890 | 0.989 | 0.973 | 0.997 | 0.987 / 0.990 / 0.989 | 11,960 |
| `wiki-chess-lead` | encyclopedia | 4125 | 0.989 | 0.977 | 1.005 | 0.990 / 0.989 / 0.988 | 1,310 |
| `legacy-docs-readme-theme` | technical-docs | 2848 | 0.990 | 0.978 | 0.994 | 0.989 / 0.990 / 0.992 | 703 |
| `comment-quote` | comments | 290 | 0.991 | 0.976 | 1.000 | 0.992 / 0.988 / 0.991 | 49 |
| `comment-review` | comments | 282 | 0.992 | 0.983 | 1.000 | 0.992 / 0.994 / 0.991 | 42 |
| `wiki-chess-first-paragraph` | encyclopedia | 1190 | 0.992 | 0.979 | 1.001 | 0.998 / 0.989 / 0.992 | 438 |
| `vite-docs-performance` | technical-docs | 8184 | 0.992 | 0.976 | 1.006 | 0.994 / 0.992 / 0.980 | 2,676 |
| `guard-angle-link` | syntax-guard | 41 | 0.992 | 0.981 | 1.000 | 0.993 / 0.994 / 0.990 | 31 |
| `legacy-docs-migration-0-4` | technical-docs | 7045 | 0.992 | 0.985 | 0.996 | 0.992 / 0.989 / 0.996 | 2,485 |
| `legacy-docs-migration-0-3` | technical-docs | 2379 | 0.993 | 0.985 | 1.005 | 0.991 / 0.996 / 0.993 | 1,046 |
| `legacy-contributing` | technical-docs | 9323 | 0.993 | 0.987 | 1.011 | 0.992 / 0.996 / 0.992 | 2,686 |
| `vite-docs-philosophy` | technical-docs | 3575 | 0.994 | 0.986 | 1.000 | 0.994 / 0.999 / 0.993 | 1,167 |
| `comment-checklist` | comments | 287 | 0.994 | 0.966 | 1.000 | 0.990 / 0.996 / 0.993 | 96 |
| `vue-docs-slots` | technical-docs | 24211 | 0.994 | 0.990 | 1.002 | 0.999 / 0.993 / 0.992 | 9,332 |
| `comment-review-long` | comments | 957 | 0.995 | 0.987 | 1.008 | 0.998 / 0.995 / 0.993 | 148 |
| `legacy-node-ferromark-readme` | readme | 9075 | 0.995 | 0.984 | 1.010 | 0.999 / 0.989 / 0.995 | 3,487 |
| `vue-docs-suspense` | technical-docs | 8291 | 0.995 | 0.982 | 1.014 | 0.998 / 0.995 / 0.991 | 3,931 |
| `legacy-docs-markdown-extensions` | technical-docs | 3707 | 0.995 | 0.987 | 1.008 | 0.995 / 0.992 / 0.997 | 1,156 |
| `wiki-volcano-plain-prose` | plain-prose | 47303 | 0.995 | 0.973 | 1.007 | 0.998 / 0.990 / 0.995 | 4,484 |
| `legacy-docs-releasing` | technical-docs | 4141 | 0.995 | 0.985 | 1.008 | 0.999 / 0.990 / 0.995 | 1,579 |
| `typescript-handbook-the-handbook` | technical-docs | 5337 | 0.996 | 0.990 | 1.008 | 0.994 / 0.996 / 0.999 | 1,197 |
| `legacy-docs-migration-0-8` | technical-docs | 2374 | 0.996 | 0.990 | 1.012 | 0.995 / 1.000 / 0.996 | 1,112 |
| `legacy-docs-mdx` | technical-docs | 7422 | 0.997 | 0.989 | 1.002 | 0.997 / 0.998 / 0.997 | 2,443 |
| `wiki-tea-plain-prose` | plain-prose | 40577 | 0.998 | 0.992 | 1.017 | 1.001 / 0.998 / 0.997 | 3,750 |
| `legacy-docs-adr-readme-theme-composition` | technical-docs | 1532 | 0.998 | 0.990 | 1.009 | 1.000 / 0.998 / 0.998 | 496 |
| `rust-book-ch03-04-comments` | technical-docs | 393 | 0.998 | 0.991 | 1.003 | 0.996 / 1.001 / 0.998 | 150 |
| `wiki-rainbow-plain-prose` | plain-prose | 38800 | 0.999 | 0.981 | 1.011 | 1.004 / 0.998 / 0.998 | 3,238 |
| `vue-docs-ways-of-using-vue` | technical-docs | 5883 | 0.999 | 0.978 | 1.022 | 1.004 / 0.993 / 1.003 | 1,885 |
| `rust-book-ch17-00-async-await` | technical-docs | 9734 | 0.999 | 0.911 | 1.009 | 1.002 / 0.997 / 0.997 | 1,497 |
| `legacy-docs-migration-0-2` | technical-docs | 1985 | 0.999 | 0.991 | 1.019 | 1.002 / 1.004 / 0.993 | 903 |
| `comment-links` | comments | 278 | 1.000 | 0.992 | 1.008 | 1.000 / 1.000 / 0.998 | 88 |
| `vue-docs-reactivity-in-depth` | technical-docs | 24001 | 1.000 | 0.992 | 1.024 | 1.000 / 0.997 / 1.002 | 7,217 |
| `legacy-docs-readme` | readme | 1825 | 1.001 | 0.991 | 1.004 | 0.999 / 0.999 / 1.002 | 1,034 |
| `typescript-handbook-compiler-options` | reference | 54026 | 1.002 | 0.996 | 1.015 | 1.000 / 1.007 / 1.005 | 56,968 |
| `comment-incident` | comments | 1124 | 1.002 | 0.983 | 1.009 | 1.003 / 1.002 / 1.001 | 285 |
| `comment-reproduction` | comments | 298 | 1.004 | 0.981 | 1.007 | 1.005 / 1.002 / 1.004 | 63 |
| `rust-book-appendix-02-operators` | reference | 22595 | 1.005 | 1.001 | 1.011 | 1.001 / 1.004 / 1.008 | 7,550 |
| `comment-table` | comments | 310 | 1.006 | 0.988 | 1.205 | 1.001 / 1.007 / 1.004 | 184 |
| `vite-docs-features` | technical-docs | 39739 | 1.011 | 0.987 | 1.041 | 0.994 / 1.013 / 1.011 | 17,242 |
| `wiki-rainbow-article-body` | encyclopedia | 48422 | 1.014 | 0.985 | 1.059 | 1.006 / 0.994 / 1.019 | 11,306 |
| `typescript-handbook-advanced-types` | reference | 36745 | 1.017 | 0.981 | 1.051 | 1.009 / 1.026 / 1.030 | 15,900 |
| `wiki-chess-plain-prose` | plain-prose | 80966 | 1.017 | 0.992 | 1.046 | 1.002 / 1.030 / 1.017 | 8,832 |
| `wiki-tea-article-body` | encyclopedia | 58814 | 1.038 | 0.984 | 1.081 | 1.036 / 1.041 / 1.038 | 18,594 |
| `wiki-volcano-article-body` | encyclopedia | 69241 | 1.051 | 0.982 | 1.135 | 1.041 / 1.051 / 1.057 | 22,074 |
| `typescript-handbook-typescript-5-0` | technical-docs | 50714 | 1.066 | 0.988 | 1.134 | 1.062 / 1.076 / 1.080 | 18,682 |
| `wiki-chess-article-body` | encyclopedia | 113609 | 1.083 | 1.025 | 1.197 | 1.070 / 1.124 / 1.087 | 36,058 |

