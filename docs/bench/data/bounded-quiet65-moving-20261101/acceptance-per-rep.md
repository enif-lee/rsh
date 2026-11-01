# Focused per-repetition comparison evidence

Each row retains one measured role in one repetition. No pooling, noise tolerance or automatic pass is applied. Observed deficits do not establish policy causality; see JSON for exact missing metrics, FPS gaps and resource increases.

| Network | Scene | Rep | Role (slot) | FPS 1% / 5% / avg | Network kbit/s | RSS client / server KiB | CPU client / server % | Age median / p95 ms | Assessment |
|---|---|---:|---|---|---:|---|---|---|---|
| moving | compress | 0 | ssh (0) | 3.8 / 29.9 / 47.9 | 1977 | 7876 / 20012 | 0.24 / 0.49 | 1760 / 11397 | control |
| moving | compress | 0 | main-A (1) | 2.6 / 12.4 / 28.5 | 115 | 2252 / 5360 | 0.55 / 2.02 | 48 / 139 | control |
| moving | compress | 0 | main-B (2) | 3.7 / 7.4 / 26.7 | 107 | 2240 / 5144 | 0.49 / 1.78 | 47 / 134 | control |
| moving | compress | 0 | candidate (3) | 3.2 / 7.0 / 26.4 | 105 | 2264 / 5192 | 0.49 / 1.79 | 47 / 129 | moving_profile_fidelity_pending_manual_review |
| moving | compress | 1 | ssh (3) | 6.0 / 27.2 / 48.1 | 1985 | 8060 / 20028 | 0.22 / 0.39 | 1776 / 11308 | control |
| moving | compress | 1 | main-A (2) | 3.0 / 12.7 / 29.8 | 120 | 2804 / 5960 | 0.56 / 1.94 | 49 / 164 | control |
| moving | compress | 1 | main-B (1) | 4.3 / 11.6 / 30.7 | 127 | 2804 / 5960 | 0.54 / 1.88 | 49 / 210 | control |
| moving | compress | 1 | candidate (0) | 3.5 / 13.8 / 31.2 | 127 | 2752 / 5908 | 0.56 / 1.97 | 49 / 151 | moving_profile_fidelity_pending_manual_review |
| moving | bandwidth | 0 | ssh (0) | 3.6 / 7.3 / 16.5 | 1694 | 8064 / 19064 | 0.14 / 0.09 | 54 / 4377 | control |
| moving | bandwidth | 0 | main-A (1) | 2.9 / 7.6 / 22.6 | 987 | 2876 / 5988 | 0.78 / 3.22 | 55 / 237 | control |
| moving | bandwidth | 0 | main-B (2) | 2.5 / 8.0 / 21.4 | 900 | 2780 / 6020 | 0.69 / 2.91 | 53 / 225 | control |
| moving | bandwidth | 0 | candidate (3) | 2.7 / 7.5 / 21.9 | 924 | 2760 / 5936 | 0.74 / 3.07 | 55 / 214 | moving_profile_fidelity_pending_manual_review |
| moving | bandwidth | 1 | ssh (3) | 3.3 / 12.4 / 16.5 | 1699 | 8032 / 18980 | 0.15 / 0.1 | 54 / 4286 | control |
| moving | bandwidth | 1 | main-A (2) | 2.4 / 6.6 / 21.1 | 888 | 2896 / 5984 | 0.74 / 3.05 | 57 / 244 | control |
| moving | bandwidth | 1 | main-B (1) | 2.9 / 6.8 / 21.8 | 931 | 2892 / 5984 | 0.79 / 3.15 | 53 / 290 | control |
| moving | bandwidth | 1 | candidate (0) | 2.3 / 7.5 / 21.7 | 911 | 2852 / 5960 | 0.8 / 3.17 | 55 / 203 | moving_profile_fidelity_pending_manual_review |
