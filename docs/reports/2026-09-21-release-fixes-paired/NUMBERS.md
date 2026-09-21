# Numbers

Ratios are baseline time over candidate time (median of the paired windows); **higher is faster, below 1.000 the candidate is slower**.
The baseline is `c4af9525` in every screen — the core the preceding native comparison measured as `7c887a2b`.
Each candidate is the named revision of `main`, so every screen is cumulative: the difference between two consecutive rows is what the commits between them cost.

## Against the baseline on the 57 broad documents

| Candidate | Stage | N | Geomean | Rounds | Above 1 | Below 1 |
| --- | --- | ---: | ---: | --- | ---: | ---: |
| HEAD `bffc89f6` | fresh | 57 | 0.9287 | 0.9275 / 0.9294 / 0.9287 | 0 | 57 |
| HEAD `bffc89f6` | reuse | 57 | 0.9217 | 0.9223 / 0.9211 / 0.9209 | 1 | 56 |
| HEAD `bffc89f6` | parse | 57 | 0.8935 | 0.8931 / 0.8935 / 0.8931 | 1 | 56 |
| HEAD `bffc89f6` | render | 57 | 1.0004 | 0.9996 / 1.0009 / 1.0008 | 14 | 43 |
| HEAD + first-byte gate | fresh | 57 | 0.9590 | 0.9575 / 0.9597 / 0.9586 | 2 | 55 |
| HEAD + first-byte gate | reuse | 57 | 0.9527 | 0.9516 / 0.9542 / 0.9520 | 1 | 56 |
| HEAD + first-byte gate | parse | 57 | 0.9340 | 0.9346 / 0.9333 / 0.9338 | 0 | 57 |
| HEAD + first-byte gate | render | 57 | 1.0013 | 1.0017 / 1.0014 / 1.0019 | 17 | 40 |
| HEAD + on-demand catch-up | fresh | 57 | 0.9720 | 0.9733 / 0.9737 / 0.9709 | 2 | 55 |
| HEAD + on-demand catch-up | reuse | 57 | 0.9683 | 0.9683 / 0.9693 / 0.9683 | 4 | 53 |
| HEAD + on-demand catch-up | parse | 57 | 0.9517 | 0.9523 / 0.9537 / 0.9502 | 0 | 57 |
| HEAD + on-demand catch-up | render | 57 | 1.0003 | 1.0017 / 1.0011 / 1.0002 | 20 | 37 |
| HEAD + on-demand catch-up + lazy memos | fresh | 57 | 0.9790 | 0.9800 / 0.9785 / 0.9794 | 6 | 51 |
| HEAD + on-demand catch-up + lazy memos | reuse | 57 | 0.9737 | 0.9729 / 0.9721 / 0.9740 | 5 | 52 |
| HEAD + on-demand catch-up + lazy memos | parse | 57 | 0.9624 | 0.9640 / 0.9616 / 0.9632 | 3 | 54 |
| HEAD + on-demand catch-up + lazy memos | render | 57 | 0.9977 | 0.9981 / 0.9990 / 0.9972 | 19 | 38 |
| HEAD + on-demand catch-up + lazy memos + closer front cache (the fix) | fresh | 57 | 0.9824 | 0.9851 / 0.9822 / 0.9824 | 5 | 52 |
| HEAD + on-demand catch-up + lazy memos + closer front cache (the fix) | reuse | 57 | 0.9778 | 0.9766 / 0.9794 / 0.9782 | 5 | 52 |
| HEAD + on-demand catch-up + lazy memos + closer front cache (the fix) | parse | 57 | 0.9669 | 0.9663 / 0.9669 / 0.9676 | 2 | 55 |
| HEAD + on-demand catch-up + lazy memos + closer front cache (the fix) | render | 57 | 0.9997 | 0.9988 / 1.0004 / 1.0001 | 14 | 43 |

By category and size group, all stages:

