# Focused per-repetition comparison evidence

Target120FPS guard. Each row retains one measured role in one repetition. No pooling, noise tolerance or automatic pass is applied. Observed deficits do not establish policy causality; see JSON for exact missing metrics, FPS gaps and resource increases.

| Network | Scene | Rep | Role (slot) | FPS 1% / 5% / avg | Network kbit/s | RSS client / server KiB | CPU client / server % | Age median / p95 ms | Assessment |
|---|---|---:|---|---|---:|---|---|---|---|
| lan | bandwidth | 0 | ssh (0) | 73.2 / 83.0 / 120.0 | 11577 | 7568 / 18804 | 0.46 / 0.59 | 3 / 5 | control |
| lan | bandwidth | 0 | main-A (1) | 61.2 / 75.5 / 119.7 | 5442 | 2736 / 5816 | 2.8 / 3.7 | 6 / 10 | control |
| lan | bandwidth | 0 | main-B (2) | 63.1 / 75.9 / 119.9 | 5440 | 2752 / 5804 | 2.78 / 3.7 | 6 / 10 | control |
| lan | bandwidth | 0 | candidate (3) | 65.7 / 78.8 / 119.5 | 5435 | 2684 / 5724 | 2.85 / 3.74 | 6 / 9 | fps_tradeoff_review_required |
| lan | bandwidth | 1 | ssh (3) | 79.8 / 85.8 / 120.0 | 11565 | 7552 / 18820 | 0.43 / 0.57 | 2 / 5 | control |
| lan | bandwidth | 1 | main-A (2) | 63.5 / 78.5 / 119.9 | 5440 | 2776 / 5804 | 2.74 / 3.56 | 5 / 9 | control |
| lan | bandwidth | 1 | main-B (1) | 67.9 / 77.2 / 119.8 | 5434 | 2740 / 5804 | 2.69 / 3.53 | 6 / 9 | control |
| lan | bandwidth | 1 | candidate (0) | 67.1 / 75.3 / 119.8 | 5433 | 2676 / 5724 | 2.74 / 3.53 | 5 / 9 | fps_tradeoff_review_required |
| wan | bandwidth | 0 | ssh (0) | 57.7 / 76.9 / 120.0 | 11588 | 8040 / 19036 | 0.51 / 0.61 | 54 / 113 | control |
| wan | bandwidth | 0 | main-A (1) | 44.6 / 70.4 / 103.5 | 4080 | 2756 / 6052 | 2.46 / 10.62 | 59 / 64 | control |
| wan | bandwidth | 0 | main-B (2) | 60.5 / 74.2 / 104.7 | 4126 | 2776 / 6096 | 2.5 / 10.79 | 59 / 65 | control |
| wan | bandwidth | 0 | candidate (3) | 56.0 / 67.8 / 100.6 | 3954 | 2688 / 5996 | 2.53 / 10.49 | 60 / 65 | fps_tradeoff_review_required |
| wan | bandwidth | 1 | ssh (3) | 49.9 / 76.8 / 120.0 | 11590 | 8084 / 19032 | 0.53 / 0.63 | 54 / 101 | control |
| wan | bandwidth | 1 | main-A (2) | 48.6 / 71.8 / 103.3 | 4095 | 2796 / 6060 | 2.53 / 11.12 | 59 / 65 | control |
| wan | bandwidth | 1 | main-B (1) | 46.1 / 68.2 / 101.9 | 4030 | 2792 / 6028 | 2.5 / 10.83 | 59 / 66 | control |
| wan | bandwidth | 1 | candidate (0) | 44.9 / 67.4 / 98.0 | 3852 | 2704 / 6180 | 2.43 / 10.64 | 60 / 66 | fps_tradeoff_review_required |
