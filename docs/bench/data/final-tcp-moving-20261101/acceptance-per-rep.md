# Focused per-repetition comparison evidence

Each row retains one measured role in one repetition. No pooling, noise tolerance or automatic pass is applied. Observed deficits do not establish policy causality; see JSON for exact missing metrics, FPS gaps and resource increases.

| Network | Scene | Rep | Role (slot) | FPS 1% / 5% / avg | Network kbit/s | RSS client / server KiB | CPU client / server % | Age median / p95 ms | Assessment |
|---|---|---:|---|---|---:|---|---|---|---|
| moving | compress | 0 | ssh (0) | 6.8 / 26.3 / 48.2 | 1987 | 8040 / 20032 | 0.24 / 0.41 | 1778 / 11274 | control |
| moving | compress | 0 | main-A (1) | 2.4 / 6.4 / 17.8 | 69 | 2652 / 5896 | 0.35 / 1.47 | 45 / 132 | control |
| moving | compress | 0 | main-B (2) | 2.5 / 6.5 / 17.9 | 72 | 2720 / 5880 | 0.35 / 1.4 | 45 / 159 | control |
| moving | compress | 0 | candidate (3) | 3.0 / 7.6 / 23.8 | 97 | 2800 / 5920 | 0.46 / 1.56 | 47 / 146 | moving_profile_fidelity_pending_manual_review |
| moving | compress | 1 | ssh (2) | 7.3 / 24.5 / 47.9 | 1976 | 7724 / 20020 | 0.25 / 0.39 | 1776 / 11430 | control |
| moving | compress | 1 | main-A (3) | 2.2 / 7.0 / 20.7 | 83 | 2692 / 5896 | 0.42 / 1.61 | 48 / 151 | control |
| moving | compress | 1 | main-B (0) | 2.4 / 6.0 / 18.3 | 71 | 2656 / 5896 | 0.38 / 1.4 | 45 / 143 | control |
| moving | compress | 1 | candidate (1) | 2.6 / 8.3 / 21.9 | 89 | 2828 / 5920 | 0.45 / 1.69 | 47 / 239 | moving_profile_fidelity_pending_manual_review |
| moving | bandwidth | 0 | ssh (0) | 2.6 / 13.6 / 27.3 | 2658 | 7704 / 20196 | 0.21 / 0.21 | 1685 / 13779 | control |
| moving | bandwidth | 0 | main-A (1) | 1.5 / 6.5 / 14.3 | 601 | 2748 / 5920 | 0.54 / 2.41 | 61 / 332 | control |
| moving | bandwidth | 0 | main-B (2) | 2.4 / 7.1 / 20.2 | 826 | 2812 / 5964 | 0.71 / 2.82 | 58 / 229 | control |
| moving | bandwidth | 0 | candidate (3) | 2.3 / 6.0 / 20.4 | 862 | 2872 / 5936 | 0.75 / 2.98 | 57 / 291 | moving_profile_fidelity_pending_manual_review |
| moving | bandwidth | 1 | ssh (2) | 3.2 / 12.9 / 27.2 | 2651 | 7704 / 20176 | 0.21 / 0.21 | 1682 / 13729 | control |
| moving | bandwidth | 1 | main-A (3) | 1.9 / 4.9 / 14.1 | 582 | 2688 / 5940 | 0.51 / 2.29 | 59 / 290 | control |
| moving | bandwidth | 1 | main-B (0) | 1.8 / 5.9 / 15.3 | 628 | 2708 / 5968 | 0.57 / 2.38 | 61 / 324 | control |
| moving | bandwidth | 1 | candidate (1) | 2.1 / 6.7 / 19.3 | 804 | 2808 / 5924 | 0.7 / 2.86 | 56 / 260 | moving_profile_fidelity_pending_manual_review |