| Group | N | Candidate | fresh | reuse | parse | render |
| --- | ---: | --- | ---: | ---: | ---: | ---: |
| comments | 12 | HEAD `bffc89f6` | 0.8789 | 0.8548 | 0.8220 | 0.9927 |
| comments | 12 | HEAD + first-byte gate | 0.9348 | 0.9149 | 0.8949 | 0.9926 |
| comments | 12 | HEAD + on-demand catch-up | 0.9515 | 0.9347 | 0.9167 | 0.9933 |
| comments | 12 | HEAD + on-demand catch-up + lazy memos | 0.9838 | 0.9774 | 0.9733 | 0.9941 |
| comments | 12 | HEAD + on-demand catch-up + lazy memos + closer front cache (the fix) | 0.9838 | 0.9747 | 0.9683 | 0.9931 |
| encyclopedia | 12 | HEAD `bffc89f6` | 0.9549 | 0.9518 | 0.9244 | 1.0122 |
| encyclopedia | 12 | HEAD + first-byte gate | 0.9647 | 0.9604 | 0.9364 | 1.0107 |
| encyclopedia | 12 | HEAD + on-demand catch-up | 0.9628 | 0.9637 | 0.9410 | 1.0067 |
| encyclopedia | 12 | HEAD + on-demand catch-up + lazy memos | 0.9549 | 0.9493 | 0.9216 | 0.9979 |
| encyclopedia | 12 | HEAD + on-demand catch-up + lazy memos + closer front cache (the fix) | 0.9702 | 0.9657 | 0.9412 | 1.0062 |
| plain-prose | 4 | HEAD `bffc89f6` | 0.9084 | 0.9066 | 0.8676 | 0.9983 |
| plain-prose | 4 | HEAD + first-byte gate | 0.9754 | 0.9723 | 0.9579 | 1.0024 |
| plain-prose | 4 | HEAD + on-demand catch-up | 0.9886 | 0.9884 | 0.9805 | 1.0004 |
| plain-prose | 4 | HEAD + on-demand catch-up + lazy memos | 0.9925 | 0.9949 | 0.9904 | 0.9925 |
| plain-prose | 4 | HEAD + on-demand catch-up + lazy memos + closer front cache (the fix) | 0.9924 | 0.9927 | 0.9906 | 1.0023 |
| readme | 2 | HEAD `bffc89f6` | 0.9849 | 0.9868 | 0.9793 | 0.9968 |
| readme | 2 | HEAD + first-byte gate | 0.9844 | 0.9874 | 0.9787 | 0.9989 |
| readme | 2 | HEAD + on-demand catch-up | 0.9840 | 0.9845 | 0.9775 | 0.9985 |
| readme | 2 | HEAD + on-demand catch-up + lazy memos | 0.9772 | 0.9791 | 0.9686 | 0.9999 |
| readme | 2 | HEAD + on-demand catch-up + lazy memos + closer front cache (the fix) | 0.9762 | 0.9799 | 0.9665 | 0.9978 |
| reference | 4 | HEAD `bffc89f6` | 0.9455 | 0.9502 | 0.9268 | 1.0003 |
| reference | 4 | HEAD + first-byte gate | 0.9675 | 0.9643 | 0.9432 | 1.0102 |
| reference | 4 | HEAD + on-demand catch-up | 0.9973 | 1.0086 | 0.9862 | 1.0044 |
| reference | 4 | HEAD + on-demand catch-up + lazy memos | 0.9880 | 0.9900 | 0.9806 | 1.0086 |
| reference | 4 | HEAD + on-demand catch-up + lazy memos + closer front cache (the fix) | 0.9884 | 0.9934 | 0.9831 | 1.0031 |
| syntax-guard | 1 | HEAD `bffc89f6` | 0.9502 | 0.9321 | 0.9069 | 0.9875 |
| syntax-guard | 1 | HEAD + first-byte gate | 0.9553 | 0.9361 | 0.9099 | 0.9891 |
| syntax-guard | 1 | HEAD + on-demand catch-up | 0.9614 | 0.9379 | 0.9042 | 0.9900 |
| syntax-guard | 1 | HEAD + on-demand catch-up + lazy memos | 0.9803 | 0.9232 | 0.9128 | 0.9918 |
| syntax-guard | 1 | HEAD + on-demand catch-up + lazy memos + closer front cache (the fix) | 0.9874 | 0.9254 | 0.9131 | 0.9924 |
| technical-docs | 22 | HEAD `bffc89f6` | 0.9373 | 0.9350 | 0.9085 | 0.9997 |
| technical-docs | 22 | HEAD + first-byte gate | 0.9626 | 0.9615 | 0.9457 | 1.0000 |
| technical-docs | 22 | HEAD + on-demand catch-up | 0.9802 | 0.9787 | 0.9658 | 1.0006 |
| technical-docs | 22 | HEAD + on-demand catch-up + lazy memos | 0.9858 | 0.9802 | 0.9726 | 0.9987 |
| technical-docs | 22 | HEAD + on-demand catch-up + lazy memos + closer front cache (the fix) | 0.9859 | 0.9830 | 0.9757 | 0.9991 |
| 0..512 | 12 | HEAD `bffc89f6` | 0.9072 | 0.8846 | 0.8568 | 0.9915 |
| 0..512 | 12 | HEAD + first-byte gate | 0.9382 | 0.9183 | 0.8980 | 0.9913 |
| 0..512 | 12 | HEAD + on-demand catch-up | 0.9499 | 0.9318 | 0.9106 | 0.9919 |
| 0..512 | 12 | HEAD + on-demand catch-up + lazy memos | 0.9849 | 0.9715 | 0.9666 | 0.9943 |
| 0..512 | 12 | HEAD + on-demand catch-up + lazy memos + closer front cache (the fix) | 0.9826 | 0.9678 | 0.9607 | 0.9926 |
| 513..2048 | 9 | HEAD `bffc89f6` | 0.9278 | 0.9182 | 0.8936 | 0.9929 |
| 513..2048 | 9 | HEAD + first-byte gate | 0.9639 | 0.9560 | 0.9412 | 0.9917 |
| 513..2048 | 9 | HEAD + on-demand catch-up | 0.9741 | 0.9657 | 0.9539 | 0.9937 |
| 513..2048 | 9 | HEAD + on-demand catch-up + lazy memos | 0.9694 | 0.9627 | 0.9493 | 0.9926 |
| 513..2048 | 9 | HEAD + on-demand catch-up + lazy memos + closer front cache (the fix) | 0.9782 | 0.9704 | 0.9578 | 0.9935 |
| 2049..16384 | 20 | HEAD `bffc89f6` | 0.9353 | 0.9332 | 0.9084 | 0.9930 |
| 2049..16384 | 20 | HEAD + first-byte gate | 0.9615 | 0.9607 | 0.9451 | 0.9943 |
| 2049..16384 | 20 | HEAD + on-demand catch-up | 0.9750 | 0.9740 | 0.9627 | 0.9939 |
| 2049..16384 | 20 | HEAD + on-demand catch-up + lazy memos | 0.9770 | 0.9744 | 0.9643 | 0.9930 |
| 2049..16384 | 20 | HEAD + on-demand catch-up + lazy memos + closer front cache (the fix) | 0.9818 | 0.9793 | 0.9723 | 0.9929 |
| 16385..65536 | 13 | HEAD `bffc89f6` | 0.9452 | 0.9474 | 0.9176 | 1.0114 |
| 16385..65536 | 13 | HEAD + first-byte gate | 0.9703 | 0.9698 | 0.9498 | 1.0129 |
| 16385..65536 | 13 | HEAD + on-demand catch-up | 0.9878 | 0.9943 | 0.9719 | 1.0105 |
| 16385..65536 | 13 | HEAD + on-demand catch-up + lazy memos | 0.9848 | 0.9828 | 0.9715 | 1.0058 |
| 16385..65536 | 13 | HEAD + on-demand catch-up + lazy memos + closer front cache (the fix) | 0.9868 | 0.9897 | 0.9745 | 1.0097 |
| 65537..18446744073709551616 | 3 | HEAD `bffc89f6` | 0.9043 | 0.8978 | 0.8437 | 1.0632 |
| 65537..18446744073709551616 | 3 | HEAD + first-byte gate | 0.9635 | 0.9557 | 0.9178 | 1.0699 |
| 65537..18446744073709551616 | 3 | HEAD + on-demand catch-up | 0.9672 | 0.9757 | 0.9542 | 1.0543 |
| 65537..18446744073709551616 | 3 | HEAD + on-demand catch-up + lazy memos | 0.9727 | 0.9711 | 0.9334 | 1.0241 |
| 65537..18446744073709551616 | 3 | HEAD + on-demand catch-up + lazy memos + closer front cache (the fix) | 0.9803 | 0.9799 | 0.9504 | 1.0503 |

