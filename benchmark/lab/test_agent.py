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


if __name__ == "__main__":
    unittest.main()
