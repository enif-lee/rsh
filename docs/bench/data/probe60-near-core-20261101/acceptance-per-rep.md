# Per-repetition acceptance evidence

Each row retains one measured role in one repetition. No pooling, noise tolerance or automatic pass is applied. Observed deficits do not establish policy causality; see JSON for exact missing metrics, FPS gaps and resource increases.

| Network | Scene | Rep | Role (slot) | FPS avg / 1% / 5% | Network kbit/s | RSS client / server KiB | CPU client / server % | Age median / p95 ms | Assessment |
|---|---|---:|---|---|---:|---|---|---|---|
| lan | compress | 0 | ssh (0) | 60.0 / 45.5 / 51.3 | 2481 | 7592 / 18824 | 0.29 / 0.44 | 1 / 3 | control |
| lan | compress | 0 | main-A (1) | 60.0 / 47.0 / 49.2 | 227 | 2628 / 5820 | 1.05 / 1.26 | 4 / 7 | control |
| lan | compress | 0 | main-B (2) | 60.0 / 45.0 / 49.6 | 227 | 2624 / 5784 | 0.91 / 1.25 | 4 / 7 | control |
| lan | compress | 0 | candidate (3) | 60.0 / 45.9 / 49.8 | 227 | 2632 / 5784 | 0.92 / 1.28 | 3 / 6 | observed_fps_gap_not_accepted |
| lan | compress | 1 | ssh (2) | 60.0 / 43.3 / 49.7 | 2480 | 7560 / 18832 | 0.4 / 0.45 | 1 / 4 | control |
| lan | compress | 1 | main-A (3) | 59.7 / 41.9 / 48.4 | 227 | 2652 / 5792 | 1.05 / 1.45 | 4 / 6 | control |
| lan | compress | 1 | main-B (0) | 59.9 / 41.6 / 48.0 | 227 | 2616 / 5828 | 1.41 / 1.65 | 4 / 7 | control |
| lan | compress | 1 | candidate (1) | 59.9 / 40.5 / 47.4 | 227 | 2640 / 5828 | 1.12 / 1.61 | 4 / 8 | observed_fps_gap_not_accepted |
| lan | bandwidth | 0 | ssh (0) | 60.0 / 47.9 / 51.8 | 5791 | 7556 / 18808 | 0.18 / 0.23 | 2 / 4 | control |
| lan | bandwidth | 0 | main-A (1) | 60.0 / 44.9 / 47.6 | 2712 | 2668 / 5880 | 1.13 / 1.49 | 5 / 9 | control |
| lan | bandwidth | 0 | main-B (2) | 60.0 / 44.6 / 48.1 | 2712 | 2664 / 5844 | 1.13 / 1.49 | 5 / 8 | control |
| lan | bandwidth | 0 | candidate (3) | 60.0 / 44.2 / 48.5 | 2713 | 2628 / 5884 | 1.18 / 1.52 | 5 / 8 | observed_fps_gap_not_accepted |
| lan | bandwidth | 1 | ssh (2) | 60.0 / 41.9 / 50.1 | 5789 | 7564 / 18824 | 0.23 / 0.29 | 2 / 5 | control |
| lan | bandwidth | 1 | main-A (3) | 59.9 / 42.0 / 47.7 | 2717 | 2648 / 5832 | 1.29 / 1.73 | 5 / 8 | control |
| lan | bandwidth | 1 | main-B (0) | 59.9 / 41.4 / 47.3 | 2717 | 2628 / 5816 | 1.38 / 1.72 | 5 / 8 | control |
| lan | bandwidth | 1 | candidate (1) | 59.9 / 44.4 / 47.7 | 2717 | 2664 / 5832 | 1.33 / 1.78 | 6 / 9 | observed_fps_gap_not_accepted |
| wan | compress | 0 | ssh (0) | 60.0 / 42.0 / 47.8 | 2512 | 7568 / 18844 | 0.33 / 0.38 | 52 / 56 | control |
| wan | compress | 0 | main-A (1) | 56.4 / 41.0 / 46.7 | 224 | 2652 / 5996 | 1.01 / 3.27 | 60 / 68 | control |
| wan | compress | 0 | main-B (2) | 56.2 / 41.4 / 45.5 | 225 | 2648 / 6068 | 1.0 / 3.34 | 60 / 69 | control |
| wan | compress | 0 | candidate (3) | 56.4 / 40.9 / 45.7 | 225 | 2644 / 5992 | 0.98 / 3.2 | 60 / 69 | observed_fps_gap_not_accepted |
| wan | compress | 1 | ssh (2) | 60.0 / 38.8 / 48.3 | 2507 | 7568 / 18820 | 0.28 / 0.3 | 52 / 56 | control |
| wan | compress | 1 | main-A (3) | 56.5 / 43.9 / 47.1 | 225 | 2652 / 6032 | 0.83 / 3.13 | 59 / 69 | control |
| wan | compress | 1 | main-B (0) | 55.9 / 42.8 / 46.5 | 222 | 2644 / 5992 | 0.83 / 3.19 | 60 / 68 | control |
| wan | compress | 1 | candidate (1) | 56.0 / 41.8 / 46.9 | 223 | 2644 / 6040 | 0.86 / 3.11 | 60 / 68 | observed_fps_gap_not_accepted |
| wan | bandwidth | 0 | ssh (0) | 60.0 / 41.3 / 48.7 | 5798 | 7624 / 18812 | 0.24 / 0.22 | 53 / 56 | control |
| wan | bandwidth | 0 | main-A (1) | 54.6 / 43.7 / 46.7 | 2136 | 2688 / 6076 | 1.03 / 4.41 | 61 / 70 | control |
| wan | bandwidth | 0 | main-B (2) | 54.7 / 30.6 / 46.6 | 2148 | 2660 / 6080 | 1.04 / 4.44 | 63 / 71 | control |
| wan | bandwidth | 0 | candidate (3) | 55.1 / 42.5 / 46.7 | 2148 | 2708 / 6064 | 1.05 / 4.54 | 62 / 71 | observed_fps_gap_not_accepted |
| wan | bandwidth | 1 | ssh (2) | 60.0 / 38.0 / 47.6 | 5795 | 7608 / 18836 | 0.24 / 0.24 | 53 / 67 | control |
| wan | bandwidth | 1 | main-A (3) | 54.1 / 22.5 / 45.5 | 2125 | 2692 / 6100 | 1.11 / 4.59 | 62 / 71 | control |
| wan | bandwidth | 1 | main-B (0) | 54.6 / 23.7 / 47.1 | 2126 | 2704 / 6128 | 1.04 / 4.57 | 62 / 71 | control |
| wan | bandwidth | 1 | candidate (1) | 54.6 / 29.5 / 46.6 | 2139 | 2672 / 6072 | 1.42 / 4.53 | 62 / 71 | observed_fps_gap_not_accepted |
| highlat | compress | 0 | ssh (0) | 60.0 / 23.4 / 46.2 | 2507 | 7636 / 18940 | 0.25 / 0.27 | 152 / 235 | control |
| highlat | compress | 0 | main-A (1) | 54.9 / 41.3 / 45.9 | 217 | 2680 / 6040 | 0.83 / 3.06 | 160 / 169 | control |
| highlat | compress | 0 | main-B (2) | 55.0 / 37.9 / 46.2 | 218 | 2712 / 6144 | 0.83 / 3.08 | 161 / 169 | control |
| highlat | compress | 0 | candidate (3) | 50.4 / 27.6 / 30.3 | 204 | 2700 / 6012 | 0.79 / 2.87 | 160 / 170 | observed_fps_gap_not_accepted |
| highlat | compress | 1 | ssh (2) | 60.0 / 24.9 / 47.7 | 2513 | 7632 / 18964 | 0.27 / 0.28 | 152 / 237 | control |
| highlat | compress | 1 | main-A (3) | 55.0 / 41.3 / 46.1 | 218 | 2044 / 4800 | 0.85 / 3.03 | 161 / 169 | control |
| highlat | compress | 1 | main-B (0) | 51.0 / 28.3 / 30.2 | 204 | 1776 / 5264 | 0.74 / 2.75 | 159 / 170 | control |
| highlat | compress | 1 | candidate (1) | 55.1 / 44.0 / 47.4 | 219 | 1908 / 5504 | 0.9 / 2.98 | 160 / 169 | observed_fps_gap_not_accepted |
| highlat | bandwidth | 0 | ssh (0) | 60.0 / 13.0 / 45.3 | 5797 | 7956 / 19124 | 0.23 / 0.24 | 153 / 489 | control |
| highlat | bandwidth | 0 | main-A (1) | 27.5 / 5.0 / 23.6 | 1126 | 2708 / 6108 | 0.64 / 2.84 | 156 / 174 | control |
| highlat | bandwidth | 0 | main-B (2) | 50.0 / 11.3 / 46.3 | 1993 | 2740 / 6164 | 1.11 / 4.43 | 162 / 172 | control |
| highlat | bandwidth | 0 | candidate (3) | 49.9 / 19.4 / 45.2 | 1996 | 2732 / 6164 | 1.08 / 4.5 | 163 / 172 | observed_fps_gap_not_accepted |
| highlat | bandwidth | 1 | ssh (2) | 60.0 / 12.6 / 46.5 | 5792 | 8024 / 19108 | 0.22 / 0.23 | 153 / 491 | control |
| highlat | bandwidth | 1 | main-A (3) | 49.6 / 30.7 / 47.0 | 1984 | 2760 / 6188 | 0.98 / 4.18 | 162 / 171 | control |
| highlat | bandwidth | 1 | main-B (0) | 49.5 / 24.8 / 46.0 | 1989 | 2736 / 6160 | 1.02 / 4.24 | 163 / 172 | control |
| highlat | bandwidth | 1 | candidate (1) | 49.9 / 25.3 / 45.8 | 1994 | 2744 / 6164 | 1.02 / 4.27 | 162 / 171 | observed_fps_gap_not_accepted |
