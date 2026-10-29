"""Timing and cleanup guarantees of the demo's changing network."""
import contextlib
import io
import unittest
from unittest.mock import Mock, patch

import lab


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
        self.demo.status = Mock()
        self.demo.rss = lambda k: (100, 200)
        self.demo.each = lambda f: [f(k) for k in range(2)]

    def test_two_cycles_share_a_clock_without_accumulating_apply_time(self):
        self.demo.shape.side_effect = lambda *a, **kw: self.clock.sleep(0.1)
        rss, changes = self.demo.moving(60, "compress")
        self.assertEqual(rss, [(100, 200), (100, 200)])
        self.assertEqual([r["scheduled_s"] for r in changes[:-1]],
                         [0, 6, 10, 14, 17, 18, 23, 30, 36, 40, 44, 47, 48, 53])
        self.assertAlmostEqual(changes[-1]["at"] - 1700000000, 60.1, places=5)
        self.assertTrue(changes[-1]["after_measurement"])
        self.demo.shape.assert_called_with("moving", 0, change=True)

    def test_ending_during_handover_restores_a_working_link(self):
        _, changes = self.demo.moving(17.5, "bandwidth")
        self.assertEqual(changes[-2]["loss_pct"], 100)
        self.assertEqual(changes[-1]["phase"], "drain: clear signal")
        self.assertEqual(changes[-1]["loss_pct"], 0.1)
        self.assertIn("clear signal (draining)", self.demo.label)
        self.demo.shape.assert_called_with("moving", 0, change=True)

    def test_interrupt_restores_the_link_before_propagating(self):
        def apply(net, phase, change):
            if phase == 2:
                raise KeyboardInterrupt
        self.demo.shape.side_effect = apply
        with self.assertRaises(KeyboardInterrupt):
            self.demo.moving(30, "compress")
        self.demo.shape.assert_called_with("moving", 0, change=True)

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

    def test_phase_changes_keep_the_qdisc_and_set_both_rates(self):
        for phase, (_, _, _, _, down, _) in enumerate(lab.MOVING):
            for container, rate in [(lab.srv(0), down), (lab.cli(0), down // 4)]:
                command = "; ".join(lab.netem(0, container, "moving", phase, change=True))
                self.assertIn("tc qdisc change", command)
                self.assertNotIn("qdisc del", command)
                self.assertIn("rate %dkbit" % rate, command)
                self.assertIn("distribution normal", command)

    def test_static_run_cannot_silently_use_only_the_first_phase(self):
        with patch.object(lab.sys, "argv", ["lab.py", "run", "--nets", "moving"]):
            with contextlib.redirect_stderr(io.StringIO()):
                with self.assertRaises(SystemExit) as error:
                    lab.main()
        self.assertEqual(error.exception.code, 2)


if __name__ == "__main__":
    unittest.main()
