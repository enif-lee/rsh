"""Timing and cleanup guarantees of the demo's changing network."""
import contextlib
import io
import json
import os
import shlex
import runpy
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import Mock, patch

import lab


class RunScheduleTests(unittest.TestCase):
    def options(self, *args):
        with tempfile.TemporaryDirectory() as directory, \
             open(Path(directory) / "lock", "a+") as lock, \
             patch.object(lab, "HERE", directory), \
             patch.object(lab, "open", return_value=lock, create=True), \
             patch.object(lab.sys, "argv", ["lab.py", "run", *args]), \
             patch.object(lab, "guard"), patch.object(lab, "run") as run:
            lab.main()
            return run.call_args.args[0]

    def test_default_full_matrix_has_three_repeats_and_one_memory_sample(self):
        opts = self.options(
            "--tests", "render,switch,echo,flood,outage,roam,reattach,udpblock,sessions",
            "--nets", "lan,wan,lowbw,verylow,slow64k,highlat,lossy,mobile,badmobile",
            "--switches", "lan:highlat,lan:lowbw,lowbw:highlat,highlat:lossy,wan:verylow")
        self.assertEqual(opts.parallel, 4)
        jobs = lab.jobs_of(opts)
        self.assertEqual(len(jobs), 755)
        self.assertEqual(sum(j["test"] == "render" for j in jobs), 270)
        self.assertEqual(sum(j["test"] == "switch" for j in jobs), 150)
        self.assertEqual([j["iteration"] for j in jobs if j["test"] == "sessions"], [0] * 5)
        self.assertEqual({j["iteration"] for j in jobs if j["test"] != "sessions"}, {0, 1, 2})

    def test_memory_repeats_can_exceed_the_other_repeats(self):
        opts = self.options("--tools", "ssh,rsh", "--tests", "echo,sessions", "--nets", "lan",
                            "--iterations", "1", "--session-iterations", "2", "--parallel", "4")
        jobs = lab.jobs_of(opts)
        self.assertEqual(opts.parallel, 4)
        self.assertEqual(sum(j["test"] == "echo" for j in jobs), 2)
        self.assertEqual(sum(j["test"] == "sessions" for j in jobs), 4)


class Clock:
    def __init__(self):
        self.at = 0.0

    def monotonic(self):
        return self.at

    def sleep(self, seconds):
        self.at += seconds


