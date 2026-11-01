# Per-repetition acceptance evidence

Each row retains one measured role in one repetition. No pooling, noise tolerance or automatic pass is applied. Observed deficits do not establish policy causality; see JSON for exact missing metrics, FPS gaps and resource increases.

| Network | Scene | Rep | Role (slot) | FPS avg / 1% / 5% | Network kbit/s | RSS client / server KiB | CPU client / server % | Age median / p95 ms | Assessment |
|---|---|---:|---|---|---:|---|---|---|---|
| lowbw | compress | 0 | ssh (0) | 23.5 / 6.7 / 7.0 | 979 | 7816 / 19692 | 0.18 / 0.56 | 2567 / 4693 | control |
| lowbw | compress | 0 | main-A (1) | 56.0 / 41.3 / 46.4 | 224 | 2640 / 6008 | 0.93 / 3.55 | 64 / 73 | control |
| lowbw | compress | 0 | main-B (2) | 57.4 / 28.2 / 40.3 | 219 | 2644 / 5884 | 0.85 / 1.4 | 60 / 65 | control |
| lowbw | compress | 0 | candidate (3) | 60.0 / 41.6 / 47.8 | 227 | 2628 / 5812 | 0.9 / 1.21 | 60 / 64 | review_required_no_automatic_pass |
| lowbw | compress | 1 | ssh (2) | 23.5 / 6.0 / 7.1 | 978 | 7556 / 19756 | 0.17 / 0.52 | 2568 / 4699 | control |
| lowbw | compress | 1 | main-A (3) | 55.4 / 40.4 / 45.7 | 221 | 2644 / 5992 | 0.89 / 3.38 | 64 / 74 | control |
| lowbw | compress | 1 | main-B (0) | 55.9 / 38.0 / 46.4 | 222 | 2640 / 6008 | 0.89 / 3.44 | 64 / 73 | control |
| lowbw | compress | 1 | candidate (1) | 60.0 / 41.3 / 46.5 | 228 | 2636 / 5848 | 0.88 / 1.2 | 59 / 63 | review_required_no_automatic_pass |
| lowbw | bandwidth | 0 | ssh (0) | 10.0 / 5.2 / 6.4 | 978 | 7568 / 20144 | 0.25 / 0.36 | 3512 / 6488 | control |
| lowbw | bandwidth | 0 | main-A (1) | 20.2 / 10.3 / 17.3 | 820 | 2648 / 6096 | 0.91 / 2.94 | 98 / 117 | control |
| lowbw | bandwidth | 0 | main-B (2) | 20.6 / 11.9 / 18.0 | 834 | 2680 / 6080 | 0.91 / 2.87 | 98 / 120 | control |
| lowbw | bandwidth | 0 | candidate (3) | 21.7 / 10.1 / 18.4 | 872 | 2620 / 6076 | 0.98 / 3.07 | 101 / 158 | observed_fps_gap_not_accepted |
| lowbw | bandwidth | 1 | ssh (2) | 10.0 / 6.2 / 6.5 | 978 | 7588 / 21044 | 0.15 / 0.31 | 3487 / 6418 | control |
| lowbw | bandwidth | 1 | main-A (3) | 21.3 / 13.9 / 18.1 | 855 | 2644 / 6072 | 0.85 / 2.87 | 101 / 126 | control |
| lowbw | bandwidth | 1 | main-B (0) | 19.9 / 9.9 / 17.5 | 808 | 2624 / 6096 | 0.78 / 2.88 | 98 / 118 | control |
| lowbw | bandwidth | 1 | candidate (1) | 21.7 / 10.3 / 17.9 | 885 | 2640 / 6056 | 0.9 / 3.03 | 101 / 120 | observed_fps_gap_not_accepted |
| verylow | compress | 0 | ssh (0) | 6.1 / 3.5 / 5.2 | 250 | 7556 / 19620 | 0.13 / 0.56 | 3818 / 7148 | control |
| verylow | compress | 0 | main-A (1) | 40.1 / 24.9 / 27.3 | 161 | 2636 / 6012 | 1.04 / 3.59 | 70 / 87 | control |
| verylow | compress | 0 | main-B (2) | 33.6 / 24.7 / 26.4 | 129 | 2644 / 6016 | 0.87 / 3.06 | 69 / 88 | control |
| verylow | compress | 0 | candidate (3) | 59.8 / 38.5 / 45.7 | 229 | 2648 / 5788 | 1.35 / 1.9 | 72 / 79 | review_required_no_automatic_pass |
| verylow | compress | 1 | ssh (2) | 6.0 / 4.2 / 5.1 | 249 | 7564 / 19308 | 0.1 / 0.5 | 3821 / 6919 | control |
| verylow | compress | 1 | main-A (3) | 41.1 / 26.3 / 27.3 | 165 | 2720 / 5988 | 0.96 / 3.35 | 70 / 88 | control |
| verylow | compress | 1 | main-B (0) | 34.1 / 25.3 / 26.8 | 131 | 2716 / 6024 | 0.77 / 2.74 | 69 / 88 | control |
| verylow | compress | 1 | candidate (1) | 60.0 / 41.3 / 46.7 | 229 | 2704 / 5824 | 1.21 / 1.76 | 72 / 76 | review_required_no_automatic_pass |
| verylow | bandwidth | 0 | ssh (0) | 2.5 / 1.5 / 1.5 | 250 | 7040 / 18952 | 0.14 / 0.45 | 4243 / 7700 | control |
| verylow | bandwidth | 0 | main-A (1) | 5.9 / 5.4 / 5.5 | 255 | 2680 / 6048 | 0.85 / 2.42 | 380 / 415 | control |
| verylow | bandwidth | 0 | main-B (2) | 6.0 / 5.3 / 5.5 | 255 | 2692 / 6040 | 0.83 / 2.43 | 380 / 407 | control |
| verylow | bandwidth | 0 | candidate (3) | 5.9 / 5.4 / 5.4 | 256 | 2708 / 6048 | 0.88 / 2.4 | 387 / 418 | observed_fps_gap_not_accepted |
| verylow | bandwidth | 1 | ssh (2) | 2.4 / 1.2 / 1.2 | 250 | 7796 / 19512 | 0.08 / 0.31 | 3854 / — | control |
| verylow | bandwidth | 1 | main-A (3) | 6.0 / 5.3 / 5.5 | 255 | 2644 / 6064 | 0.75 / 2.51 | 383 / 407 | control |
| verylow | bandwidth | 1 | main-B (0) | 5.9 / 5.5 / 5.5 | 256 | 2620 / 6100 | 0.77 / 2.54 | 385 / 411 | control |
| verylow | bandwidth | 1 | candidate (1) | 5.9 / 5.4 / 5.5 | 255 | 2628 / 6072 | 0.81 / 2.49 | 384 / 411 | observed_fps_gap_not_accepted |
| slow64k | compress | 0 | ssh (0) | 1.5 / 1.1 / 1.1 | 63 | 7556 / 19396 | 0.04 / 0.72 | 4581 / — | control |
| slow64k | compress | 0 | main-A (1) | 15.9 / 11.4 / 12.7 | 64 | 2632 / 6024 | 0.62 / 2.5 | 172 / 354 | control |
| slow64k | compress | 0 | main-B (2) | 15.7 / 11.3 / 13.1 | 62 | 2644 / 6016 | 0.63 / 2.39 | 122 / 355 | control |
| slow64k | compress | 0 | candidate (3) | 1.5 / 15.5 / 15.5 | 64 | 2652 / 5980 | 0.61 / 2.37 | 388 / — | observed_fps_gap_not_accepted |
| slow64k | compress | 1 | ssh (2) | 1.5 / 1.2 / 1.2 | 61 | 7560 / 19412 | 0.05 / 0.75 | 4495 / — | control |
| slow64k | compress | 1 | main-A (3) | 15.9 / 11.6 / 13.0 | 63 | 2644 / 6044 | 0.54 / 2.37 | 148 / 375 | control |
| slow64k | compress | 1 | main-B (0) | 16.0 / 9.2 / 12.7 | 64 | 2660 / 6040 | 0.56 / 2.47 | 153 / 381 | control |
| slow64k | compress | 1 | candidate (1) | 15.2 / 6.1 / 12.1 | 62 | 2664 / 5992 | 0.53 / 2.42 | 147 / 368 | observed_fps_gap_not_accepted |
| slow64k | bandwidth | 0 | ssh (0) | 0.6 / — / — | 60 | 7552 / 19340 | 0.11 / 0.26 | 4817 / — | control |
| slow64k | bandwidth | 0 | main-A (1) | 1.6 / 1.4 / 1.4 | 63 | 2648 / 6088 | 0.2 / 1.91 | 2038 / — | control |
| slow64k | bandwidth | 0 | main-B (2) | 1.5 / 1.4 / 1.4 | 63 | 2604 / 6056 | 0.22 / 2.03 | 1732 / — | control |
| slow64k | bandwidth | 0 | candidate (3) | 1.5 / 1.4 / 1.4 | 63 | 2652 / 6068 | 0.23 / 1.91 | 1730 / — | missing_metrics_not_accepted |
| slow64k | bandwidth | 1 | ssh (2) | 0.6 / — / — | 62 | 7540 / 19448 | 0.17 / 0.21 | 4871 / — | control |
| slow64k | bandwidth | 1 | main-A (3) | 1.4 / 1.3 / 1.3 | 64 | 2652 / 6040 | 0.3 / 1.72 | 1047 / — | control |
| slow64k | bandwidth | 1 | main-B (0) | 1.4 / 1.3 / 1.3 | 63 | 2624 / 6060 | 0.3 / 1.65 | 1057 / — | control |
| slow64k | bandwidth | 1 | candidate (1) | 1.5 / 1.4 / 1.4 | 63 | 2604 / 6060 | 0.2 / 1.76 | 1730 / — | missing_metrics_not_accepted |
