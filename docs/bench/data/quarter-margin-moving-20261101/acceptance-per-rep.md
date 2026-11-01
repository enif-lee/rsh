# Focused per-repetition comparison evidence

Each row retains one measured role in one repetition. No pooling, noise tolerance or automatic pass is applied. Observed deficits do not establish policy causality; see JSON for exact missing metrics, FPS gaps and resource increases.

| Network | Scene | Rep | Role (slot) | FPS 1% / 5% / avg | Network kbit/s | RSS client / server KiB | CPU client / server % | Age median / p95 ms | Assessment |
|---|---|---:|---|---|---:|---|---|---|---|
| moving | compress | 0 | ssh (0) | 6.9 / 25.4 / 48.0 | 1977 | 7712 / 19984 | 0.23 / 0.37 | 1785 / 11301 | control |
| moving | compress | 0 | main-A (1) | 2.6 / 10.5 / 20.5 | 81 | 2736 / 5856 | 0.35 / 1.27 | 44 / 95 | control |
| moving | compress | 0 | main-B (2) | 2.9 / 7.6 / 23.0 | 90 | 2800 / 5888 | 0.41 / 1.35 | 45 / 129 | control |
| moving | compress | 0 | candidate (3) | 3.2 / 8.7 / 26.2 | 105 | 2808 / 5964 | 0.47 / 1.61 | 47 / 129 | moving_profile_fidelity_pending_manual_review |
| moving | compress | 1 | ssh (2) | 3.6 / 20.2 / 47.9 | 1973 | 7740 / 20548 | 0.25 / 0.53 | 1783 / 11423 | control |
| moving | compress | 1 | main-A (3) | 2.8 / 9.2 / 20.2 | 78 | 2760 / 5924 | 0.4 / 1.61 | 45 / 131 | control |
| moving | compress | 1 | main-B (0) | 2.9 / 6.8 / 21.5 | 84 | 2812 / 5884 | 0.42 / 1.55 | 46 / 154 | control |
| moving | compress | 1 | candidate (1) | 3.6 / 13.4 / 31.1 | 130 | 2832 / 5968 | 0.68 / 2.41 | 50 / 183 | moving_profile_fidelity_pending_manual_review |
| moving | bandwidth | 0 | ssh (0) | 3.4 / 13.7 / 27.1 | 2645 | 7768 / 20092 | 0.19 / 0.19 | 1699 / 13816 | control |
| moving | bandwidth | 0 | main-A (1) | 1.8 / 6.3 / 18.4 | 786 | 2828 / 5940 | 0.63 / 2.73 | 57 / 291 | control |
| moving | bandwidth | 0 | main-B (2) | 3.1 / 8.6 / 20.3 | 847 | 2844 / 5984 | 0.7 / 2.88 | 57 / 272 | control |
| moving | bandwidth | 0 | candidate (3) | 2.2 / 6.7 / 20.9 | 883 | 2752 / 6000 | 0.7 / 2.91 | 55 / 239 | moving_profile_fidelity_pending_manual_review |
| moving | bandwidth | 1 | ssh (2) | 3.4 / 13.3 / 27.3 | 2664 | 7972 / 20328 | 0.2 / 0.2 | 1702 / 13740 | control |
| moving | bandwidth | 1 | main-A (3) | 2.6 / 7.2 / 20.2 | 841 | 2784 / 5980 | 0.68 / 2.81 | 57 / 243 | control |
| moving | bandwidth | 1 | main-B (0) | 2.3 / 7.1 / 17.7 | 760 | 2832 / 5924 | 0.63 / 2.64 | 58 / 219 | control |
| moving | bandwidth | 1 | candidate (1) | 2.9 / 6.6 / 21.0 | 911 | 2844 / 6000 | 0.77 / 3.03 | 55 / 226 | moving_profile_fidelity_pending_manual_review |
