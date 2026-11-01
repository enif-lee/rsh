# Focused per-repetition comparison evidence

Each row retains one measured role in one repetition. No pooling, noise tolerance or automatic pass is applied. Observed deficits do not establish policy causality; see JSON for exact missing metrics, FPS gaps and resource increases.

| Network | Scene | Rep | Role (slot) | FPS 1% / 5% / avg | Network kbit/s | RSS client / server KiB | CPU client / server % | Age median / p95 ms | Assessment |
|---|---|---:|---|---|---:|---|---|---|---|
| lowbw | compress | 0 | ssh (0) | 6.3 / 7.1 / 23.7 | 978 | 7668 / 19748 | 0.14 / 0.49 | 2613 / 4700 | control |
| lowbw | compress | 0 | main-A (1) | 41.0 / 46.3 / 60.0 | 228 | 2688 / 5688 | 0.82 / 1.12 | 59 / 62 | control |
| lowbw | compress | 0 | main-B (2) | 41.2 / 46.3 / 60.0 | 228 | 2692 / 5712 | 0.82 / 1.12 | 59 / 62 | control |
| lowbw | compress | 0 | candidate (3) | 41.8 / 46.6 / 60.0 | 228 | 2692 / 5804 | 0.84 / 1.17 | 60 / 64 | qualitative_review_required |
| lowbw | compress | 1 | ssh (2) | 6.6 / 7.2 / 23.7 | 976 | 7936 / 19780 | 0.16 / 0.5 | 2584 / 4741 | control |
| lowbw | compress | 1 | main-A (3) | 41.9 / 45.5 / 60.0 | 228 | 2692 / 5712 | 0.81 / 1.09 | 59 / 63 | control |
| lowbw | compress | 1 | main-B (0) | 42.1 / 45.8 / 60.0 | 228 | 2696 / 5680 | 0.81 / 1.12 | 60 / 64 | control |
| lowbw | compress | 1 | candidate (1) | 42.7 / 46.2 / 60.0 | 228 | 2692 / 5756 | 0.83 / 1.11 | 60 / 64 | qualitative_review_required |
| lowbw | bandwidth | 0 | ssh (0) | 6.3 / 6.7 / 10.0 | 979 | 7852 / 20048 | 0.12 / 0.26 | 3534 / 6398 | control |
| lowbw | bandwidth | 0 | main-A (1) | 12.8 / 17.9 / 20.5 | 817 | 2700 / 5968 | 0.72 / 2.63 | 97 / 117 | control |
| lowbw | bandwidth | 0 | main-B (2) | 13.1 / 18.5 / 22.7 | 914 | 2696 / 5936 | 0.82 / 2.89 | 111 / 172 | control |
| lowbw | bandwidth | 0 | candidate (3) | 17.9 / 19.6 / 24.2 | 967 | 2708 / 6012 | 0.9 / 2.95 | 125 / 178 | qualitative_review_required |
| lowbw | bandwidth | 1 | ssh (2) | 6.7 / 6.8 / 10.0 | 977 | 7580 / 20128 | 0.11 / 0.3 | 3522 / 6385 | control |
| lowbw | bandwidth | 1 | main-A (3) | 10.1 / 18.3 / 22.5 | 908 | 2720 / 5996 | 0.89 / 2.9 | 113 / 173 | control |
| lowbw | bandwidth | 1 | main-B (0) | 14.5 / 18.2 / 22.6 | 900 | 2688 / 5984 | 0.82 / 2.92 | 113 / 170 | control |
| lowbw | bandwidth | 1 | candidate (1) | 11.1 / 19.0 / 23.6 | 968 | 2692 / 6016 | 0.91 / 3.05 | 126 / 182 | fps_tradeoff_review_required |
| verylow | compress | 0 | ssh (0) | 1.9 / 5.1 / 5.6 | 250 | 7532 / 19844 | 0.07 / 0.4 | 3555 / 6727 | control |
| verylow | compress | 0 | main-A (1) | 40.5 / 46.6 / 60.0 | 229 | 2692 / 5704 | 0.87 / 1.19 | 72 / 75 | control |
| verylow | compress | 0 | main-B (2) | 41.8 / 47.3 / 60.0 | 229 | 2692 / 5728 | 0.87 / 1.22 | 71 / 75 | control |
| verylow | compress | 0 | candidate (3) | 41.6 / 46.5 / 60.0 | 230 | 2692 / 5792 | 0.9 / 1.24 | 72 / 76 | fps_tradeoff_review_required |
| verylow | compress | 1 | ssh (2) | 3.0 / 5.1 / 6.0 | 248 | 7540 / 19828 | 0.07 / 0.41 | 3795 / 6924 | control |
| verylow | compress | 1 | main-A (3) | 43.5 / 46.9 / 60.0 | 229 | 2692 / 5728 | 0.9 / 1.26 | 72 / 75 | control |
| verylow | compress | 1 | main-B (0) | 41.0 / 45.8 / 60.0 | 229 | 2700 / 5692 | 0.91 / 1.26 | 72 / 76 | control |
| verylow | compress | 1 | candidate (1) | 42.5 / 46.6 / 60.0 | 229 | 2692 / 5760 | 0.9 / 1.29 | 72 / 75 | fps_tradeoff_review_required |
| verylow | bandwidth | 0 | ssh (0) | 1.5 / 1.5 / 2.4 | 249 | 7548 / 19936 | 0.07 / 0.33 | 4197 / — | control |
| verylow | bandwidth | 0 | main-A (1) | 5.2 / 5.5 / 5.9 | 256 | 2692 / 5920 | 0.59 / 2.3 | 385 / 420 | control |
| verylow | bandwidth | 0 | main-B (2) | 5.2 / 5.6 / 5.9 | 256 | 2692 / 5940 | 0.59 / 2.18 | 376 / 411 | control |
| verylow | bandwidth | 0 | candidate (3) | 5.5 / 5.5 / 5.9 | 256 | 2700 / 5968 | 0.6 / 2.19 | 389 / 422 | fps_tradeoff_review_required |
| verylow | bandwidth | 1 | ssh (2) | 1.5 / 1.5 / 2.5 | 250 | 7568 / 19956 | 0.08 / 0.35 | 4443 / 7377 | control |
| verylow | bandwidth | 1 | main-A (3) | 5.5 / 5.5 / 6.0 | 255 | 1540 / 5296 | 0.6 / 2.41 | 382 / 417 | control |
| verylow | bandwidth | 1 | main-B (0) | 5.5 / 5.5 / 6.0 | 255 | 2296 / 5332 | 0.61 / 2.42 | 380 / 409 | control |
| verylow | bandwidth | 1 | candidate (1) | 5.3 / 5.5 / 6.0 | 256 | 2272 / 5152 | 0.61 / 2.36 | 381 / 409 | fps_tradeoff_review_required |
| slow64k | compress | 0 | ssh (0) | 1.0 / 1.0 / 1.4 | 61 | 7544 / 19788 | 0.07 / 0.54 | 4088 / — | control |
| slow64k | compress | 0 | main-A (1) | 5.8 / 12.2 / 15.9 | 63 | 2724 / 5876 | 0.54 / 2.36 | 185 / 346 | control |
| slow64k | compress | 0 | main-B (2) | 9.1 / 12.3 / 15.6 | 64 | 2712 / 5908 | 0.55 / 2.24 | 188 / 370 | control |
| slow64k | compress | 0 | candidate (3) | 6.9 / 13.1 / 16.0 | 64 | 2720 / 5948 | 0.56 / 2.29 | 187 / 438 | fps_tradeoff_review_required |
| slow64k | compress | 1 | ssh (2) | — / — / 1.2 | 62 | 7556 / 19668 | 0.06 / 0.53 | 3840 / — | control |
| slow64k | compress | 1 | main-A (3) | 11.3 / 12.5 / 15.6 | 63 | 2696 / 5908 | 0.56 / 2.47 | 182 / 362 | control |
| slow64k | compress | 1 | main-B (0) | 9.1 / 12.3 / 15.6 | 63 | 2708 / 5916 | 0.57 / 2.37 | 176 / 368 | control |
| slow64k | compress | 1 | candidate (1) | 8.5 / 13.0 / 15.9 | 64 | 2704 / 5948 | 0.6 / 2.46 | 189 / 374 | partial_metrics_review_required |
| slow64k | bandwidth | 0 | ssh (0) | — / — / 0.6 | 62 | 7788 / 19812 | 0.03 / 0.22 | 4867 / — | control |
| slow64k | bandwidth | 0 | main-A (1) | 1.3 / 1.3 / 1.4 | 63 | 2708 / 5912 | 0.24 / 1.77 | 1059 / — | control |
| slow64k | bandwidth | 0 | main-B (2) | 0.8 / 0.8 / 1.4 | 64 | 2676 / 5960 | 0.22 / 1.78 | 1802 / — | control |
| slow64k | bandwidth | 0 | candidate (3) | 1.4 / 1.4 / 1.6 | 63 | 2696 / 6008 | 0.2 / 1.74 | 1824 / — | partial_metrics_review_required |
| slow64k | bandwidth | 1 | ssh (2) | — / — / 0.6 | 60 | 7564 / 19892 | 0.03 / 0.23 | 4807 / — | control |
| slow64k | bandwidth | 1 | main-A (3) | 1.3 / 1.3 / 1.5 | 64 | 2672 / 5944 | 0.22 / 1.67 | 1033 / — | control |
| slow64k | bandwidth | 1 | main-B (0) | 1.3 / 1.3 / 1.4 | 64 | 2676 / 5904 | 0.24 / 1.7 | 1035 / — | control |
| slow64k | bandwidth | 1 | candidate (1) | 1.3 / 1.3 / 1.5 | 64 | 2672 / 5968 | 0.24 / 1.6 | 1034 / — | partial_metrics_review_required |
