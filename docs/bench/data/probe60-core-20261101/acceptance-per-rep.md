# Per-repetition acceptance evidence

Each row retains one measured role in one repetition. No pooling, noise tolerance or automatic pass is applied. Observed deficits do not establish policy causality; see JSON for exact missing metrics, FPS gaps and resource increases.

| Network | Scene | Rep | Role (slot) | FPS avg / 1% / 5% | Network kbit/s | RSS client / server KiB | CPU client / server % | Age median / p95 ms | Assessment |
|---|---|---:|---|---|---:|---|---|---|---|
| lan | compress | 0 | ssh (0) | 60.0 / 46.0 / 50.0 | 2483 | 7556 / 18808 | 0.43 / 0.57 | 1 / 4 | control |
| lan | compress | 0 | main-A (1) | 59.9 / 37.9 / 48.2 | 227 | 2624 / 5788 | 1.25 / 1.72 | 3 / 7 | control |
| lan | compress | 0 | main-B (2) | 59.9 / 39.8 / 48.5 | 227 | 2644 / 5828 | 1.26 / 1.67 | 4 / 8 | control |
| lan | compress | 0 | candidate (3) | 59.7 / 43.0 / 48.8 | 227 | 2656 / 5788 | 1.34 / 1.68 | 4 / 7 | observed_fps_gap_not_accepted |
| lan | compress | 1 | ssh (2) | 60.0 / 45.8 / 50.5 | 2478 | 7540 / 18816 | 0.36 / 0.52 | 1 / 4 | control |
| lan | compress | 1 | main-A (3) | 60.0 / 42.6 / 48.1 | 227 | 2636 / 5792 | 1.17 / 1.65 | 4 / 7 | control |
| lan | compress | 1 | main-B (0) | 60.0 / 42.7 / 46.8 | 227 | 2636 / 5804 | 1.16 / 1.62 | 3 / 8 | control |
| lan | compress | 1 | candidate (1) | 59.7 / 42.3 / 47.8 | 227 | 2636 / 5804 | 1.16 / 1.69 | 4 / 7 | observed_fps_gap_not_accepted |
| lan | bandwidth | 0 | ssh (0) | 60.0 / 47.6 / 50.3 | 5780 | 7560 / 18812 | 0.26 / 0.32 | 2 / 4 | control |
| lan | bandwidth | 0 | main-A (1) | 60.0 / 44.2 / 48.5 | 2720 | 2648 / 5824 | 1.33 / 1.76 | 5 / 8 | control |
| lan | bandwidth | 0 | main-B (2) | 60.0 / 44.2 / 47.6 | 2712 | 2644 / 5840 | 1.24 / 1.62 | 5 / 8 | control |
| lan | bandwidth | 0 | candidate (3) | 60.0 / 44.6 / 47.6 | 2716 | 2628 / 5848 | 1.25 / 1.71 | 5 / 8 | observed_fps_gap_not_accepted |
| lan | bandwidth | 1 | ssh (2) | 60.0 / 45.0 / 49.2 | 5776 | 7568 / 18828 | 0.31 / 0.44 | 3 / 6 | control |
| lan | bandwidth | 1 | main-A (3) | 60.0 / 43.0 / 48.5 | 2716 | 1720 / 4732 | 1.67 / 2.16 | 5 / 9 | control |
| lan | bandwidth | 1 | main-B (0) | 60.0 / 42.1 / 46.9 | 2717 | 1928 / 4912 | 1.42 / 1.98 | 6 / 9 | control |
| lan | bandwidth | 1 | candidate (1) | 59.9 / 44.0 / 47.4 | 2717 | 2432 / 4752 | 1.62 / 2.23 | 5 / 9 | observed_fps_gap_not_accepted |
| wan | compress | 0 | ssh (0) | 60.0 / 30.7 / 48.5 | 2509 | 7560 / 18800 | 0.34 / 0.44 | 52 / 56 | control |
| wan | compress | 0 | main-A (1) | 55.4 / 30.3 / 46.5 | 220 | 2676 / 6012 | 1.04 / 3.57 | 60 / 70 | control |
| wan | compress | 0 | main-B (2) | 33.9 / 22.5 / 27.4 | 128 | 2648 / 6044 | 0.68 / 2.84 | 53 / 71 | control |
| wan | compress | 0 | candidate (3) | 56.4 / 30.1 / 46.9 | 226 | 2656 / 6040 | 1.03 / 3.68 | 60 / 69 | observed_fps_gap_not_accepted |
| wan | compress | 1 | ssh (2) | 60.0 / 46.5 / 49.3 | 2505 | 7556 / 18884 | 0.23 / 0.25 | 52 / 54 | control |
| wan | compress | 1 | main-A (3) | 56.5 / 44.8 / 47.2 | 225 | 2652 / 5992 | 0.8 / 3.05 | 60 / 69 | control |
| wan | compress | 1 | main-B (0) | 56.7 / 44.8 / 47.4 | 227 | 2636 / 5996 | 0.78 / 3.07 | 60 / 69 | control |
| wan | compress | 1 | candidate (1) | 56.2 / 45.0 / 47.8 | 223 | 2640 / 6056 | 0.8 / 2.98 | 59 / 68 | observed_fps_gap_not_accepted |
| wan | bandwidth | 0 | ssh (0) | 60.0 / 25.8 / 45.8 | 5792 | 7348 / 18796 | 0.42 / 0.37 | 54 / 82 | control |
| wan | bandwidth | 0 | main-A (1) | 55.1 / 27.8 / 46.3 | 2150 | 1864 / 5452 | 1.18 / 4.8 | 63 / 71 | control |
| wan | bandwidth | 0 | main-B (2) | 54.5 / 31.6 / 46.4 | 2127 | 1760 / 5316 | 1.16 / 4.64 | 62 / 70 | control |
| wan | bandwidth | 0 | candidate (3) | 54.9 / 41.9 / 46.9 | 2151 | 2080 / 5520 | 1.19 / 4.85 | 63 / 71 | observed_fps_gap_not_accepted |
| wan | bandwidth | 1 | ssh (2) | 60.0 / 24.3 / 45.0 | 5792 | 7716 / 18680 | 0.32 / 0.34 | 54 / 67 | control |
| wan | bandwidth | 1 | main-A (3) | 29.2 / 11.5 / 24.0 | 1157 | 2728 / 6080 | 0.99 / 3.75 | 56 / 75 | control |
| wan | bandwidth | 1 | main-B (0) | 29.0 / 11.3 / 23.1 | 1147 | 2716 / 6184 | 0.85 / 3.31 | 56 / 75 | control |
| wan | bandwidth | 1 | candidate (1) | 53.9 / 17.7 / 43.4 | 2102 | 2684 / 6084 | 1.46 / 5.81 | 63 / 73 | observed_fps_gap_not_accepted |
| highlat | compress | 0 | ssh (0) | 60.0 / 23.0 / 45.6 | 2508 | 7612 / 18704 | 0.31 / 0.36 | 152 / 234 | control |
| highlat | compress | 0 | main-A (1) | 46.0 / 25.6 / 28.8 | 182 | 2696 / 6036 | 0.73 / 2.67 | 160 / 170 | control |
| highlat | compress | 0 | main-B (2) | 54.9 / 41.7 / 45.3 | 219 | 2696 / 6060 | 0.91 / 3.16 | 161 / 170 | control |
| highlat | compress | 0 | candidate (3) | 55.0 / 39.2 / 45.4 | 219 | 2680 / 6036 | 0.9 / 3.07 | 161 / 170 | observed_fps_gap_not_accepted |
| highlat | compress | 1 | ssh (2) | 60.0 / 22.6 / 46.7 | 2510 | 7620 / 18732 | 0.25 / 0.28 | 152 / 237 | control |
| highlat | compress | 1 | main-A (3) | 37.6 / 26.5 / 28.0 | 147 | 2700 / 6040 | 0.57 / 2.28 | 154 / 170 | control |
| highlat | compress | 1 | main-B (0) | 55.0 / 44.1 / 47.3 | 219 | 2692 / 6044 | 0.81 / 3.2 | 161 / 169 | control |
| highlat | compress | 1 | candidate (1) | 55.2 / 41.6 / 46.9 | 219 | 2736 / 6076 | 0.82 / 3.09 | 161 / 169 | observed_fps_gap_not_accepted |
| highlat | bandwidth | 0 | ssh (0) | 60.0 / 12.4 / 44.3 | 5799 | 7196 / 18376 | 0.24 / 0.23 | 153 / 498 | control |
| highlat | bandwidth | 0 | main-A (1) | 50.2 / 11.3 / 45.8 | 1997 | 2484 / 5744 | 1.06 / 4.43 | 163 / 172 | control |
| highlat | bandwidth | 0 | main-B (2) | 50.2 / 17.5 / 45.9 | 1999 | 2484 / 5748 | 1.05 / 4.41 | 163 / 172 | control |
| highlat | bandwidth | 0 | candidate (3) | 49.5 / 13.4 / 44.9 | 1982 | 2480 / 5796 | 1.04 / 4.46 | 162 / 172 | observed_fps_gap_not_accepted |
| highlat | bandwidth | 1 | ssh (2) | 60.0 / 12.0 / 46.3 | 5797 | 7916 / 18900 | 0.24 / 0.24 | 153 / 495 | control |
| highlat | bandwidth | 1 | main-A (3) | 50.0 / 14.4 / 45.0 | 1994 | 2736 / 6204 | 1.02 / 4.27 | 163 / 172 | control |
| highlat | bandwidth | 1 | main-B (0) | 49.5 / 10.7 / 45.6 | 1982 | 2728 / 6204 | 1.02 / 4.24 | 162 / 171 | control |
| highlat | bandwidth | 1 | candidate (1) | 50.1 / 22.4 / 45.3 | 1994 | 2740 / 6160 | 1.04 / 4.37 | 162 / 172 | observed_fps_gap_not_accepted |
