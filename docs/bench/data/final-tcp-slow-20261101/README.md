# Final TCP slow: saved-data archive

Study 41: 48 observations / 12 blocks, four parallel roles and two evaluations. CAN+PTY-fixed reference source `4f98/c25` and combined source `295f/fb04` share CAN and caller-owned PTY/account fixes. These are development artifacts, not release adoption.

All 48 observations pass saved data checks and permit qualitative profile comparison. Final selection remains null. Do not pool separate artifacts/pipelines or static/moving studies. 1%/5% low and average FPS are primary; CPU/RSS are secondary. Age and recovery remain visible. No invented weights, automatic metric veto or noise allowance are used.

- [Full report](../../final-tcp-20261101.md)
- [Metrics CSV](core-metrics.csv), [complete-body compact timelines](core-results.json)
- [Exact independent audit](saved-data-audit.json), [audit narrative](saved-data-audit.md)
- [Extraction scope](extraction-provenance.json), [source/header audit](source-header-join-audit.json)
- [Paired build](pty-account-release-pair.json), [controller-only delta](controller.patch), [controller identities](controller-source.json)
- [CPU admission/runtime context](cpu30-summary.json), [original source imports](source-imports.json)

CPU admission requires one user+system<30% interval, with no 60-second streak or per-process limit. Runtime CPU is context, not an abort threshold. Source/clock/identity/fence errors remain invalidating. Actual windows are November 1; environment creation predates them. Body metrics join actual source frames. Lows use adjacent gaps excluding window edges; unmatched tail remains. Sparse lows/p95 ages remain null. RSS is midpoint KiB in compact/CSV and MiB in report, RX is kbit/s, CPU is process percent. Resource checks cover saved raw-job→summary→CSV consistency, not independent raw-counter reconstruction.

## Reproduce saved-data checks and plots

```sh
python3 verify-metrics.py
uv run --with matplotlib python plot.py
```

These commands use bundled compact data, not product binaries, network shaping or terminal replay. `package-source.py` requires the retained local study directory. Gzip logs preserve original uncompressed bytes and whitespace; their original hashes are recorded. Plot dependencies are transient, not product dependencies.

This is a partial archive. Full capture/source-ledger payloads and executable binaries remain local. Original raw hashes are imported from the independent audit, not described as a new publication rehash. The manifest covers bundled files and SVGs, not unbundled captures. Original proofs/logs retain creation-stage wording; actual paired-build results supersede plans.
