# Focused per-repetition comparison evidence

Each row retains one measured role in one repetition. No pooling, noise tolerance or automatic pass is applied. Observed deficits do not establish policy causality; see JSON for exact missing metrics, FPS gaps and resource increases.

| Network | Scene | Rep | Role (slot) | FPS 1% / 5% / avg | Network kbit/s | RSS client / server KiB | CPU client / server % | Age median / p95 ms | Assessment |
|---|---|---:|---|---|---:|---|---|---|---|
| lowbw | compress | 0 | ssh (0) | 5.9 / 7.2 / 23.5 | 978 | 7532 / 19708 | 0.14 / 0.5 | 2547 / 4680 | control |
| lowbw | compress | 0 | main-A (1) | 43.3 / 46.4 / 56.0 | 223 | 2640 / 5876 | 0.84 / 3.37 | 64 / 74 | control |
| lowbw | compress | 0 | main-B (2) | 39.8 / 46.9 / 55.6 | 222 | 2636 / 5896 | 0.85 / 3.38 | 64 / 73 | control |
| lowbw | compress | 0 | candidate (3) | 42.1 / 46.5 / 60.0 | 228 | 2644 / 5716 | 0.84 / 1.15 | 60 / 63 | fps_tradeoff_review_required |
| lowbw | compress | 1 | ssh (3) | 6.5 / 7.0 / 23.5 | 977 | 7564 / 19804 | 0.16 / 0.56 | 2573 / 4689 | control |
| lowbw | compress | 1 | main-A (2) | 40.5 / 46.9 / 55.9 | 222 | 2648 / 5900 | 0.82 / 3.4 | 64 / 74 | control |
| lowbw | compress | 1 | main-B (1) | 43.8 / 46.6 / 55.9 | 222 | 2644 / 5876 | 0.83 / 3.38 | 64 / 73 | control |
| lowbw | compress | 1 | candidate (0) | 43.4 / 46.7 / 60.0 | 228 | 2636 / 5688 | 0.83 / 1.13 | 60 / 63 | fps_tradeoff_review_required |
| lowbw | bandwidth | 0 | ssh (0) | 6.3 / 6.5 / 9.7 | 977 | 7584 / 20128 | 0.12 / 0.29 | 3427 / 6431 | control |
| lowbw | bandwidth | 0 | main-A (1) | 17.5 / 19.3 / 22.9 | 905 | 2644 / 5928 | 0.79 / 2.79 | 103 / 124 | control |
| lowbw | bandwidth | 0 | main-B (2) | 17.7 / 18.6 / 22.4 | 885 | 2632 / 5964 | 0.77 / 2.77 | 101 / 120 | control |
| lowbw | bandwidth | 0 | candidate (3) | 19.2 / 20.9 / 24.4 | 966 | 2676 / 5980 | 0.87 / 2.99 | 118 / 160 | qualitative_review_required |
| lowbw | bandwidth | 1 | ssh (3) | 5.4 / 6.3 / 9.9 | 979 | 7576 / 20152 | 0.13 / 0.32 | 3508 / 6491 | control |
| lowbw | bandwidth | 1 | main-A (2) | 16.4 / 18.3 / 21.2 | 842 | 2632 / 5936 | 0.74 / 2.69 | 100 / 124 | control |
| lowbw | bandwidth | 1 | main-B (1) | 13.6 / 18.6 / 23.2 | 924 | 2636 / 5964 | 0.83 / 2.86 | 107 / 128 | control |
| lowbw | bandwidth | 1 | candidate (0) | 14.8 / 19.5 / 24.1 | 956 | 2628 / 5948 | 0.84 / 2.95 | 113 / 146 | fps_tradeoff_review_required |
| verylow | compress | 0 | ssh (0) | 2.1 / 5.1 / 6.0 | 249 | 7536 / 19808 | 0.09 / 0.58 | 3762 / 6954 | control |
| verylow | compress | 0 | main-A (1) | 23.8 / 27.0 / 34.0 | 131 | 2644 / 5888 | 0.8 / 2.85 | 69 / 88 | control |
| verylow | compress | 0 | main-B (2) | 25.3 / 27.7 / 37.6 | 150 | 2652 / 5888 | 0.86 / 3.14 | 70 / 88 | control |
| verylow | compress | 0 | candidate (3) | 43.8 / 47.1 / 60.0 | 229 | 2628 / 5696 | 1.22 / 1.73 | 72 / 76 | qualitative_review_required |
| verylow | compress | 1 | ssh (3) | 2.4 / 4.1 / 6.1 | 250 | 7556 / 19888 | 0.08 / 0.53 | 3868 / 7249 | control |
| verylow | compress | 1 | main-A (2) | 24.9 / 26.8 / 35.0 | 135 | 2644 / 5888 | 0.8 / 2.76 | 70 / 88 | control |
| verylow | compress | 1 | main-B (1) | 25.1 / 27.7 / 35.9 | 140 | 2652 / 5928 | 0.85 / 3.0 | 70 / 89 | control |
| verylow | compress | 1 | candidate (0) | 38.7 / 44.6 / 60.0 | 228 | 2640 / 5728 | 1.17 / 1.67 | 72 / 76 | qualitative_review_required |
| verylow | bandwidth | 0 | ssh (0) | 1.3 / 1.3 / 2.4 | 250 | 7552 / 19948 | 0.12 / 0.36 | 3919 / — | control |
| verylow | bandwidth | 0 | main-A (1) | 5.4 / 5.6 / 5.9 | 256 | 2628 / 5944 | 0.71 / 2.32 | 384 / 404 | control |
| verylow | bandwidth | 0 | main-B (2) | 5.4 / 5.5 / 6.0 | 256 | 2648 / 5928 | 0.66 / 2.39 | 381 / 408 | control |
| verylow | bandwidth | 0 | candidate (3) | 5.3 / 5.5 / 5.9 | 256 | 2620 / 5948 | 0.7 / 2.36 | 385 / 412 | fps_tradeoff_review_required |
| verylow | bandwidth | 1 | ssh (3) | 1.6 / 1.6 / 2.5 | 249 | 7568 / 19960 | 0.12 / 0.39 | 4227 / 7647 | control |
| verylow | bandwidth | 1 | main-A (2) | 5.4 / 5.5 / 6.0 | 256 | 2616 / 5916 | 0.82 / 2.65 | 382 / 395 | control |
| verylow | bandwidth | 1 | main-B (1) | 5.4 / 5.5 / 6.0 | 256 | 2620 / 5936 | 0.82 / 2.62 | 382 / 409 | control |
| verylow | bandwidth | 1 | candidate (0) | 5.2 / 5.4 / 5.9 | 256 | 2652 / 5988 | 0.84 / 2.62 | 386 / 420 | fps_tradeoff_review_required |
| slow64k | compress | 0 | ssh (0) | 1.3 / 1.3 / 1.5 | 62 | 7596 / 19808 | 0.12 / 0.75 | 4496 / — | control |
| slow64k | compress | 0 | main-A (1) | 11.5 / 12.6 / 15.6 | 63 | 2640 / 5912 | 0.68 / 2.61 | 171 / 413 | control |
| slow64k | compress | 0 | main-B (2) | 11.3 / 12.0 / 15.6 | 63 | 2644 / 5912 | 0.68 / 2.63 | 165 / 352 | control |
| slow64k | compress | 0 | candidate (3) | 6.9 / 12.6 / 15.6 | 64 | 2656 / 5936 | 0.67 / 2.7 | 194 / 429 | fps_tradeoff_review_required |
| slow64k | compress | 1 | ssh (3) | 1.3 / 1.3 / 1.4 | 60 | 7548 / 19804 | 0.09 / 0.65 | 4122 / — | control |
| slow64k | compress | 1 | main-A (2) | 11.9 / 12.5 / 16.0 | 63 | 1756 / 5164 | 0.64 / 2.36 | 164 / 374 | control |
| slow64k | compress | 1 | main-B (1) | 11.3 / 12.8 / 15.7 | 63 | 2256 / 5464 | 0.69 / 2.36 | 142 / 350 | control |
| slow64k | compress | 1 | candidate (0) | 8.9 / 12.5 / 15.7 | 63 | 2452 / 5420 | 0.62 / 2.43 | 189 / 345 | fps_tradeoff_review_required |
| slow64k | bandwidth | 0 | ssh (0) | — / — / 0.6 | 60 | 7564 / 19900 | 0.09 / 0.33 | 4933 / — | control |
| slow64k | bandwidth | 0 | main-A (1) | — / — / 1.2 | 64 | 2640 / 5948 | 0.29 / 1.71 | 1020 / — | control |
| slow64k | bandwidth | 0 | main-B (2) | 0.8 / 0.8 / 1.4 | 64 | 2648 / 5916 | 0.29 / 1.69 | 997 / — | control |
| slow64k | bandwidth | 0 | candidate (3) | — / — / 1.2 | 63 | 2644 / 5916 | 0.35 / 1.73 | 1015 / — | partial_metrics_review_required |
| slow64k | bandwidth | 1 | ssh (3) | — / — / 0.6 | 61 | 7796 / 19828 | 0.09 / 0.33 | 5067 / — | control |
| slow64k | bandwidth | 1 | main-A (2) | 1.3 / 1.3 / 1.4 | 63 | 2608 / 5936 | 0.29 / 1.75 | 1079 / — | control |
| slow64k | bandwidth | 1 | main-B (1) | 1.3 / 1.3 / 1.4 | 64 | 2660 / 5904 | 0.29 / 1.72 | 1022 / — | control |
| slow64k | bandwidth | 1 | candidate (0) | 0.8 / 0.8 / 1.5 | 63 | 2612 / 5948 | 0.23 / 1.93 | 2179 / — | partial_metrics_review_required |
