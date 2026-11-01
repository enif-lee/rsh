# Focused per-repetition comparison evidence

Each row retains one measured role in one repetition. No pooling, noise tolerance or automatic pass is applied. Observed deficits do not establish policy causality; see JSON for exact missing metrics, FPS gaps and resource increases.

| Network | Scene | Rep | Role (slot) | FPS 1% / 5% / avg | Network kbit/s | RSS client / server KiB | CPU client / server % | Age median / p95 ms | Assessment |
|---|---|---:|---|---|---:|---|---|---|---|
| lan | compress | 0 | ssh (0) | 47.0 / 49.1 / 60.0 | 2483 | 7548 / 18800 | 0.27 / 0.34 | 1 / 4 | control |
| lan | compress | 0 | main-A (1) | 43.5 / 48.0 / 60.0 | 228 | 2692 / 5656 | 0.83 / 1.15 | 3 / 6 | control |
| lan | compress | 0 | main-B (2) | 43.6 / 47.8 / 60.0 | 228 | 2696 / 5700 | 0.85 / 1.16 | 3 / 6 | control |
| lan | compress | 0 | candidate (3) | 44.5 / 46.9 / 60.0 | 227 | 2692 / 5788 | 0.85 / 1.21 | 4 / 6 | fps_tradeoff_review_required |
| lan | compress | 1 | ssh (2) | 46.4 / 50.8 / 60.0 | 2485 | 7536 / 18828 | 0.26 / 0.31 | 1 / 3 | control |
| lan | compress | 1 | main-A (3) | 41.9 / 46.7 / 60.0 | 227 | 2692 / 5660 | 0.74 / 1.05 | 4 / 6 | control |
| lan | compress | 1 | main-B (0) | 45.6 / 48.6 / 60.0 | 227 | 2704 / 5684 | 0.76 / 1.05 | 3 / 6 | control |
| lan | compress | 1 | candidate (1) | 44.7 / 48.1 / 60.0 | 228 | 2716 / 5816 | 0.76 / 1.09 | 4 / 6 | fps_tradeoff_review_required |
| lan | bandwidth | 0 | ssh (0) | 45.3 / 49.8 / 60.0 | 5788 | 7552 / 18820 | 0.18 / 0.22 | 2 / 4 | control |
| lan | bandwidth | 0 | main-A (1) | 43.4 / 46.7 / 60.0 | 2721 | 2716 / 5728 | 1.07 / 1.42 | 5 / 8 | control |
| lan | bandwidth | 0 | main-B (2) | 41.9 / 45.6 / 60.0 | 2718 | 2732 / 5728 | 1.05 / 1.45 | 5 / 9 | control |
| lan | bandwidth | 0 | candidate (3) | 43.1 / 46.3 / 60.0 | 2716 | 2732 / 5840 | 1.05 / 1.47 | 5 / 8 | fps_tradeoff_review_required |
| lan | bandwidth | 1 | ssh (2) | 43.4 / 49.2 / 60.0 | 5791 | 7552 / 18816 | 0.2 / 0.23 | 2 / 4 | control |
| lan | bandwidth | 1 | main-A (3) | 41.5 / 46.1 / 60.0 | 2716 | 2720 / 5752 | 1.13 / 1.46 | 5 / 9 | control |
| lan | bandwidth | 1 | main-B (0) | 42.3 / 46.6 / 60.0 | 2717 | 2708 / 5732 | 1.1 / 1.44 | 5 / 9 | control |
| lan | bandwidth | 1 | candidate (1) | 43.1 / 46.5 / 60.0 | 2718 | 2720 / 5876 | 1.1 / 1.47 | 5 / 8 | fps_tradeoff_review_required |
| wan | compress | 0 | ssh (0) | 41.7 / 47.5 / 60.0 | 2516 | 7560 / 18828 | 0.27 / 0.29 | 52 / 56 | control |
| wan | compress | 0 | main-A (1) | 42.1 / 46.3 / 60.0 | 228 | 2692 / 5724 | 0.85 / 1.16 | 55 / 58 | control |
| wan | compress | 0 | main-B (2) | 41.4 / 47.2 / 60.0 | 228 | 2700 / 5688 | 0.88 / 1.16 | 55 / 58 | control |
| wan | compress | 0 | candidate (3) | 41.6 / 47.1 / 60.0 | 228 | 2696 / 5824 | 0.86 / 1.19 | 55 / 58 | fps_tradeoff_review_required |
| wan | compress | 1 | ssh (2) | 42.5 / 46.6 / 60.0 | 2506 | 7560 / 18828 | 0.24 / 0.3 | 52 / 56 | control |
| wan | compress | 1 | main-A (3) | 41.2 / 46.3 / 60.0 | 228 | 2696 / 5700 | 0.87 / 1.15 | 55 / 59 | control |
| wan | compress | 1 | main-B (0) | 41.2 / 46.8 / 60.0 | 228 | 2692 / 5688 | 0.86 / 1.19 | 55 / 59 | control |
| wan | compress | 1 | candidate (1) | 42.6 / 44.8 / 60.0 | 228 | 2704 / 5844 | 0.89 / 1.2 | 56 / 60 | fps_tradeoff_review_required |
| wan | bandwidth | 0 | ssh (0) | 41.3 / 47.9 / 60.0 | 5793 | 7600 / 18804 | 0.23 / 0.23 | 53 / 57 | control |
| wan | bandwidth | 0 | main-A (1) | 40.8 / 45.2 / 54.6 | 2133 | 2728 / 5968 | 1.09 / 4.77 | 62 / 71 | control |
| wan | bandwidth | 0 | main-B (2) | 39.4 / 45.0 / 54.9 | 2138 | 2696 / 5964 | 1.09 / 4.72 | 61 / 71 | control |
| wan | bandwidth | 0 | candidate (3) | 39.4 / 45.5 / 54.5 | 2132 | 2760 / 6104 | 1.1 / 4.82 | 63 / 72 | fps_tradeoff_review_required |
| wan | bandwidth | 1 | ssh (2) | 33.5 / 45.3 / 59.9 | 5785 | 7680 / 18824 | 0.21 / 0.22 | 53 / 58 | control |
| wan | bandwidth | 1 | main-A (3) | 39.8 / 44.8 / 54.4 | 2125 | 2732 / 5996 | 1.08 / 4.8 | 62 / 71 | control |
| wan | bandwidth | 1 | main-B (0) | 28.6 / 43.4 / 54.2 | 2113 | 2712 / 5996 | 1.09 / 4.82 | 62 / 71 | control |
| wan | bandwidth | 1 | candidate (1) | 31.9 / 43.9 / 54.6 | 2131 | 2708 / 6136 | 1.08 / 4.84 | 61 / 71 | fps_tradeoff_review_required |
| highlat | compress | 0 | ssh (0) | 24.8 / 45.0 / 60.0 | 2513 | 7592 / 18908 | 0.25 / 0.27 | 152 / 241 | control |
| highlat | compress | 0 | main-A (1) | 42.7 / 45.7 / 54.6 | 218 | 2788 / 5896 | 0.81 / 2.99 | 161 / 170 | control |
| highlat | compress | 0 | main-B (2) | 41.7 / 46.2 / 54.5 | 217 | 2780 / 5928 | 0.79 / 3.1 | 161 / 170 | control |
| highlat | compress | 0 | candidate (3) | 43.6 / 46.7 / 54.6 | 218 | 2788 / 6084 | 0.81 / 3.1 | 161 / 170 | fps_tradeoff_review_required |
| highlat | compress | 1 | ssh (2) | 21.8 / 47.0 / 60.0 | 2507 | 7608 / 18844 | 0.25 / 0.27 | 152 / 227 | control |
| highlat | compress | 1 | main-A (3) | 41.7 / 46.1 / 54.6 | 219 | 1728 / 5160 | 0.8 / 3.07 | 160 / 170 | control |
| highlat | compress | 1 | main-B (0) | 42.0 / 45.2 / 54.6 | 219 | 2200 / 5240 | 0.81 / 2.92 | 161 / 170 | control |
| highlat | compress | 1 | candidate (1) | 42.4 / 46.4 / 54.6 | 219 | 2180 / 5288 | 0.8 / 3.02 | 160 / 169 | fps_tradeoff_review_required |
| highlat | bandwidth | 0 | ssh (0) | 13.7 / 44.4 / 60.0 | 5798 | 7868 / 19108 | 0.21 / 0.24 | 154 / 496 | control |
| highlat | bandwidth | 0 | main-A (1) | 11.2 / 44.3 / 49.5 | 1970 | 2808 / 6068 | 1.08 / 4.6 | 162 / 172 | control |
| highlat | bandwidth | 0 | main-B (2) | 11.9 / 44.0 / 50.4 | 1994 | 2776 / 6048 | 1.07 / 4.59 | 163 / 173 | control |
| highlat | bandwidth | 0 | candidate (3) | 21.4 / 44.6 / 49.7 | 1985 | 2796 / 6164 | 1.07 / 4.61 | 163 / 172 | fps_tradeoff_review_required |
| highlat | bandwidth | 1 | ssh (2) | 11.4 / 44.4 / 60.0 | 5794 | 7984 / 18996 | 0.21 / 0.22 | 154 / 492 | control |
| highlat | bandwidth | 1 | main-A (3) | 12.8 / 44.3 / 49.7 | 1979 | 2812 / 6084 | 1.02 / 4.44 | 162 / 171 | control |
| highlat | bandwidth | 1 | main-B (0) | 11.6 / 45.1 / 50.2 | 1980 | 2772 / 6072 | 1.03 / 4.4 | 163 / 172 | control |
| highlat | bandwidth | 1 | candidate (1) | 12.8 / 43.1 / 50.6 | 1993 | 2744 / 6180 | 1.04 / 4.47 | 162 / 172 | fps_tradeoff_review_required |
