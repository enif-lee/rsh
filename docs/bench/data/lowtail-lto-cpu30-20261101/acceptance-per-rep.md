# Per-repetition acceptance evidence

Each row retains one measured role in one repetition. No pooling, noise tolerance or automatic pass is applied. Observed deficits do not establish policy causality; see JSON for exact missing metrics, FPS gaps and resource increases.

| Network | Scene | Rep | Role (slot) | FPS avg / 1% / 5% | Network kbit/s | RSS client / server KiB | CPU client / server % | Age median / p95 ms | Assessment |
|---|---|---:|---|---|---:|---|---|---|---|
| lan | compress | 0 | ssh (0) | 60.0 / 47.7 / 51.4 | 2472 | 7580 / 18864 | 0.35 / 0.49 | 1 / 4 | control |
| lan | compress | 0 | main-A (1) | 60.0 / 45.9 / 48.2 | 227 | 2620 / 5816 | 1.16 / 1.57 | 4 / 7 | control |
| lan | compress | 0 | main-B (2) | 60.0 / 45.7 / 49.1 | 227 | 2628 / 5784 | 1.18 / 1.59 | 4 / 7 | control |
| lan | compress | 0 | candidate (3) | 60.0 / 45.4 / 49.2 | 227 | 2556 / 5628 | 1.19 / 1.57 | 4 / 7 | observed_fps_gap_not_accepted |
| lan | compress | 1 | ssh (2) | 60.0 / 46.1 / 51.0 | 2479 | 7400 / 18764 | 0.4 / 0.37 | 1 / 3 | control |
| lan | compress | 1 | main-A (3) | 59.7 / 43.8 / 49.0 | 227 | 2380 / 5340 | 1.15 / 1.55 | 4 / 7 | control |
| lan | compress | 1 | main-B (0) | 60.0 / 43.6 / 48.7 | 227 | 2368 / 5376 | 0.93 / 1.49 | 4 / 6 | control |
| lan | compress | 1 | candidate (1) | 59.9 / 45.3 / 48.8 | 228 | 2376 / 5348 | 1.03 / 1.37 | 4 / 6 | observed_fps_gap_not_accepted |
| lan | bandwidth | 0 | ssh (0) | 60.0 / 48.4 / 51.4 | 5777 | 7560 / 18820 | 0.2 / 0.26 | 2 / 4 | control |
| lan | bandwidth | 0 | main-A (1) | 60.0 / 43.8 / 47.9 | 2717 | 2628 / 5820 | 1.19 / 1.57 | 5 / 8 | control |
| lan | bandwidth | 0 | main-B (2) | 60.0 / 44.9 / 47.6 | 2718 | 2664 / 5856 | 1.18 / 1.6 | 5 / 8 | control |
| lan | bandwidth | 0 | candidate (3) | 60.0 / 44.9 / 47.6 | 2717 | 2572 / 5656 | 1.18 / 1.66 | 5 / 8 | observed_fps_gap_not_accepted |
| lan | bandwidth | 1 | ssh (2) | 60.0 / 48.2 / 51.6 | 5789 | 7572 / 18808 | 0.22 / 0.29 | 2 / 4 | control |
| lan | bandwidth | 1 | main-A (3) | 60.0 / 43.5 / 47.3 | 2718 | 2644 / 5844 | 1.33 / 1.68 | 5 / 9 | control |
| lan | bandwidth | 1 | main-B (0) | 59.9 / 43.0 / 48.0 | 2717 | 2688 / 5856 | 1.2 / 1.72 | 5 / 8 | control |
| lan | bandwidth | 1 | candidate (1) | 59.9 / 44.6 / 47.4 | 2718 | 2580 / 5712 | 1.2 / 1.61 | 5 / 8 | observed_fps_gap_not_accepted |
| wan | compress | 0 | ssh (0) | 60.0 / 43.1 / 48.1 | 2504 | 7444 / 18824 | 0.38 / 0.4 | 52 / 55 | control |
| wan | compress | 0 | main-A (1) | 56.4 / 42.7 / 47.3 | 225 | 2204 / 5572 | 1.06 / 3.62 | 60 / 69 | control |
| wan | compress | 0 | main-B (2) | 56.5 / 44.2 / 47.9 | 226 | 2208 / 5352 | 1.04 / 3.53 | 60 / 69 | control |
| wan | compress | 0 | candidate (3) | 56.1 / 44.1 / 46.6 | 223 | 2240 / 5592 | 1.05 / 3.42 | 61 / 69 | observed_fps_gap_not_accepted |
| wan | compress | 1 | ssh (2) | 60.0 / 46.4 / 48.9 | 2511 | 7544 / 18828 | 0.32 / 0.36 | 52 / 55 | control |
| wan | compress | 1 | main-A (3) | 56.4 / 42.6 / 47.2 | 225 | 2672 / 6072 | 1.04 / 3.49 | 60 / 69 | control |
| wan | compress | 1 | main-B (0) | 55.5 / 43.1 / 46.8 | 221 | 2640 / 6016 | 1.02 / 3.43 | 60 / 68 | control |
| wan | compress | 1 | candidate (1) | 56.2 / 42.6 / 47.3 | 224 | 2564 / 5856 | 1.07 / 3.59 | 60 / 69 | observed_fps_gap_not_accepted |
| wan | bandwidth | 0 | ssh (0) | 60.0 / 40.9 / 47.4 | 5800 | 7832 / 18824 | 0.29 / 0.3 | 53 / 57 | control |
| wan | bandwidth | 0 | main-A (1) | 54.2 / 27.0 / 45.4 | 2134 | 2704 / 6088 | 1.17 / 4.71 | 62 / 71 | control |
| wan | bandwidth | 0 | main-B (2) | 54.9 / 40.1 / 45.7 | 2139 | 2708 / 6088 | 1.15 / 4.78 | 62 / 71 | control |
| wan | bandwidth | 0 | candidate (3) | 29.4 / 14.8 / 26.3 | 1155 | 2624 / 5896 | 0.67 / 2.73 | 55 / 74 | observed_fps_gap_not_accepted |
| wan | bandwidth | 1 | ssh (2) | 60.0 / 28.5 / 47.3 | 5796 | 7648 / 18820 | 0.28 / 0.29 | 53 / 91 | control |
| wan | bandwidth | 1 | main-A (3) | 54.6 / 29.3 / 45.3 | 2137 | 2712 / 6088 | 1.18 / 4.76 | 62 / 71 | control |
| wan | bandwidth | 1 | main-B (0) | 54.9 / 30.6 / 46.4 | 2145 | 2704 / 6072 | 1.12 / 4.69 | 62 / 72 | control |
| wan | bandwidth | 1 | candidate (1) | 29.5 / 14.5 / 26.4 | 1160 | 2624 / 5880 | 0.64 / 2.84 | 55 / 73 | observed_fps_gap_not_accepted |
| highlat | compress | 0 | ssh (0) | 60.0 / 23.6 / 45.1 | 2513 | 7628 / 18824 | 0.44 / 0.52 | 152 / 233 | control |
| highlat | compress | 0 | main-A (1) | 36.9 / 24.0 / 27.2 | 147 | 2684 / 6076 | 0.88 / 2.81 | 154 / 172 | control |
| highlat | compress | 0 | main-B (2) | 54.5 / 33.9 / 44.7 | 217 | 2712 / 6044 | 1.37 / 4.15 | 162 / 171 | control |
| highlat | compress | 0 | candidate (3) | 34.2 / 25.4 / 27.0 | 134 | 2592 / 5832 | 0.85 / 2.81 | 154 / 173 | observed_fps_gap_not_accepted |
| highlat | compress | 1 | ssh (2) | 60.0 / 22.9 / 45.8 | 2506 | 7628 / 18788 | 0.32 / 0.35 | 152 / 239 | control |
| highlat | compress | 1 | main-A (3) | 55.0 / 42.3 / 45.7 | 218 | 2672 / 6060 | 0.98 / 3.49 | 161 / 169 | control |
| highlat | compress | 1 | main-B (0) | 52.0 / 27.8 / 31.1 | 207 | 2716 / 6036 | 0.94 / 3.4 | 161 / 170 | control |
| highlat | compress | 1 | candidate (1) | 54.5 / 40.2 / 45.7 | 217 | 2628 / 5884 | 1.01 / 3.37 | 162 / 170 | observed_fps_gap_not_accepted |
| highlat | bandwidth | 0 | ssh (0) | 60.0 / 15.1 / 44.2 | 5797 | 8004 / 18936 | 0.23 / 0.23 | 154 / 487 | control |
| highlat | bandwidth | 0 | main-A (1) | 49.4 / 15.8 / 45.6 | 1977 | 2792 / 6180 | 1.01 / 4.33 | 163 / 171 | control |
| highlat | bandwidth | 0 | main-B (2) | 49.6 / 8.0 / 43.4 | 1972 | 2716 / 6180 | 1.02 / 4.27 | 163 / 174 | control |
| highlat | bandwidth | 0 | candidate (3) | 49.9 / 11.7 / 44.6 | 1984 | 2632 / 5960 | 1.0 / 4.17 | 162 / 172 | observed_fps_gap_not_accepted |
| highlat | bandwidth | 1 | ssh (2) | 60.0 / 12.1 / 43.8 | 5794 | 7996 / 18996 | 0.27 / 0.28 | 154 / 488 | control |
| highlat | bandwidth | 1 | main-A (3) | 16.4 / 6.1 / 9.2 | 660 | 2680 / 6060 | 0.42 / 2.04 | 156 / 176 | control |
| highlat | bandwidth | 1 | main-B (0) | 16.2 / 7.0 / 9.6 | 670 | 2764 / 6088 | 0.43 / 2.04 | 157 / 175 | control |
| highlat | bandwidth | 1 | candidate (1) | 15.9 / 7.3 / 9.7 | 654 | 2656 / 5884 | 0.41 / 2.02 | 156 / 175 | observed_fps_gap_not_accepted |
