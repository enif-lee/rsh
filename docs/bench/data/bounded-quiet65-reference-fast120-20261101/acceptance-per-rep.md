# Focused per-repetition comparison evidence

Target120FPS guard. Each row retains one measured role in one repetition. No pooling, noise tolerance or automatic pass is applied. Observed deficits do not establish policy causality; see JSON for exact missing metrics, FPS gaps and resource increases.

| Network | Scene | Rep | Role (slot) | FPS 1% / 5% / avg | Network kbit/s | RSS client / server KiB | CPU client / server % | Age median / p95 ms | Assessment |
|---|---|---:|---|---|---:|---|---|---|---|
| lan | bandwidth | 0 | ssh (0) | 75.7 / 86.4 / 120.0 | 11567 | 7548 / 18820 | 0.4 / 0.51 | 2 / 5 | control |
| lan | bandwidth | 0 | main-A (1) | 66.2 / 77.7 / 119.5 | 5438 | 2664 / 5744 | 2.62 / 3.39 | 5 / 9 | control |
| lan | bandwidth | 0 | main-B (2) | 64.0 / 77.0 / 120.0 | 5441 | 2652 / 5716 | 2.61 / 3.33 | 6 / 10 | control |
| lan | bandwidth | 0 | candidate (3) | 62.1 / 76.3 / 119.2 | 5435 | 2680 / 5776 | 2.71 / 3.35 | 5 / 9 | fps_tradeoff_review_required |
| lan | bandwidth | 1 | ssh (3) | 74.4 / 84.4 / 120.0 | 11577 | 7592 / 18824 | 0.45 / 0.58 | 2 / 5 | control |
| lan | bandwidth | 1 | main-A (2) | 67.3 / 76.7 / 119.9 | 5439 | 2680 / 5752 | 2.7 / 3.57 | 5 / 9 | control |
| lan | bandwidth | 1 | main-B (1) | 63.5 / 75.6 / 119.5 | 5439 | 2680 / 5720 | 2.66 / 3.56 | 5 / 10 | control |
| lan | bandwidth | 1 | candidate (0) | 65.1 / 75.6 / 119.3 | 5433 | 2700 / 5760 | 2.71 / 3.63 | 6 / 9 | fps_tradeoff_review_required |
| wan | bandwidth | 0 | ssh (0) | 50.9 / 75.1 / 120.0 | 11589 | 7968 / 19036 | 0.44 / 0.53 | 54 / 142 | control |
| wan | bandwidth | 0 | main-A (1) | 54.7 / 68.9 / 100.4 | 3956 | 2720 / 5984 | 2.17 / 9.54 | 58 / 64 | control |
| wan | bandwidth | 0 | main-B (2) | 55.0 / 68.9 / 100.7 | 3972 | 2704 / 6044 | 2.22 / 9.64 | 58 / 65 | control |
| wan | bandwidth | 0 | candidate (3) | 51.9 / 69.6 / 99.9 | 3917 | 2696 / 6000 | 2.22 / 9.71 | 60 / 66 | fps_tradeoff_review_required |
| wan | bandwidth | 1 | ssh (3) | 48.9 / 77.4 / 120.0 | 11584 | 8132 / 19036 | 0.47 / 0.57 | 54 / 117 | control |
| wan | bandwidth | 1 | main-A (2) | 42.5 / 69.2 / 99.7 | 3943 | 2704 / 5980 | 2.33 / 9.98 | 59 / 65 | control |
| wan | bandwidth | 1 | main-B (1) | 45.9 / 69.8 / 100.9 | 3967 | 2672 / 6024 | 2.33 / 10.14 | 59 / 65 | control |
| wan | bandwidth | 1 | candidate (0) | 55.9 / 67.8 / 101.0 | 3959 | 2704 / 5996 | 2.37 / 10.3 | 60 / 65 | fps_tradeoff_review_required |
