# Focused per-repetition comparison evidence

Each row retains one measured role in one repetition. No pooling, noise tolerance or automatic pass is applied. Observed deficits do not establish policy causality; see JSON for exact missing metrics, FPS gaps and resource increases.

| Network | Scene | Rep | Role (slot) | FPS 1% / 5% / avg | Network kbit/s | RSS client / server KiB | CPU client / server % | Age median / p95 ms | Assessment |
|---|---|---:|---|---|---:|---|---|---|---|
| lowbw | compress | 0 | ssh (0) | 6.6 / 7.6 / 23.7 | 979 | 7788 / 19740 | 0.2 / 0.68 | 2547 / 4711 | control |
| lowbw | compress | 0 | main-A (1) | 40.0 / 45.7 / 60.0 | 227 | 2700 / 5724 | 1.05 / 1.41 | 60 / 64 | control |
| lowbw | compress | 0 | main-B (2) | 41.6 / 46.4 / 60.0 | 227 | 2692 / 5696 | 1.05 / 1.45 | 60 / 64 | control |
| lowbw | compress | 0 | candidate (3) | 38.6 / 45.1 / 60.0 | 228 | 2692 / 5820 | 1.09 / 1.44 | 60 / 64 | fps_tradeoff_review_required |
| lowbw | compress | 1 | ssh (2) | 6.6 / 7.1 / 23.5 | 978 | 7652 / 19736 | 0.19 / 0.54 | 2568 / 4695 | control |
| lowbw | compress | 1 | main-A (3) | 41.3 / 46.5 / 60.0 | 228 | 2700 / 5700 | 0.98 / 1.3 | 60 / 64 | control |
| lowbw | compress | 1 | main-B (0) | 40.6 / 46.8 / 60.0 | 227 | 2692 / 5700 | 0.96 / 1.32 | 60 / 64 | control |
| lowbw | compress | 1 | candidate (1) | 41.5 / 45.7 / 60.0 | 227 | 2688 / 5800 | 0.96 / 1.31 | 59 / 63 | fps_tradeoff_review_required |
| lowbw | bandwidth | 0 | ssh (0) | 4.6 / 6.8 / 9.9 | 976 | 7828 / 20096 | 0.14 / 0.31 | 3489 / 6496 | control |
| lowbw | bandwidth | 0 | main-A (1) | 13.8 / 18.6 / 23.1 | 925 | 2704 / 5964 | 0.84 / 2.83 | 114 / 178 | control |
| lowbw | bandwidth | 0 | main-B (2) | 10.8 / 17.9 / 21.4 | 867 | 2724 / 5904 | 0.77 / 2.74 | 99 / 120 | control |
| lowbw | bandwidth | 0 | candidate (3) | 18.0 / 20.2 / 24.2 | 961 | 2692 / 6072 | 0.9 / 2.93 | 123 / 160 | qualitative_review_required |
| lowbw | bandwidth | 1 | ssh (2) | 5.8 / 6.5 / 9.9 | 978 | 7572 / 20108 | 0.12 / 0.28 | 3492 / 6493 | control |
| lowbw | bandwidth | 1 | main-A (3) | 14.2 / 18.3 / 21.6 | 858 | 2724 / 5936 | 0.72 / 2.6 | 101 / 133 | control |
| lowbw | bandwidth | 1 | main-B (0) | 11.2 / 17.9 / 23.0 | 930 | 2700 / 5952 | 0.79 / 2.76 | 119 / 169 | control |
| lowbw | bandwidth | 1 | candidate (1) | 17.4 / 18.8 / 23.4 | 926 | 2692 / 6056 | 0.78 / 2.8 | 108 / 139 | qualitative_review_required |
| verylow | compress | 0 | ssh (0) | 2.1 / 4.3 / 6.0 | 249 | 7416 / 19864 | 0.08 / 0.5 | 3757 / 6949 | control |
| verylow | compress | 0 | main-A (1) | 41.8 / 46.5 / 60.0 | 229 | 1996 / 5248 | 1.01 / 1.37 | 71 / 75 | control |
| verylow | compress | 0 | main-B (2) | 41.4 / 46.3 / 60.0 | 229 | 1696 / 5280 | 0.97 / 1.38 | 72 / 76 | control |
| verylow | compress | 0 | candidate (3) | 41.8 / 47.2 / 60.0 | 229 | 1752 / 5216 | 0.99 / 1.4 | 72 / 76 | qualitative_review_required |
| verylow | compress | 1 | ssh (2) | 2.4 / 4.2 / 5.7 | 250 | 7556 / 19916 | 0.09 / 0.58 | 3650 / 6771 | control |
| verylow | compress | 1 | main-A (3) | 42.2 / 45.9 / 60.0 | 229 | 2692 / 5696 | 1.25 / 1.83 | 72 / 76 | control |
| verylow | compress | 1 | main-B (0) | 41.6 / 45.9 / 60.0 | 229 | 2692 / 5700 | 1.28 / 1.84 | 72 / 77 | control |
| verylow | compress | 1 | candidate (1) | 39.7 / 45.3 / 60.0 | 229 | 2692 / 5824 | 1.28 / 1.84 | 72 / 77 | fps_tradeoff_review_required |
| verylow | bandwidth | 0 | ssh (0) | 1.2 / 1.2 / 2.4 | 249 | 7560 / 19908 | 0.1 / 0.37 | 3907 / — | control |
| verylow | bandwidth | 0 | main-A (1) | 5.3 / 5.5 / 6.0 | 256 | 2684 / 5936 | 0.68 / 2.34 | 383 / 409 | control |
| verylow | bandwidth | 0 | main-B (2) | 2.8 / 5.5 / 5.7 | 256 | 2704 / 5904 | 0.71 / 2.27 | 382 / 409 | control |
| verylow | bandwidth | 0 | candidate (3) | 5.3 / 5.5 / 5.9 | 256 | 2680 / 6096 | 0.7 / 2.35 | 384 / 409 | fps_tradeoff_review_required |
| verylow | bandwidth | 1 | ssh (2) | 1.2 / 1.2 / 2.4 | 248 | 7576 / 19908 | 0.12 / 0.33 | 3849 / — | control |
| verylow | bandwidth | 1 | main-A (3) | 5.4 / 5.5 / 6.0 | 256 | 2684 / 5920 | 0.71 / 2.38 | 386 / 409 | control |
| verylow | bandwidth | 1 | main-B (0) | 5.3 / 5.5 / 5.9 | 256 | 2696 / 5968 | 0.76 / 2.42 | 388 / 408 | control |
| verylow | bandwidth | 1 | candidate (1) | 5.3 / 5.5 / 6.0 | 256 | 2692 / 6048 | 0.77 / 2.4 | 384 / 410 | fps_tradeoff_review_required |
| slow64k | compress | 0 | ssh (0) | 1.1 / 1.1 / 1.4 | 61 | 7552 / 19832 | 0.06 / 0.83 | 4312 / — | control |
| slow64k | compress | 0 | main-A (1) | 11.6 / 12.6 / 15.6 | 64 | 2696 / 5908 | 0.62 / 2.55 | 179 / 360 | control |
| slow64k | compress | 0 | main-B (2) | 9.3 / 12.0 / 15.7 | 63 | 2688 / 5864 | 0.64 / 2.66 | 179 / 367 | control |
| slow64k | compress | 0 | candidate (3) | 9.5 / 12.1 / 15.5 | 63 | 2704 / 6044 | 0.62 / 2.72 | 198 / 441 | fps_tradeoff_review_required |
| slow64k | compress | 1 | ssh (2) | 1.3 / 1.3 / 1.4 | 62 | 7532 / 19708 | 0.08 / 0.6 | 4239 / — | control |
| slow64k | compress | 1 | main-A (3) | 5.5 / 8.9 / 15.0 | 63 | 2708 / 5900 | 0.46 / 1.81 | 244 / 638 | control |
| slow64k | compress | 1 | main-B (0) | 8.7 / 12.8 / 16.2 | 64 | 2704 / 5872 | 0.5 / 1.98 | 188 / 372 | control |
| slow64k | compress | 1 | candidate (1) | 9.0 / 13.3 / 16.1 | 64 | 2684 / 6000 | 0.49 / 2.03 | 200 / 385 | fps_tradeoff_review_required |
| slow64k | bandwidth | 0 | ssh (0) | — / — / 0.6 | 62 | 7796 / 19856 | 0.05 / 0.23 | 5079 / — | control |
| slow64k | bandwidth | 0 | main-A (1) | 1.4 / 1.4 / 1.6 | 63 | 2668 / 5976 | 0.19 / 1.62 | 2041 / — | control |
| slow64k | bandwidth | 0 | main-B (2) | 1.3 / 1.3 / 1.4 | 64 | 2700 / 5944 | 0.29 / 1.67 | 1062 / — | control |
| slow64k | bandwidth | 0 | candidate (3) | 1.3 / 1.3 / 1.5 | 64 | 2692 / 6044 | 0.28 / 1.61 | 1037 / — | partial_metrics_review_required |
| slow64k | bandwidth | 1 | ssh (2) | — / — / 0.6 | 60 | 7548 / 19844 | 0.04 / 0.25 | 4829 / — | control |
| slow64k | bandwidth | 1 | main-A (3) | 1.4 / 1.4 / 1.6 | 63 | 2712 / 5960 | 0.2 / 1.71 | 2039 / — | control |
| slow64k | bandwidth | 1 | main-B (0) | 0.8 / 0.8 / 1.4 | 64 | 2688 / 5992 | 0.25 / 1.68 | 1584 / — | control |
| slow64k | bandwidth | 1 | candidate (1) | 1.4 / 1.4 / 1.5 | 63 | 2712 / 6060 | 0.21 / 1.63 | 1732 / — | partial_metrics_review_required |