## Stage geomeans per screen, 16 documents

| Candidate | fresh | reuse | parse | render |
| --- | ---: | ---: | ---: | ---: |
| A/A control (baseline against itself) | 1.0068 | 1.0040 | 0.9994 | 1.0083 |
| `2887b2bd` #369 bound inline bracket nesting | 1.0031 | 1.0027 | 0.9964 | 1.0061 |
| `e8439fd8` #372 bound emphasis nesting | 0.9824 | 0.9793 | 0.9851 | 0.9585 |
| `ca82ec2f` #374 linear nested link probing | 0.9766 | 0.9686 | 0.9747 | 0.9542 |
| `c84743cb` #376/#377 base URL, MDX closer memo | 0.9740 | 0.9688 | 0.9700 | 0.9574 |
| `1037fb5d` #380 renderer fixes | 0.9755 | 0.9683 | 0.9733 | 0.9575 |
| `e532dcf4` #382 tabs, HTML closers, MDX flow | 0.9774 | 0.9741 | 0.9765 | 0.9607 |
| `8d957e30` #383 laziness, container and inline cost bounds | 0.8692 | 0.8545 | 0.8125 | 0.9966 |
| `bffc89f6` #384 linear math/MDX/definition-list scans (HEAD) | 0.8562 | 0.8408 | 0.7962 | 0.9979 |
| HEAD + tracker classifiers gated on the first byte | 0.9366 | 0.9289 | 0.9042 | 1.0010 |
| ablation: the same, with `emphasis.rs` as of #382 (measures the delimiter list of #383) | 0.9433 | 0.9351 | 0.9127 | 0.9982 |
| HEAD + tracker catching up on demand | 0.9719 | 0.9650 | 0.9497 | 0.9997 |
| HEAD + on-demand tracker + lazily allocated memos | 0.9833 | 0.9834 | 0.9712 | 0.9974 |
| HEAD + on-demand tracker + lazy memos + closer front cache (the fix) | 0.9871 | 0.9838 | 0.9749 | 0.9983 |

