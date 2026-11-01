# Study34 slot3: frozen-body/header disagreement

**The saved12-body result is reproduced exactly, but it is not evidence of a7.344s physical output stop.** Pyte0.8's handling of a nested ESC inside an unfinished CSI leaves three literal-looking characters that then prevent the full-body oracle from matching. This is a specific parser/model disagreement candidate; current Rust Terminal/actual-terminal controls are still required before classifying product corruption or correcting a criterion.

Target: `34-transit-floor-slow-cpu30-screen/08-slow64k-compress-i0`, slot3 candidate. Original inputs and all prepared/capture/ledger hashes remain unchanged. Four own offline containers used the exact recorded image1ecf08a7, networknone, CPU1, noNET_ADMIN/ports, original data/harness read-only and own notes output only; all four --rm names were verified absent afterward. Two initial investigator assertions needed tuple/JSON-list normalization; the successful replay exactly matches saved complete frames, body/header statistics and visible-state percentages. No product executable/new benchmark/build ran.

## Preserved measurement facts

- Window8.000709s; frozen body12/1.5FPS, header121/15.1FPS. Last matching body n11 atwindow+0.656425s; trailing no-matching-body interval7.344284s. Edge-inclusive maxgap7344ms includes this tail.
- Low1/low5=15.5 are reciprocals of the quantiles of only11 adjacent matched-frame gaps. They exclude the trailing no-match interval and cannot be cited as a tail/recovery improvement. Frozen visible incomplete90.0708% and no-body tail91.7954% use different definitions.
- Raw source has480completed records,0pending, writep99.242543ms/max1.191091ms. All121 replayed visible header N/T/scene entries independently match actual source payloads. Source generation/write stall and missing ledger records do not explain the gap.

## Exact character evidence

The first incomplete visible observation, header13 atwindow+0.780723s, has24 body rows tagged13 and five bottom rows tagged12 (34 differing cells). That is a genuine partial program-screen observation in the captured representation; it need not mean lost bytes or a transport tear.

At header20 (+0.900986s) and subsequent selected snapshots, all29 body row tags agree. The remaining mismatch is only three characters at zero-based row24, columns94–96: actual `[0m`, expected spaces. Header477 at+7.958109s still has this exact residue and otherwise matches its source body. This is incompatible with a simple80×24 clipping explanation: all29 body rows update and content/cursors beyond column80 are present. Source creation also passes the same cols/rows to Pty and Terminal; the child setsTIOCSWINSZ beforeexec (`main-source/src/pty.rs:183–194,270–276`; `server.rs:1291–1313`). No runtime geometry race was observed or proved.

Raw capture has no unescaped literal `[0m`. At captured chunk35 (+0.897128s), a partial CSI is followed by a new ESC reset. The relevant minimal bytes are:

```text
ESC [ 25 ; 95 H  ESC [  ESC [ 0 m
```

Direct pyte feed of those bytes, all-at-once or segmented, produces `[0m` at that cursor. Adding explicit CAN before the new ESC or supplying an ordinary reset leaves spaces. Actual full-capture per-read, whole-prefix and bytewise-prefix controls all reproduce the residue, independently of the frozen replay's boundary splitting. The observed discrepancy therefore is not introduced by that boundary algorithm alone. It is also not sufficient to generalize a mandatory error-recovery behavior for every terminal implementation.

## Concrete source path, still conditional

`Session::screen` appends the unfinished program tail to a full render (`main-source/src/server.rs:2294–2296`), and `refresh` builds its shadow from that screen (`2336–2341`). A later at-ground update can use `Terminal::diff_from` (`2308–2314`). Full render starts with CAN (`terminal.rs:735`), while the delta prefix starts synchronized-update+SGR reset without CAN (`terminal.rs:951`). Client `without_sync` strips synchronized markers (`client.rs:1384`), making an old held partial CSI adjacent to the new reset. This explains how the captured nested-ESC form can arise without a literal unescaped `[0m` in source output.

The next decisive comparison is the same minimal whole/segmented sequence through the current Rust Terminal/parser and a known actual terminal implementation. Do not mask three columns, rewrite the original12-body result, or infer an overall candidate regression/recovery until semantics are checked. Any future oracle change needs explicit provenance and consistent application to all roles. The old arithmetic remains a valid description of the frozen criterion.

Evidence: `far34-body-mismatch-diagnostic.{py,json}`, `far34-pyte-stream-controls.{py,json}`, `far34-all-header-source-joins.json`, `far34-transition-raw-snippets.json`, `far34-raw-escape-scan.json`, `far34-offline-replay-provenance.json`; receiver independent arithmetic is `far34-body-header-arithmetic-review.md/json`.

## Follow-up scope notes

Root's isolated tmux3.7c native controls reproduce blank cells for nestedESC whole/bytewise andCAN, and visible `[0m` only for the literal negative control;100×30 geometry and own-session cleanup were checked (`terminal-parser-control/tmux-native-controls.json/log/script/screens`). Thus disagreement with pyte0.8 is now observed for that native implementation, not a universal/GPU terminal theorem. Current Rust parser control is still a root build step.

The productPTY ordering above is distinct from the benchmark harness: frozen `agent.py:59–62` calls pty.fork/childexec before parent TIOCSWINSZ, so a harness startup-size race is possible in principle. This target's all29row/column94updates do not identify it as the cause. Neither raw measurement inputs nor original oracle/frame counts were changed.
