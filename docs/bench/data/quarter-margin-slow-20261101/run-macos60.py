from pathlib import Path
import os,subprocess,json,shutil,hashlib
s=Path('/Users/ed/personal/rsh/benchmark/lab/out/lowtail-research-20261031');w=Path('/Users/ed/personal/rsh-quarter-margin')
env=os.environ.copy();env.pop('NO_COLOR',None);env['CARGO_TARGET_DIR']=str(s/'macos60-target');env['CARGO_BUILD_JOBS']='4'
assert not (s/'macos60-build-result.json').exists()
with (s/'macos60-all-tests-build.jsonl').open('w') as out:
 subprocess.run(['cargo','test','--locked','--all-targets','--no-run','--message-format=json'],cwd=w,env=env,stdout=out,check=True)
results=[]
for line in (s/'macos60-all-tests-build.jsonl').read_text().splitlines():
 d=json.loads(line)
 if d.get('reason')!='compiler-artifact' or not d.get('executable') or not d.get('profile',{}).get('test'):continue
 name='library' if 'lib' in d['target']['kind'] else d['target']['name']
 dest=s/f'macos60-{name}-tests';shutil.copy2(d['executable'],dest)
 with (s/f'macos60-{name}-tests.log').open('w') as out:
  run=subprocess.run([str(dest),'--test-threads=4'],cwd=w,env=env,stdout=out,stderr=subprocess.STDOUT,timeout=180)
 results.append({'name':name,'executable':str(dest),'sha256':hashlib.sha256(dest.read_bytes()).hexdigest(),'exit_code':run.returncode})
 assert run.returncode==0,results[-1]
with (s/'macos60-release-build.jsonl').open('w') as out:
 subprocess.run(['cargo','build','--locked','--release','--message-format=json'],cwd=w,env=env,stdout=out,check=True)
release=s/'rsh-cadence60-macos';shutil.copy2(s/'macos60-target/release/rsh',release)
subprocess.run([str(release),'--version'],check=True)
(s/'macos60-build-result.json').write_text(json.dumps({'tests':results,'release_sha256':hashlib.sha256(release.read_bytes()).hexdigest(),'target':'aarch64-apple-darwin','no_color_removed':True,'fd_limit':4096},indent=2)+'\n')
print('macOS tests/releases completed',hashlib.sha256(release.read_bytes()).hexdigest(),flush=True)