## Per-document parse ratios across screens

| Document | aa | c369 | c372 | c374 | c377 | c380 | c382 | c383 | head-screen | fix1-screen | abl-emph-screen | fix2-screen | fix3-screen | fix4-screen |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `comment-checklist` | 1.006 | 0.997 | 0.984 | 0.977 | 0.970 | 0.971 | 0.975 | 0.626 | 0.621 | 0.836 | 0.838 | 0.946 | 0.949 | 0.957 |
| `comment-quote` | 1.001 | 0.982 | 0.982 | 0.955 | 0.944 | 0.954 | 0.950 | 0.685 | 0.652 | 0.853 | 0.857 | 0.917 | 1.033 | 1.027 |
| `comment-review-long` | 0.992 | 1.008 | 0.990 | 0.971 | 0.976 | 0.981 | 0.983 | 0.715 | 0.699 | 0.908 | 0.914 | 0.964 | 0.950 | 0.952 |
| `typescript-handbook-the-handbook` | 0.997 | 0.998 | 0.996 | 0.990 | 0.984 | 0.985 | 0.987 | 0.720 | 0.701 | 0.868 | 0.889 | 0.969 | 0.958 | 0.954 |
| `comment-incident` | 1.004 | 1.000 | 0.987 | 0.966 | 0.962 | 0.963 | 0.978 | 0.766 | 0.750 | 0.910 | 0.915 | 0.963 | 0.981 | 0.984 |
| `vite-docs-api-plugin` | 0.995 | 1.001 | 0.996 | 0.987 | 0.973 | 0.980 | 0.982 | 0.754 | 0.764 | 0.828 | 0.832 | 0.954 | 0.932 | 0.951 |
| `vue-docs-ways-of-using-vue` | 0.997 | 1.004 | 0.984 | 0.972 | 0.975 | 0.975 | 0.981 | 0.830 | 0.787 | 0.869 | 0.924 | 0.911 | 0.941 | 0.947 |
| `wiki-volcano-plain-prose` | 0.998 | 0.993 | 0.983 | 0.981 | 0.978 | 0.976 | 0.979 | 0.796 | 0.796 | 0.947 | 0.944 | 0.978 | 0.981 | 0.986 |
| `wiki-chess-plain-prose` | 0.997 | 0.996 | 0.978 | 0.983 | 0.980 | 0.977 | 0.982 | 0.809 | 0.804 | 0.941 | 0.944 | 0.978 | 0.985 | 0.987 |
| `comment-ack` | 1.001 | 0.992 | 0.965 | 0.935 | 0.918 | 0.917 | 0.918 | 0.898 | 0.822 | 0.816 | 0.823 | 0.817 | 0.979 | 0.967 |
| `legacy-docs-releasing` | 1.001 | 0.997 | 0.993 | 0.993 | 0.987 | 0.995 | 0.999 | 0.827 | 0.827 | 0.938 | 0.940 | 0.982 | 0.985 | 0.992 |
| `vite-docs-performance` | 1.002 | 0.997 | 0.988 | 0.980 | 0.970 | 0.981 | 0.976 | 0.845 | 0.829 | 0.922 | 0.929 | 0.954 | 0.951 | 0.959 |
| `vite-docs-philosophy` | 0.997 | 0.988 | 0.983 | 0.967 | 0.961 | 0.967 | 0.973 | 0.883 | 0.856 | 0.913 | 0.939 | 0.936 | 0.940 | 0.952 |
| `legacy-docs-migration-0-2` | 0.994 | 0.983 | 0.959 | 0.951 | 0.960 | 0.971 | 0.972 | 0.974 | 0.968 | 0.965 | 0.951 | 0.965 | 0.990 | 0.987 |
| `typescript-handbook-advanced-types` | 0.997 | 0.999 | 1.000 | 0.988 | 0.987 | 0.988 | 0.994 | 0.984 | 0.975 | 0.984 | 0.988 | 0.978 | 0.995 | 1.003 |
| `rust-book-appendix-02-operators` | 1.012 | 1.007 | 0.995 | 1.002 | 0.998 | 0.995 | 0.999 | 0.998 | 1.000 | 0.993 | 1.000 | 0.999 | 0.995 | 0.996 |

