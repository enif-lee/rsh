# Cadence0.9 slow: separate saved-data cohort

Study 51: 48 observations/12 blocks, four parallel roles and two evaluations. References A/B are identical combined `295f/fb04`, candidate `ea6c/e34a`. This is not the earlier CAN+PTY-only reference. The baseline is reused and the candidate separately built in a pinned image; no new same-container pair is claimed. Selection remains null/unselected.

- [Full report](../../cadence-margin-20261101.md)
- [Metrics](core-metrics.csv), [complete-body compact timelines](core-results.json)
- [Independent audit](saved-data-audit.json), [narrative](saved-data-audit.md)
- [Actual build/artifacts](paired-build-provenance.json), [controller-only patch](controller.patch)
- [Negative-control proof](48-tcp-cadence-margin-control-proof.json), [three fixtures](48-tcp-cadence-margin-tests.rs)
- [CPU admission/runtime context](cpu30-summary.json), [source imports](source-imports.json)

1%/5% low and avg FPS are primary; CPU/RSS secondary. Age/network costs remain. Every evaluation, original51loss and predeclared53replication remains separate; no pooling with older studies or functional 52, automatic single-metric veto, numerical weights or noise allowance. Admission is one host user+system<30% interval, runtimeCPU context only; no 60 s streak or per-process gate. Timing/source/identity errors still reject. Process CPU100%=onecore. RSS is midpoint KiB in CSV and MiB in report; client RX kbit/s. Lows exclude window edges; tail/maxgap retained; missing lows/p95 ages stay null.

```sh
python3 verify-metrics.py
uv run --with matplotlib python plot.py
```

Commands use bundled compacts, not product binaries, networks or terminal replay. package-source.py requires retained local inputs. This partial archive excludes raw capture/source payloads and executables. Raw hashes come from the independent audit, not a new publisher raw parse. Original gzip logs preserve uncompressed bytes and whitespace. The manifest covers bundled files/linkedSVGs, not unbundled payloads. Resource validation is saved-job/summary/CSV consistency, not independent raw counter reconstruction. Actual windows are November 1, while environment creation predates them. Sourceproofs retain creation-stage wording; actual metadata/logs supersede plans.
