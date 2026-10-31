# Per-repetition acceptance evidence

Each row retains one measured role in one repetition. No pooling, noise tolerance or automatic pass is applied. Observed deficits do not establish policy causality; see JSON for exact missing metrics, FPS gaps and resource increases.

| Network | Scene | Rep | Role (slot) | FPS avg / 1% / 5% | Network kbit/s | RSS client / server KiB | CPU client / server % | Age median / p95 ms | Assessment |
|---|---|---:|---|---|---:|---|---|---|---|
| lowbw | compress | 0 | ssh (0) | 23.9 / 6.6 / 7.2 | 978 | 7692 / 19736 | 0.19 / 0.54 | 2630 / 4717 | control |
| lowbw | compress | 0 | main-A (1) | 56.4 / 42.4 / 47.5 | 224 | 2636 / 6020 | 0.97 / 3.98 | 64 / 73 | control |
| lowbw | compress | 0 | main-B (2) | 55.7 / 41.1 / 47.2 | 222 | 2648 / 6052 | 1.17 / 3.85 | 64 / 73 | control |
| lowbw | compress | 0 | candidate (3) | 59.9 / 38.4 / 48.8 | 228 | 2628 / 5792 | 1.05 / 1.35 | 60 / 63 | observed_fps_gap_not_accepted |
| lowbw | compress | 1 | ssh (2) | 23.7 / 6.6 / 7.1 | 978 | 7520 / 19684 | 0.19 / 0.54 | 2623 / 4703 | control |
| lowbw | compress | 1 | main-A (3) | 57.4 / 29.3 / 43.8 | 219 | 2628 / 5924 | 0.83 / 1.39 | 59 / 63 | control |
| lowbw | compress | 1 | main-B (0) | 56.0 / 44.5 / 47.1 | 222 | 2652 / 5996 | 0.9 / 3.27 | 64 / 73 | control |
| lowbw | compress | 1 | candidate (1) | 60.0 / 44.1 / 47.4 | 227 | 2628 / 5836 | 0.88 / 1.16 | 60 / 62 | observed_fps_gap_not_accepted |
| lowbw | bandwidth | 0 | ssh (0) | 10.0 / 5.4 / 6.7 | 979 | 7584 / 20168 | 0.15 / 0.32 | 3522 / 6504 | control |
| lowbw | bandwidth | 0 | main-A (1) | 20.4 / 13.9 / 18.5 | 813 | 2648 / 6052 | 0.75 / 2.48 | 97 / 115 | control |
| lowbw | bandwidth | 0 | main-B (2) | 21.6 / 17.3 / 18.8 | 857 | 2636 / 6080 | 0.77 / 2.64 | 97 / 137 | control |
| lowbw | bandwidth | 0 | candidate (3) | 22.2 / 11.3 / 18.5 | 897 | 2644 / 6044 | 0.85 / 2.83 | 103 / 154 | observed_fps_gap_not_accepted |
| lowbw | bandwidth | 1 | ssh (2) | 10.0 / 5.8 / 6.8 | 977 | 7596 / 20088 | 0.18 / 0.38 | 3488 / 6395 | control |
| lowbw | bandwidth | 1 | main-A (3) | 19.9 / 10.8 / 17.9 | 799 | 2644 / 6084 | 1.0 / 3.0 | 97 / 121 | control |
| lowbw | bandwidth | 1 | main-B (0) | 20.2 / 14.3 / 18.1 | 803 | 2640 / 6040 | 0.98 / 3.14 | 97 / 117 | control |
| lowbw | bandwidth | 1 | candidate (1) | 20.2 / 13.6 / 16.7 | 808 | 2644 / 6048 | 0.97 / 3.1 | 98 / 117 | observed_fps_gap_not_accepted |
| verylow | compress | 0 | ssh (0) | 6.1 / 3.0 / 5.0 | 250 | 7556 / 19696 | 0.11 / 0.43 | 3884 / 7247 | control |
| verylow | compress | 0 | main-A (1) | 36.0 / 26.0 / 27.9 | 142 | 2644 / 6060 | 0.73 / 2.81 | 69 / 87 | control |
| verylow | compress | 0 | main-B (2) | 37.9 / 26.8 / 27.8 | 151 | 2640 / 6028 | 0.9 / 2.89 | 69 / 87 | control |
| verylow | compress | 0 | candidate (3) | 60.0 / 41.7 / 47.0 | 229 | 2628 / 5848 | 1.14 / 1.51 | 71 / 75 | review_required_no_automatic_pass |
| verylow | compress | 1 | ssh (2) | 6.0 / 2.7 / 5.2 | 248 | 7560 / 19812 | 0.11 / 0.51 | 3778 / 6929 | control |
| verylow | compress | 1 | main-A (3) | 36.4 / 24.8 / 27.1 | 144 | 2648 / 6052 | 1.1 / 3.35 | 69 / 87 | control |
| verylow | compress | 1 | main-B (0) | 35.6 / 25.9 / 27.5 | 139 | 2648 / 6060 | 1.21 / 2.95 | 69 / 87 | control |
| verylow | compress | 1 | candidate (1) | 59.6 / 40.6 / 47.3 | 229 | 2636 / 5860 | 1.32 / 1.81 | 72 / 79 | review_required_no_automatic_pass |
| verylow | bandwidth | 0 | ssh (0) | 2.5 / 1.3 / 1.3 | 248 | 7552 / 20044 | 0.13 / 0.4 | 4262 / 7704 | control |
| verylow | bandwidth | 0 | main-A (1) | 6.0 / 5.2 / 5.6 | 256 | 2372 / 5612 | 0.92 / 2.46 | 379 / 406 | control |
| verylow | bandwidth | 0 | main-B (2) | 6.0 / 5.2 / 5.5 | 256 | 2372 / 5624 | 0.93 / 2.62 | 383 / 414 | control |
| verylow | bandwidth | 0 | candidate (3) | 5.7 / 3.0 / 5.4 | 255 | 2380 / 5656 | 0.92 / 2.54 | 383 / 402 | observed_fps_gap_not_accepted |
| verylow | bandwidth | 1 | ssh (2) | 2.5 / 1.6 / 1.6 | 250 | 7572 / 20020 | 0.15 / 0.36 | 4076 / 7720 | control |
| verylow | bandwidth | 1 | main-A (3) | 5.9 / 5.3 / 5.5 | 256 | 2632 / 6064 | 0.87 / 2.39 | 384 / 409 | control |
| verylow | bandwidth | 1 | main-B (0) | 6.0 / 5.5 / 5.6 | 256 | 2620 / 6064 | 0.82 / 2.45 | 376 / 401 | control |
| verylow | bandwidth | 1 | candidate (1) | 5.9 / 5.3 / 5.5 | 255 | 2616 / 6048 | 0.88 / 2.38 | 387 / 412 | observed_fps_gap_not_accepted |
| slow64k | compress | 0 | ssh (0) | 1.5 / 1.0 / 1.0 | 63 | 7544 / 19716 | 0.11 / 0.77 | 4623 / — | control |
| slow64k | compress | 0 | main-A (1) | 15.6 / 11.7 / 12.3 | 63 | 2628 / 6012 | 0.64 / 2.61 | 169 / 341 | control |
| slow64k | compress | 0 | main-B (2) | 15.5 / 11.5 / 13.0 | 64 | 2636 / 6008 | 0.68 / 2.56 | 171 / 346 | control |
| slow64k | compress | 0 | candidate (3) | 15.9 / 11.0 / 12.7 | 63 | 2632 / 6004 | 0.69 / 2.6 | 147 / 358 | observed_fps_gap_not_accepted |
| slow64k | compress | 1 | ssh (2) | 1.5 / 1.1 / 1.1 | 62 | 7552 / 19692 | 0.05 / 0.83 | 4693 / — | control |
| slow64k | compress | 1 | main-A (3) | 15.6 / 11.6 / 12.5 | 63 | 2644 / 6000 | 0.66 / 2.75 | 146 / 351 | control |
| slow64k | compress | 1 | main-B (0) | 16.0 / 11.4 / 12.2 | 64 | 2640 / 6040 | 0.71 / 2.71 | 176 / 351 | control |
| slow64k | compress | 1 | candidate (1) | 15.5 / 9.2 / 12.6 | 63 | 2620 / 5992 | 0.69 / 2.72 | 166 / 376 | observed_fps_gap_not_accepted |
| slow64k | bandwidth | 0 | ssh (0) | 0.6 / — / — | 61 | 7800 / 19868 | 0.14 / 0.31 | 5076 / — | control |
| slow64k | bandwidth | 0 | main-A (1) | 1.4 / 1.3 / 1.3 | 64 | 2632 / 6060 | 0.36 / 2.03 | 1014 / — | control |
| slow64k | bandwidth | 0 | main-B (2) | 1.4 / 1.2 / 1.2 | 64 | 2652 / 6040 | 0.38 / 1.9 | 1094 / — | control |
| slow64k | bandwidth | 0 | candidate (3) | 1.4 / 1.3 / 1.3 | 64 | 2644 / 6028 | 0.44 / 2.02 | 1086 / — | missing_metrics_not_accepted |
| slow64k | bandwidth | 1 | ssh (2) | 0.6 / — / — | 61 | 7560 / 19976 | 0.08 / 0.35 | 4981 / — | control |
| slow64k | bandwidth | 1 | main-A (3) | 1.4 / 0.7 / 0.7 | 64 | 1984 / 5308 | 0.38 / 1.69 | 1006 / — | control |
| slow64k | bandwidth | 1 | main-B (0) | 1.4 / 1.3 / 1.3 | 64 | 2184 / 5776 | 0.33 / 1.66 | 1019 / — | control |
| slow64k | bandwidth | 1 | candidate (1) | 1.4 / 1.3 / 1.3 | 63 | 1976 / 5552 | 0.3 / 1.73 | 1021 / — | missing_metrics_not_accepted |
