"""The two-build comparison must execute and sample the requested binary."""
import unittest
from unittest.mock import patch

import agent


class ClientCommandTests(unittest.TestCase):
    def test_tcp_aliases_keep_their_binary_port_session_and_screen_observer(self):
        for tool, binary, peer in (("rsh-tcp", "rsh", "server:2222"),
                                   ("rsh-b", "rsh-b", "server:2223"),
                                   ("rsh-b-tcp", "rsh-b", "server:2223")):
            with self.subTest(tool=tool), patch.object(agent.pty, "fork", return_value=(0, 0)), \
                 patch.object(agent.os, "execvpe", side_effect=RuntimeError("stop at exec")) as execute:
                client = agent.Client.__new__(agent.Client)
                with self.assertRaisesRegex(RuntimeError, "stop at exec"):
                    client.__init__(tool, "comparison-session", 100, 30)
                program, argv, env = execute.call_args.args
                self.assertEqual(program, binary)
                self.assertEqual(argv, [binary, "server", "comparison-session"])
                self.assertEqual(env["RSH_PEER"], peer)
                self.assertEqual(agent.PROCESS[tool], binary)
                self.assertTrue(client.diffs)


class BufferedBurstFilterTests(unittest.TestCase):
    def test_buffered_catchup_frames_are_excluded_and_capped_at_max_fps(self):
        # Second 0 stalls after 40 frames; second 1 flushes a 25-frame burst in
        # 2ms plus 55 normal frames (80 raw frames in second 1).
        seen = []
        for i in range(40):
            t = 100.0 + i / 60.0
            seen.append((t, i, t - 0.15, "bandwidth"))
        for i in range(25):
            t = 101.0 + i * 0.0001
            seen.append((t, 40 + i, 100.7 + i / 60.0, "bandwidth"))
        for i in range(1, 60):
            t = 101.0 + i / 60.0
            seen.append((t, 65 + i, t - 0.15, "bandwidth"))

        stats = agent.render_stats(seen, 100.0, 102.0, max_fps=60)
        self.assertEqual(stats["raw_frames"], 124)
        # Second 0 has 40 valid frames; second 1's 25-frame burst collapses to 1
        # frame + 59 paced frames = 60 frames (never 84!).
        self.assertEqual(stats["per_s"], [40, 60])
        self.assertEqual(stats["fps"], 50.0)
        self.assertLessEqual(stats["low5_fps"], 60.0)
        self.assertLessEqual(stats["low1_fps"], 60.0)

    def test_steady_cadence_Never_exceeds_target_fps_on_boundary_alignment(self):
        # 61 frames landing inside [0, 1.0) at ~60Hz boundary alignment must cap at 60.
        seen = [(100.0 + i * (0.99 / 60.0), i, 100.0 + i * (0.99 / 60.0), "compress") for i in range(61)]
        stats = agent.render_stats(seen, 100.0, 101.0, max_fps=60)
        self.assertLessEqual(stats["fps"], 60.0)
        self.assertEqual(stats["first_s_fps"], 60)
        self.assertEqual(stats["per_s"], [60])


if __name__ == "__main__":
    unittest.main()
