# Actual compiled fixture → frozen pyte control

**PASS: the candidate's one CAN byte removes the three-character mismatch in this fixture.** These are actual Rust fixture HEX outputs from `build-diff-cancel.log` and `diff-cancel-control-tests.log`, not manually reconstructed delta bytes. Candidate/control snapshots, client snapshots and expected full render are identical; delta and client delta differ only by the candidate's initial0x18.

| Actual client byte feed | Old control differing cells | CAN candidate differing cells | Cursor versus expected |
|---|---:|---:|---|
|Whole snapshot+delta|3 (`[0m`)|0|Both match x94/y24|
|Split at snapshot/delta|3 (`[0m`)|0|Both match x94/y24|
|Bytewise|3 (`[0m`)|0|Both match x94/y24|

Comparison includes all100×30 character cells. In the control, only row24/columns94–96 differ from expected spaces. The candidate matches the compiled expected render in all three modes. Pyte0.8.0 screen source SHA is `b3316cd3a5140ef5e4fe3eaa3d88a9769d8a4c482405595411f542979b561658`, matching the measurement image. Both final states accept one printableZ at the cursor. Pyte exposes no public `at_ground` API in this environment, so that bounded continuation observation is not a general internal parser-ground theorem. Root's actual Rust Terminal fixtures separately check at_ground for both control and candidate.

Read-only offline containers used pinned actual image1ecf08a7, CPU1/networknone, noNET_ADMIN/ports, source study mountedRO and own notes outputRW. Both own --rm containers exited0 and were verified absent. Source logs and original34 prepared/capture/ledger hashes remain unchanged. No new benchmark/product executable/compilation ran here.

This demonstrates this compiled screen-delta/client-sync-filter fixture's pyte interoperability. It does not execute the full receive/Paced path, prove every terminal/GPU output, establish performance gains or rewrite study34's original12-body observation. Root's native tmux controls and any affected-scene validation remain separate evidence.

Evidence: `diff-cancel-interop-inputs.json`, `diff-cancel-frozen-pyte-interop.py/json/log`, `diff-cancel-frozen-pyte-provenance.json`.
