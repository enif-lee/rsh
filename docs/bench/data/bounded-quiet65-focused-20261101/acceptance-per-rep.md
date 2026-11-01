# Focused per-repetition comparison evidence

Each row retains one measured role in one repetition. No pooling, noise tolerance or automatic pass is applied. Observed deficits do not establish policy causality; see JSON for exact missing metrics, FPS gaps and resource increases.

| Network | Scene | Rep | Role (slot) | FPS 1% / 5% / avg | Network kbit/s | RSS client / server KiB | CPU client / server % | Age median / p95 ms | Assessment |
|---|---|---:|---|---|---:|---|---|---|---|
| wan | bandwidth | 0 | ssh (0) | 29.6 / 48.0 / 60.0 | 5796 | 7632 / 18808 | 0.2 / 0.21 | 53 / 57 | control |
| wan | bandwidth | 0 | main-A (1) | 29.1 / 45.2 / 58.0 | 2268 | 2728 / 6008 | 1.07 / 4.62 | 55 / 65 | control |
| wan | bandwidth | 0 | main-B (2) | 29.4 / 44.9 / 57.4 | 2238 | 2692 / 6040 | 1.07 / 4.62 | 58 / 64 | control |
| wan | bandwidth | 0 | candidate (3) | 43.4 / 48.6 / 58.6 | 2280 | 2704 / 5968 | 1.09 / 4.76 | 59 / 63 | fps_tradeoff_review_required |
| wan | bandwidth | 1 | ssh (3) | 36.3 / 46.6 / 60.0 | 5800 | 7396 / 18568 | 0.21 / 0.22 | 53 / 58 | control |
| wan | bandwidth | 1 | main-A (2) | 28.1 / 46.1 / 57.7 | 2277 | 2164 / 5068 | 1.12 / 4.71 | 55 / 64 | control |
| wan | bandwidth | 1 | main-B (1) | 30.9 / 46.4 / 58.0 | 2277 | 1624 / 5092 | 1.09 / 4.68 | 55 / 65 | control |
| wan | bandwidth | 1 | candidate (0) | 41.5 / 46.8 / 58.4 | 2285 | 2248 / 5396 | 1.11 / 4.83 | 58 / 63 | fps_tradeoff_review_required |
| highlat | bandwidth | 0 | ssh (0) | 12.4 / 43.5 / 60.0 | 5799 | 8064 / 19016 | 0.21 / 0.23 | 154 / 497 | control |
| highlat | bandwidth | 0 | main-A (1) | 24.3 / 45.2 / 52.1 | 2080 | 2828 / 6124 | 1.05 / 4.37 | 156 / 167 | control |
| highlat | bandwidth | 0 | main-B (2) | 16.1 / 47.0 / 53.2 | 2124 | 2800 / 6092 | 1.04 / 4.48 | 156 / 167 | control |
| highlat | bandwidth | 0 | candidate (3) | 37.5 / 47.4 / 53.2 | 2123 | 2780 / 6028 | 1.07 / 4.55 | 160 / 168 | fps_tradeoff_review_required |
| highlat | bandwidth | 1 | ssh (3) | 13.6 / 42.8 / 60.0 | 5795 | 8012 / 19124 | 0.22 / 0.24 | 153 / 502 | control |
| highlat | bandwidth | 1 | main-A (2) | 17.1 / 45.5 / 53.2 | 2126 | 2796 / 6128 | 1.1 / 4.55 | 159 / 171 | control |
| highlat | bandwidth | 1 | main-B (1) | 30.0 / 47.0 / 53.1 | 2114 | 2784 / 6140 | 1.09 / 4.51 | 154 / 169 | control |
| highlat | bandwidth | 1 | candidate (0) | 20.7 / 47.3 / 53.6 | 2124 | 2740 / 6040 | 1.13 / 4.63 | 159 / 163 | fps_tradeoff_review_required |
