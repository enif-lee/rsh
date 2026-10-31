"""The source ledger must preserve output bytes and report discarded records."""
import base64
import builtins
import io
import json
import os
from pathlib import Path
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import probe
import probe_report


class LedgerTests(unittest.TestCase):
    def execute(self, text, scene):
        clock = [1700000000.0]
        output = io.BytesIO()
        def sleep(delay):
            clock[0] += delay
        with tempfile.TemporaryDirectory() as temp:
            destination = Path(temp) / "source.json"
            real_open = builtins.open
            def open_record(path, *args, **kwargs):
                self.assertEqual(path, "/tmp/source-ledger.json.tmp")
                return real_open(destination, *args, **kwargs)
            with patch.object(sys, "argv", ["scene.py", scene, ".05", "60"]), \
                 patch.object(sys, "stdout", SimpleNamespace(buffer=output, write=lambda s: output.write(s.encode()))), \
                 patch("os.get_terminal_size", return_value=os.terminal_size((100, 30))), \
                 patch("time.time", side_effect=lambda: clock[0]), \
                 patch("time.time_ns", side_effect=lambda: int(clock[0]*1e9)), \
                 patch("time.monotonic_ns", side_effect=lambda: int((clock[0]-1700000000)*1e9)), \
                 patch("time.sleep", side_effect=sleep), \
                 patch("os.path.exists", return_value=True), \
                 patch("os.replace"), \
                 patch("builtins.open", side_effect=open_record):
                exec(compile(text, "scene.py", "exec"), {})
            ledger = json.loads(destination.read_text()) if destination.exists() else None
        return output.getvalue(), ledger

    def test_byte_identical_and_exact_draw_payload(self):
        original = (Path(probe.lab.HERE) / "scene.py").read_text()
        for scene in ("bandwidth", "compress"):
            with self.subTest(scene=scene):
                baseline, _ = self.execute(original, scene)
                traced, ledger = self.execute(probe.ledger_scene(original), scene)
                self.assertEqual(baseline, traced)
                self.assertEqual(ledger["dropped"], 0)
                self.assertGreater(len(ledger["frames"]), 1)
                payload = b"".join(base64.b64decode(f["payload_b64"]) for f in ledger["frames"])
                self.assertIn(payload, baseline)
                self.assertEqual(ledger["bytes"], len(payload))

    def test_overflow_is_explicit(self):
        original = (Path(probe.lab.HERE) / "scene.py").read_text()
        small = probe.ledger_scene(original).replace("LEDGER_LIMIT = 32 * 1024 * 1024", "LEDGER_LIMIT = 1")
        baseline, _ = self.execute(original, "bandwidth")
        traced, ledger = self.execute(small, "bandwidth")
        self.assertEqual(baseline, traced)
        self.assertEqual(ledger["frames"], [])
        self.assertGreater(ledger["dropped"], 0)

    def test_agent_hooks_compile_for_both_trace_states(self):
        with tempfile.TemporaryDirectory() as directory:
            with patch.object(probe, "OUT", Path(directory)), \
                 patch.object(probe.lab, "slots", return_value=[]), \
                 patch("probe.time.sleep"):
                for traced in (False, True):
                    probe.configure(traced)
                    text = (Path(directory) / "agent.py").read_text()
                    compile(text, "agent.py", "exec")
                    self.assertEqual(text.count('ask("probe-end")'), 1)
                    self.assertEqual(text.count("os.kill(c.pid, signal.SIGUSR1)"), int(traced))


class TraceAnalysisTests(unittest.TestCase):
    def test_gate_poll_interval_and_clock_validation(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            folder = root / "on/0-cli"
            folder.mkdir(parents=True)
            meta = dict(pid=7, events=4, dropped=0, anchor_realtime_ns=10000*10**9,
                anchor_bracket_ns=50, dump_realtime_ns=10010*10**9, dump_monotonic_ns=10*10**9)
            (folder / "meta-7.json").write_text(json.dumps(meta))
            (folder / "trace-7.csv").write_text(
                "ns,kind,a,b,c,d,e,f,g,h\n"
                "1000000,gate,500000,20000000,100,129,500,30000000,0,0\n"
                "2000000,poll_begin,18,500000,20000000,100,0,0,0,0\n"
                "12000000,poll_end,10000000,1,0,0,0,0,0,0\n"
                "13000000,enqueue,2,10,100,30000000,20000000,0,0,0\n")
            job = dict(batch="on", slot=0, window_start=10000, window_end=10008)
            result, _, sleeps = probe_report.trace(root, job, "cli")
            self.assertTrue(result["valid"])
            self.assertAlmostEqual(result["pending_poll"]["token"]["total_ms"], 10)
            self.assertAlmostEqual(sleeps[0]["due_epoch"], 10000.02)
            self.assertAlmostEqual(sleeps[0]["decision_staleness_ms"], 1.5)
            self.assertEqual(result["enqueue_origins"]["2"]["cadence_changes"], 1)
            meta["dropped"] = 1
            (folder / "meta-7.json").write_text(json.dumps(meta))
            result, _, _ = probe_report.trace(root, job, "cli")
            self.assertFalse(result["valid"])
            self.assertIn("recorder overflow", result["reasons"])


if __name__ == "__main__":
    unittest.main()
