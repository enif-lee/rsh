# Per-repetition acceptance evidence

Each row retains one measured role in one repetition. No pooling, noise tolerance or automatic pass is applied. Observed deficits do not establish policy causality; see JSON for exact missing metrics, FPS gaps and resource increases.

| Network | Scene | Rep | Role (slot) | FPS avg / 1% / 5% | Network kbit/s | RSS client / server KiB | CPU client / server % | Age median / p95 ms | Assessment |
|---|---|---:|---|---|---:|---|---|---|---|
| lan | compress | 0 | ssh (0) | 60.0 / 47.7 / 50.5 | 2478 | 7548 / 18832 | 0.34 / 0.44 | 1 / 4 | control |
| lan | compress | 0 | main-A (1) | 60.0 / 43.7 / 46.8 | 228 | 2620 / 5340 | 1.0 / 1.37 | 3 / 6 | control |
| lan | compress | 0 | main-B (2) | 60.0 / 43.5 / 47.6 | 228 | 2636 / 5340 | 1.01 / 1.35 | 3 / 6 | control |
| lan | compress | 0 | candidate (3) | 60.0 / 41.5 / 45.8 | 227 | 2628 / 5368 | 1.03 / 1.38 | 3 / 7 | observed_fps_gap_not_accepted |
| lan | compress | 1 | ssh (2) | 60.0 / 45.3 / 49.5 | 2479 | 7544 / 18800 | 0.34 / 0.45 | 1 / 4 | control |
| lan | compress | 1 | main-A (3) | 60.0 / 39.3 / 45.1 | 227 | 2620 / 5816 | 1.06 / 1.44 | 4 / 7 | control |
| lan | compress | 1 | main-B (0) | 60.0 / 43.2 / 46.7 | 227 | 2632 / 5820 | 1.07 / 1.44 | 4 / 7 | control |
| lan | compress | 1 | candidate (1) | 59.9 / 41.2 / 45.8 | 227 | 2636 / 5828 | 1.11 / 1.49 | 4 / 8 | observed_fps_gap_not_accepted |
| lan | bandwidth | 0 | ssh (0) | 60.0 / 43.3 / 49.0 | 5795 | 7552 / 18816 | 0.29 / 0.35 | 2 / 5 | control |
| lan | bandwidth | 0 | main-A (1) | 59.7 / 43.3 / 46.2 | 2721 | 2688 / 5880 | 1.36 / 2.1 | 5 / 9 | control |
| lan | bandwidth | 0 | main-B (2) | 59.6 / 40.4 / 45.2 | 2719 | 2700 / 5864 | 1.41 / 2.01 | 6 / 10 | control |
| lan | bandwidth | 0 | candidate (3) | 59.6 / 40.8 / 45.7 | 2716 | 2736 / 5900 | 1.75 / 1.99 | 5 / 9 | observed_fps_gap_not_accepted |
| lan | bandwidth | 1 | ssh (2) | 60.0 / 44.2 / 49.4 | 5777 | 7544 / 18796 | 0.21 / 0.26 | 2 / 4 | control |
| lan | bandwidth | 1 | main-A (3) | 60.0 / 42.1 / 45.5 | 2716 | 2628 / 5824 | 1.29 / 1.72 | 6 / 9 | control |
| lan | bandwidth | 1 | main-B (0) | 60.0 / 42.2 / 46.2 | 2717 | 2628 / 5828 | 1.25 / 1.63 | 5 / 9 | control |
| lan | bandwidth | 1 | candidate (1) | 60.0 / 41.4 / 45.8 | 2721 | 2652 / 5860 | 1.27 / 1.65 | 5 / 8 | observed_fps_gap_not_accepted |
| wan | compress | 0 | ssh (0) | 60.0 / 32.7 / 46.0 | 2509 | 7564 / 18828 | 0.3 / 0.39 | 52 / 57 | control |
| wan | compress | 0 | main-A (1) | 56.0 / 39.0 / 44.9 | 223 | 2652 / 6024 | 0.96 / 3.47 | 60 / 69 | control |
| wan | compress | 0 | main-B (2) | 55.2 / 40.8 / 44.8 | 219 | 2644 / 6008 | 0.99 / 3.32 | 60 / 69 | control |
| wan | compress | 0 | candidate (3) | 60.0 / 36.0 / 44.5 | 229 | 2628 / 5824 | 0.97 / 1.7 | 55 / 60 | observed_fps_gap_not_accepted |
| wan | compress | 1 | ssh (2) | 60.0 / 39.9 / 46.3 | 2512 | 7552 / 18804 | 0.3 / 0.31 | 52 / 56 | control |
| wan | compress | 1 | main-A (3) | 56.1 / 41.8 / 46.4 | 223 | 2660 / 6088 | 0.88 / 3.24 | 60 / 69 | control |
| wan | compress | 1 | main-B (0) | 55.5 / 34.7 / 45.9 | 220 | 2656 / 6020 | 0.89 / 3.3 | 60 / 69 | control |
| wan | compress | 1 | candidate (1) | 59.7 / 43.7 / 46.5 | 227 | 2636 / 5824 | 0.85 / 1.15 | 54 / 58 | observed_fps_gap_not_accepted |
| wan | bandwidth | 0 | ssh (0) | 60.0 / 39.9 / 47.6 | 5796 | 7720 / 18812 | 0.24 / 0.25 | 53 / 57 | control |
| wan | bandwidth | 0 | main-A (1) | 54.2 / 39.0 / 45.0 | 2122 | 2696 / 6084 | 1.13 / 4.89 | 62 / 71 | control |
| wan | bandwidth | 0 | main-B (2) | 54.6 / 42.6 / 45.7 | 2136 | 2672 / 6084 | 1.13 / 4.92 | 62 / 71 | control |
| wan | bandwidth | 0 | candidate (3) | 54.4 / 27.8 / 45.0 | 2136 | 2708 / 6132 | 1.16 / 4.91 | 63 / 72 | observed_fps_gap_not_accepted |
| wan | bandwidth | 1 | ssh (2) | 60.0 / 29.4 / 45.6 | 5787 | 7860 / 18828 | 0.24 / 0.25 | 53 / 75 | control |
| wan | bandwidth | 1 | main-A (3) | 55.1 / 31.3 / 46.1 | 2152 | 2672 / 6084 | 1.15 / 5.01 | 63 / 71 | control |
| wan | bandwidth | 1 | main-B (0) | 54.2 / 28.0 / 44.9 | 2126 | 2692 / 6292 | 1.25 / 5.17 | 63 / 72 | control |
| wan | bandwidth | 1 | candidate (1) | 54.6 / 32.9 / 45.4 | 2137 | 2688 / 6076 | 1.29 / 5.11 | 63 / 73 | observed_fps_gap_not_accepted |
| highlat | compress | 0 | ssh (0) | 60.0 / 23.4 / 46.1 | 2506 | 7624 / 18928 | 0.26 / 0.29 | 152 / 242 | control |
| highlat | compress | 0 | main-A (1) | 54.2 / 35.4 / 44.9 | 216 | 2688 / 6084 | 0.83 / 3.14 | 161 / 170 | control |
| highlat | compress | 0 | main-B (2) | 54.6 / 33.5 / 45.1 | 217 | 2680 / 6036 | 0.83 / 3.33 | 161 / 169 | control |
| highlat | compress | 0 | candidate (3) | 54.6 / 37.3 / 45.3 | 219 | 2688 / 6040 | 0.87 / 3.18 | 161 / 170 | observed_fps_gap_not_accepted |
| highlat | compress | 1 | ssh (2) | 60.0 / 24.3 / 45.4 | 2506 | 7604 / 18964 | 0.35 / 0.32 | 152 / 254 | control |
| highlat | compress | 1 | main-A (3) | 54.5 / 35.9 / 45.8 | 217 | 2700 / 6056 | 0.94 / 3.26 | 161 / 169 | control |
| highlat | compress | 1 | main-B (0) | 54.6 / 39.2 / 44.7 | 217 | 2676 / 6040 | 0.9 / 3.16 | 162 / 170 | control |
| highlat | compress | 1 | candidate (1) | 54.2 / 39.8 / 45.6 | 217 | 2700 / 6084 | 0.9 / 3.2 | 161 / 170 | observed_fps_gap_not_accepted |
| highlat | bandwidth | 0 | ssh (0) | 60.0 / 12.4 / 42.7 | 5797 | 8036 / 19048 | 0.23 / 0.24 | 154 / 504 | control |
| highlat | bandwidth | 0 | main-A (1) | 49.7 / 10.0 / 44.1 | 1979 | 2664 / 6172 | 1.06 / 4.54 | 163 / 173 | control |
| highlat | bandwidth | 0 | main-B (2) | 49.7 / 15.9 / 43.4 | 1982 | 2736 / 6168 | 1.07 / 4.68 | 163 / 174 | control |
| highlat | bandwidth | 0 | candidate (3) | 49.6 / 9.8 / 43.3 | 1969 | 2736 / 6188 | 1.08 / 4.38 | 162 / 172 | observed_fps_gap_not_accepted |
| highlat | bandwidth | 1 | ssh (2) | 60.0 / 11.1 / 43.0 | 5796 | 8048 / 19148 | 0.24 / 0.26 | 153 / 491 | control |
| highlat | bandwidth | 1 | main-A (3) | 16.0 / 6.6 / 10.0 | 670 | 2740 / 6080 | 0.38 / 1.88 | 156 / 175 | control |
| highlat | bandwidth | 1 | main-B (0) | 49.6 / 13.0 / 45.5 | 1984 | 2760 / 6180 | 1.07 / 4.57 | 163 / 172 | control |
| highlat | bandwidth | 1 | candidate (1) | 49.7 / 20.0 / 44.2 | 1990 | 2768 / 6192 | 1.1 / 4.58 | 163 / 172 | observed_fps_gap_not_accepted |
