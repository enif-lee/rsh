# Development core-screen evidence

See [the report](../../lowtail-20261031.md) for the decision and measurement
limits. This directory contains the 2026-10-31 freshness-quarter experiment,
not the TCP-batch product candidate and not a full-suite rerun.

- `core-results.json`: compact final analysis, all 48 repetitions and 21,590
  complete-body observations. Data validity passed; performance acceptance
  failed. Source/capture raw-file hashes are included, but those raw files are
  not published here, so body-oracle replay requires the local originals.
- `core-metrics.csv`: per-repetition complete-body FPS/lows, frame age, received
  traffic, midpoint client/server RSS in **KiB**, and CPU percent of one core.
  Figures convert RSS to MiB and retain both repetitions individually.
- `source-header-join-audit.json`: observed complete bodies' frame IDs, actual
  source header timestamps and scenes matched the recorded payloads.
- `run-status.json`: immutable status written after measurement and **before**
  analysis; its `analysis_pending` text is historical, not the final verdict.
- `fresh-quarter-release-pair.json`: immutable paired-build transcript written
  **before** measurement. Its `not_measured` field describes that stage.
- `fresh-quarter-control-result.json`: the same seven fixtures on unchanged
  main; one expected behavior failure, six guard passes, no unexpected failure.
- Source/harness manifests identify frozen inputs. `package-source.py` records
  the local extraction procedure; its absolute paths refer to that machine.
- `plot.py` regenerates the two report SVGs from `core-results.json` using
  Matplotlib. `plot-environment.json` records the version used for export.

The source ledger statistics include generated frames and write times. An
observed completed body does not prove GPU presentation, full ANSI-attribute
equality or source write completion at the observer timestamp. Host guards
sample load; they do not establish physical P-core placement or exclude every
short scheduling delay. Runtime image IDs were not separately captured in this
run. Individual container process names and host process lists are omitted.