class MovingDemoTests(unittest.TestCase):
    def setUp(self):
        self.clock = Clock()
        self.addCleanup(patch.stopall)
        patch.object(lab.time, "monotonic", self.clock.monotonic).start()
        patch.object(lab.time, "sleep", self.clock.sleep).start()
        self.demo = lab.Demo.__new__(lab.Demo)
        self.demo.tools = ["ssh", "rsh"]
        self.demo.now = lambda: 1700000000 + self.clock.at
        self.demo.shape = Mock()
        self.workers = []
        def launch(*args, **kwargs):
            proc = Mock(stdin=io.StringIO(), stdout=io.StringIO("ready\n"), returncode=0)
            pending = proc.stdin
            pending.close = Mock()
            def finish(**kwargs):
                job = json.loads(pending.getvalue())
                self.workers.append(job)
                proc.poll.return_value = 0
                return json.dumps([job["start"] + p["offset_s"] for p in job["trace"]]), ""
            proc.communicate.side_effect = finish
            proc.poll.return_value = None
            return proc
        patch.object(lab.subprocess, "Popen", launch).start()
        self.demo.status = Mock()
        self.demo.rss = lambda k: (100, 200)
        self.demo.each = lambda f: [f(k) for k in range(2)]

    def test_two_cycles_share_a_clock_without_accumulating_apply_time(self):
        self.demo.shape.side_effect = lambda *a, **kw: self.clock.sleep(0.1)
        rss, changes = self.demo.moving(60, "compress")
        self.assertEqual(rss, [(100, 200), (100, 200)])
        self.assertEqual([r["scheduled_s"] for r in changes[:-1]],
                         [0, 6, 10, 14, 17, 18, 23, 30, 36, 40, 44, 47, 48, 53])
        self.assertAlmostEqual(changes[-1]["at"] - 1700000000, 60.35, places=5)
        self.assertEqual(len(self.workers), 4)
        self.assertTrue(all(job == self.workers[0] for job in self.workers))
        self.assertTrue(all(len(p["applied_at"]) == 4 for p in self.demo.network_trace))
        self.assertTrue(changes[-1]["after_measurement"])
        self.demo.shape.assert_called_with("moving", 0, change=True)

    def test_ending_during_handover_restores_a_working_link(self):
        _, changes = self.demo.moving(17.5, "bandwidth")
        self.assertEqual(changes[-2]["outage_time_pct"], 100)
        self.assertEqual(changes[-1]["phase"], "drain: clear signal")
        self.assertEqual(changes[-1]["loss_pct"], 0)
        self.assertIn("clear signal (draining)", self.demo.label)
        self.demo.shape.assert_called_with("moving", 0, change=True)

    def test_interrupt_restores_the_link_before_propagating(self):
        def status(text):
            if "weak signal" in text:
                raise KeyboardInterrupt
        self.demo.status.side_effect = status
        with self.assertRaises(KeyboardInterrupt):
            self.demo.moving(30, "compress")
        self.demo.shape.assert_called_with("moving", 0, change=True)
        self.assertEqual(len(self.workers), 4)

    def test_default_duration_contains_a_full_cycle_for_each_scene(self):
        self.demo.everyone = Mock()
        self.demo.shown = lambda *a: [True, True]
        patch.object(lab, "rx_total", lambda k: 1000).start()
        windows = []
        self.demo.render(windows, ["moving"], ["compress", "bandwidth"], 4, 60, 3)
        self.assertEqual([w["seconds"] for w in windows], [30, 30])
        for w in windows:
            self.assertEqual(w["end"] - w["start"], 30)
            self.assertEqual([c["scheduled_s"] for c in w["network_changes"][:-1]],
                             [0, 6, 10, 14, 17, 18, 23])
            self.assertTrue(all(w["done"]))
        self.assertEqual(len(self.demo.everyone.call_args_list), 2)
        for call, window in zip(self.demo.everyone.call_args_list, windows):
            command = shlex.split(call.args[0].split(";")[1])
            source_seconds, source_start = float(command[3]), float(command[-1])
            self.assertEqual(source_start, window["start"] - lab.DEMO_SKIP)
            self.assertEqual(source_start + source_seconds, window["end"])

    def test_phase_changes_keep_the_qdisc_and_set_both_rates(self):
        for phase, (_, _, _, _, down, _) in enumerate(lab.MOVING):
            for container, rate in [(lab.srv(0), down), (lab.cli(0), down // 4)]:
                command = "; ".join(lab.netem(0, container, "moving", phase, change=True))
                self.assertIn("tc qdisc change", command)
                self.assertNotIn("qdisc del", command)
                self.assertIn("rate %dkbit" % rate, command)
                self.assertNotIn("distribution", command)
                self.assertNotIn("random", command)

    def test_trace_repeats_identically_and_loss_is_a_common_time_interval(self):
        trace = lab.timeline(lab.MOVING, 60)
        first = [p for p in trace if p["offset_s"] < 30]
        second = [dict(p, offset_s=round(p["offset_s"] - 30, 6)) for p in trace if p["offset_s"] >= 30]
        self.assertEqual(first, second)
        self.assertEqual([p["offset_s"] for p in trace], sorted(set(p["offset_s"] for p in trace)))
        for phase, (seconds, delay, jitter, loss, down, _) in enumerate(lab.MOVING):
            points = [p for p in first if p["phase"] == phase]
            end = sum(p[0] for p in lab.MOVING[:phase + 1])
            lost = sum((points[i + 1]["offset_s"] if i + 1 < len(points) else end) - p["offset_s"]
                       for i, p in enumerate(points) if p["loss_pct"] == 100)
            self.assertAlmostEqual(lost, seconds * loss / 100)
            self.assertTrue(all(p["down_kbit"] == down and p["up_kbit"] == down // 4 for p in points))

    def test_static_run_cannot_silently_use_only_the_first_phase(self):
        with patch.object(lab.sys, "argv", ["lab.py", "run", "--nets", "moving"]):
            with contextlib.redirect_stderr(io.StringIO()):
                with self.assertRaises(SystemExit) as error:
                    lab.main()
        self.assertEqual(error.exception.code, 2)


class MovingSummaryTests(unittest.TestCase):
    def test_stalled_tool_does_not_shift_its_window_into_the_drain(self):
        window = dict(kind="render", net="moving", scene="compress", start=100, end=110,
                      until=120, seconds=10, done=[True, True])
        frames = {lab.cli(0): "101 1 100 compress\n109 2 108 compress\n111 3 109 compress\n",
                  lab.cli(1): "105 1 100 compress\n109 2 108 compress\n111 3 109 compress\n"}
        with tempfile.TemporaryDirectory() as directory:
            with patch.object(lab, "HERE", directory), patch.object(lab, "dexec", side_effect=lambda c, *a, **kw: Mock(stdout=frames[c])):
                with contextlib.redirect_stdout(io.StringIO()):
                    lab.demo_summary(["ssh", "rsh"], [window])
            results = json.loads(next((Path(directory) / "out").glob("demo-*.json")).read_text())["results"]
            self.assertEqual([r["fps"] for r in results], [0.2, 0.2])
            self.assertEqual([r["max_gap_ms"] for r in results], [8000, 5000])

    def test_late_scene_start_keeps_the_scheduled_end(self):
        clock = Clock()
        clock.at = 100
        output = io.StringIO()
        output.buffer = io.BytesIO()
        argv = ["scene.py", "ticker", "0.2", "10", "0", "", "99.8"]
        with patch.object(sys, "argv", argv), patch.object(sys, "stdout", output), \
             patch.object(os, "get_terminal_size", return_value=os.terminal_size((100, 30))), \
             patch.object(time, "time", clock.monotonic), patch.object(time, "sleep", clock.sleep):
            runpy.run_path(str(Path(lab.HERE) / "scene.py"), run_name="__main__")
        self.assertNotIn(b"FRAME", output.buffer.getvalue())
        self.assertEqual(clock.at, 100)


class MovingWorkerTests(unittest.TestCase):
    def test_disconnect_stops_worker_and_restores_link(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            tc = root / "tc"
            tc.write_text('#!/bin/sh\nprintf "%s\\n" "$*" >> "$MOVING_TEST_LOG"\n')
            tc.chmod(0o755)
            log = root / "commands"
            proc = subprocess.Popen([sys.executable, str(Path(lab.HERE) / "moving.py"), "test0", "down"],
                                    stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
                                    env=dict(os.environ, PATH=directory + os.pathsep + os.environ["PATH"], MOVING_TEST_LOG=str(log)))
            self.addCleanup(lambda: proc.poll() is None and proc.kill())
            self.assertEqual(proc.stdout.readline().strip(), "ready")
            point = dict(offset_s=0, delay_ms=100, loss_pct=100, down_kbit=128, up_kbit=32)
            restore = dict(point, delay_ms=35, loss_pct=0, down_kbit=8000)
            proc.stdin.write(json.dumps(dict(start=time.time(), seconds=30, trace=[point], restore=restore)) + "\n")
            proc.stdin.flush()
            until = time.monotonic() + 2
            while not log.exists() and time.monotonic() < until:
                time.sleep(0.01)
            self.assertTrue(log.exists())
            output, error = proc.communicate(timeout=2)
            self.assertEqual(proc.returncode, 0, error)
            self.assertEqual(len(json.loads(output)), 1)
            self.assertEqual(log.read_text().splitlines(), [
                "qdisc change dev test0 root netem delay 100ms loss 100% rate 128kbit",
                "qdisc change dev test0 root netem delay 35ms loss 0% rate 8000kbit"])


if __name__ == "__main__":
    unittest.main()
