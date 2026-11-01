# Focused per-repetition comparison evidence

Each row retains one measured role in one repetition. No pooling, noise tolerance or automatic pass is applied. Observed deficits do not establish policy causality; see JSON for exact missing metrics, FPS gaps and resource increases.

| Network | Scene | Rep | Role (slot) | FPS 1% / 5% / avg | Network kbit/s | RSS client / server KiB | CPU client / server % | Age median / p95 ms | Assessment |
|---|---|---:|---|---|---:|---|---|---|---|
| lowbw | compress | 0 | ssh (0) | 6.6 / 7.2 / 23.5 | 977 | 7540 / 19692 | 0.18 / 0.54 | 2573 / 4673 | control |
| lowbw | compress | 0 | main-A (1) | 42.1 / 45.1 / 55.5 | 221 | 2652 / 5988 | 0.88 / 3.28 | 65 / 74 | control |
| lowbw | compress | 0 | main-B (2) | 41.2 / 45.8 / 55.6 | 222 | 2656 / 6040 | 0.87 / 3.24 | 65 / 74 | control |
| lowbw | compress | 0 | candidate (3) | 42.8 / 46.8 / 60.0 | 227 | 2628 / 5844 | 0.87 / 1.14 | 59 / 62 | qualitative_review_required |
| lowbw | compress | 1 | ssh (2) | 6.3 / 7.1 / 23.7 | 979 | 7800 / 19704 | 0.18 / 0.53 | 2553 / 4726 | control |
| lowbw | compress | 1 | main-A (3) | 40.8 / 44.8 / 55.7 | 222 | 2644 / 6016 | 0.9 / 3.45 | 64 / 74 | control |
| lowbw | compress | 1 | main-B (0) | 42.1 / 45.8 / 55.9 | 222 | 2640 / 6004 | 0.9 / 3.44 | 64 / 74 | control |
| lowbw | compress | 1 | candidate (1) | 40.7 / 46.7 / 60.0 | 227 | 2628 / 5804 | 0.89 / 1.23 | 59 / 63 | fps_tradeoff_review_required |
| lowbw | bandwidth | 0 | ssh (0) | 5.5 / 6.3 / 9.9 | 980 | 7564 / 21200 | 0.15 / 0.33 | 3501 / 6459 | control |
| lowbw | bandwidth | 0 | main-A (1) | 15.8 / 17.9 / 20.4 | 814 | 2644 / 6064 | 0.84 / 2.76 | 97 / 132 | control |
| lowbw | bandwidth | 0 | main-B (2) | 9.5 / 17.9 / 20.0 | 801 | 2640 / 6048 | 0.81 / 2.65 | 97 / 116 | control |
| lowbw | bandwidth | 0 | candidate (3) | 13.1 / 17.8 / 21.2 | 852 | 2652 / 6072 | 0.87 / 2.91 | 98 / 141 | fps_tradeoff_review_required |
| lowbw | bandwidth | 1 | ssh (2) | 5.4 / 6.8 / 9.9 | 978 | 7332 / 21000 | 0.14 / 0.3 | 3441 / 6423 | control |
| lowbw | bandwidth | 1 | main-A (3) | 12.8 / 19.1 / 23.6 | 939 | 1668 / 4944 | 0.89 / 2.86 | 109 / 134 | control |
| lowbw | bandwidth | 1 | main-B (0) | 10.0 / 18.4 / 22.0 | 886 | 1924 / 5196 | 0.82 / 2.71 | 105 / 138 | control |
| lowbw | bandwidth | 1 | candidate (1) | 12.2 / 18.6 / 23.7 | 953 | 1828 / 5356 | 0.89 / 2.92 | 128 / 178 | fps_tradeoff_review_required |
| verylow | compress | 0 | ssh (0) | 1.5 / 2.3 / 5.9 | 250 | 7552 / 19820 | 0.09 / 0.53 | 3703 / 7176 | control |
| verylow | compress | 0 | main-A (1) | 26.0 / 27.4 / 37.1 | 147 | 2652 / 6012 | 0.87 / 3.15 | 70 / 89 | control |
| verylow | compress | 0 | main-B (2) | 26.7 / 27.6 / 40.5 | 162 | 2644 / 6016 | 0.96 / 3.36 | 70 / 88 | control |
| verylow | compress | 0 | candidate (3) | 41.2 / 46.2 / 60.0 | 229 | 2640 / 5824 | 1.26 / 1.79 | 72 / 76 | qualitative_review_required |
| verylow | compress | 1 | ssh (2) | 4.1 / 5.0 / 6.1 | 250 | 7536 / 19832 | 0.12 / 0.52 | 3833 / 7122 | control |
| verylow | compress | 1 | main-A (3) | 25.5 / 27.9 / 43.0 | 171 | 1900 / 5148 | 0.97 / 2.95 | 71 / 86 | control |
| verylow | compress | 1 | main-B (0) | 25.9 / 28.1 / 42.1 | 171 | 1624 / 5232 | 0.97 / 3.21 | 70 / 88 | control |
| verylow | compress | 1 | candidate (1) | 42.6 / 45.4 / 60.0 | 229 | 1988 / 5044 | 1.26 / 1.86 | 72 / 76 | qualitative_review_required |
| verylow | bandwidth | 0 | ssh (0) | 1.3 / 1.3 / 2.4 | 247 | 7576 / 19904 | 0.08 / 0.31 | 3915 / — | control |
| verylow | bandwidth | 0 | main-A (1) | 5.4 / 5.6 / 6.0 | 256 | 2632 / 6044 | 0.57 / 2.39 | 379 / 417 | control |
| verylow | bandwidth | 0 | main-B (2) | 5.3 / 5.4 / 6.0 | 256 | 2624 / 6056 | 0.57 / 2.23 | 380 / 407 | control |
| verylow | bandwidth | 0 | candidate (3) | 5.5 / 5.5 / 6.0 | 256 | 2628 / 6084 | 0.57 / 2.37 | 380 / 411 | fps_tradeoff_review_required |
| verylow | bandwidth | 1 | ssh (2) | 1.3 / 1.3 / 2.5 | 250 | 7560 / 19892 | 0.07 / 0.3 | 4092 / 7324 | control |
| verylow | bandwidth | 1 | main-A (3) | 5.4 / 5.5 / 6.0 | 256 | 2640 / 6060 | 0.57 / 2.14 | 385 / 407 | control |
| verylow | bandwidth | 1 | main-B (0) | 4.3 / 5.6 / 5.9 | 256 | 2624 / 6044 | 0.55 / 2.08 | 383 / 426 | control |
| verylow | bandwidth | 1 | candidate (1) | 5.4 / 5.5 / 5.9 | 256 | 2628 / 6060 | 0.56 / 2.11 | 384 / 401 | fps_tradeoff_review_required |
| slow64k | compress | 0 | ssh (0) | 1.3 / 1.3 / 1.5 | 61 | 7560 / 19724 | 0.07 / 0.56 | 4523 / — | control |
| slow64k | compress | 0 | main-A (1) | 11.1 / 12.6 / 15.5 | 63 | 2320 / 5592 | 0.57 / 2.43 | 166 / 380 | control |
| slow64k | compress | 0 | main-B (2) | 9.2 / 12.6 / 15.5 | 63 | 2328 / 5552 | 0.56 / 2.41 | 172 / 370 | control |
| slow64k | compress | 0 | candidate (3) | 8.6 / 13.0 / 16.1 | 63 | 2320 / 5560 | 0.59 / 2.54 | 184 / 378 | fps_tradeoff_review_required |
| slow64k | compress | 1 | ssh (2) | 1.2 / 1.2 / 1.5 | 61 | 7568 / 19712 | 0.05 / 0.47 | 4508 / — | control |
| slow64k | compress | 1 | main-A (3) | 11.0 / 12.3 / 15.7 | 63 | 2636 / 6008 | 0.46 / 1.75 | 163 / 375 | control |
| slow64k | compress | 1 | main-B (0) | 11.6 / 12.8 / 15.6 | 63 | 2632 / 6040 | 0.45 / 1.77 | 169 / 373 | control |
| slow64k | compress | 1 | candidate (1) | 5.8 / 12.2 / 15.5 | 63 | 2660 / 6040 | 0.43 / 1.75 | 180 / 352 | fps_tradeoff_review_required |
| slow64k | bandwidth | 0 | ssh (0) | — / — / 0.6 | 62 | 7564 / 19900 | 0.07 / 0.33 | 4844 / — | control |
| slow64k | bandwidth | 0 | main-A (1) | 1.4 / 1.4 / 1.6 | 63 | 2648 / 6064 | 0.21 / 1.83 | 2041 / — | control |
| slow64k | bandwidth | 0 | main-B (2) | 1.3 / 1.3 / 1.5 | 64 | 2652 / 6024 | 0.3 / 1.78 | 1008 / — | control |
| slow64k | bandwidth | 0 | candidate (3) | 1.3 / 1.3 / 1.4 | 64 | 2652 / 6064 | 0.3 / 1.65 | 1045 / — | partial_metrics_review_required |
| slow64k | bandwidth | 1 | ssh (2) | — / — / 0.5 | 60 | 7548 / 19844 | 0.04 / 0.28 | 3973 / — | control |
| slow64k | bandwidth | 1 | main-A (3) | 0.8 / 0.8 / 1.4 | 63 | 2632 / 6104 | 0.23 / 1.67 | 1590 / — | control |
| slow64k | bandwidth | 1 | main-B (0) | 1.4 / 1.4 / 1.6 | 63 | 2628 / 6088 | 0.2 / 1.57 | 2048 / — | control |
| slow64k | bandwidth | 1 | candidate (1) | 1.4 / 1.4 / 1.6 | 63 | 2624 / 6108 | 0.2 / 1.57 | 1684 / — | partial_metrics_review_required |
