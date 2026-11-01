"""The observer must reject partial bodies and retain split sync markers."""
import struct
import base64
import tempfile
from pathlib import Path
import unittest
from types import SimpleNamespace

import replay
from ledger_oracle import LedgerSource


class ReplayTests(unittest.TestCase):
    def test_actual_ledger_handles_a_late_source_that_skips_frame_ids(self):
        source = replay.Source("bandwidth", 100, 30)
        # A synchronized scene can start late at n=5 while its PRNG is still
        # at its first draw. Regenerating draws 0..5 gives the wrong body.
        payload = source.draw(5).encode()
        ledger = LedgerSource([dict(n=5, payload_b64=base64.b64encode(payload).decode())], 100, 30)
        data = b"\x1b[?7l" + payload
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "capture.bin"
            path.write_bytes(struct.pack("!dI", 100, len(data)) + data)
            job = dict(scene="bandwidth", cols=100, rows=30, window_start=99, window_end=101)
            self.assertEqual(replay.observe(path, job, ledger)["complete_body"]["frames"], 1)
            self.assertEqual(replay.observe(path, job, replay.Source("bandwidth", 100, 30))["complete_body"]["frames"], 0)
            self.assertFalse(ledger.missing)

    def test_complete_body_and_fragmented_sync(self):
        source = replay.Source("bandwidth", 100, 30)
        data = b"\x1b[?7l" + replay.BEGIN + source.draw(0).encode() + replay.END
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "capture.bin"
            with path.open("wb") as f:
                for i, part in enumerate((data[:-3], data[-3:-1], data[-1:])):
                    f.write(struct.pack("!dI", 100 + i / 10, len(part)) + part)
            job = dict(scene="bandwidth", cols=100, rows=30, window_start=99, window_end=101)
            # A separate source starts its deterministic PRNG at frame zero.
            result = replay.observe(path, job, replay.Source("bandwidth", 100, 30))
            self.assertEqual(result["complete_body"]["frames"], 1)
            self.assertEqual(result["uniform_header"]["frames"], 1)

    def test_header_alone_is_not_a_complete_body(self):
        source = replay.Source("compress", 100, 30)
        data = b"\x1b[?7l" + source.namespace["header"](0).encode()
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "capture.bin"
            path.write_bytes(struct.pack("!dI", 100, len(data)) + data)
            result = replay.observe(path, dict(scene="compress", cols=100, rows=30,
                                    window_start=99, window_end=101), source)
            self.assertEqual(result["uniform_header"]["frames"], 1)
            self.assertEqual(result["complete_body"]["frames"], 0)

    def test_two_complete_frames_in_one_read_are_both_retained(self):
        source = replay.Source("bandwidth", 100, 30)
        data = b"\x1b[?7l" + source.draw(0).encode() + source.draw(1).encode()
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "capture.bin"
            path.write_bytes(struct.pack("!dI", 100, len(data)) + data)
            result = replay.observe(path, dict(scene="bandwidth", cols=100, rows=30,
                                    window_start=99, window_end=101), replay.Source("bandwidth", 100, 30))
            self.assertEqual(result["complete_body"]["frames"], 2)

    def test_complete_erase_is_not_delayed_until_the_next_read(self):
        first = (b"\x1b[?7l\x1b[1;1HFRAME 000000 T=100000 compress"
                 b"\x1b[2;1H000000" + b" " * 93 + b"X")
        source = SimpleNamespace(body=lambda n: ("000000" + " " * 94,))
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "capture.bin"
            with path.open("wb") as f:
                for at, data in ((100., first), (100.1, b"\x1b[2;7H\x1b[K"), (100.2, b"\r")):
                    f.write(struct.pack("!dI", at, len(data)) + data)
            result = replay.observe(path, dict(scene="compress", cols=100, rows=2,
                                    window_start=99, window_end=101), source)
            self.assertEqual(result["complete_body"]["frames"], 1)
            self.assertEqual(result["complete_body"]["age_ms"], 100)
            self.assertAlmostEqual(result["visible_state"]["incomplete_pct"], 5.)
            self.assertAlmostEqual(result["visible_state"]["complete_pct"], 45.)
            self.assertAlmostEqual(result["visible_state"]["unknown_pct"], 50.)

    def test_state_time_is_clipped_to_the_post_switch_window(self):
        result = replay.state_time([(99., True), (101., False), (103., None), (105., True)], 102., 106.)
        self.assertAlmostEqual(result["incomplete_pct"], 25.)
        self.assertAlmostEqual(result["unknown_pct"], 50.)
        self.assertAlmostEqual(result["complete_pct"], 25.)


if __name__ == "__main__":
    unittest.main()
