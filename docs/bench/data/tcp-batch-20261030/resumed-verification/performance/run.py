from pathlib import Path
import concurrent.futures,contextlib,fcntl,hashlib,json,os,signal,subprocess,sys,time
OUT=Path(__file__).resolve().parent
sys.path.insert(0,str(OUT/"harness"))
import lab,probe,experiment as ex,load_watch
probe.OUT=OUT;ex.OUT=OUT
PAIR=OUT.parent/"parity-release-pair-20261030"
ARTIFACTS={r["variant"]:r for r in json.loads((PAIR/"artifact-identities.json").read_text())}
SCHEDULE=[("00-udp-highlat","udp",False,[("highlat","compress"),("highlat","bandwidth")]),("01-tcp-highlat","tcp",True,[("highlat","compress"),("highlat","bandwidth")]),("02-udp-guards","udp",True,[("wan","bandwidth"),("lowbw","bandwidth")]),("03-tcp-guards","tcp",False,[("wan","bandwidth"),("lowbw","bandwidth")])]

def specs(protocol,cases):
    tools=("ssh","mosh","rsh-tcp","rsh-b-tcp") if protocol=="tcp" else ("ssh","mosh","rsh","rsh-b")
    rows=[]
    for net,scene in cases:
        for tool in tools:
            variant="candidate" if tool.startswith("rsh-b") else "baseline" if tool.startswith("rsh") else tool
            row=dict(test="render",tool=tool,net=net,scene=scene,iteration=0,duration=8,scene_fps=60,cols=100,rows=30,variant=variant)
            if variant in ARTIFACTS:row["binary_sha256"]=ARTIFACTS[variant]["sha256"]
            rows.append(row)
    assert len(rows)==8
    return rows

def verify_inputs():
    manifest=json.loads((OUT/"harness/manifest.json").read_text())
    assert all(hashlib.sha256((OUT/"harness"/n).read_bytes()).hexdigest()==sha for n,sha in manifest.items())
    for artifact in ARTIFACTS.values():assert hashlib.sha256(Path(artifact["binary"]).read_bytes()).hexdigest()==artifact["sha256"]
    return manifest

def installed_identity():
    expected={"rsh":ARTIFACTS["baseline"]["sha256"],"rsh-b":ARTIFACTS["candidate"]["sha256"]}
    rows=[]
    for k in lab.slots():
        for container in (lab.cli(k),lab.srv(k)):
            found=lab.dexec(container,"sha256sum","/usr/local/bin/rsh","/usr/local/bin/rsh-b").stdout.splitlines()
            for line in found:
                sha,path=line.split();assert sha==expected[Path(path).name],(container,path,sha)
            rows.append(dict(container=container,checks=found))
    return rows

def restart_second(k):
    lab.dexec(lab.srv(k),"sh","-c","pkill -x rsh-b; sleep .3; su dev -c 'nohup rsh-b serve --bind 0.0.0.0:2223 --authorized /home/dev/.rsh/authorized >/tmp/rsh-b.log 2>&1 &'",check=False)

def run():
    hashes=verify_inputs()
    assert not list(OUT.glob("*/batch.json")),"Fresh run required"
    signal.signal(signal.SIGINT,signal.default_int_handler)
    locks=[OUT.parents[0]/".lock",Path("/Users/ed/personal/rsh-parity/benchmark/lab/out/.lock")]
    with contextlib.ExitStack() as stack:
        for path in sorted(locks):
            path.parent.mkdir(parents=True,exist_ok=True)
            lock=stack.enter_context(path.open("a+"));fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
            lock.seek(0);lock.truncate();lock.write(str(os.getpid()));lock.flush()
        assert not lab.slots(),"Existing lab slots must not be overwritten"
        lab.guard()
        ex.save(OUT/"environment.json",dict(date=time.strftime("%Y-%m-%dT%H:%M:%S%z"),iterations=1,parallel=8,source_ledger="same observer on every participant; dumps after all receive windows close",schedule=SCHEDULE,binaries=ARTIFACTS,source_hashes=hashes,note="Paired development screen only; one observation per case; no publication or no-regression claim."))
        print("Waiting for 60 seconds of sampled headroom",flush=True)
        load_watch.wait_quiet(OUT/"preflight-host.jsonl",seconds=60)
        print("Headroom preflight passed",flush=True)
        try:
            lab.up(8,ARTIFACTS["baseline"]["binary"])
            lab.install_rsh_b(ARTIFACTS["candidate"]["binary"],lab.slots())
            ex.save(OUT/"installed-identities-before.json",installed_identity())
            for tag,protocol,reverse,cases in SCHEDULE:
                load_watch.wait_quiet(OUT/(tag+"-quiet.jsonl"))
                with load_watch.Watch(OUT/(tag+"-host.jsonl")) as watch:
                    with concurrent.futures.ThreadPoolExecutor(8) as pool:list(pool.map(restart_second,lab.slots()))
                    probe.batch(tag,False,reverse=reverse,specs=specs(protocol,cases),parallel=8)
                status=watch.status();ex.save(OUT/(tag+"-host-status.json"),status)
                if not status["accepted"]:raise RuntimeError("Host guard rejected phase: "+str(status))
                batch=json.loads((OUT/tag/"batch.json").read_text())
                if any(r.get("error") for r in batch["results"]):raise RuntimeError("Participant failed; do not continue")
            ex.save(OUT/"installed-identities-after.json",installed_identity())
        finally:
            lab.remove_slots()
    verify_inputs()

action=sys.argv[1] if len(sys.argv)>1 else "plan"
if action=="run":run()
elif action=="analyze":probe.analyze()
elif action=="plan":
    verify_inputs();print(json.dumps(dict(parallel=8,iterations=1,jobs=32,blocks=[dict(tag=t,protocol=p,reverse=r,jobs=specs(p,c)) for t,p,r,c in SCHEDULE]),indent=2))
else:raise SystemExit("plan, run or analyze")
