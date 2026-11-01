# Focused per-repetition comparison evidence

Each row retains one measured role in one repetition. No pooling, noise tolerance or automatic pass is applied. Observed deficits do not establish policy causality; see JSON for exact missing metrics, FPS gaps and resource increases.

| Network | Scene | Rep | Role (slot) | FPS 1% / 5% / avg | Network kbit/s | RSS client / server KiB | CPU client / server % | Age median / p95 ms | Assessment |
|---|---|---:|---|---|---:|---|---|---|---|
| lan | compress | 0 | ssh (0) | 45.2 / 50.1 / 60.0 | 2476 | 7536 / 18808 | 0.22 / 0.28 | 1 / 3 | control |
| lan | compress | 0 | main-A (1) | 45.2 / 48.6 / 60.0 | 227 | 2624 / 5656 | 0.78 / 1.09 | 3 / 6 | control |
| lan | compress | 0 | main-B (2) | 45.0 / 47.8 / 60.0 | 228 | 2628 / 5688 | 0.78 / 1.1 | 3 / 6 | control |
| lan | compress | 0 | candidate (3) | 43.5 / 47.3 / 60.0 | 228 | 2636 / 5652 | 0.83 / 1.13 | 3 / 6 | fps_tradeoff_review_required |
| lan | compress | 1 | ssh (3) | 45.2 / 49.6 / 60.0 | 2483 | 7552 / 18828 | 0.24 / 0.29 | 1 / 3 | control |
| lan | compress | 1 | main-A (2) | 45.2 / 47.3 / 60.0 | 228 | 2024 / 4664 | 0.76 / 1.08 | 4 / 6 | control |
| lan | compress | 1 | main-B (1) | 43.5 / 48.1 / 60.0 | 227 | 1740 / 4888 | 0.77 / 1.03 | 3 / 6 | control |
| lan | compress | 1 | candidate (0) | 43.7 / 48.1 / 60.0 | 227 | 1724 / 4876 | 0.78 / 1.08 | 3 / 6 | fps_tradeoff_review_required |
| lan | bandwidth | 0 | ssh (0) | 46.5 / 48.8 / 59.9 | 5781 | 7560 / 18820 | 0.19 / 0.25 | 2 / 4 | control |
| lan | bandwidth | 0 | main-A (1) | 40.5 / 45.0 / 60.0 | 2718 | 2652 / 5736 | 1.15 / 1.55 | 5 / 9 | control |
| lan | bandwidth | 0 | main-B (2) | 42.4 / 46.9 / 60.0 | 2723 | 2656 / 5692 | 1.19 / 1.55 | 5 / 9 | control |
| lan | bandwidth | 0 | candidate (3) | 42.4 / 46.3 / 60.0 | 2718 | 2656 / 5736 | 1.19 / 1.56 | 5 / 9 | fps_tradeoff_review_required |
| lan | bandwidth | 1 | ssh (3) | 46.0 / 49.1 / 60.0 | 5789 | 7596 / 18832 | 0.18 / 0.22 | 2 / 4 | control |
| lan | bandwidth | 1 | main-A (2) | 41.7 / 46.6 / 60.0 | 2717 | 2652 / 5716 | 1.11 / 1.44 | 5 / 9 | control |
| lan | bandwidth | 1 | main-B (1) | 41.9 / 45.4 / 60.0 | 2716 | 2648 / 5756 | 1.07 / 1.41 | 4 / 8 | control |
| lan | bandwidth | 1 | candidate (0) | 43.3 / 46.1 / 59.9 | 2715 | 2652 / 5712 | 1.11 / 1.44 | 5 / 8 | fps_tradeoff_review_required |
| wan | compress | 0 | ssh (0) | 43.2 / 48.6 / 60.0 | 2511 | 7568 / 18824 | 0.22 / 0.26 | 52 / 55 | control |
| wan | compress | 0 | main-A (1) | 43.0 / 46.9 / 56.0 | 224 | 2036 / 5464 | 0.82 / 3.13 | 60 / 69 | control |
| wan | compress | 0 | main-B (2) | 44.0 / 46.8 / 56.0 | 223 | 2268 / 5052 | 0.8 / 3.18 | 60 / 69 | control |
| wan | compress | 0 | candidate (3) | 43.4 / 46.0 / 60.0 | 228 | 1500 / 4872 | 0.8 / 1.08 | 55 / 58 | fps_tradeoff_review_required |
| wan | compress | 1 | ssh (3) | 43.8 / 48.8 / 60.0 | 2515 | 7548 / 18832 | 0.23 / 0.25 | 52 / 55 | control |
| wan | compress | 1 | main-A (2) | 42.1 / 46.4 / 55.6 | 221 | 2640 / 5912 | 0.8 / 3.09 | 60 / 69 | control |
| wan | compress | 1 | main-B (1) | 42.2 / 46.3 / 55.8 | 223 | 2644 / 5888 | 0.81 / 3.17 | 60 / 69 | control |
| wan | compress | 1 | candidate (0) | 43.8 / 47.9 / 60.0 | 228 | 2628 / 5720 | 0.8 / 1.06 | 55 / 58 | fps_tradeoff_review_required |
| wan | bandwidth | 0 | ssh (0) | 38.2 / 46.8 / 60.0 | 5798 | 7652 / 18828 | 0.23 / 0.23 | 53 / 57 | control |
| wan | bandwidth | 0 | main-A (1) | 30.6 / 46.0 / 54.2 | 2127 | 2696 / 5968 | 1.09 / 4.63 | 63 / 72 | control |
| wan | bandwidth | 0 | main-B (2) | 35.6 / 45.9 / 54.4 | 2124 | 2712 / 5984 | 1.08 / 4.62 | 62 / 71 | control |
| wan | bandwidth | 0 | candidate (3) | 35.5 / 46.5 / 58.4 | 2281 | 2716 / 6000 | 1.19 / 5.11 | 58 / 65 | fps_tradeoff_review_required |
| wan | bandwidth | 1 | ssh (3) | 31.5 / 47.4 / 60.0 | 5790 | 7404 / 18848 | 0.22 / 0.22 | 53 / 57 | control |
| wan | bandwidth | 1 | main-A (2) | 29.0 / 43.9 / 53.6 | 2092 | 2704 / 5960 | 1.04 / 4.34 | 62 / 72 | control |
| wan | bandwidth | 1 | main-B (1) | 31.2 / 45.6 / 54.4 | 2124 | 2720 / 6024 | 1.04 / 4.46 | 62 / 71 | control |
| wan | bandwidth | 1 | candidate (0) | 42.5 / 47.5 / 58.5 | 2279 | 2708 / 5964 | 1.14 / 4.7 | 59 / 67 | fps_tradeoff_review_required |
| highlat | compress | 0 | ssh (0) | 22.9 / 46.5 / 60.0 | 2505 | 7588 / 18932 | 0.23 / 0.25 | 152 / 239 | control |
| highlat | compress | 0 | main-A (1) | 29.3 / 44.6 / 53.6 | 214 | 2688 / 5908 | 0.75 / 2.94 | 160 / 170 | control |
| highlat | compress | 0 | main-B (2) | 40.2 / 45.9 / 54.7 | 218 | 2736 / 5904 | 0.76 / 3.02 | 161 / 169 | control |
| highlat | compress | 0 | candidate (3) | 45.5 / 49.2 / 58.9 | 235 | 2692 / 5960 | 0.85 / 3.27 | 158 / 165 | fps_tradeoff_review_required |
| highlat | compress | 1 | ssh (3) | 22.9 / 45.3 / 60.0 | 2508 | 7612 / 18904 | 0.23 / 0.27 | 152 / 240 | control |
| highlat | compress | 1 | main-A (2) | 38.0 / 43.7 / 54.4 | 216 | 2676 / 5924 | 0.84 / 3.14 | 162 / 170 | control |
| highlat | compress | 1 | main-B (1) | 18.9 / 28.0 / 39.7 | 160 | 2692 / 5904 | 0.64 / 2.42 | 153 / 172 | control |
| highlat | compress | 1 | candidate (0) | 31.4 / 42.8 / 56.1 | 220 | 2708 / 5748 | 0.81 / 1.79 | 157 / 167 | fps_tradeoff_review_required |
| highlat | bandwidth | 0 | ssh (0) | 12.1 / 44.2 / 60.0 | 5796 | 7916 / 19000 | 0.2 / 0.21 | 153 / 489 | control |
| highlat | bandwidth | 0 | main-A (1) | 13.6 / 43.5 / 48.2 | 1926 | 2744 / 5852 | 0.94 / 4.07 | 163 / 173 | control |
| highlat | bandwidth | 0 | main-B (2) | 14.4 / 44.9 / 49.4 | 1978 | 2744 / 5604 | 0.97 / 4.12 | 162 / 172 | control |
| highlat | bandwidth | 0 | candidate (3) | 27.6 / 47.3 / 53.5 | 2129 | 2744 / 5496 | 1.05 / 4.46 | 161 / 168 | fps_tradeoff_review_required |
| highlat | bandwidth | 1 | ssh (3) | 14.9 / 45.0 / 60.0 | 5792 | 7924 / 19072 | 0.2 / 0.2 | 153 / 477 | control |
| highlat | bandwidth | 1 | main-A (2) | 15.1 / 45.4 / 49.7 | 1981 | 2744 / 6028 | 0.96 / 4.08 | 163 / 172 | control |
| highlat | bandwidth | 1 | main-B (1) | 13.5 / 45.3 / 50.1 | 1994 | 2756 / 6060 | 1.02 / 4.12 | 162 / 172 | control |
| highlat | bandwidth | 1 | candidate (0) | 14.5 / 47.4 / 53.4 | 2130 | 2784 / 6124 | 1.05 / 4.43 | 160 / 166 | fps_tradeoff_review_required |
