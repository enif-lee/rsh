# Focused per-repetition comparison evidence

Each row retains one measured role in one repetition. No pooling, noise tolerance or automatic pass is applied. Observed deficits do not establish policy causality; see JSON for exact missing metrics, FPS gaps and resource increases.

| Network | Scene | Rep | Role (slot) | FPS 1% / 5% / avg | Network kbit/s | RSS client / server KiB | CPU client / server % | Age median / p95 ms | Assessment |
|---|---|---:|---|---|---:|---|---|---|---|
| lowbw | compress | 0 | ssh (0) | 6.4 / 7.1 / 23.6 | 977 | 7804 / 19644 | 0.17 / 0.63 | 2590 / 4679 | control |
| lowbw | compress | 0 | main-A (1) | 43.3 / 46.8 / 60.0 | 227 | 2696 / 5660 | 0.92 / 1.22 | 60 / 63 | control |
| lowbw | compress | 0 | main-B (2) | 42.5 / 46.0 / 60.0 | 228 | 2696 / 5716 | 0.89 / 1.22 | 59 / 62 | control |
| lowbw | compress | 0 | candidate (3) | 42.9 / 46.3 / 60.0 | 227 | 2624 / 5656 | 0.91 / 1.24 | 60 / 64 | fps_tradeoff_review_required |
| lowbw | compress | 1 | ssh (2) | 6.4 / 7.1 / 23.7 | 977 | 7676 / 19780 | 0.15 / 0.54 | 2613 / 4709 | control |
| lowbw | compress | 1 | main-A (3) | 41.5 / 45.4 / 60.0 | 228 | 2688 / 5712 | 0.83 / 1.14 | 60 / 63 | control |
| lowbw | compress | 1 | main-B (0) | 42.2 / 45.6 / 60.0 | 227 | 2700 / 5688 | 0.83 / 1.12 | 59 / 63 | control |
| lowbw | compress | 1 | candidate (1) | 44.9 / 47.9 / 60.0 | 228 | 2636 / 5724 | 0.85 / 1.17 | 60 / 63 | qualitative_review_required |
| lowbw | bandwidth | 0 | ssh (0) | 4.6 / 6.4 / 9.9 | 977 | 7588 / 20016 | 0.11 / 0.27 | 3468 / 6431 | control |
| lowbw | bandwidth | 0 | main-A (1) | 13.4 / 18.1 / 22.4 | 891 | 2688 / 5960 | 0.75 / 2.77 | 112 / 179 | control |
| lowbw | bandwidth | 0 | main-B (2) | 13.4 / 18.6 / 22.6 | 909 | 2700 / 5932 | 0.78 / 2.81 | 113 / 178 | control |
| lowbw | bandwidth | 0 | candidate (3) | 13.3 / 18.0 / 21.5 | 857 | 2632 / 5928 | 0.74 / 2.67 | 100 / 160 | fps_tradeoff_review_required |
| lowbw | bandwidth | 1 | ssh (2) | 5.4 / 6.5 / 10.0 | 978 | 7816 / 20116 | 0.11 / 0.27 | 3495 / 6429 | control |
| lowbw | bandwidth | 1 | main-A (3) | 13.2 / 18.8 / 22.9 | 910 | 2688 / 5932 | 0.75 / 2.72 | 112 / 161 | control |
| lowbw | bandwidth | 1 | main-B (0) | 17.5 / 18.6 / 22.6 | 895 | 2704 / 5964 | 0.75 / 2.65 | 111 / 164 | control |
| lowbw | bandwidth | 1 | candidate (1) | 11.4 / 19.4 / 23.6 | 950 | 2664 / 6008 | 0.79 / 2.81 | 125 / 187 | fps_tradeoff_review_required |
