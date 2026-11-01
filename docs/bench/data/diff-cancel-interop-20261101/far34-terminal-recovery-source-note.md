# Far34 nested ESC: primary-source scope note

Source/document reading only; no new terminal process, compilation or replay. The capture reduction and pyte controls are measurement agent evidence (`far34-pyte-stream-controls.*`), not independently repeated here.

Xterm's own [control-sequence documentation](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html) says ECMA-48 error recovery is implementation-dependent (Control Bytes, Characters, and Sequences, lines120–126 in the retrieved text). This prevents a blanket claim that one parser's recovery behavior establishes all terminals' output.

For a concrete implementation, the maintainer's [CSI parse table](https://github.com/ThomasDickey/xterm-snapshots/blob/master/VTPrsTbl.c#L468-L504) maps an ESC while in CSI to `CASE_ESC`. The [CASE_ESC handler](https://github.com/ThomasDickey/xterm-snapshots/blob/master/charproc.c#L3524-L3530) switches to `esc_table` (or the VT52 escape table in that mode). Thus restarting escape parsing is supported by xterm source, rather than inferred from pyte or rsh alone. Retrieved November1,2026; these master links are mutable.

This is source evidence for a parser transition. It does not independently prove the full captured stream's final cells, actual installed xterm behavior, other terminal implementations, GPU presentation, or whether changing rsh's output strategy is preferable. The original frozen-pyte body criterion and its12-body result remain preserved, labeled separately from header121 and from a physical terminal correctness verdict.
