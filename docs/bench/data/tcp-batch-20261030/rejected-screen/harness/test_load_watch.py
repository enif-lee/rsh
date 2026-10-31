"""Load evidence must use interval CPU data and reject unobserved/busy runs."""
import unittest
import tempfile
from pathlib import Path
from unittest.mock import patch

import load_watch as watch


class HostLoadTests(unittest.TestCase):
    def test_since_boot_row_is_not_used_as_interval_idle(self):
        result = watch.parse_iostat("cpu load average\n us sy id 1m 5m 15m\n 1 1 98 1.00 2.00 3.00\n 50 30 20 8.00 6.00 4.00\n")
        self.assertEqual(result["idle_pct"], 20)
        self.assertFalse(watch.quiet(dict(result, max_external_process_cpu_pct=0)))
        with self.assertRaises(AssertionError):
            watch.parse_iostat("1 1 98 1.00 2.00 3.00\n")

    def test_busy_process_or_cpu_and_missing_sampling_fail_the_guard(self):
        sample = lambda at, idle=80, cpu=0: dict(at=at, idle_pct=idle, max_external_process_cpu_pct=cpu)
        self.assertTrue(watch.assess([sample(0), sample(5), sample(10)])["accepted"])
        self.assertFalse(watch.assess([sample(0,20), sample(5,20), sample(10,20)])["accepted"])
        self.assertFalse(watch.assess([sample(0,cpu=200), sample(5,cpu=200)])["accepted"])
        self.assertFalse(watch.assess([sample(0), sample(20)])["accepted"])
        self.assertFalse(watch.assess([])["accepted"])
        self.assertFalse(watch.assess([sample(0), dict(at=5,error="no sample")])["accepted"])

    def test_phase_edges_and_sleep_gap_cannot_pass(self):
        samples = [dict(at=t, idle_pct=80, max_external_process_cpu_pct=0) for t in (20,25)]
        self.assertFalse(watch.assess(samples, phase_start=0, phase_end=25)["accepted"])
        self.assertFalse(watch.assess(samples, phase_start=20, phase_end=45)["accepted"])
        self.assertFalse(watch.assess(samples, phase_start=20, phase_end=25, monotonic_elapsed=0)["accepted"])
        self.assertTrue(watch.assess(samples, phase_start=19, phase_end=26, monotonic_elapsed=7)["accepted"])

    def test_quiet_streak_restarts_after_a_sampling_gap(self):
        samples = [dict(at=t, idle_pct=80, max_external_process_cpu_pct=0) for t in (1,6,26,31,36)]
        with tempfile.TemporaryDirectory() as directory, patch.object(watch,"sample",side_effect=samples) as take, patch.object(watch.time,"sleep"):
            watch.wait_quiet(Path(directory)/"load.jsonl",seconds=10)
            self.assertEqual(take.call_count,5)


if __name__ == "__main__":
    unittest.main()
