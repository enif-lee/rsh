# CAN interoperability follow-up: saved controls

These saved controls are separate from studies 33/34; they do not replace
the original frozen-pyte body metrics or constitute a new performance run.

The focused product fix is [PR 4](https://github.com/enif-lee/rsh/pull/4).
Its scope is distinct from the adaptive-research PR 3 and far-policy selection.

The CAN-only candidate adds one byte before a generated screen delta. Its
three targeted Rust fixtures and 99 library tests pass (two ignored); main
with the same fixtures passes two and fails the expected CAN-prefix contract.
Both old/new fixture outputs match the unchanged Rust Terminal model. The actual
compiled fixture HEX is retained, rather than manually reconstructed output.
Functional integration separately passes 4 CLI and 25 session tests; together
with 99 library passes this is 128 passing tests, with two library ignores.
The test fixture accepts Bash's existing bracketed-paste reset before the
first sequence number. Its exact fixture/proof and fresh build metadata are
included; production remains the same CAN candidate.

In frozen pyte 0.8, old actual client-filtered bytes leave three`[0m`
characters, while CAN-prefixed bytes match all 100×30 characters. Whole,
snapshot/delta-split and bytewise feeds agree. Both cursors are x 94/y 24.
The single-printable continuation is a bounded observation; pyte has no
public ground-state API here. This is character/cursor evidence, without
full receive/Paced-path, SGR/attribute, GPU or performance validation.

The separate native tmux 3.7c **minimal sequence** controls leave blank cells
for ordinary/nested/CAN sequences, including nested bytewise input, and
detect the literal-negative control. They establish behavior of that
implementation, not every terminal. A failed initial image attempt with no
tmux installed is retained separately from the successful native controls.

Raw diagnostic snippets and the independent arithmetic/coherence notes
preserve the original 34 result: 12 body matches, 121 headers and a 7.344-second
interval without a new frozen-oracle body match. That is not proof of a
physical terminal output stop. No columns were masked or metrics rewritten.

Build/preparation metadata retains its original creation-stage status.
Archived native scripts have local environment/path assumptions and are
provenance, not a portable replay runner. `verify-controls.py` checks saved
hashes, fixture bytes and result consistency without starting any terminal.
The executor's Docker lifecycle transcript is explicitly a transcription of
prior tool history; effective live cgroup/network state was not inspected.
