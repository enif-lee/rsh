# Per-repetition acceptance evidence

Each row retains one measured role in one repetition. No pooling, noise tolerance or automatic pass is applied. Observed deficits do not establish policy causality; see JSON for exact missing metrics, FPS gaps and resource increases.

| Network | Scene | Rep | Role (slot) | FPS avg / 1% / 5% | Network kbit/s | RSS client / server KiB | CPU client / server % | Age median / p95 ms | Assessment |
|---|---|---:|---|---|---:|---|---|---|---|
| lowbw | compress | 0 | ssh (0) | 23.6 / 6.5 / 7.0 | 980 | 7576 / 19724 | 0.2 / 0.54 | 2584 / 4715 | control |
| lowbw | compress | 0 | main-A (1) | 56.2 / 41.1 / 46.4 | 224 | 2640 / 5996 | 0.97 / 3.47 | 64 / 73 | control |
| lowbw | compress | 0 | main-B (2) | 53.7 / 29.4 / 33.4 | 214 | 2644 / 6056 | 0.93 / 3.35 | 64 / 74 | control |
| lowbw | compress | 0 | candidate (3) | 56.2 / 41.2 / 46.5 | 224 | 2636 / 6028 | 0.98 / 3.45 | 64 / 73 | review_required_no_automatic_pass |
| lowbw | compress | 1 | ssh (2) | 23.7 / 6.5 / 7.0 | 979 | 7276 / 19636 | 0.17 / 0.49 | 2539 / 4714 | control |
| lowbw | compress | 1 | main-A (3) | 56.2 / 44.2 / 47.2 | 224 | 2644 / 6036 | 0.87 / 3.36 | 64 / 73 | control |
| lowbw | compress | 1 | main-B (0) | 56.2 / 44.3 / 47.4 | 223 | 2648 / 6032 | 0.87 / 3.35 | 63 / 73 | control |
| lowbw | compress | 1 | candidate (1) | 52.4 / 28.5 / 30.5 | 203 | 2636 / 5872 | 0.75 / 1.74 | 59 / 68 | observed_fps_gap_not_accepted |
| lowbw | bandwidth | 0 | ssh (0) | 10.0 / 5.9 / 6.7 | 977 | 7712 / 20140 | 0.14 / 0.33 | 3543 / 6396 | control |
| lowbw | bandwidth | 0 | main-A (1) | 20.1 / 9.3 / 15.2 | 817 | 2636 / 6052 | 0.77 / 2.7 | 97 / 116 | control |
| lowbw | bandwidth | 0 | main-B (2) | 21.4 / 12.6 / 18.3 | 856 | 2644 / 6108 | 0.81 / 2.7 | 98 / 142 | control |
| lowbw | bandwidth | 0 | candidate (3) | 21.2 / 14.0 / 18.3 | 847 | 2624 / 6068 | 0.81 / 2.71 | 97 / 117 | observed_fps_gap_not_accepted |
| lowbw | bandwidth | 1 | ssh (2) | 10.0 / 4.9 / 6.2 | 977 | 7584 / 19952 | 0.15 / 0.31 | 3516 / 6429 | control |
| lowbw | bandwidth | 1 | main-A (3) | 20.0 / 11.3 / 18.1 | 802 | 2660 / 6052 | 0.87 / 2.95 | 97 / 117 | control |
| lowbw | bandwidth | 1 | main-B (0) | 19.9 / 12.5 / 17.7 | 796 | 2660 / 6056 | 0.88 / 2.85 | 97 / 116 | control |
| lowbw | bandwidth | 1 | candidate (1) | 20.4 / 10.0 / 17.7 | 818 | 2640 / 6064 | 0.93 / 2.86 | 97 / 118 | observed_fps_gap_not_accepted |
| verylow | compress | 0 | ssh (0) | 6.0 / 5.1 / 5.2 | 248 | 7540 / 19784 | 0.11 / 0.56 | 3790 / 6984 | control |
| verylow | compress | 0 | main-A (1) | 40.7 / 26.3 / 27.6 | 164 | 2392 / 5568 | 1.1 / 3.84 | 69 / 88 | control |
| verylow | compress | 0 | main-B (2) | 33.9 / 25.3 / 26.8 | 130 | 2396 / 5544 | 0.91 / 3.3 | 69 / 87 | control |
| verylow | compress | 0 | candidate (3) | 36.2 / 25.8 / 27.3 | 143 | 2384 / 5632 | 0.98 / 3.41 | 70 / 87 | observed_fps_gap_not_accepted |
| verylow | compress | 1 | ssh (2) | 6.1 / 5.1 / 5.1 | 250 | 7556 / 19800 | 0.13 / 0.52 | 3855 / 7137 | control |
| verylow | compress | 1 | main-A (3) | 40.9 / 26.6 / 28.0 | 165 | 2640 / 6040 | 1.13 / 3.88 | 70 / 87 | control |
| verylow | compress | 1 | main-B (0) | 40.4 / 25.3 / 27.3 | 164 | 2644 / 6052 | 1.23 / 3.88 | 69 / 88 | control |
| verylow | compress | 1 | candidate (1) | 38.7 / 24.7 / 26.8 | 154 | 2636 / 6016 | 1.1 / 3.86 | 69 / 89 | observed_fps_gap_not_accepted |
| verylow | bandwidth | 0 | ssh (0) | 2.4 / 1.8 / 1.8 | 248 | 7540 / 19940 | 0.12 / 0.37 | 3836 / — | control |
| verylow | bandwidth | 0 | main-A (1) | 5.9 / 5.3 / 5.5 | 255 | 2668 / 6052 | 0.82 / 2.48 | 386 / 406 | control |
| verylow | bandwidth | 0 | main-B (2) | 5.7 / 2.9 / 5.4 | 256 | 2628 / 6048 | 0.81 / 2.46 | 377 / 407 | control |
| verylow | bandwidth | 0 | candidate (3) | 5.9 / 5.4 / 5.5 | 256 | 2632 / 6068 | 0.8 / 2.53 | 386 / 405 | review_required_no_automatic_pass |
| verylow | bandwidth | 1 | ssh (2) | 2.4 / 1.2 / 1.2 | 250 | 7576 / 19928 | 0.11 / 0.37 | 3899 / — | control |
| verylow | bandwidth | 1 | main-A (3) | 5.9 / 5.3 / 5.5 | 256 | 2640 / 6068 | 0.71 / 2.24 | 382 / 409 | control |
| verylow | bandwidth | 1 | main-B (0) | 6.0 / 5.5 / 5.6 | 256 | 2620 / 6096 | 0.67 / 2.16 | 384 / 405 | control |
| verylow | bandwidth | 1 | candidate (1) | 5.9 / 5.1 / 5.4 | 256 | 2628 / 6104 | 0.71 / 2.26 | 392 / 420 | observed_fps_gap_not_accepted |
| slow64k | compress | 0 | ssh (0) | 1.5 / 1.2 / 1.2 | 62 | 7536 / 19816 | 0.05 / 0.65 | 4442 / — | control |
| slow64k | compress | 0 | main-A (1) | 15.5 / 11.6 / 12.9 | 63 | 2648 / 6020 | 0.58 / 2.27 | 160 / 379 | control |
| slow64k | compress | 0 | main-B (2) | 16.2 / 11.6 / 13.3 | 64 | 2640 / 5992 | 0.58 / 2.36 | 168 / 419 | control |
| slow64k | compress | 0 | candidate (3) | 16.0 / 11.4 / 12.7 | 64 | 2644 / 6000 | 0.66 / 2.44 | 170 / 353 | observed_fps_gap_not_accepted |
| slow64k | compress | 1 | ssh (2) | 1.4 / 1.0 / 1.0 | 62 | 7552 / 19748 | 0.09 / 0.6 | 4194 / — | control |
| slow64k | compress | 1 | main-A (3) | 16.0 / 11.1 / 12.9 | 63 | 2636 / 5984 | 0.55 / 2.25 | 156 / 375 | control |
| slow64k | compress | 1 | main-B (0) | 16.1 / 11.0 / 12.4 | 64 | 2648 / 6012 | 0.56 / 2.19 | 189 / 376 | control |
| slow64k | compress | 1 | candidate (1) | 15.6 / 11.1 / 12.3 | 63 | 2636 / 6008 | 0.55 / 2.18 | 178 / 375 | observed_fps_gap_not_accepted |
| slow64k | bandwidth | 0 | ssh (0) | 0.6 / — / — | 60 | 7544 / 20216 | 0.07 / 0.25 | 4879 / — | control |
| slow64k | bandwidth | 0 | main-A (1) | 1.2 / — / — | 63 | 2624 / 6088 | 0.23 / 1.37 | 1011 / — | control |
| slow64k | bandwidth | 0 | main-B (2) | 1.2 / — / — | 63 | 2604 / 6056 | 0.24 / 1.36 | 1002 / — | control |
| slow64k | bandwidth | 0 | candidate (3) | 1.5 / 0.8 / 0.8 | 63 | 2616 / 6056 | 0.17 / 1.5 | 2187 / — | missing_metrics_not_accepted |
| slow64k | bandwidth | 1 | ssh (2) | 0.6 / — / — | 60 | 7564 / 19912 | 0.04 / 0.25 | 4811 / — | control |
| slow64k | bandwidth | 1 | main-A (3) | 1.6 / 1.4 / 1.4 | 63 | 2652 / 6084 | 0.19 / 1.46 | 1575 / — | control |
| slow64k | bandwidth | 1 | main-B (0) | 1.6 / 1.3 / 1.3 | 64 | 2636 / 6076 | 0.19 / 1.54 | 1597 / — | control |
| slow64k | bandwidth | 1 | candidate (1) | 1.4 / 1.3 / 1.3 | 64 | 2608 / 6024 | 0.23 / 1.48 | 1068 / — | missing_metrics_not_accepted |
