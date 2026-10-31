# Focused per-repetition comparison evidence

Each row retains one measured role in one repetition. No pooling, noise tolerance or automatic pass is applied. Observed deficits do not establish policy causality; see JSON for exact missing metrics, FPS gaps and resource increases.

| Network | Scene | Rep | Role (slot) | FPS 1% / 5% / avg | Network kbit/s | RSS client / server KiB | CPU client / server % | Age median / p95 ms | Assessment |
|---|---|---:|---|---|---:|---|---|---|---|
| lowbw | compress | 0 | ssh (0) | 6.5 / 7.2 / 23.7 | 977 | 7568 / 19656 | 0.15 / 0.48 | 2546 / 4712 | control |
| lowbw | compress | 0 | main-A (1) | 41.3 / 45.4 / 60.0 | 228 | 2688 / 5700 | 0.81 / 1.1 | 60 / 64 | control |
| lowbw | compress | 0 | main-B (2) | 40.8 / 45.8 / 60.0 | 228 | 2692 / 5700 | 0.8 / 1.11 | 60 / 64 | control |
| lowbw | compress | 0 | candidate (3) | 44.1 / 46.8 / 60.0 | 227 | 2628 / 5704 | 0.81 / 1.12 | 59 / 63 | qualitative_review_required |
| lowbw | compress | 1 | ssh (2) | 6.4 / 7.1 / 23.5 | 979 | 8080 / 19732 | 0.15 / 0.53 | 2507 / 4637 | control |
| lowbw | compress | 1 | main-A (3) | 41.3 / 46.3 / 60.0 | 227 | 2692 / 5696 | 0.84 / 1.14 | 59 / 63 | control |
| lowbw | compress | 1 | main-B (0) | 41.2 / 46.6 / 60.0 | 227 | 2688 / 5676 | 0.84 / 1.17 | 60 / 63 | control |
| lowbw | compress | 1 | candidate (1) | 41.3 / 45.4 / 60.0 | 228 | 2628 / 5692 | 0.85 / 1.17 | 59 / 63 | fps_tradeoff_review_required |
| lowbw | bandwidth | 0 | ssh (0) | 5.2 / 6.5 / 10.0 | 979 | 7580 / 20108 | 0.11 / 0.3 | 3523 / 6485 | control |
| lowbw | bandwidth | 0 | main-A (1) | 17.5 / 19.0 / 22.7 | 901 | 2720 / 5536 | 0.8 / 2.83 | 110 / 169 | control |
| lowbw | bandwidth | 0 | main-B (2) | 12.0 / 19.2 / 23.2 | 944 | 2692 / 5504 | 0.82 / 2.93 | 129 / 184 | control |
| lowbw | bandwidth | 0 | candidate (3) | 8.5 / 18.4 / 22.2 | 902 | 2660 / 5508 | 0.81 / 2.86 | 109 / 175 | fps_tradeoff_review_required |
| lowbw | bandwidth | 1 | ssh (2) | 5.1 / 6.3 / 10.0 | 977 | 7676 / 20028 | 0.12 / 0.28 | 3525 / 6449 | control |
| lowbw | bandwidth | 1 | main-A (3) | 14.7 / 19.3 / 23.6 | 949 | 2688 / 5992 | 0.87 / 2.97 | 126 / 179 | control |
| lowbw | bandwidth | 1 | main-B (0) | 13.7 / 19.0 / 23.5 | 938 | 2700 / 5992 | 0.86 / 2.9 | 114 / 169 | control |
| lowbw | bandwidth | 1 | candidate (1) | 9.7 / 18.7 / 23.2 | 938 | 2676 / 5944 | 0.88 / 2.93 | 119 / 175 | fps_tradeoff_review_required |
| verylow | compress | 0 | ssh (0) | 2.6 / 5.1 / 6.0 | 250 | 7560 / 19840 | 0.09 / 0.46 | 3828 / 6927 | control |
| verylow | compress | 0 | main-A (1) | 41.7 / 46.7 / 60.0 | 230 | 2704 / 5696 | 1.01 / 1.43 | 72 / 76 | control |
| verylow | compress | 0 | main-B (2) | 42.5 / 47.0 / 60.0 | 229 | 2692 / 5696 | 1.0 / 1.43 | 72 / 75 | control |
| verylow | compress | 0 | candidate (3) | 42.0 / 46.9 / 60.0 | 229 | 2628 / 5696 | 1.03 / 1.46 | 72 / 76 | fps_tradeoff_review_required |
| verylow | compress | 1 | ssh (2) | 3.0 / 5.2 / 6.1 | 249 | 7552 / 19860 | 0.06 / 0.39 | 3860 / 7213 | control |
| verylow | compress | 1 | main-A (3) | 41.4 / 48.3 / 60.0 | 229 | 2692 / 5684 | 0.88 / 1.23 | 71 / 75 | control |
| verylow | compress | 1 | main-B (0) | 41.3 / 45.6 / 60.0 | 230 | 2692 / 5680 | 0.89 / 1.23 | 72 / 78 | control |
| verylow | compress | 1 | candidate (1) | 41.6 / 46.8 / 60.0 | 229 | 2628 / 5664 | 0.89 / 1.26 | 72 / 76 | fps_tradeoff_review_required |
| verylow | bandwidth | 0 | ssh (0) | 1.5 / 1.5 / 2.4 | 251 | 7560 / 19832 | 0.1 / 0.37 | 3878 / — | control |
| verylow | bandwidth | 0 | main-A (1) | 3.1 / 5.5 / 5.9 | 255 | 2724 / 5900 | 0.69 / 2.46 | 386 / 417 | control |
| verylow | bandwidth | 0 | main-B (2) | 3.0 / 5.4 / 5.7 | 256 | 2724 / 5936 | 0.72 / 2.34 | 384 / 411 | control |
| verylow | bandwidth | 0 | candidate (3) | 5.5 / 5.5 / 6.0 | 256 | 2620 / 5976 | 0.73 / 2.47 | 379 / 401 | qualitative_review_required |
| verylow | bandwidth | 1 | ssh (2) | 1.5 / 1.5 / 2.5 | 250 | 7572 / 19900 | 0.13 / 0.37 | 4068 / 7762 | control |
| verylow | bandwidth | 1 | main-A (3) | 5.4 / 5.5 / 5.9 | 256 | 2344 / 5036 | 0.83 / 2.48 | 382 / 405 | control |
| verylow | bandwidth | 1 | main-B (0) | 5.3 / 5.5 / 5.9 | 256 | 1992 / 5332 | 0.8 / 2.59 | 387 / 420 | control |
| verylow | bandwidth | 1 | candidate (1) | 5.4 / 5.5 / 6.0 | 255 | 2112 / 5428 | 0.8 / 2.57 | 383 / 416 | qualitative_review_required |
| slow64k | compress | 0 | ssh (0) | 1.0 / 1.0 / 1.5 | 61 | 7568 / 19860 | 0.07 / 0.76 | 4575 / — | control |
| slow64k | compress | 0 | main-A (1) | 9.0 / 11.9 / 15.4 | 63 | 2704 / 5896 | 0.63 / 2.56 | 178 / 376 | control |
| slow64k | compress | 0 | main-B (2) | 6.3 / 12.5 / 15.4 | 64 | 2728 / 5940 | 0.63 / 2.54 | 182 / 372 | control |
| slow64k | compress | 0 | candidate (3) | 5.9 / 12.1 / 15.4 | 63 | 2664 / 5872 | 0.61 / 2.58 | 191 / 353 | fps_tradeoff_review_required |
| slow64k | compress | 1 | ssh (2) | 1.0 / 1.0 / 1.5 | 62 | 7544 / 19756 | 0.07 / 0.7 | 4622 / — | control |
| slow64k | compress | 1 | main-A (3) | 10.3 / 12.4 / 15.6 | 63 | 2708 / 5912 | 0.61 / 2.55 | 192 / 417 | control |
| slow64k | compress | 1 | main-B (0) | 10.9 / 12.7 / 15.6 | 64 | 2700 / 5872 | 0.72 / 2.51 | 186 / 416 | control |
| slow64k | compress | 1 | candidate (1) | 8.6 / 12.8 / 16.1 | 64 | 2640 / 5888 | 0.65 / 2.66 | 186 / 376 | fps_tradeoff_review_required |
| slow64k | bandwidth | 0 | ssh (0) | — / — / 0.6 | 60 | 7540 / 19904 | 0.06 / 0.33 | 5031 / — | control |
| slow64k | bandwidth | 0 | main-A (1) | — / — / 1.2 | 63 | 2688 / 5464 | 0.28 / 1.52 | 1036 / — | control |
| slow64k | bandwidth | 0 | main-B (2) | 0.8 / 0.8 / 1.5 | 64 | 2676 / 5532 | 0.19 / 1.64 | 2101 / — | control |
| slow64k | bandwidth | 0 | candidate (3) | — / — / 1.2 | 64 | 2608 / 5500 | 0.27 / 1.52 | 1035 / — | partial_metrics_review_required |
| slow64k | bandwidth | 1 | ssh (2) | — / — / 0.6 | 62 | 7552 / 19912 | 0.1 / 0.33 | 4861 / — | control |
| slow64k | bandwidth | 1 | main-A (3) | 1.3 / 1.3 / 1.4 | 64 | 2700 / 5904 | 0.3 / 1.68 | 1029 / — | control |
| slow64k | bandwidth | 1 | main-B (0) | — / — / 1.2 | 64 | 2700 / 5912 | 0.28 / 1.63 | 1035 / — | control |
| slow64k | bandwidth | 1 | candidate (1) | 0.8 / 0.8 / 1.5 | 63 | 2632 / 5944 | 0.2 / 1.7 | 1913 / — | partial_metrics_review_required |
