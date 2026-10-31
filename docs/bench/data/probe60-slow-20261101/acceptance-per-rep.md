# Per-repetition acceptance evidence

Each row retains one measured role in one repetition. No pooling, noise tolerance or automatic pass is applied. Observed deficits do not establish policy causality; see JSON for exact missing metrics, FPS gaps and resource increases.

| Network | Scene | Rep | Role (slot) | FPS avg / 1% / 5% | Network kbit/s | RSS client / server KiB | CPU client / server % | Age median / p95 ms | Assessment |
|---|---|---:|---|---|---:|---|---|---|---|
| lowbw | compress | 0 | ssh (0) | 23.6 / 6.3 / 7.1 | 977 | 7656 / 19676 | 0.18 / 0.54 | 2536 / 4716 | control |
| lowbw | compress | 0 | main-A (1) | 56.0 / 43.0 / 46.3 | 223 | 2644 / 6044 | 0.94 / 3.43 | 64 / 73 | control |
| lowbw | compress | 0 | main-B (2) | 57.1 / 28.9 / 35.2 | 218 | 2636 / 5892 | 0.87 / 1.38 | 60 / 64 | control |
| lowbw | compress | 0 | candidate (3) | 56.1 / 43.3 / 46.2 | 224 | 2656 / 6080 | 0.91 / 3.55 | 65 / 74 | observed_fps_gap_not_accepted |
| lowbw | compress | 1 | ssh (2) | 23.6 / 5.9 / 7.1 | 977 | 7552 / 19768 | 0.18 / 0.47 | 2585 / 4686 | control |
| lowbw | compress | 1 | main-A (3) | 56.1 / 43.3 / 47.3 | 224 | 2640 / 6008 | 0.8 / 3.24 | 64 / 73 | control |
| lowbw | compress | 1 | main-B (0) | 56.4 / 45.0 / 47.3 | 224 | 2640 / 5988 | 0.8 / 3.19 | 64 / 73 | control |
| lowbw | compress | 1 | candidate (1) | 56.2 / 43.3 / 47.2 | 224 | 2652 / 6052 | 0.8 / 3.31 | 65 / 73 | observed_fps_gap_not_accepted |
| lowbw | bandwidth | 0 | ssh (0) | 10.0 / 5.5 / 6.5 | 978 | 7576 / 20068 | 0.12 / 0.31 | 3533 / 6488 | control |
| lowbw | bandwidth | 0 | main-A (1) | 19.6 / 10.2 / 17.4 | 788 | 2648 / 5624 | 0.78 / 2.54 | 96 / 116 | control |
| lowbw | bandwidth | 0 | main-B (2) | 20.0 / 14.2 / 18.1 | 799 | 2660 / 5608 | 0.77 / 2.63 | 96 / 114 | control |
| lowbw | bandwidth | 0 | candidate (3) | 23.0 / 12.8 / 18.9 | 922 | 2636 / 5720 | 0.87 / 2.97 | 115 / 176 | observed_fps_gap_not_accepted |
| lowbw | bandwidth | 1 | ssh (2) | 9.9 / 5.4 / 6.4 | 979 | 7576 / 20156 | 0.15 / 0.36 | 3504 / 6475 | control |
| lowbw | bandwidth | 1 | main-A (3) | 20.2 / 13.9 / 18.0 | 807 | 2636 / 6052 | 0.77 / 2.68 | 97 / 115 | control |
| lowbw | bandwidth | 1 | main-B (0) | 19.9 / 10.6 / 17.9 | 798 | 2640 / 6052 | 0.76 / 2.64 | 97 / 116 | control |
| lowbw | bandwidth | 1 | candidate (1) | 22.6 / 12.7 / 18.9 | 910 | 2648 / 6056 | 0.87 / 2.96 | 100 / 131 | observed_fps_gap_not_accepted |
| verylow | compress | 0 | ssh (0) | 6.0 / 4.2 / 5.1 | 249 | 7540 / 19816 | 0.12 / 0.49 | 3796 / 6996 | control |
| verylow | compress | 0 | main-A (1) | 38.1 / 25.9 / 27.2 | 152 | 2012 / 5084 | 0.84 / 3.2 | 69 / 87 | control |
| verylow | compress | 0 | main-B (2) | 40.9 / 26.6 / 27.8 | 166 | 2012 / 5444 | 0.9 / 3.27 | 70 / 87 | control |
| verylow | compress | 0 | candidate (3) | 41.6 / 26.4 / 28.3 | 169 | 1576 / 5440 | 0.95 / 3.43 | 69 / 88 | observed_fps_gap_not_accepted |
| verylow | compress | 1 | ssh (2) | 6.0 / 4.3 / 5.1 | 249 | 7552 / 19732 | 0.1 / 0.53 | 3775 / 6922 | control |
| verylow | compress | 1 | main-A (3) | 40.1 / 26.2 / 27.6 | 161 | 2636 / 6032 | 0.96 / 3.42 | 69 / 86 | control |
| verylow | compress | 1 | main-B (0) | 36.5 / 24.3 / 27.3 | 144 | 2640 / 6052 | 0.85 / 3.07 | 69 / 88 | control |
| verylow | compress | 1 | candidate (1) | 40.9 / 26.6 / 28.1 | 165 | 2652 / 6028 | 1.0 / 3.55 | 69 / 87 | review_required_no_automatic_pass |
| verylow | bandwidth | 0 | ssh (0) | 2.5 / 1.5 / 1.5 | 248 | 7580 / 19892 | 0.15 / 0.35 | 4024 / 7715 | control |
| verylow | bandwidth | 0 | main-A (1) | 5.9 / 5.4 / 5.5 | 256 | 2632 / 6092 | 0.8 / 2.43 | 385 / 414 | control |
| verylow | bandwidth | 0 | main-B (2) | 5.9 / 5.3 / 5.5 | 256 | 2652 / 6092 | 0.81 / 2.42 | 388 / 406 | control |
| verylow | bandwidth | 0 | candidate (3) | 5.9 / 5.3 / 5.5 | 255 | 2632 / 6036 | 0.83 / 2.51 | 384 / 413 | observed_fps_gap_not_accepted |
| verylow | bandwidth | 1 | ssh (2) | 2.4 / 1.3 / 1.3 | 248 | 7568 / 19932 | 0.11 / 0.38 | 3845 / — | control |
| verylow | bandwidth | 1 | main-A (3) | 5.9 / 5.4 / 5.5 | 256 | 2628 / 6056 | 0.78 / 2.37 | 385 / 411 | control |
| verylow | bandwidth | 1 | main-B (0) | 5.9 / 5.4 / 5.5 | 255 | 2628 / 6048 | 0.75 / 2.45 | 382 / 406 | control |
| verylow | bandwidth | 1 | candidate (1) | 5.9 / 5.4 / 5.5 | 256 | 2628 / 6040 | 0.77 / 2.5 | 382 / 417 | review_required_no_automatic_pass |
| slow64k | compress | 0 | ssh (0) | 1.4 / 1.0 / 1.0 | 62 | 7552 / 19832 | 0.12 / 0.79 | 4417 / — | control |
| slow64k | compress | 0 | main-A (1) | 16.0 / 8.5 / 12.4 | 64 | 2672 / 6000 | 0.62 / 2.53 | 166 / 381 | control |
| slow64k | compress | 0 | main-B (2) | 15.6 / 8.8 / 12.5 | 64 | 2660 / 6048 | 0.61 / 2.56 | 173 / 387 | control |
| slow64k | compress | 0 | candidate (3) | 15.9 / 11.3 / 12.1 | 63 | 2652 / 5996 | 0.64 / 2.65 | 188 / 379 | observed_fps_gap_not_accepted |
| slow64k | compress | 1 | ssh (2) | 1.4 / 1.1 / 1.1 | 62 | 7556 / 20276 | 0.12 / 0.8 | 4194 / — | control |
| slow64k | compress | 1 | main-A (3) | 16.0 / 11.4 / 12.9 | 64 | 2644 / 6000 | 0.71 / 2.78 | 171 / 426 | control |
| slow64k | compress | 1 | main-B (0) | 15.9 / 11.6 / 12.9 | 63 | 2644 / 6004 | 0.71 / 2.75 | 174 / 371 | control |
| slow64k | compress | 1 | candidate (1) | 15.6 / 11.2 / 12.3 | 63 | 2644 / 6044 | 0.72 / 2.98 | 180 / 347 | observed_fps_gap_not_accepted |
| slow64k | bandwidth | 0 | ssh (0) | 0.6 / — / — | 61 | 7808 / 19964 | 0.13 / 0.28 | 4869 / — | control |
| slow64k | bandwidth | 0 | main-A (1) | 1.6 / 1.4 / 1.4 | 63 | 2640 / 6072 | 0.25 / 1.97 | 2040 / — | control |
| slow64k | bandwidth | 0 | main-B (2) | 1.6 / 1.4 / 1.4 | 63 | 2640 / 6072 | 0.28 / 2.11 | 1592 / — | control |
| slow64k | bandwidth | 0 | candidate (3) | 1.6 / 1.4 / 1.4 | 63 | 2624 / 6068 | 0.25 / 2.13 | 2035 / — | missing_metrics_not_accepted |
| slow64k | bandwidth | 1 | ssh (2) | 0.6 / — / — | 63 | 7564 / 19984 | 0.13 / 0.38 | 4802 / — | control |
| slow64k | bandwidth | 1 | main-A (3) | 1.5 / 1.4 / 1.4 | 63 | 2640 / 6084 | 0.24 / 1.78 | 1726 / — | control |
| slow64k | bandwidth | 1 | main-B (0) | 1.4 / 1.3 / 1.3 | 64 | 2648 / 6064 | 0.32 / 1.81 | 1041 / — | control |
| slow64k | bandwidth | 1 | candidate (1) | 1.4 / 1.3 / 1.3 | 63 | 2616 / 6076 | 0.32 / 1.8 | 1014 / — | missing_metrics_not_accepted |
