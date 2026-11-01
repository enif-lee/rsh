# Focused per-repetition comparison evidence

Each row retains one measured role in one repetition. No pooling, noise tolerance or automatic pass is applied. Observed deficits do not establish policy causality; see JSON for exact missing metrics, FPS gaps and resource increases.

| Network | Scene | Rep | Role (slot) | FPS 1% / 5% / avg | Network kbit/s | RSS client / server KiB | CPU client / server % | Age median / p95 ms | Assessment |
|---|---|---:|---|---|---:|---|---|---|---|
| lan | compress | 0 | ssh (0) | 44.5 / 49.5 / 60.0 | 2472 | 7536 / 18816 | 0.26 / 0.34 | 1 / 4 | control |
| lan | compress | 0 | main-A (1) | 43.2 / 47.0 / 60.0 | 227 | 2648 / 5696 | 0.86 / 1.17 | 4 / 7 | control |
| lan | compress | 0 | main-B (2) | 44.6 / 48.0 / 60.0 | 228 | 2628 / 5688 | 0.84 / 1.16 | 3 / 6 | control |
| lan | compress | 0 | candidate (3) | 44.7 / 47.6 / 60.0 | 227 | 2696 / 5660 | 0.86 / 1.19 | 3 / 7 | fps_tradeoff_review_required |
| lan | compress | 1 | ssh (2) | 46.1 / 51.0 / 60.0 | 2476 | 7540 / 18824 | 0.28 / 0.34 | 1 / 3 | control |
| lan | compress | 1 | main-A (3) | 43.1 / 47.0 / 60.0 | 227 | 2636 / 5688 | 0.8 / 1.11 | 3 / 6 | control |
| lan | compress | 1 | main-B (0) | 44.1 / 47.1 / 60.0 | 228 | 2624 / 5684 | 0.79 / 1.1 | 3 / 6 | control |
| lan | compress | 1 | candidate (1) | 43.4 / 47.0 / 60.0 | 227 | 2692 / 5656 | 0.82 / 1.15 | 3 / 6 | fps_tradeoff_review_required |
| lan | bandwidth | 0 | ssh (0) | 45.0 / 49.7 / 60.0 | 5781 | 7356 / 18572 | 0.28 / 0.32 | 2 / 5 | control |
| lan | bandwidth | 0 | main-A (1) | 39.2 / 44.4 / 60.0 | 2718 | 1804 / 4784 | 1.29 / 1.62 | 5 / 9 | control |
| lan | bandwidth | 0 | main-B (2) | 40.0 / 45.5 / 60.0 | 2716 | 1980 / 4728 | 1.31 / 1.65 | 5 / 9 | control |
| lan | bandwidth | 0 | candidate (3) | 40.7 / 46.5 / 60.0 | 2717 | 1776 / 4820 | 1.31 / 1.77 | 5 / 9 | fps_tradeoff_review_required |
| lan | bandwidth | 1 | ssh (2) | 43.1 / 48.3 / 60.0 | 5791 | 7552 / 18820 | 0.28 / 0.35 | 2 / 5 | control |
| lan | bandwidth | 1 | main-A (3) | 40.5 / 46.0 / 60.0 | 2718 | 2640 / 5748 | 1.45 / 1.89 | 6 / 10 | control |
| lan | bandwidth | 1 | main-B (0) | 41.0 / 45.7 / 60.0 | 2717 | 2628 / 5752 | 1.48 / 1.9 | 5 / 9 | control |
| lan | bandwidth | 1 | candidate (1) | 40.9 / 46.2 / 60.0 | 2716 | 2700 / 5688 | 1.47 / 1.91 | 5 / 9 | fps_tradeoff_review_required |
| wan | compress | 0 | ssh (0) | 42.3 / 47.4 / 60.0 | 2504 | 7556 / 18840 | 0.24 / 0.27 | 52 / 55 | control |
| wan | compress | 0 | main-A (1) | 43.5 / 46.3 / 56.1 | 224 | 2640 / 6072 | 0.84 / 3.29 | 60 / 69 | control |
| wan | compress | 0 | main-B (2) | 42.0 / 45.8 / 55.7 | 222 | 2644 / 5892 | 0.84 / 3.29 | 60 / 69 | control |
| wan | compress | 0 | candidate (3) | 42.9 / 46.6 / 60.0 | 228 | 2700 / 5660 | 0.82 / 1.11 | 55 / 58 | fps_tradeoff_review_required |
| wan | compress | 1 | ssh (2) | 42.4 / 47.8 / 60.0 | 2509 | 7424 / 18844 | 0.23 / 0.27 | 52 / 55 | control |
| wan | compress | 1 | main-A (3) | 43.6 / 47.1 / 56.3 | 224 | 2260 / 5240 | 0.86 / 3.22 | 60 / 69 | control |
| wan | compress | 1 | main-B (0) | 43.2 / 45.5 / 56.0 | 224 | 2256 / 5464 | 0.83 / 3.17 | 60 / 69 | control |
| wan | compress | 1 | candidate (1) | 43.0 / 47.1 / 60.0 | 228 | 2308 / 4844 | 0.8 / 1.1 | 54 / 58 | fps_tradeoff_review_required |
| wan | bandwidth | 0 | ssh (0) | 31.7 / 46.2 / 60.0 | 5793 | 7724 / 18824 | 0.25 / 0.27 | 53 / 58 | control |
| wan | bandwidth | 0 | main-A (1) | 14.1 / 25.9 / 29.2 | 1155 | 2692 / 5992 | 0.66 / 2.94 | 56 / 74 | control |
| wan | bandwidth | 0 | main-B (2) | 41.0 / 44.2 / 54.6 | 2132 | 2668 / 5944 | 1.18 / 5.09 | 63 / 72 | control |
| wan | bandwidth | 0 | candidate (3) | 34.8 / 45.4 / 54.4 | 2130 | 2740 / 5944 | 1.19 / 4.95 | 62 / 72 | fps_tradeoff_review_required |
| wan | bandwidth | 1 | ssh (2) | 26.5 / 46.2 / 60.0 | 5791 | 7652 / 18836 | 0.28 / 0.3 | 54 / 88 | control |
| wan | bandwidth | 1 | main-A (3) | 40.2 / 45.4 / 54.5 | 2131 | 2704 / 5964 | 1.24 / 5.23 | 63 / 72 | control |
| wan | bandwidth | 1 | main-B (0) | 14.2 / 25.4 / 29.2 | 1150 | 2692 / 5940 | 0.71 / 3.04 | 56 / 74 | control |
| wan | bandwidth | 1 | candidate (1) | 37.3 / 45.5 / 54.6 | 2135 | 2736 / 5944 | 1.27 / 5.28 | 63 / 71 | fps_tradeoff_review_required |
| highlat | compress | 0 | ssh (0) | 25.5 / 46.2 / 60.0 | 2510 | 7640 / 18832 | 0.26 / 0.29 | 152 / 230 | control |
| highlat | compress | 0 | main-A (1) | 25.8 / 27.5 / 40.4 | 160 | 2692 / 5908 | 0.66 / 2.54 | 155 / 170 | control |
| highlat | compress | 0 | main-B (2) | 42.2 / 45.2 / 54.7 | 218 | 2676 / 5956 | 0.88 / 3.09 | 161 / 170 | control |
| highlat | compress | 0 | candidate (3) | 42.0 / 45.7 / 54.6 | 218 | 2776 / 5924 | 0.9 / 3.25 | 161 / 169 | fps_tradeoff_review_required |
| highlat | compress | 1 | ssh (2) | 22.5 / 43.5 / 60.0 | 2502 | 7592 / 18908 | 0.26 / 0.29 | 152 / 243 | control |
| highlat | compress | 1 | main-A (3) | 24.8 / 28.2 / 40.4 | 163 | 2692 / 5904 | 0.64 / 2.52 | 154 / 171 | control |
| highlat | compress | 1 | main-B (0) | 40.2 / 45.7 / 54.7 | 218 | 2712 / 5932 | 0.86 / 3.26 | 161 / 170 | control |
| highlat | compress | 1 | candidate (1) | 43.4 / 46.4 / 54.4 | 217 | 2776 / 5916 | 0.85 / 3.28 | 161 / 169 | fps_tradeoff_review_required |
| highlat | bandwidth | 0 | ssh (0) | 12.1 / 42.6 / 60.0 | 5791 | 8024 / 19080 | 0.24 / 0.27 | 154 / 495 | control |
| highlat | bandwidth | 0 | main-A (1) | 11.7 / 45.2 / 50.4 | 1983 | 2740 / 5996 | 1.19 / 4.97 | 163 / 173 | control |
| highlat | bandwidth | 0 | main-B (2) | 10.7 / 44.6 / 49.9 | 1984 | 2732 / 6064 | 1.24 / 4.85 | 163 / 173 | control |
| highlat | bandwidth | 0 | candidate (3) | 12.4 / 29.9 / 47.7 | 1896 | 2768 / 6048 | 1.15 / 4.86 | 162 / 173 | fps_tradeoff_review_required |
| highlat | bandwidth | 1 | ssh (2) | 12.1 / 43.1 / 60.0 | 5799 | 8028 / 19044 | 0.26 / 0.28 | 154 / 496 | control |
| highlat | bandwidth | 1 | main-A (3) | 9.0 / 42.6 / 48.9 | 1954 | 2780 / 6032 | 1.17 / 4.86 | 164 / 173 | control |
| highlat | bandwidth | 1 | main-B (0) | 9.7 / 44.7 / 49.4 | 1974 | 2792 / 6040 | 1.19 / 4.88 | 163 / 172 | control |
| highlat | bandwidth | 1 | candidate (1) | 12.5 / 44.2 / 49.7 | 1983 | 2804 / 6024 | 1.2 / 4.94 | 163 / 172 | fps_tradeoff_review_required |
