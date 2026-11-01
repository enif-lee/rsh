# Quarter + margin60 core archive

48 observations/12 blocks, four parallel jobs × two evaluations. Candidate `fa934e198646` / `4b5273…6c2d`; duplicate baseline `295f81d54ea4` / `fb0438…eec4`, plus SSH. Eligibility is `True` and final selection remains `None`. This dataset is separate from the other three cohorts and from studies19/48/55; it does not isolate quarter versus48 or establish additive effects.

`core-results.json` retains complete-frame timestamps, header/body/partial-state metrics, source metadata and source/PID/nonce/five-second closure proofs. `core-metrics.csv` retains every role/evaluation including nulls and resource values. Body total 22195; header total 22215; source records 23040; pending writes 0, pending body joins 0. Missing low jobs 0; missing p95-age jobs 0. Resource checking is saved-derived consistency, not independent raw RX/CPU counter or RSS-sample reconstruction.

Raw captures, source payloads, full trace replays and GPU evidence are **not** bundled. Original capture/source hashes are imported from the independently checked local audit; the publisher does not parse payloads again. Compact verification recomputes metric arithmetic and stored proof/hash consistency. `package-source.py` requires the original machine's study folder and sibling research notes; it is not a portable raw experiment runner. Source manifests/proofs/patches and root test logs are included, not full product source directories or executables.

Linux candidate:116 library +4 CLI +25 session +1 Linux concurrency test =146 passes/2 ignored. Mac native:116 library +4 CLI +25 session =145 passes/2 ignored; Linux-only concurrency has0 Mac tests. Negative control is exact48+the same adapted quarter fixtures:2 pass/1 expected fail, with existing margin3 passing. No baseline fresh-pair rebuild is claimed. Mac toolchain observation is distinguished from the actual test/guard logs.

Portable check:

```sh
python3 verify-metrics.py
```

Re-export requires local original evidence:

```sh
python3 package-source.py /absolute/path/to/lowtail-research-20261031
uv run --with matplotlib python plot.py
```

Matplotlib version/backend are in `plot-environment.json`; figure timestamps are omitted. `plot.py --preview` creates local PNGs for visual QA; previews are outside the curated archive. Averages and both low metrics are separate points for each evaluation. Adjacent-gap lows exclude window edge gaps; max-gap and no-new-complete-tail retain them. Missing values remain null/blank and are not plotted as zero. Age is conditional on detected complete bodies, not RTT/input/GPU latency. FPS1%/5%low and average are primary; CPU/RSS are secondary. No numeric weight, automatic parity/promotion or single-field veto is applied.
