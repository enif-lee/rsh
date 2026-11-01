# Focused per-repetition comparison evidence

Each row retains one measured role in one repetition. No pooling, noise tolerance or automatic pass is applied. Observed deficits do not establish policy causality; see JSON for exact missing metrics, FPS gaps and resource increases.

| Network | Scene | Rep | Role (slot) | FPS 1% / 5% / avg | Network kbit/s | RSS client / server KiB | CPU client / server % | Age median / p95 ms | Assessment |
|---|---|---:|---|---|---:|---|---|---|---|
| wan | bandwidth | 0 | ssh (0) | 31.3 / 46.5 / 60.0 | 5802 | 7896 / 18828 | 0.21 / 0.22 | 53 / 57 | control |
| wan | bandwidth | 0 | main-A (1) | 37.9 / 44.8 / 54.9 | 2133 | 2436 / 5524 | 1.02 / 4.52 | 62 / 71 | control |
| wan | bandwidth | 0 | main-B (2) | 42.2 / 45.3 / 54.5 | 2126 | 2500 / 5560 | 1.02 / 4.43 | 61 / 71 | control |
| wan | bandwidth | 0 | candidate (3) | 29.8 / 46.8 / 58.1 | 2270 | 2472 / 5528 | 1.1 / 4.71 | 58 / 64 | fps_tradeoff_review_required |
| wan | bandwidth | 1 | ssh (3) | 39.0 / 48.1 / 60.0 | 5802 | 7700 / 18840 | 0.23 / 0.24 | 53 / 58 | control |
| wan | bandwidth | 1 | main-A (2) | 41.7 / 46.6 / 55.0 | 2141 | 2728 / 5976 | 1.07 / 4.58 | 63 / 71 | control |
| wan | bandwidth | 1 | main-B (1) | 36.4 / 45.3 / 54.4 | 2125 | 2772 / 6000 | 1.07 / 4.64 | 61 / 71 | control |
| wan | bandwidth | 1 | candidate (0) | 29.5 / 46.0 / 58.4 | 2280 | 2756 / 6008 | 1.17 / 4.77 | 55 / 67 | fps_tradeoff_review_required |