## Per-document fresh ratios across screens

| Document | aa | c369 | c372 | c374 | c377 | c380 | c382 | c383 | head-screen | fix1-screen | abl-emph-screen | fix2-screen | fix3-screen | fix4-screen |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `comment-checklist` | 0.999 | 1.002 | 0.973 | 0.968 | 0.971 | 0.972 | 0.970 | 0.706 | 0.702 | 0.880 | 0.888 | 0.967 | 0.959 | 0.959 |
| `comment-quote` | 1.006 | 0.995 | 0.979 | 0.966 | 0.966 | 0.963 | 0.958 | 0.796 | 0.762 | 0.914 | 0.912 | 0.959 | 1.021 | 1.013 |
| `typescript-handbook-the-handbook` | 1.002 | 1.004 | 0.988 | 0.983 | 0.980 | 0.979 | 0.983 | 0.788 | 0.767 | 0.898 | 0.922 | 0.981 | 0.971 | 0.974 |
| `comment-review-long` | 1.001 | 1.004 | 0.982 | 0.972 | 0.966 | 0.979 | 0.977 | 0.775 | 0.773 | 0.942 | 0.949 | 0.973 | 0.978 | 0.981 |
| `vite-docs-api-plugin` | 1.016 | 1.016 | 0.998 | 0.993 | 0.982 | 0.989 | 0.987 | 0.808 | 0.812 | 0.859 | 0.863 | 0.978 | 0.944 | 0.974 |
| `comment-incident` | 1.006 | 0.997 | 0.982 | 0.963 | 0.961 | 0.970 | 0.973 | 0.837 | 0.822 | 0.934 | 0.938 | 0.973 | 0.982 | 0.991 |
| `vue-docs-ways-of-using-vue` | 1.003 | 0.999 | 0.973 | 0.966 | 0.966 | 0.966 | 0.972 | 0.887 | 0.849 | 0.919 | 0.949 | 0.938 | 0.961 | 0.968 |
| `wiki-volcano-plain-prose` | 0.999 | 0.997 | 0.967 | 0.973 | 0.974 | 0.964 | 0.979 | 0.850 | 0.850 | 0.969 | 0.967 | 0.985 | 0.988 | 0.992 |
| `wiki-chess-plain-prose` | 1.003 | 1.005 | 0.986 | 0.980 | 0.979 | 0.977 | 0.975 | 0.868 | 0.862 | 0.969 | 0.966 | 0.997 | 0.991 | 0.991 |
| `legacy-docs-releasing` | 1.001 | 1.000 | 0.986 | 0.974 | 0.975 | 0.984 | 0.987 | 0.875 | 0.880 | 0.952 | 0.953 | 0.991 | 0.993 | 0.989 |
| `vite-docs-performance` | 1.007 | 1.003 | 0.975 | 0.970 | 0.968 | 0.971 | 0.974 | 0.893 | 0.880 | 0.949 | 0.950 | 0.972 | 0.971 | 0.975 |
| `comment-ack` | 1.005 | 1.002 | 0.988 | 0.975 | 0.969 | 0.969 | 0.961 | 0.971 | 0.882 | 0.882 | 0.885 | 0.885 | 1.000 | 0.995 |
| `vite-docs-philosophy` | 1.008 | 0.994 | 0.970 | 0.958 | 0.956 | 0.964 | 0.963 | 0.931 | 0.911 | 0.959 | 0.968 | 0.962 | 0.966 | 0.972 |
| `legacy-docs-migration-0-2` | 1.005 | 1.001 | 0.962 | 0.962 | 0.966 | 0.978 | 0.979 | 0.991 | 0.994 | 0.992 | 0.984 | 0.987 | 1.002 | 1.003 |
| `rust-book-appendix-02-operators` | 1.015 | 1.003 | 0.992 | 0.995 | 0.990 | 0.988 | 0.995 | 0.996 | 1.005 | 0.993 | 0.997 | 0.998 | 1.001 | 0.994 |
| `typescript-handbook-advanced-types` | 1.032 | 1.027 | 1.019 | 1.028 | 1.016 | 0.995 | 1.007 | 1.003 | 1.018 | 0.989 | 1.017 | 1.013 | 1.010 | 1.024 |

