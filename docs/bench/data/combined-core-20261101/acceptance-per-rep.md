# Focused per-repetition comparison evidence

Each row retains one measured role in one repetition. No pooling, noise tolerance or automatic pass is applied. Observed deficits do not establish policy causality; see JSON for exact missing metrics, FPS gaps and resource increases.

| Network | Scene | Rep | Role (slot) | FPS 1% / 5% / avg | Network kbit/s | RSS client / server KiB | CPU client / server % | Age median / p95 ms | Assessment |
|---|---|---:|---|---|---:|---|---|---|---|
| lan | compress | 0 | ssh (0) | 48.3 / 51.9 / 60.0 | 2485 | 7548 / 18836 | 0.28 / 0.32 | 1 / 3 | control |
| lan | compress | 0 | main-A (1) | 48.1 / 50.2 / 60.0 | 227 | 2012 / 5152 | 0.72 / 0.99 | 3 / 5 | control |
| lan | compress | 0 | main-B (2) | 43.9 / 48.1 / 60.0 | 228 | 1680 / 4616 | 0.71 / 0.99 | 3 / 6 | control |
| lan | compress | 0 | candidate (3) | 43.9 / 49.3 / 60.0 | 227 | 1876 / 4732 | 0.71 / 1.02 | 4 / 5 | fps_tradeoff_review_required |
| lan | compress | 1 | ssh (2) | 46.6 / 50.8 / 60.0 | 2478 | 7580 / 18808 | 0.28 / 0.35 | 1 / 4 | control |
| lan | compress | 1 | main-A (3) | 45.2 / 49.4 / 60.0 | 227 | 2632 / 5832 | 0.92 / 1.25 | 4 / 7 | control |
| lan | compress | 1 | main-B (0) | 43.7 / 49.0 / 60.0 | 227 | 2640 / 5784 | 0.94 / 1.25 | 4 / 7 | control |
| lan | compress | 1 | candidate (1) | 45.7 / 49.1 / 60.0 | 227 | 2624 / 5792 | 0.93 / 1.29 | 4 / 7 | fps_tradeoff_review_required |
| lan | bandwidth | 0 | ssh (0) | 48.4 / 51.3 / 60.0 | 5789 | 7556 / 18904 | 0.21 / 0.26 | 2 / 4 | control |
| lan | bandwidth | 0 | main-A (1) | 44.0 / 46.4 / 60.0 | 2718 | 2648 / 5860 | 1.15 / 1.62 | 6 / 8 | control |
| lan | bandwidth | 0 | main-B (2) | 45.1 / 47.7 / 60.0 | 2717 | 2656 / 5828 | 1.16 / 1.51 | 5 / 8 | control |
| lan | bandwidth | 0 | candidate (3) | 44.4 / 48.7 / 60.0 | 2718 | 2636 / 5888 | 1.21 / 1.55 | 5 / 8 | fps_tradeoff_review_required |
| lan | bandwidth | 1 | ssh (2) | 48.4 / 50.6 / 60.0 | 5793 | 7536 / 18896 | 0.24 / 0.29 | 2 / 4 | control |
| lan | bandwidth | 1 | main-A (3) | 43.9 / 48.2 / 59.9 | 2717 | 2660 / 5856 | 1.21 / 1.62 | 5 / 8 | control |
| lan | bandwidth | 1 | main-B (0) | 44.1 / 47.0 / 60.0 | 2718 | 2636 / 5888 | 1.21 / 1.66 | 5 / 8 | control |
| lan | bandwidth | 1 | candidate (1) | 41.6 / 48.4 / 60.0 | 2722 | 2668 / 5824 | 1.27 / 1.61 | 5 / 8 | fps_tradeoff_review_required |
| wan | compress | 0 | ssh (0) | 40.6 / 46.8 / 60.0 | 2506 | 7560 / 18848 | 0.27 / 0.31 | 52 / 55 | control |
| wan | compress | 0 | main-A (1) | 42.0 / 46.2 / 56.2 | 224 | 2644 / 6008 | 0.93 / 3.41 | 60 / 69 | control |
| wan | compress | 0 | main-B (2) | 42.4 / 45.6 / 55.8 | 223 | 2636 / 5992 | 0.91 / 3.47 | 60 / 70 | control |
| wan | compress | 0 | candidate (3) | 43.0 / 47.0 / 60.0 | 228 | 2624 / 5796 | 0.87 / 1.22 | 55 / 58 | qualitative_review_required |
| wan | compress | 1 | ssh (2) | 45.3 / 48.3 / 60.0 | 2511 | 7564 / 18888 | 0.27 / 0.29 | 52 / 55 | control |
| wan | compress | 1 | main-A (3) | 42.2 / 45.3 / 55.9 | 223 | 2664 / 6016 | 0.9 / 3.27 | 60 / 69 | control |
| wan | compress | 1 | main-B (0) | 42.6 / 45.8 / 55.7 | 222 | 2652 / 5988 | 0.83 / 3.14 | 60 / 69 | control |
| wan | compress | 1 | candidate (1) | 43.3 / 46.8 / 60.0 | 228 | 2628 / 5796 | 0.82 / 1.11 | 55 / 58 | fps_tradeoff_review_required |
| wan | bandwidth | 0 | ssh (0) | 32.8 / 46.9 / 60.0 | 5790 | 7656 / 18816 | 0.23 / 0.25 | 53 / 58 | control |
| wan | bandwidth | 0 | main-A (1) | 28.6 / 45.9 / 54.5 | 2131 | 1968 / 5320 | 1.12 / 4.69 | 62 / 71 | control |
| wan | bandwidth | 0 | main-B (2) | 40.7 / 46.9 / 54.7 | 2138 | 1764 / 5076 | 1.12 / 4.71 | 62 / 71 | control |
| wan | bandwidth | 0 | candidate (3) | 30.5 / 45.7 / 54.5 | 2130 | 2028 / 5348 | 1.12 / 4.68 | 63 / 71 | fps_tradeoff_review_required |
| wan | bandwidth | 1 | ssh (2) | 37.0 / 46.3 / 60.0 | 5794 | 7840 / 18800 | 0.27 / 0.27 | 53 / 58 | control |
| wan | bandwidth | 1 | main-A (3) | 39.7 / 45.2 / 54.5 | 2125 | 2664 / 6084 | 1.16 / 4.65 | 62 / 71 | control |
| wan | bandwidth | 1 | main-B (0) | 39.5 / 44.5 / 54.7 | 2127 | 2616 / 6128 | 1.16 / 4.8 | 63 / 71 | control |
| wan | bandwidth | 1 | candidate (1) | 21.4 / 34.2 / 52.1 | 2055 | 2688 / 6100 | 1.12 / 4.62 | 63 / 71 | fps_tradeoff_review_required |
| highlat | compress | 0 | ssh (0) | 24.7 / 46.7 / 60.0 | 2511 | 7588 / 18856 | 0.26 / 0.29 | 152 / 234 | control |
| highlat | compress | 0 | main-A (1) | 27.5 / 29.1 / 44.7 | 181 | 2688 / 6056 | 0.71 / 2.74 | 156 / 170 | control |
| highlat | compress | 0 | main-B (2) | 40.5 / 46.0 / 54.6 | 218 | 2696 / 6032 | 0.84 / 3.18 | 161 / 170 | control |
| highlat | compress | 0 | candidate (3) | 40.7 / 46.5 / 54.6 | 219 | 2712 / 6088 | 0.87 / 3.26 | 160 / 169 | fps_tradeoff_review_required |
| highlat | compress | 1 | ssh (2) | 23.1 / 46.0 / 60.0 | 2501 | 7600 / 18816 | 0.28 / 0.29 | 152 / 235 | control |
| highlat | compress | 1 | main-A (3) | 28.9 / 40.0 / 53.0 | 211 | 2248 / 5640 | 0.83 / 2.99 | 161 / 170 | control |
| highlat | compress | 1 | main-B (0) | 40.9 / 45.4 / 55.0 | 218 | 2228 / 5644 | 0.87 / 3.07 | 161 / 169 | control |
| highlat | compress | 1 | candidate (1) | 40.9 / 45.1 / 54.6 | 218 | 2272 / 5592 | 0.88 / 3.27 | 162 / 170 | fps_tradeoff_review_required |
| highlat | bandwidth | 0 | ssh (0) | 13.0 / 42.8 / 60.0 | 5788 | 7940 / 19008 | 0.25 / 0.27 | 154 / 496 | control |
| highlat | bandwidth | 0 | main-A (1) | 13.8 / 44.3 / 50.0 | 1990 | 2744 / 6168 | 1.16 / 4.71 | 163 / 172 | control |
| highlat | bandwidth | 0 | main-B (2) | 8.4 / 11.6 / 17.0 | 693 | 2748 / 6064 | 0.42 / 1.98 | 156 / 175 | control |
| highlat | bandwidth | 0 | candidate (3) | 12.7 / 44.2 / 50.1 | 2000 | 2732 / 6180 | 1.16 / 4.69 | 163 / 171 | fps_tradeoff_review_required |
| highlat | bandwidth | 1 | ssh (2) | 14.5 / 45.6 / 60.0 | 5791 | 7984 / 19000 | 0.25 / 0.28 | 154 / 476 | control |
| highlat | bandwidth | 1 | main-A (3) | 15.9 / 43.8 / 49.1 | 1971 | 2796 / 6164 | 1.18 / 4.94 | 162 / 172 | control |
| highlat | bandwidth | 1 | main-B (0) | 28.0 / 45.7 / 50.4 | 1999 | 2768 / 6156 | 1.22 / 5.07 | 162 / 172 | control |
| highlat | bandwidth | 1 | candidate (1) | 16.8 / 43.6 / 49.2 | 1987 | 2792 / 6224 | 1.21 / 5.08 | 163 / 172 | fps_tradeoff_review_required |
