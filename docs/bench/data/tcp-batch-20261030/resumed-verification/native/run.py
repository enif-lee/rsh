from pathlib import Path
import hashlib,json,os,re,shutil,subprocess,time
base=Path(__file__).resolve().parent
source=base/'source';target=base/'target'
env=dict(os.environ)
for key in ('NO_COLOR','RSH_BUILD_SALT'):env.pop(key,None)
env['CARGO_TARGET_DIR']=str(target)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
files=[source/'Cargo.toml',source/'Cargo.lock',source/'build.rs',*sorted((source/'src').rglob('*')),*sorted((source/'tests').rglob('*'))]
manifest={str(p.relative_to(source)):sha(p) for p in files if p.is_file()}
(base/'source-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
(base/'compiler.log').write_text(subprocess.check_output(['rustc','-Vv'],text=True)+subprocess.check_output(['cargo','--version'],text=True)+'\n'+json.dumps({k:env.get(k) for k in ('CARGO_TARGET_DIR','RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','RSH_BUILD_SALT','NO_COLOR')}))
if not (base/'build.jsonl').exists():
    with (base/'build.jsonl').open('w') as out,(base/'build.log').open('w') as err:
        built=subprocess.run(['cargo','test','--locked','--offline','--all-targets','--no-run','--message-format=json'],cwd=source,env=env,stdout=out,stderr=err)
    assert built.returncode==0,'Build failed; inspect build.log'
messages=[json.loads(l) for l in (base/'build.jsonl').read_text().splitlines()]
assert any(m.get('reason')=='build-finished' and m.get('success') for m in messages)
product=target/'debug/rsh'
product_guard=subprocess.run([str(source/'scripts/check-build.sh'),str(source),str(product)],env=env,capture_output=True,text=True)
(base/'product-guard.log').write_text(product_guard.stdout+product_guard.stderr);assert product_guard.returncode==0
artifacts=[m for m in messages if m.get('reason')=='compiler-artifact' and m.get('executable') and m.get('profile',{}).get('test')]
assert len(artifacts)==4,len(artifacts)
assert all(not m['fresh'] for m in artifacts)
results=[];identities=[]
for artifact in artifacts:
    original=Path(artifact['executable']);frozen=base/original.name
    shutil.copy2(original,frozen)
    identity={'target':artifact['target'],'sha256':sha(frozen),'fresh':artifact['fresh'],'binary':str(frozen)}
    identities.append(identity)
    embedded=(artifact['target']['kind']==['lib'] or artifact['target']['name'] in ('session','binary_cli'))
    if embedded:
        guard=subprocess.run([str(source/'scripts/check-build.sh'),str(source),str(frozen)],env=env,capture_output=True,text=True)
        (base/(original.name+'-guard.log')).write_text(guard.stdout+guard.stderr)
        assert guard.returncode==0
        identity['embedded_source_check']='passed'
    else:
        identity['embedded_source_check']='not applicable: CLI driver or empty binary unit target; compiled product checked separately'
        (base/(original.name+'-guard.log')).write_text(identity['embedded_source_check']+'\n')
    start=time.monotonic()
    with (base/(original.name+'-tests.log')).open('w') as log:
        result=subprocess.run([str(frozen),'--nocapture'],cwd=source,env=env,stdout=log,stderr=subprocess.STDOUT)
    text=(base/(original.name+'-tests.log')).read_text()
    rows=[l for l in text.splitlines() if l.startswith('test result:')]
    results.append({'target':artifact['target']['name'],'kind':artifact['target']['kind'],'exit_code':result.returncode,'summary':rows,'elapsed_seconds':time.monotonic()-start})
    print(json.dumps(results[-1]),flush=True)
product=target/'debug/rsh'
guard=subprocess.run([str(source/'scripts/check-build.sh'),str(source),str(product)],env=env,capture_output=True,text=True)
(base/'product-guard.log').write_text(guard.stdout+guard.stderr);assert guard.returncode==0
identities.append({'target':'product-rsh','sha256':sha(product),'binary':str(product)})
assert all(sha(source/n)==h for n,h in manifest.items())
(base/'identities.json').write_text(json.dumps(identities,indent=2)+'\n')
(base/'results.json').write_text(json.dumps({'performance_jobs':0,'product_instrumented':False,'source_unchanged':True,'root_test_artifacts_fresh':False,'results':results},indent=2)+'\n')
assert all(r['exit_code']==0 for r in results),'Functional failure; compare baseline before drawing cause'
