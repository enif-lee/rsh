# Focused per-repetition comparison evidence

Each row retains one measured role in one repetition. No pooling, noise tolerance or automatic pass is applied. Observed deficits do not establish policy causality; see JSON for exact missing metrics, FPS gaps and resource increases.

| Network | Scene | Rep | Role (slot) | FPS 1% / 5% / avg | Network kbit/s | RSS client / server KiB | CPU client / server % | Age median / p95 ms | Assessment |
|---|---|---:|---|---|---:|---|---|---|---|
| lan | compress | 0 | ssh (0) | 45.9 / 49.2 / 60.0 | 2478 | 7556 / 18820 | 0.31 / 0.4 | 1 / 4 | control |
| lan | compress | 0 | main-A (1) | 42.8 / 46.9 / 60.0 | 228 | 2708 / 5680 | 1.02 / 1.58 | 4 / 7 | control |
| lan | compress | 0 | main-B (2) | 41.2 / 46.6 / 60.0 | 228 | 2692 / 5692 | 1.03 / 1.42 | 3 / 8 | control |
| lan | compress | 0 | candidate (3) | 44.1 / 47.0 / 60.0 | 228 | 2696 / 5720 | 1.04 / 1.43 | 4 / 7 | fps_tradeoff_review_required |
| lan | compress | 1 | ssh (2) | 45.9 / 49.7 / 60.0 | 2479 | 7544 / 18884 | 0.3 / 0.37 | 1 / 3 | control |
| lan | compress | 1 | main-A (3) | 42.6 / 47.7 / 60.0 | 227 | 2692 / 5656 | 0.89 / 1.24 | 3 / 7 | control |
| lan | compress | 1 | main-B (0) | 43.3 / 47.5 / 60.0 | 227 | 2692 / 5688 | 0.88 / 1.23 | 4 / 7 | control |
| lan | compress | 1 | candidate (1) | 41.7 / 46.5 / 60.0 | 227 | 2696 / 5724 | 0.9 / 1.25 | 3 / 7 | fps_tradeoff_review_required |
| lan | bandwidth | 0 | ssh (0) | 45.0 / 48.2 / 60.0 | 5788 | 7552 / 18820 | 0.23 / 0.29 | 2 / 5 | control |
| lan | bandwidth | 0 | main-A (1) | 41.5 / 46.3 / 60.0 | 2718 | 2704 / 5236 | 1.3 / 1.68 | 5 / 9 | control |
| lan | bandwidth | 0 | main-B (2) | 41.7 / 46.6 / 60.0 | 2723 | 2708 / 5280 | 1.28 / 1.66 | 5 / 9 | control |
| lan | bandwidth | 0 | candidate (3) | 43.0 / 47.6 / 60.0 | 2717 | 2708 / 5268 | 1.3 / 1.69 | 5 / 9 | fps_tradeoff_review_required |
| lan | bandwidth | 1 | ssh (2) | 46.0 / 48.5 / 60.0 | 5775 | 7552 / 18800 | 0.22 / 0.29 | 2 / 5 | control |
| lan | bandwidth | 1 | main-A (3) | 42.5 / 46.4 / 60.0 | 2721 | 2704 / 5696 | 1.27 / 1.7 | 5 / 8 | control |
| lan | bandwidth | 1 | main-B (0) | 42.4 / 45.7 / 60.0 | 2717 | 2716 / 5752 | 1.26 / 1.68 | 5 / 9 | control |
| lan | bandwidth | 1 | candidate (1) | 41.1 / 45.6 / 60.0 | 2717 | 2724 / 5824 | 1.26 / 1.75 | 5 / 9 | fps_tradeoff_review_required |
| wan | compress | 0 | ssh (0) | 40.9 / 47.5 / 60.0 | 2511 | 7564 / 18820 | 0.3 / 0.33 | 52 / 56 | control |
| wan | compress | 0 | main-A (1) | 42.2 / 47.0 / 60.0 | 228 | 2688 / 5664 | 1.04 / 1.29 | 54 / 58 | control |
| wan | compress | 0 | main-B (2) | 42.0 / 46.7 / 60.0 | 228 | 2696 / 5680 | 0.97 / 1.3 | 54 / 58 | control |
| wan | compress | 0 | candidate (3) | 43.2 / 47.2 / 60.0 | 228 | 2692 / 5748 | 0.97 / 1.29 | 55 / 58 | fps_tradeoff_review_required |
| wan | compress | 1 | ssh (2) | 41.0 / 48.1 / 60.0 | 2505 | 7552 / 18812 | 0.29 / 0.34 | 52 / 56 | control |
| wan | compress | 1 | main-A (3) | 40.0 / 46.4 / 60.0 | 228 | 2692 / 5664 | 1.01 / 1.37 | 55 / 58 | control |
| wan | compress | 1 | main-B (0) | 41.7 / 47.1 / 60.0 | 228 | 2692 / 5688 | 0.98 / 1.38 | 55 / 59 | control |
| wan | compress | 1 | candidate (1) | 41.8 / 46.2 / 60.0 | 228 | 2692 / 5748 | 1.02 / 1.37 | 55 / 58 | fps_tradeoff_review_required |
| wan | bandwidth | 0 | ssh (0) | 36.3 / 46.6 / 60.0 | 5799 | 7628 / 18792 | 0.25 / 0.26 | 53 / 58 | control |
| wan | bandwidth | 0 | main-A (1) | 28.9 / 45.0 / 54.1 | 2130 | 2780 / 5972 | 1.15 / 4.94 | 62 / 71 | control |
| wan | bandwidth | 0 | main-B (2) | 40.2 / 45.6 / 54.7 | 2132 | 2724 / 6004 | 1.16 / 5.02 | 62 / 71 | control |
| wan | bandwidth | 0 | candidate (3) | 29.7 / 47.3 / 58.5 | 2279 | 2764 / 6028 | 1.28 / 5.14 | 57 / 67 | fps_tradeoff_review_required |
| wan | bandwidth | 1 | ssh (2) | 42.5 / 47.6 / 60.0 | 5885 | 7660 / 18828 | 0.22 / 0.23 | 53 / 58 | control |
| wan | bandwidth | 1 | main-A (3) | 33.2 / 45.3 / 55.0 | 2142 | 2680 / 5972 | 1.07 / 4.56 | 63 / 71 | control |
| wan | bandwidth | 1 | main-B (0) | 41.6 / 45.8 / 54.7 | 2136 | 2768 / 5956 | 1.04 / 4.54 | 62 / 72 | control |
| wan | bandwidth | 1 | candidate (1) | 31.6 / 47.9 / 58.3 | 2270 | 2728 / 6028 | 1.13 / 4.71 | 57 / 68 | fps_tradeoff_review_required |
| highlat | compress | 0 | ssh (0) | 22.8 / 45.2 / 60.0 | 2501 | 7632 / 18952 | 0.23 / 0.25 | 152 / 240 | control |
| highlat | compress | 0 | main-A (1) | 43.0 / 46.2 / 54.5 | 218 | 2756 / 5964 | 0.8 / 2.87 | 161 / 170 | control |
| highlat | compress | 0 | main-B (2) | 41.7 / 45.8 / 54.6 | 218 | 2776 / 5956 | 0.79 / 2.89 | 161 / 170 | control |
| highlat | compress | 0 | candidate (3) | 44.0 / 48.7 / 58.9 | 235 | 2764 / 5988 | 0.86 / 3.12 | 155 / 164 | fps_tradeoff_review_required |
| highlat | compress | 1 | ssh (2) | 24.2 / 44.4 / 60.0 | 2505 | 7640 / 18832 | 0.27 / 0.31 | 152 / 233 | control |
| highlat | compress | 1 | main-A (3) | 41.0 / 45.1 / 54.5 | 218 | 2768 / 5912 | 0.88 / 3.0 | 161 / 170 | control |
| highlat | compress | 1 | main-B (0) | 41.0 / 46.0 / 54.6 | 219 | 2780 / 5928 | 0.9 / 3.01 | 161 / 170 | control |
| highlat | compress | 1 | candidate (1) | 42.5 / 47.7 / 59.0 | 235 | 2760 / 5980 | 0.94 / 3.25 | 154 / 162 | fps_tradeoff_review_required |
| highlat | bandwidth | 0 | ssh (0) | 12.2 / 44.5 / 60.0 | 5794 | 7936 / 19068 | 0.22 / 0.24 | 154 / 484 | control |
| highlat | bandwidth | 0 | main-A (1) | 8.8 / 44.8 / 49.6 | 1989 | 2456 / 5604 | 1.04 / 4.39 | 163 / 172 | control |
| highlat | bandwidth | 0 | main-B (2) | 14.9 / 45.4 / 50.0 | 1984 | 2532 / 5612 | 1.05 / 4.37 | 163 / 172 | control |
| highlat | bandwidth | 0 | candidate (3) | 11.8 / 43.5 / 53.0 | 2116 | 2644 / 6120 | 1.14 / 4.69 | 158 / 169 | fps_tradeoff_review_required |
| highlat | bandwidth | 1 | ssh (2) | 12.4 / 44.7 / 60.0 | 5800 | 8008 / 19016 | 0.22 / 0.22 | 154 / 494 | control |
| highlat | bandwidth | 1 | main-A (3) | 14.0 / 44.6 / 49.9 | 1979 | 2824 / 6040 | 0.97 / 4.13 | 163 / 172 | control |
| highlat | bandwidth | 1 | main-B (0) | 15.0 / 28.8 / 46.9 | 1847 | 2808 / 6036 | 0.91 / 3.91 | 162 / 172 | control |
| highlat | bandwidth | 1 | candidate (1) | 11.5 / 47.5 / 53.4 | 2117 | 2828 / 6084 | 1.04 / 4.37 | 157 / 168 | fps_tradeoff_review_required |
