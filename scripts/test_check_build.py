"""Reject stale shared-target artifacts before preparing distribution checksums."""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent
CHECK = ROOT / "scripts/check-build.sh"


class BuildIdentityTests(unittest.TestCase):
    def run_command(self, args, cwd, env, ok=True):
        result = subprocess.run(args, cwd=cwd, env=env, text=True, capture_output=True)
        if ok:
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        return result

    def test_shared_cache_foreign_source_and_salt_are_rejected(self):
        with tempfile.TemporaryDirectory(prefix="rsh build identity ") as temporary:
            work = Path(temporary)
            baseline, candidate, target = (work/n for n in ("baseline tree", "candidate tree", "shared target"))
            baseline.mkdir(); (baseline/"src").mkdir()
            dependency = work/"shared helper"
            (dependency/"src").mkdir(parents=True)
            (dependency/"Cargo.toml").write_text('[package]\nname="buildguarddep"\nversion="0.0.0"\nedition="2021"\n')
            (dependency/"src/lib.rs").write_text('pub fn ping() -> u8 { 42 }\n')
            (baseline/"Cargo.toml").write_text('[package]\nname="rsh"\nversion="0.0.0"\nedition="2021"\n[dependencies]\nbuildguarddep={path="../shared helper"}\n')
            (baseline/"Cargo.lock").write_text('version=4\n[[package]]\nname="buildguarddep"\nversion="0.0.0"\n[[package]]\nname="rsh"\nversion="0.0.0"\ndependencies=["buildguarddep"]\n')
            shutil.copyfile(ROOT/"build.rs", baseline/"build.rs")
            (baseline/"src/main.rs").write_text('fn main() { std::hint::black_box(buildguarddep::ping()); println!("baseline {}", concat!("rsh-build-mark:", env!("RSH_BUILD"), ":end")); }\n')
            shutil.copytree(baseline, candidate)
            (candidate/"src/main.rs").write_text((candidate/"src/main.rs").read_text().replace("baseline", "candidate"))
            # Both snapshots predate the first compile, as exported trees do.
            for tree in (baseline, candidate):
                for path in tree.rglob("*"): os.utime(path, (1000000000, 1000000000))
                os.utime(tree, (1000000000, 1000000000))
            env = dict(os.environ, CARGO_TARGET_DIR=str(target))
            env.pop("RSH_BUILD_SALT", None)
            build = ["cargo", "build", "--release", "--locked", "--offline"]
            self.run_command(build, baseline, env)
            first = work/"baseline binary"
            shutil.copyfile(target/"release/rsh", first)
            self.run_command([str(CHECK), str(baseline), str(first)], work, env)
            wrong = self.run_command([str(CHECK), str(candidate), str(first)], work, env, ok=False)
            self.assertNotEqual(wrong.returncode, 0)
            self.assertIn("build mismatch", wrong.stderr)
            # A future Cargo may fix this cache collision; the guard must work either way.
            self.run_command(build, candidate, env)
            cached = self.run_command([str(CHECK), str(candidate), str(target/"release/rsh")], work, env, ok=False)
            print("shared target reused foreign source:", cached.returncode != 0, flush=True)
            dependencies = list((target/"release/deps").glob("libbuildguarddep-*.rlib"))
            self.assertTrue(dependencies)
            self.run_command(["cargo", "clean", "--release", "--locked", "--offline", "-p", "rsh"], candidate, env)
            self.assertTrue(all(p.is_file() for p in dependencies), "dependency cache is retained")
            self.run_command(build, candidate, env)
            fresh = work/"candidate binary"
            shutil.copyfile(target/"release/rsh", fresh)
            self.run_command([str(CHECK), str(candidate), str(fresh)], work, env)
            self.assertNotEqual(first.read_bytes(), fresh.read_bytes())
            salted = dict(env, RSH_BUILD_SALT="identity-test")
            self.assertNotEqual(self.run_command([str(CHECK), str(candidate), str(fresh)], work, salted, ok=False).returncode, 0)
            self.run_command(build, candidate, salted)
            self.run_command([str(CHECK), str(candidate), str(target/"release/rsh")], work, salted)
            self.check_distribution_checksum_gate(candidate, first, fresh, work, env)


    def check_distribution_checksum_gate(self, source, wrong, correct, work, env):
        (source/"scripts").mkdir()
        for name in ("dist.sh", "check-build.sh"):
            shutil.copy2(ROOT/"scripts"/name, source/"scripts"/name)
        commands=work/"fake tools"; commands.mkdir()
        stubs={
            "cargo": """import os,sys,shutil,pathlib
args=sys.argv[1:]
target=pathlib.Path(args[args.index("--target-dir")+1] if "--target-dir" in args else os.environ.get("CARGO_TARGET_DIR",str(pathlib.Path.cwd()/"target")))
if sys.argv[1] == "build":
    out=target/"release/rsh"; out.parent.mkdir(parents=True,exist_ok=True)
    shutil.copyfile(os.environ["FIXTURE_CORRECT"],out);out.chmod(0o755)
""",
            "docker": """import os,sys,shutil,pathlib
args=sys.argv[1:]
out=pathlib.Path(next(a[:-5] for a in args if a.endswith(":/out")))
assert out.is_absolute(), "Docker bind paths must be absolute"
assert "RSH_BUILD_SALT="+os.environ.get("RSH_BUILD_SALT", "") in args
arch="x86_64" if "linux/amd64" in args else "aarch64"
shutil.copyfile(os.environ["FIXTURE_LINUX"],out/("rsh-linux-"+arch))
""",
            "uname": """import sys
print("Linux aarch64" if sys.argv[1:] == ["-sm"] else "Linux")
""",
        }
        for name,body in stubs.items():
            p=commands/name;p.write_text("#!/usr/bin/env python3\n"+body);p.chmod(0o755)
        out=work/"distribution";out.mkdir()
        sums=out/"SHA256SUMS";sums.write_text("previous manifest\n")
        fake=dict(env,PATH=str(commands)+os.pathsep+env["PATH"],FIXTURE_CORRECT=str(correct),FIXTURE_LINUX=str(wrong))
        # The stubbed platform builders deliberately supply a stale artifact.
        failed=self.run_command(["sh",str(source/"scripts/dist.sh"),out.name],work,fake,ok=False)
        self.assertNotEqual(failed.returncode,0)
        self.assertIn("build mismatch",failed.stderr)
        self.assertEqual(sums.read_text(),"previous manifest\n")
        self.assertFalse((out/"SHA256SUMS.new").exists())
        # After correct artifacts arrive, the same script can commit the checksum manifest.
        fake["FIXTURE_LINUX"]=str(correct)
        self.run_command(["sh",str(source/"scripts/dist.sh"),out.name],work,fake)
        self.assertNotEqual(sums.read_text(),"previous manifest\n")
        self.assertFalse((out/"SHA256SUMS.new").exists())


if __name__ == "__main__":
    unittest.main()
