# Focused per-repetition comparison evidence

Each row retains one measured role in one repetition. No pooling, noise tolerance or automatic pass is applied. Observed deficits do not establish policy causality; see JSON for exact missing metrics, FPS gaps and resource increases.

| Network | Scene | Rep | Role (slot) | FPS 1% / 5% / avg | Network kbit/s | RSS client / server KiB | CPU client / server % | Age median / p95 ms | Assessment |
|---|---|---:|---|---|---:|---|---|---|---|
| moving | compress | 0 | ssh (0) | 3.6 / 32.5 / 47.9 | 1969 | 7968 / 20100 | 0.22 / 0.38 | 1685 / 12033 | control |
| moving | compress | 0 | main-A (1) | 2.5 / 5.4 / 15.2 | 59 | 2724 / 5868 | 0.31 / 1.49 | 50 / 164 | control |
| moving | compress | 0 | main-B (2) | 2.3 / 7.4 / 18.5 | 73 | 2748 / 5892 | 0.36 / 1.45 | 47 / 135 | control |
| moving | compress | 0 | candidate (3) | 3.2 / 8.4 / 26.9 | 107 | 2736 / 5912 | 0.47 / 1.54 | 46 / 131 | moving_profile_fidelity_pending_manual_review |
| moving | compress | 1 | ssh (3) | 6.3 / 27.0 / 48.2 | 1989 | 7696 / 19988 | 0.24 / 0.37 | 1780 / 11258 | control |
| moving | compress | 1 | main-A (2) | 2.8 / 5.0 / 15.7 | 60 | 2652 / 5896 | 0.31 / 1.46 | 49 / 204 | control |
| moving | compress | 1 | main-B (1) | 2.8 / 5.3 / 18.0 | 70 | 2628 / 5888 | 0.33 / 1.38 | 46 / 164 | control |
| moving | compress | 1 | candidate (0) | 3.4 / 9.8 / 28.2 | 113 | 2728 / 5888 | 0.51 / 1.8 | 48 / 140 | moving_profile_fidelity_pending_manual_review |
| moving | bandwidth | 0 | ssh (0) | 3.5 / 16.6 / 16.5 | 1694 | 8048 / 19100 | 0.14 / 0.09 | 54 / 4527 | control |
| moving | bandwidth | 0 | main-A (1) | 1.6 / 4.8 / 14.2 | 602 | 2772 / 5488 | 0.57 / 2.44 | 61 / 354 | control |
| moving | bandwidth | 0 | main-B (2) | 1.5 / 5.1 / 12.9 | 537 | 2728 / 5496 | 0.53 / 2.31 | 65 / 356 | control |
| moving | bandwidth | 0 | candidate (3) | 2.5 / 6.5 / 21.7 | 910 | 2716 / 5516 | 0.81 / 3.12 | 55 / 208 | moving_profile_fidelity_pending_manual_review |
| moving | bandwidth | 1 | ssh (3) | 2.6 / 12.8 / 27.2 | 2652 | 7744 / 21176 | 0.21 / 0.21 | 1700 / 13834 | control |
| moving | bandwidth | 1 | main-A (2) | 1.9 / 6.0 / 15.9 | 692 | 2768 / 5520 | 0.63 / 2.61 | 60 / 268 | control |
| moving | bandwidth | 1 | main-B (1) | 1.4 / 3.7 / 13.9 | 585 | 2768 / 5496 | 0.51 / 2.33 | 57 / 339 | control |
| moving | bandwidth | 1 | candidate (0) | 2.6 / 7.1 / 20.3 | 856 | 2712 / 5504 | 0.73 / 2.99 | 54 / 222 | moving_profile_fidelity_pending_manual_review |
