# Quarter55 slow: separate saved-data cohort

Study57: 48 observations/12 blocks, four parallel jobs and two evaluations. ReferencesA/B are identical combined295f/fb04, candidate515d/6d7f. Margin48 and later source60 are excluded. The baseline is reused; the candidate was separately compiled in a pinned image. No new same-container pair is claimed. Performance eligibility is True; selection remains null/unselected. This static cohort is separate from all older or later measurements.

- [Full report](../../fresh-quarter55-20261101.md)
- [Metrics](core-metrics.csv), [complete-body timelines](core-results.json)
- [Independent audit](saved-data-audit.json), [narrative](saved-data-audit.md)
- [Actual artifact/builds](paired-build-provenance.json), [exact product delta](controller.patch)
- [Scope fixtures](55-tcp-fresh-quarter-tests.rs), [negative control](55-tcp-fresh-quarter-control-proof.json)
- [CPU admission/runtime context](cpu30-summary.json), [exact source imports](source-imports.json)

1%/5%low andavgFPS are primary, CPU/RSS secondary. Age and network costs remain. No invented weights, automatic single-metric veto or noise tolerance. Admission uses one host user+system<30% interval, runtime load is context; source/identity/timing errors still reject. ProcessCPU100%=onecore. CSV RSS is midpointKiB; reportMiB. ClientRX kbit/s; bodyage is conditional generation-to-read latency, not RTT/input/GPU. Adjacent lows exclude window edges; maxgap/tail remain, sparse lows/p95ages staynull. Actual windows areNovember1; environment creation predates them.

```sh
python3 verify-metrics.py
uv run --with matplotlib python plot.py

```

These commands read bundled compacts, with no product/network/terminal replay. package-source.py needs original local paths. Raw capture/source payloads and executables are excluded from this partial archive. Raw hashes are imported from independent audits, not a new publisher rawparse. Resources are checked for saved-job→summary→CSV consistency, not independently reconstructed counters. Deterministic gzip logs preserve original uncompressed bytes/whitespace. The manifest covers bundled files and linked SVGs, not missing payloads. Source proofs retain planned-stage wording; actual logs/pair metadata supersede planned build counts. Old19 adverse evidence remains separate and unresolved. No source60 Mac test proof is attributed to55.
