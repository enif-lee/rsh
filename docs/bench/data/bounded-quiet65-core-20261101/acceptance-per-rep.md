# Focused per-repetition comparison evidence

Each row retains one measured role in one repetition. No pooling, noise tolerance or automatic pass is applied. Observed deficits do not establish policy causality; see JSON for exact missing metrics, FPS gaps and resource increases.

| Network | Scene | Rep | Role (slot) | FPS 1% / 5% / avg | Network kbit/s | RSS client / server KiB | CPU client / server % | Age median / p95 ms | Assessment |
|---|---|---:|---|---|---:|---|---|---|---|
| lan | compress | 0 | ssh (0) | 46.5 / 49.6 / 60.0 | 2477 | 7544 / 18812 | 0.28 / 0.33 | 1 / 4 | control |
| lan | compress | 0 | main-A (1) | 45.3 / 48.4 / 60.0 | 228 | 2692 / 5752 | 0.75 / 1.04 | 3 / 6 | control |
| lan | compress | 0 | main-B (2) | 43.2 / 47.9 / 60.0 | 228 | 2692 / 5720 | 0.74 / 1.02 | 3 / 6 | control |
| lan | compress | 0 | candidate (3) | 45.1 / 48.2 / 60.0 | 227 | 2628 / 5688 | 0.77 / 1.07 | 3 / 6 | fps_tradeoff_review_required |
| lan | compress | 1 | ssh (3) | 46.1 / 49.7 / 60.0 | 2477 | 7556 / 18816 | 0.26 / 0.34 | 1 / 4 | control |
| lan | compress | 1 | main-A (2) | 45.2 / 47.6 / 60.0 | 227 | 2124 / 4888 | 0.86 / 1.17 | 3 / 6 | control |
| lan | compress | 1 | main-B (1) | 44.9 / 47.2 / 60.0 | 227 | 2116 / 5160 | 0.86 / 1.18 | 3 / 7 | control |
| lan | compress | 1 | candidate (0) | 42.8 / 46.7 / 60.0 | 228 | 2180 / 5172 | 0.9 / 1.24 | 3 / 6 | fps_tradeoff_review_required |
| lan | bandwidth | 0 | ssh (0) | 44.6 / 48.5 / 60.0 | 5792 | 7536 / 18824 | 0.2 / 0.25 | 2 / 5 | control |
| lan | bandwidth | 0 | main-A (1) | 43.3 / 47.7 / 60.0 | 2721 | 2712 / 5816 | 1.23 / 1.55 | 5 / 8 | control |
| lan | bandwidth | 0 | main-B (2) | 41.9 / 45.9 / 60.0 | 2717 | 2716 / 5796 | 1.23 / 1.57 | 5 / 9 | control |
| lan | bandwidth | 0 | candidate (3) | 43.5 / 46.7 / 60.0 | 2721 | 2656 / 5716 | 1.25 / 1.61 | 5 / 8 | fps_tradeoff_review_required |
| lan | bandwidth | 1 | ssh (3) | 45.2 / 49.3 / 60.0 | 5788 | 7556 / 18812 | 0.2 / 0.27 | 2 / 5 | control |
| lan | bandwidth | 1 | main-A (2) | 39.5 / 45.6 / 60.0 | 2717 | 2720 / 5776 | 1.3 / 1.73 | 5 / 9 | control |
| lan | bandwidth | 1 | main-B (1) | 42.1 / 47.0 / 60.0 | 2717 | 2700 / 5812 | 1.22 / 1.59 | 5 / 8 | control |
| lan | bandwidth | 1 | candidate (0) | 43.5 / 46.5 / 60.0 | 2717 | 2644 / 5720 | 1.24 / 1.66 | 5 / 9 | fps_tradeoff_review_required |
| wan | compress | 0 | ssh (0) | 43.4 / 48.0 / 60.0 | 2504 | 7548 / 18820 | 0.23 / 0.27 | 52 / 55 | control |
| wan | compress | 0 | main-A (1) | 41.7 / 46.9 / 60.0 | 228 | 1852 / 5184 | 0.81 / 1.12 | 55 / 58 | control |
| wan | compress | 0 | main-B (2) | 41.5 / 45.3 / 60.0 | 228 | 2128 / 4916 | 0.85 / 1.11 | 55 / 59 | control |
| wan | compress | 0 | candidate (3) | 42.7 / 46.7 / 60.0 | 228 | 2180 / 4856 | 0.83 / 1.14 | 55 / 58 | fps_tradeoff_review_required |
| wan | compress | 1 | ssh (3) | 41.9 / 47.7 / 60.0 | 2516 | 7548 / 18900 | 0.26 / 0.28 | 52 / 55 | control |
| wan | compress | 1 | main-A (2) | 41.3 / 45.9 / 60.0 | 228 | 2696 / 5780 | 0.81 / 1.09 | 55 / 58 | control |
| wan | compress | 1 | main-B (1) | 40.9 / 46.2 / 60.0 | 228 | 2692 / 5760 | 0.8 / 1.11 | 55 / 59 | control |
| wan | compress | 1 | candidate (0) | 42.1 / 46.8 / 60.0 | 228 | 2624 / 5660 | 0.84 / 1.14 | 55 / 58 | fps_tradeoff_review_required |
| wan | bandwidth | 0 | ssh (0) | 38.9 / 45.4 / 60.0 | 5794 | 7664 / 18808 | 0.24 / 0.25 | 53 / 57 | control |
| wan | bandwidth | 0 | main-A (1) | 32.6 / 47.4 / 58.2 | 2275 | 2760 / 6048 | 1.17 / 4.87 | 57 / 69 | control |
| wan | bandwidth | 0 | main-B (2) | 30.6 / 47.1 / 58.0 | 2269 | 2756 / 6080 | 1.15 / 4.78 | 58 / 64 | control |
| wan | bandwidth | 0 | candidate (3) | 37.9 / 47.7 / 58.4 | 2280 | 2704 / 5960 | 1.18 / 4.92 | 59 / 64 | fps_tradeoff_review_required |
| wan | bandwidth | 1 | ssh (3) | 40.9 / 47.2 / 60.0 | 5794 | 7384 / 18832 | 0.24 / 0.23 | 53 / 61 | control |
| wan | bandwidth | 1 | main-A (2) | 31.1 / 47.3 / 58.2 | 2280 | 2180 / 5008 | 1.11 / 4.7 | 55 / 66 | control |
| wan | bandwidth | 1 | main-B (1) | 29.1 / 46.2 / 58.2 | 2277 | 2164 / 5488 | 1.1 / 4.66 | 56 / 67 | control |
| wan | bandwidth | 1 | candidate (0) | 40.1 / 48.0 / 58.6 | 2280 | 2408 / 5212 | 1.12 / 4.8 | 59 / 64 | fps_tradeoff_review_required |
| highlat | compress | 0 | ssh (0) | 24.0 / 46.3 / 60.0 | 2500 | 7636 / 18872 | 0.22 / 0.26 | 152 / 233 | control |
| highlat | compress | 0 | main-A (1) | 44.8 / 48.2 / 59.0 | 234 | 2764 / 6020 | 0.87 / 3.38 | 154 / 159 | control |
| highlat | compress | 0 | main-B (2) | 43.6 / 48.5 / 59.0 | 235 | 2784 / 5980 | 0.88 / 3.37 | 154 / 162 | control |
| highlat | compress | 0 | candidate (3) | 43.5 / 49.0 / 59.0 | 235 | 2712 / 5960 | 0.91 / 3.44 | 157 / 162 | fps_tradeoff_review_required |
| highlat | compress | 1 | ssh (3) | 22.0 / 46.8 / 60.0 | 2505 | 7620 / 18832 | 0.24 / 0.28 | 152 / 230 | control |
| highlat | compress | 1 | main-A (2) | 44.7 / 49.1 / 59.0 | 235 | 2776 / 6024 | 0.89 / 3.3 | 154 / 160 | control |
| highlat | compress | 1 | main-B (1) | 43.2 / 47.7 / 59.0 | 235 | 2756 / 5976 | 0.89 / 3.31 | 154 / 160 | control |
| highlat | compress | 1 | candidate (0) | 41.9 / 46.8 / 58.0 | 226 | 2712 / 5780 | 0.83 / 1.59 | 156 / 163 | fps_tradeoff_review_required |
| highlat | bandwidth | 0 | ssh (0) | 13.6 / 42.2 / 60.0 | 5800 | 7992 / 19032 | 0.24 / 0.25 | 154 / 475 | control |
| highlat | bandwidth | 0 | main-A (1) | 19.9 / 47.3 / 53.0 | 2123 | 2836 / 5612 | 1.14 / 4.77 | 157 / 168 | control |
| highlat | bandwidth | 0 | main-B (2) | 30.8 / 46.5 / 52.9 | 2113 | 2804 / 5616 | 1.13 / 4.82 | 155 / 168 | control |
| highlat | bandwidth | 0 | candidate (3) | 13.4 / 46.6 / 53.6 | 2124 | 2744 / 5604 | 1.16 / 4.89 | 160 / 167 | fps_tradeoff_review_required |
| highlat | bandwidth | 1 | ssh (3) | 14.5 / 44.6 / 60.0 | 5791 | 7968 / 19020 | 0.21 / 0.23 | 154 / 481 | control |
| highlat | bandwidth | 1 | main-A (2) | 20.6 / 44.5 / 53.1 | 2127 | 2800 / 6100 | 1.15 / 4.82 | 156 / 167 | control |
| highlat | bandwidth | 1 | main-B (1) | 23.4 / 45.3 / 53.2 | 2119 | 2804 / 6164 | 1.15 / 4.86 | 156 / 170 | control |
| highlat | bandwidth | 1 | candidate (0) | 18.2 / 43.9 / 51.5 | 2065 | 2796 / 6036 | 1.13 / 4.8 | 160 / 165 | fps_tradeoff_review_required |
