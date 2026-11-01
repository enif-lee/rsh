from pathlib import Path
import subprocess,os,json,hashlib,re
base=Path(__file__).resolve().parent
checker=Path("/Users/ed/personal/rsh-parity/scripts/check-build.sh")
env=dict(os.environ);env.pop("NO_COLOR",None);env.pop("RSH_BUILD_SALT",None)
env["CARGO_TARGET_DIR"]=str(base/"isolated-target")
executables={};builds=[]
for variant in ("candidate","baseline"):
    source=base/variant
    with (base/(variant+"-clean.log")).open("w") as log:
        clean=subprocess.run(["cargo","clean","--locked","--offline","--profile","dev","-p","rsh","--manifest-path",str(source/"Cargo.toml")],env=env,stdout=log,stderr=subprocess.STDOUT)
    assert clean.returncode==0
    with (base/(variant+"-build.jsonl")).open("w") as out,(base/(variant+"-build.log")).open("w") as err:
        built=subprocess.run(["cargo","test","--locked","--offline","--manifest-path",str(source/"Cargo.toml"),"--test","fallback_matrix","--no-run","--message-format=json"],env=env,stdout=out,stderr=err)
    assert built.returncode==0,variant
    messages=[json.loads(l) for l in (base/(variant+"-build.jsonl")).read_text().splitlines()]
    artifact=next(m for m in messages if m.get("target",{}).get("name")=="fallback_matrix" and m.get("executable"))
    assert artifact["fresh"] is False
    frozen=base/(variant+"-tests");frozen.write_bytes(Path(artifact["executable"]).read_bytes());frozen.chmod(0o755)
    check=subprocess.run([str(checker),str(source),str(frozen)],capture_output=True,text=True,env=env)
    assert check.returncode==0,check.stderr
    (base/(variant+"-guard.log")).write_text(check.stdout+check.stderr)
    expected_build=re.search(r"verified build ([0-9a-f]{12}):",check.stdout).group(1)
    identity=subprocess.run([str(frozen),"--exact","diagnostic_identity","--nocapture"],capture_output=True,text=True,env=env)
    (base/(variant+"-identity.log")).write_text(identity.stdout+identity.stderr)
    assert identity.returncode==0
    found=re.search(r"DIAG identity build=([0-9a-f]{12}) fixture_sha256=([0-9a-f]{64})",identity.stdout+identity.stderr)
    expected_fixture=hashlib.sha256((source/"tests/fallback_matrix.rs").read_bytes()).hexdigest()
    assert found and found.groups()==(expected_build,expected_fixture)
    manifest=json.loads((base/(variant+"-source-manifest.json")).read_text())
    assert all(hashlib.sha256((source/n).read_bytes()).hexdigest()==sha for n,sha in manifest.items())
    builds.append(dict(variant=variant,build_id=expected_build,fixture_sha256=expected_fixture,executable_sha256=hashlib.sha256(frozen.read_bytes()).hexdigest(),root_artifact_fresh=artifact["fresh"],source_unchanged=True))
    executables[variant]=frozen;print(json.dumps(builds[-1]),flush=True)
assert builds[0]["fixture_sha256"]==builds[1]["fixture_sha256"]
assert builds[0]["build_id"]!=builds[1]["build_id"]
assert builds[0]["executable_sha256"]!=builds[1]["executable_sha256"]
(base/"build-identities.json").write_text(json.dumps(builds,indent=2)+"\n")
results=[]
for index,variant in enumerate(("candidate","baseline","baseline","candidate")):
    directory=base/(str(index)+"-"+variant);directory.mkdir();runenv=dict(env,DIAG_DIR=str(directory))
    with (directory/"test.log").open("w") as log:
        result=subprocess.run([str(executables[variant]),"--nocapture"],env=runenv,stdout=log,stderr=subprocess.STDOUT)
    text=(directory/"test.log").read_text()
    rows=[line for line in text.splitlines() if line.startswith("test result:")]
    target_failed="---- where_udp_is_blocked_the_session_goes_over_tcp stdout ----" in text or "test where_udp_is_blocked_the_session_goes_over_tcp ... FAILED" in text
    summary=dict(run=index,variant=variant,exit_code=result.returncode,totals=rows,target_failed=target_failed,fallback=[l for l in text.splitlines() if "DIAG fallback" in l])
    results.append(summary);print(json.dumps(summary),flush=True)
(base/"run-summary.json").write_text(json.dumps(dict(performance_jobs=0,diagnostic_only=True,max_attempts=4,builds=builds,results=results),indent=2)+"\n")
