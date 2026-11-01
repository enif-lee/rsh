import json,tempfile,time,unittest
from pathlib import Path
from unittest.mock import patch
import probe

class PairedProbeTests(unittest.TestCase):
    def test_second_tcp_build_is_actually_forced_to_tcp(self):
        for tool,dropped in (("rsh",False),("rsh-b",False),("rsh-tcp",True),("rsh-b-tcp",True)):
            with self.subTest(tool=tool), patch.object(probe.lab,"prepare") as prepare, patch.object(probe.subprocess,"Popen",side_effect=RuntimeError("stop at launch")):
                with self.assertRaisesRegex(RuntimeError,"stop at launch"):
                    probe.run_job(0,dict(tool=tool,test="render",net="lan"),None,{})
                prepare.assert_called_once_with(0,"lan",dropped)

    def test_declared_build_identity_survives_barrier_batch(self):
        with tempfile.TemporaryDirectory() as directory:
            jobs=[dict(tool="rsh-b" if n%2 else "rsh",test="echo",net="wan",variant="candidate" if n%2 else "baseline") for n in range(8)]
            def job(k,spec,barrier,ends):
                ends[k]=time.time()
                barrier.wait(timeout=5)
                return dict(spec,slot=k)
            with patch.object(probe,"OUT",Path(directory)), patch.object(probe,"configure"), patch.object(probe,"run_job",side_effect=job):
                probe.batch("paired",False,specs=jobs,parallel=8)
            for k,spec in enumerate(jobs):
                saved=json.loads((Path(directory)/"paired"/(str(k)+".json")).read_text())
                self.assertEqual(saved["variant"],spec["variant"])
