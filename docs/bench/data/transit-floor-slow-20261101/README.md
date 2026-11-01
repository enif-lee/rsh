# Sampled-floor transit: slow development screen

Saved records, charts and verification sources are included; no new benchmark was run to assemble this archive.

This archive represents study 34 only: 48 jobs/12 blocks, four parallel roles,
two evaluations each, 1M/256k/64k and dashboard/random scenes. Internal main-A/B
roles are identical actual-main artifacts; candidate is the sampled-floor
transit allowance, retaining 80% exploration. The source pipeline is RAM
pre-write/schema 2 with nonce-bound pidfd/`SIGUSR2` collection after all four
windows close, using the original five-second deadline. Do not pool with
the legacy completed-write core screen.

The original 64 kbit/s dashboard evaluation 0 candidate is retained as 12 complete
bodies/1.5 FPS **under the frozen pyte 0.8 criterion**, versus 121 headers/15.1
FPS. Its 11 adjacent gaps yield 15.5/15.5 lows but omit the 7.344-second interval
without a new complete-body match. That interval is right-censored at the
receive-window end. Edge-inclusive maxgap 7344 ms and per-second body counts
`[12,0,0,0,0,0,0,0]` describe the criterion's whole-window result.

Pyte retains three`[0m` cells after an unfinished CSI/new ESC combination;
native tmux and Rust controls have different recovery behavior. This does
not establish a physical terminal output stop. No columns are masked and
no original body/header/visible-state metric is rewritten. CAN follow-up
controls and later measurements belong to separate evidence.

`core-results.json` and CSV retain both body/header criteria, visible state,
adjacent-gap sample counts, edge-inclusive gaps and trailing no-new-body time.
Missing lows/ages remain null/empty, and evaluations remain separate. Body
age describes only detected complete bodies, not an unobserved remainder.

FPS avg/1%/5% are primary, CPU/RSS secondary; age/network/recovery costs stay
visible. Raw gap labels are not automatic rejection or a weighted score.
The candidate is currently unselected, without a global parity claim.

Raw full captures/source bodies remain local. All 96 original hashes and
partial header audit are retained. Compact data allows saved metric
recalculation, not a new body-oracle replay. Resource checks establish
saved-derived batch→summary→CSV consistency and units; raw counter endpoints
were not retained. CPU admission is one interval user+system<30%; runtime
load remains context.

`verify-metrics.py` will check archive/figure manifest hashes and recompute
frame/CSV statistics with the standard library. `plot.py` retains both
evaluations, and the additional completion timeline contrasts body matches
with header counts. The local-input extractor and archived native scripts
are provenance, not portable replay tools. Original CSV/patch/log bytes and
creation-stage metadata remain preserved; gzip keeps original log bytes.
