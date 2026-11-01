# Focused per-repetition comparison evidence

Each row retains one measured role in one repetition. No pooling, noise tolerance or automatic pass is applied. Observed deficits do not establish policy causality; see JSON for exact missing metrics, FPS gaps and resource increases.

| Network | Scene | Rep | Role (slot) | FPS 1% / 5% / avg | Network kbit/s | RSS client / server KiB | CPU client / server % | Age median / p95 ms | Assessment |
|---|---|---:|---|---|---:|---|---|---|---|
| lan | compress | 0 | ssh (0) | 47.2 / 51.1 / 60.0 | 2484 | 7524 / 18792 | 0.26 / 0.31 | 1 / 3 | control |
| lan | compress | 0 | main-A (1) | 43.2 / 46.9 / 60.0 | 228 | 2700 / 5652 | 0.82 / 1.13 | 3 / 6 | control |
| lan | compress | 0 | main-B (2) | 42.4 / 47.1 / 60.0 | 227 | 2692 / 5684 | 0.82 / 1.16 | 4 / 6 | control |
| lan | compress | 0 | candidate (3) | 42.5 / 47.4 / 60.0 | 228 | 2624 / 5680 | 0.84 / 1.17 | 4 / 7 | fps_tradeoff_review_required |
| lan | compress | 1 | ssh (2) | 44.6 / 48.7 / 60.0 | 2478 | 7544 / 18804 | 0.25 / 0.32 | 1 / 4 | control |
| lan | compress | 1 | main-A (3) | 43.3 / 46.9 / 60.0 | 227 | 2700 / 5664 | 0.83 / 1.16 | 4 / 6 | control |
| lan | compress | 1 | main-B (0) | 43.9 / 47.0 / 60.0 | 227 | 2700 / 5680 | 0.81 / 1.12 | 4 / 6 | control |
| lan | compress | 1 | candidate (1) | 42.4 / 45.8 / 60.0 | 227 | 2636 / 5652 | 0.87 / 1.17 | 3 / 7 | fps_tradeoff_review_required |
| lan | bandwidth | 0 | ssh (0) | 46.6 / 50.4 / 60.0 | 5776 | 7376 / 18632 | 0.2 / 0.26 | 2 / 4 | control |
| lan | bandwidth | 0 | main-A (1) | 42.3 / 46.6 / 60.0 | 2717 | 2716 / 5696 | 1.21 / 1.57 | 5 / 8 | control |
| lan | bandwidth | 0 | main-B (2) | 41.5 / 45.7 / 60.0 | 2717 | 2724 / 5700 | 1.22 / 1.56 | 5 / 9 | control |
| lan | bandwidth | 0 | candidate (3) | 41.5 / 46.5 / 60.0 | 2716 | 2636 / 5748 | 1.23 / 1.57 | 5 / 8 | fps_tradeoff_review_required |
| lan | bandwidth | 1 | ssh (2) | 46.4 / 49.7 / 60.0 | 5794 | 7552 / 18900 | 0.21 / 0.24 | 2 / 4 | control |
| lan | bandwidth | 1 | main-A (3) | 41.3 / 46.5 / 60.0 | 2717 | 2720 / 5700 | 1.2 / 1.53 | 5 / 9 | control |
| lan | bandwidth | 1 | main-B (0) | 43.8 / 47.6 / 60.0 | 2717 | 2704 / 5752 | 1.19 / 1.48 | 5 / 8 | control |
| lan | bandwidth | 1 | candidate (1) | 43.0 / 46.2 / 60.0 | 2722 | 2656 / 5732 | 1.17 / 1.51 | 5 / 8 | fps_tradeoff_review_required |
| wan | compress | 0 | ssh (0) | 42.4 / 46.5 / 60.0 | 2512 | 7556 / 18888 | 0.25 / 0.3 | 52 / 55 | control |
| wan | compress | 0 | main-A (1) | 42.4 / 47.6 / 60.0 | 228 | 2692 / 5688 | 0.87 / 1.17 | 55 / 59 | control |
| wan | compress | 0 | main-B (2) | 40.9 / 46.1 / 60.0 | 228 | 2696 / 5688 | 0.87 / 1.17 | 55 / 59 | control |
| wan | compress | 0 | candidate (3) | 42.1 / 45.8 / 60.0 | 228 | 2628 / 5684 | 0.87 / 1.2 | 54 / 58 | fps_tradeoff_review_required |
| wan | compress | 1 | ssh (2) | 41.9 / 47.1 / 60.0 | 2510 | 7552 / 18816 | 0.24 / 0.25 | 52 / 55 | control |
| wan | compress | 1 | main-A (3) | 41.3 / 45.4 / 60.0 | 228 | 2688 / 5720 | 0.77 / 1.06 | 55 / 59 | control |
| wan | compress | 1 | main-B (0) | 43.2 / 47.1 / 60.0 | 228 | 2700 / 5660 | 0.77 / 1.04 | 55 / 58 | control |
| wan | compress | 1 | candidate (1) | 42.7 / 46.0 / 60.0 | 228 | 2628 / 5680 | 0.78 / 1.06 | 54 / 57 | fps_tradeoff_review_required |
| wan | bandwidth | 0 | ssh (0) | 40.7 / 46.5 / 60.0 | 5795 | 7656 / 18832 | 0.25 / 0.26 | 54 / 58 | control |
| wan | bandwidth | 0 | main-A (1) | 20.8 / 44.1 / 53.5 | 2087 | 2764 / 6000 | 1.11 / 4.83 | 62 / 72 | control |
| wan | bandwidth | 0 | main-B (2) | 29.7 / 45.4 / 54.5 | 2130 | 2768 / 5956 | 1.16 / 4.86 | 62 / 71 | control |
| wan | bandwidth | 0 | candidate (3) | 28.2 / 44.1 / 57.4 | 2248 | 2672 / 5956 | 1.25 / 5.05 | 56 / 67 | fps_tradeoff_review_required |
| wan | bandwidth | 1 | ssh (2) | 31.1 / 46.8 / 60.0 | 5790 | 7700 / 18820 | 0.22 / 0.23 | 53 / 57 | control |
| wan | bandwidth | 1 | main-A (3) | 29.7 / 45.0 / 54.4 | 2131 | 2764 / 5960 | 1.13 / 4.84 | 62 / 72 | control |
| wan | bandwidth | 1 | main-B (0) | 39.9 / 43.9 / 54.5 | 2131 | 2760 / 5956 | 1.14 / 4.87 | 62 / 71 | control |
| wan | bandwidth | 1 | candidate (1) | 28.8 / 44.6 / 57.1 | 2231 | 2676 / 5960 | 1.18 / 4.94 | 56 / 65 | fps_tradeoff_review_required |
| highlat | compress | 0 | ssh (0) | 24.2 / 46.9 / 60.0 | 2509 | 7616 / 18804 | 0.25 / 0.28 | 152 / 243 | control |
| highlat | compress | 0 | main-A (1) | 39.4 / 45.5 / 54.6 | 218 | 2760 / 5900 | 0.84 / 2.97 | 161 / 170 | control |
| highlat | compress | 0 | main-B (2) | 29.5 / 36.1 / 52.7 | 212 | 2756 / 5956 | 0.82 / 3.0 | 161 / 171 | control |
| highlat | compress | 0 | candidate (3) | 29.6 / 41.4 / 56.9 | 220 | 2688 / 5768 | 0.8 / 1.29 | 156 / 160 | fps_tradeoff_review_required |
| highlat | compress | 1 | ssh (2) | 23.7 / 45.6 / 60.0 | 2501 | 7608 / 18820 | 0.24 / 0.26 | 152 / 239 | control |
| highlat | compress | 1 | main-A (3) | 41.3 / 45.4 / 54.7 | 219 | 2772 / 5924 | 0.82 / 3.09 | 161 / 170 | control |
| highlat | compress | 1 | main-B (0) | 43.0 / 46.7 / 54.5 | 218 | 2776 / 5924 | 0.84 / 3.08 | 161 / 169 | control |
| highlat | compress | 1 | candidate (1) | 45.6 / 49.4 / 59.0 | 235 | 2736 / 5916 | 0.88 / 3.36 | 154 / 161 | fps_tradeoff_review_required |
| highlat | bandwidth | 0 | ssh (0) | 11.9 / 44.0 / 60.0 | 5797 | 8056 / 19140 | 0.24 / 0.25 | 154 / 487 | control |
| highlat | bandwidth | 0 | main-A (1) | 11.4 / 44.6 / 49.7 | 1976 | 2808 / 6056 | 1.05 / 4.56 | 163 / 172 | control |
| highlat | bandwidth | 0 | main-B (2) | 24.2 / 44.7 / 49.5 | 1984 | 2824 / 6040 | 1.05 / 4.47 | 163 / 171 | control |
| highlat | bandwidth | 0 | candidate (3) | 23.2 / 46.2 / 53.6 | 2119 | 2776 / 6064 | 1.13 / 4.75 | 155 / 167 | fps_tradeoff_review_required |
| highlat | bandwidth | 1 | ssh (2) | 12.6 / 44.8 / 60.0 | 5794 | 7764 / 19008 | 0.23 / 0.25 | 154 / 573 | control |
| highlat | bandwidth | 1 | main-A (3) | 8.4 / 44.5 / 49.9 | 1988 | 2780 / 6064 | 1.1 / 4.6 | 163 / 172 | control |
| highlat | bandwidth | 1 | main-B (0) | 10.2 / 45.1 / 50.5 | 1990 | 2780 / 6032 | 1.13 / 4.59 | 162 / 171 | control |
| highlat | bandwidth | 1 | candidate (1) | 16.1 / 46.9 / 53.4 | 2129 | 2768 / 6044 | 1.2 / 5.02 | 160 / 169 | fps_tradeoff_review_required |
