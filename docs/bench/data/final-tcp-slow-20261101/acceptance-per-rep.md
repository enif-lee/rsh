# Focused per-repetition comparison evidence

Each row retains one measured role in one repetition. No pooling, noise tolerance or automatic pass is applied. Observed deficits do not establish policy causality; see JSON for exact missing metrics, FPS gaps and resource increases.

| Network | Scene | Rep | Role (slot) | FPS 1% / 5% / avg | Network kbit/s | RSS client / server KiB | CPU client / server % | Age median / p95 ms | Assessment |
|---|---|---:|---|---|---:|---|---|---|---|
| lowbw | compress | 0 | ssh (0) | 6.5 / 7.1 / 23.4 | 976 | 7300 / 19712 | 0.16 / 0.5 | 2552 / 4634 | control |
| lowbw | compress | 0 | main-A (1) | 42.3 / 45.6 / 55.9 | 222 | 2264 / 5464 | 0.85 / 3.32 | 65 / 73 | control |
| lowbw | compress | 0 | main-B (2) | 28.2 / 31.1 / 55.1 | 212 | 1476 / 5336 | 0.76 / 1.5 | 60 / 64 | control |
| lowbw | compress | 0 | candidate (3) | 40.2 / 46.8 / 60.0 | 227 | 1856 / 4908 | 0.82 / 1.15 | 60 / 63 | fps_tradeoff_review_required |
| lowbw | compress | 1 | ssh (2) | 6.9 / 7.2 / 23.5 | 977 | 7664 / 19712 | 0.18 / 0.54 | 2567 / 4669 | control |
| lowbw | compress | 1 | main-A (3) | 41.1 / 45.9 / 55.9 | 223 | 2652 / 5920 | 0.91 / 3.43 | 65 / 74 | control |
| lowbw | compress | 1 | main-B (0) | 29.5 / 41.7 / 57.2 | 219 | 2628 / 5792 | 0.87 / 1.38 | 60 / 64 | control |
| lowbw | compress | 1 | candidate (1) | 43.2 / 46.9 / 60.0 | 227 | 2688 / 5676 | 0.89 / 1.22 | 60 / 63 | qualitative_review_required |
| lowbw | bandwidth | 0 | ssh (0) | 5.8 / 6.5 / 10.0 | 979 | 7588 / 20092 | 0.14 / 0.31 | 3524 / 6509 | control |
| lowbw | bandwidth | 0 | main-A (1) | 16.2 / 18.4 / 21.5 | 852 | 2636 / 5920 | 0.84 / 2.84 | 98 / 118 | control |
| lowbw | bandwidth | 0 | main-B (2) | 12.0 / 18.7 / 22.6 | 906 | 2660 / 5916 | 0.9 / 3.02 | 105 / 123 | control |
| lowbw | bandwidth | 0 | candidate (3) | 13.7 / 17.8 / 20.2 | 809 | 2712 / 5964 | 0.82 / 2.74 | 97 / 117 | fps_tradeoff_review_required |
| lowbw | bandwidth | 1 | ssh (2) | 5.4 / 6.4 / 9.9 | 977 | 7440 / 20440 | 0.12 / 0.34 | 3503 / 6463 | control |
| lowbw | bandwidth | 1 | main-A (3) | 13.6 / 17.8 / 20.7 | 828 | 2392 / 5512 | 0.83 / 2.82 | 98 / 122 | control |
| lowbw | bandwidth | 1 | main-B (0) | 11.5 / 19.3 / 22.9 | 920 | 2396 / 5656 | 0.95 / 3.12 | 103 / 121 | control |
| lowbw | bandwidth | 1 | candidate (1) | 14.3 / 18.1 / 21.1 | 842 | 2504 / 5496 | 0.87 / 2.85 | 99 / 151 | fps_tradeoff_review_required |
| verylow | compress | 0 | ssh (0) | 2.6 / 5.1 / 6.0 | 248 | 7552 / 19804 | 0.09 / 0.46 | 3776 / 6921 | control |
| verylow | compress | 0 | main-A (1) | 25.2 / 26.4 / 33.6 | 127 | 2644 / 5884 | 0.71 / 2.62 | 69 / 88 | control |
| verylow | compress | 0 | main-B (2) | 25.9 / 27.4 / 34.6 | 133 | 2652 / 5896 | 0.72 / 2.51 | 69 / 87 | control |
| verylow | compress | 0 | candidate (3) | 40.3 / 46.3 / 60.0 | 229 | 2708 / 5664 | 1.09 / 1.57 | 72 / 77 | qualitative_review_required |
| verylow | compress | 1 | ssh (2) | 1.5 / 5.2 / 6.1 | 251 | 7528 / 19852 | 0.1 / 0.52 | 3896 / 7401 | control |
| verylow | compress | 1 | main-A (3) | 25.9 / 26.8 / 33.9 | 130 | 2644 / 5888 | 0.78 / 2.75 | 69 / 88 | control |
| verylow | compress | 1 | main-B (0) | 25.5 / 27.2 / 34.4 | 134 | 2644 / 5888 | 0.79 / 2.79 | 69 / 88 | control |
| verylow | compress | 1 | candidate (1) | 40.1 / 45.5 / 60.0 | 229 | 2700 / 5728 | 1.17 / 1.7 | 72 / 77 | qualitative_review_required |
| verylow | bandwidth | 0 | ssh (0) | 1.3 / 1.3 / 2.5 | 248 | 7516 / 19904 | 0.11 / 0.31 | 4193 / 7755 | control |
| verylow | bandwidth | 0 | main-A (1) | 5.3 / 5.5 / 6.0 | 256 | 2644 / 5940 | 0.62 / 2.24 | 384 / 407 | control |
| verylow | bandwidth | 0 | main-B (2) | 2.9 / 5.4 / 5.7 | 256 | 2640 / 5936 | 0.66 / 2.08 | 387 / 405 | control |
| verylow | bandwidth | 0 | candidate (3) | 5.5 / 5.5 / 5.9 | 256 | 2720 / 5924 | 0.64 / 2.26 | 381 / 405 | fps_tradeoff_review_required |
| verylow | bandwidth | 1 | ssh (2) | 1.8 / 1.8 / 2.4 | 249 | 7576 / 19924 | 0.08 / 0.31 | 3877 / — | control |
| verylow | bandwidth | 1 | main-A (3) | 5.4 / 5.5 / 5.9 | 256 | 2628 / 5932 | 0.67 / 2.24 | 387 / 403 | control |
| verylow | bandwidth | 1 | main-B (0) | 5.5 / 5.6 / 5.9 | 256 | 2636 / 5928 | 0.66 / 2.2 | 381 / 407 | control |
| verylow | bandwidth | 1 | candidate (1) | 5.5 / 5.5 / 6.0 | 256 | 2704 / 5936 | 0.7 / 2.23 | 383 / 412 | fps_tradeoff_review_required |
| slow64k | compress | 0 | ssh (0) | 1.1 / 1.1 / 1.4 | 63 | 7560 / 19968 | 0.06 / 0.63 | 4125 / — | control |
| slow64k | compress | 0 | main-A (1) | 8.9 / 12.7 / 15.6 | 63 | 2664 / 5872 | 0.52 / 2.28 | 154 / 382 | control |
| slow64k | compress | 0 | main-B (2) | 11.6 / 12.3 / 16.0 | 63 | 2640 / 5856 | 0.55 / 2.28 | 176 / 370 | control |
| slow64k | compress | 0 | candidate (3) | 5.9 / 12.3 / 15.4 | 64 | 2732 / 5948 | 0.54 / 2.29 | 191 / 349 | fps_tradeoff_review_required |
| slow64k | compress | 1 | ssh (2) | 1.0 / 1.0 / 1.5 | 62 | 7568 / 19768 | 0.13 / 0.63 | 4637 / — | control |
| slow64k | compress | 1 | main-A (3) | 11.1 / 12.9 / 16.0 | 63 | 2636 / 5856 | 0.57 / 2.35 | 152 / 363 | control |
| slow64k | compress | 1 | main-B (0) | 11.6 / 12.9 / 15.9 | 64 | 2632 / 5872 | 0.61 / 2.27 | 170 / 372 | control |
| slow64k | compress | 1 | candidate (1) | 8.8 / 12.7 / 15.7 | 64 | 2704 / 5860 | 0.57 / 2.33 | 191 / 375 | fps_tradeoff_review_required |
| slow64k | bandwidth | 0 | ssh (0) | — / — / 0.5 | 60 | 7540 / 19804 | 0.04 / 0.24 | 3974 / — | control |
| slow64k | bandwidth | 0 | main-A (1) | 1.3 / 1.3 / 1.4 | 63 | 2612 / 5924 | 0.24 / 1.68 | 998 / — | control |
| slow64k | bandwidth | 0 | main-B (2) | 1.3 / 1.3 / 1.4 | 64 | 2632 / 5944 | 0.25 / 1.67 | 1068 / — | control |
| slow64k | bandwidth | 0 | candidate (3) | 1.4 / 1.4 / 1.6 | 63 | 2668 / 5956 | 0.2 / 1.75 | 2047 / — | partial_metrics_review_required |
| slow64k | bandwidth | 1 | ssh (2) | — / — / 0.6 | 62 | 7544 / 19880 | 0.03 / 0.29 | 5051 / — | control |
| slow64k | bandwidth | 1 | main-A (3) | 1.4 / 1.4 / 1.6 | 63 | 2640 / 5980 | 0.17 / 1.64 | 2038 / — | control |
| slow64k | bandwidth | 1 | main-B (0) | 1.3 / 1.3 / 1.4 | 64 | 2632 / 5960 | 0.25 / 1.6 | 1079 / — | control |
| slow64k | bandwidth | 1 | candidate (1) | 1.3 / 1.3 / 1.6 | 63 | 2712 / 5968 | 0.19 / 1.53 | 1981 / — | partial_metrics_review_required |
