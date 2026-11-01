# Focused per-repetition comparison evidence

Each row retains one measured role in one repetition. No pooling, noise tolerance or automatic pass is applied. Observed deficits do not establish policy causality; see JSON for exact missing metrics, FPS gaps and resource increases.

| Network | Scene | Rep | Role (slot) | FPS 1% / 5% / avg | Network kbit/s | RSS client / server KiB | CPU client / server % | Age median / p95 ms | Assessment |
|---|---|---:|---|---|---:|---|---|---|---|
| lowbw | compress | 0 | ssh (0) | 6.2 / 7.2 / 23.5 | 977 | 7568 / 19792 | 0.16 / 0.57 | 2608 / 4646 | control |
| lowbw | compress | 0 | main-A (1) | 41.2 / 45.3 / 60.0 | 227 | 2692 / 5804 | 0.9 / 1.24 | 60 / 64 | control |
| lowbw | compress | 0 | main-B (2) | 41.4 / 45.4 / 60.0 | 227 | 2692 / 5760 | 0.9 / 1.2 | 60 / 64 | control |
| lowbw | compress | 0 | candidate (3) | 41.4 / 47.0 / 60.0 | 227 | 2640 / 5692 | 0.93 / 1.25 | 60 / 64 | qualitative_review_required |
| lowbw | compress | 1 | ssh (3) | 6.4 / 7.1 / 23.5 | 978 | 7560 / 19700 | 0.15 / 0.52 | 2547 / 4658 | control |
| lowbw | compress | 1 | main-A (2) | 41.3 / 46.9 / 60.0 | 227 | 2112 / 4908 | 0.78 / 1.05 | 59 / 62 | control |
| lowbw | compress | 1 | main-B (1) | 42.9 / 46.5 / 60.0 | 228 | 1704 / 5192 | 0.78 / 1.06 | 59 / 62 | control |
| lowbw | compress | 1 | candidate (0) | 42.8 / 46.2 / 60.0 | 227 | 2184 / 4908 | 0.79 / 1.09 | 60 / 63 | fps_tradeoff_review_required |
| lowbw | bandwidth | 0 | ssh (0) | 5.9 / 6.8 / 9.9 | 977 | 7576 / 20076 | 0.12 / 0.34 | 3501 / 6476 | control |
| lowbw | bandwidth | 0 | main-A (1) | 18.3 / 21.3 / 24.5 | 970 | 2708 / 6000 | 0.86 / 2.99 | 103 / 129 | control |
| lowbw | bandwidth | 0 | main-B (2) | 12.5 / 19.7 / 23.9 | 968 | 2700 / 6052 | 0.87 / 2.98 | 128 / 175 | control |
| lowbw | bandwidth | 0 | candidate (3) | 16.3 / 19.9 / 24.0 | 949 | 2644 / 5988 | 0.85 / 2.96 | 121 / 152 | fps_tradeoff_review_required |
| lowbw | bandwidth | 1 | ssh (3) | 5.8 / 6.4 / 10.0 | 980 | 7828 / 20108 | 0.12 / 0.31 | 3537 / 6472 | control |
| lowbw | bandwidth | 1 | main-A (2) | 14.0 / 18.8 / 23.4 | 931 | 2712 / 6016 | 0.87 / 2.95 | 115 / 145 | control |
| lowbw | bandwidth | 1 | main-B (1) | 13.6 / 19.9 / 24.0 | 953 | 2704 / 6012 | 0.88 / 3.0 | 108 / 150 | control |
| lowbw | bandwidth | 1 | candidate (0) | 15.3 / 19.9 / 23.6 | 941 | 2632 / 5936 | 0.87 / 3.08 | 103 / 134 | fps_tradeoff_review_required |
| verylow | compress | 0 | ssh (0) | 2.3 / 4.2 / 6.0 | 249 | 7432 / 19856 | 0.09 / 0.46 | 3787 / 7270 | control |
| verylow | compress | 0 | main-A (1) | 43.5 / 47.2 / 60.0 | 229 | 2380 / 5224 | 0.86 / 1.2 | 71 / 75 | control |
| verylow | compress | 0 | main-B (2) | 41.1 / 48.1 / 60.0 | 229 | 2388 / 5284 | 0.86 / 1.2 | 71 / 75 | control |
| verylow | compress | 0 | candidate (3) | 42.2 / 46.3 / 60.0 | 229 | 2304 / 5272 | 0.88 / 1.19 | 72 / 75 | fps_tradeoff_review_required |
| verylow | compress | 1 | ssh (3) | 2.6 / 4.9 / 6.1 | 250 | 7516 / 19740 | 0.09 / 0.43 | 3893 / 7123 | control |
| verylow | compress | 1 | main-A (2) | 42.5 / 46.2 / 60.0 | 229 | 2696 / 5748 | 0.99 / 1.38 | 72 / 76 | control |
| verylow | compress | 1 | main-B (1) | 41.1 / 46.2 / 60.0 | 229 | 2688 / 5756 | 0.97 / 1.39 | 72 / 76 | control |
| verylow | compress | 1 | candidate (0) | 42.8 / 46.3 / 60.0 | 229 | 2628 / 5688 | 1.02 / 1.42 | 72 / 76 | qualitative_review_required |
| verylow | bandwidth | 0 | ssh (0) | 1.2 / 1.2 / 2.4 | 250 | 7556 / 19956 | 0.13 / 0.38 | 4338 / — | control |
| verylow | bandwidth | 0 | main-A (1) | 5.3 / 5.5 / 6.0 | 256 | 2708 / 5988 | 0.77 / 2.36 | 381 / 402 | control |
| verylow | bandwidth | 0 | main-B (2) | 5.5 / 5.6 / 5.9 | 256 | 2688 / 5984 | 0.76 / 2.38 | 383 / 403 | control |
| verylow | bandwidth | 0 | candidate (3) | 4.4 / 5.4 / 5.9 | 255 | 2632 / 5912 | 0.8 / 2.42 | 385 / 407 | fps_tradeoff_review_required |
| verylow | bandwidth | 1 | ssh (3) | 1.5 / 1.5 / 2.5 | 250 | 7536 / 19960 | 0.12 / 0.45 | 4445 / 7382 | control |
| verylow | bandwidth | 1 | main-A (2) | 5.3 / 5.6 / 6.0 | 256 | 2692 / 5988 | 0.73 / 2.58 | 383 / 402 | control |
| verylow | bandwidth | 1 | main-B (1) | 5.3 / 5.5 / 6.0 | 256 | 2696 / 6000 | 0.73 / 2.52 | 378 / 404 | control |
| verylow | bandwidth | 1 | candidate (0) | 5.3 / 5.4 / 5.9 | 256 | 2640 / 5912 | 0.76 / 2.39 | 384 / 404 | fps_tradeoff_review_required |
| slow64k | compress | 0 | ssh (0) | 1.1 / 1.1 / 1.4 | 60 | 7560 / 19812 | 0.05 / 0.7 | 4129 / — | control |
| slow64k | compress | 0 | main-A (1) | 4.0 / 11.7 / 15.1 | 63 | 2700 / 5912 | 0.55 / 2.17 | 181 / 412 | control |
| slow64k | compress | 0 | main-B (2) | 9.1 / 13.0 / 16.1 | 64 | 2704 / 5972 | 0.58 / 2.24 | 196 / 369 | control |
| slow64k | compress | 0 | candidate (3) | 9.1 / 12.3 / 15.2 | 63 | 2640 / 5876 | 0.55 / 2.26 | 184 / 341 | fps_tradeoff_review_required |
| slow64k | compress | 1 | ssh (3) | 1.3 / 1.3 / 1.5 | 60 | 7548 / 19684 | 0.05 / 0.58 | 4497 / — | control |
| slow64k | compress | 1 | main-A (2) | 8.3 / 12.6 / 15.9 | 64 | 2704 / 5980 | 0.62 / 2.33 | 201 / 439 | control |
| slow64k | compress | 1 | main-B (1) | 8.5 / 12.4 / 15.7 | 64 | 2704 / 5948 | 0.6 / 2.34 | 197 / 418 | control |
| slow64k | compress | 1 | candidate (0) | 5.7 / 12.4 / 15.9 | 63 | 2660 / 5908 | 0.62 / 2.48 | 184 / 355 | fps_tradeoff_review_required |
| slow64k | bandwidth | 0 | ssh (0) | — / — / 0.6 | 62 | 7548 / 19900 | 0.09 / 0.28 | 4819 / — | control |
| slow64k | bandwidth | 0 | main-A (1) | 1.4 / 1.4 / 1.6 | 63 | 2668 / 6008 | 0.19 / 1.66 | 2037 / — | control |
| slow64k | bandwidth | 0 | main-B (2) | 1.4 / 1.4 / 1.5 | 63 | 2688 / 6040 | 0.21 / 1.58 | 1742 / — | control |
| slow64k | bandwidth | 0 | candidate (3) | 1.3 / 1.3 / 1.5 | 64 | 2616 / 5932 | 0.28 / 1.62 | 1020 / — | partial_metrics_review_required |
| slow64k | bandwidth | 1 | ssh (3) | — / — / 0.6 | 60 | 7792 / 19840 | 0.05 / 0.22 | 5059 / — | control |
| slow64k | bandwidth | 1 | main-A (2) | 0.8 / 0.8 / 1.4 | 63 | 2700 / 6028 | 0.18 / 1.33 | 1663 / — | control |
| slow64k | bandwidth | 1 | main-B (1) | 1.3 / 1.3 / 1.4 | 63 | 2692 / 5984 | 0.23 / 1.31 | 1050 / — | control |
| slow64k | bandwidth | 1 | candidate (0) | 1.3 / 1.3 / 1.4 | 64 | 2612 / 5932 | 0.24 / 1.35 | 1016 / — | partial_metrics_review_required |
