# Focused per-repetition comparison evidence

Each row retains one measured role in one repetition. No pooling, noise tolerance or automatic pass is applied. Observed deficits do not establish policy causality; see JSON for exact missing metrics, FPS gaps and resource increases.

| Network | Scene | Rep | Role (slot) | FPS 1% / 5% / avg | Network kbit/s | RSS client / server KiB | CPU client / server % | Age median / p95 ms | Assessment |
|---|---|---:|---|---|---:|---|---|---|---|
| moving | compress | 0 | ssh (0) | 3.7 / 31.5 / 48.0 | 1978 | 7500 / 20088 | 0.22 / 0.33 | 1683 / 11992 | control |
| moving | compress | 0 | main-A (1) | 2.5 / 7.8 / 21.4 | 86 | 2804 / 5888 | 0.41 / 1.51 | 47 / 155 | control |
| moving | compress | 0 | main-B (2) | 2.9 / 9.4 / 23.7 | 94 | 2792 / 5912 | 0.43 / 1.36 | 46 / 131 | control |
| moving | compress | 0 | candidate (3) | 3.6 / 8.6 / 27.9 | 113 | 2800 / 6004 | 0.49 / 1.56 | 48 / 133 | moving_profile_fidelity_pending_manual_review |
| moving | compress | 1 | ssh (2) | 6.9 / 32.0 / 49.4 | 2033 | 7612 / 19804 | 0.25 / 0.46 | 1691 / 11381 | control |
| moving | compress | 1 | main-A (3) | 2.8 / 6.9 / 21.2 | 85 | 1856 / 5240 | 0.46 / 1.82 | 47 / 161 | control |
| moving | compress | 1 | main-B (0) | 2.4 / 9.3 / 22.0 | 91 | 2416 / 5288 | 0.49 / 2.04 | 47 / 195 | control |
| moving | compress | 1 | candidate (1) | 3.4 / 7.3 / 25.2 | 101 | 1400 / 4904 | 0.5 / 1.81 | 47 / 118 | moving_profile_fidelity_pending_manual_review |
| moving | bandwidth | 0 | ssh (0) | 3.5 / 13.6 / 27.0 | 2636 | 7516 / 21144 | 0.2 / 0.2 | 1626 / 13826 | control |
| moving | bandwidth | 0 | main-A (1) | 2.2 / 6.0 / 19.0 | 808 | 2472 / 5476 | 0.67 / 2.8 | 58 / 251 | control |
| moving | bandwidth | 0 | main-B (2) | 1.9 / 6.3 / 19.8 | 816 | 2420 / 5252 | 0.65 / 2.81 | 56 / 254 | control |
| moving | bandwidth | 0 | candidate (3) | 1.7 / 7.4 / 19.6 | 849 | 1812 / 5312 | 0.68 / 2.9 | 58 / 292 | moving_profile_fidelity_pending_manual_review |
| moving | bandwidth | 1 | ssh (2) | 3.5 / 13.4 / 27.6 | 2695 | 7460 / 19968 | 0.2 / 0.2 | 1658 / 13760 | control |
| moving | bandwidth | 1 | main-A (3) | 2.4 / 6.5 / 18.9 | 784 | 2472 / 5524 | 0.62 / 2.61 | 56 / 248 | control |
| moving | bandwidth | 1 | main-B (0) | 2.2 / 7.0 / 19.1 | 796 | 2436 / 5496 | 0.67 / 2.72 | 58 / 233 | control |
| moving | bandwidth | 1 | candidate (1) | 1.9 / 7.1 / 21.3 | 921 | 2204 / 5488 | 0.68 / 2.92 | 58 / 233 | moving_profile_fidelity_pending_manual_review |
