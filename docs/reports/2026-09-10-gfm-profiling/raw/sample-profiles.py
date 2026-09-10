from pathlib import Path
import subprocess,time,json
root=Path('/Users/sebastian/Workspace/ferromark')
out=root/'target/gfm-profile'
cases=[('plain','commonmark'),('plain','gfm'),('commonmark-50k','commonmark'),('commonmark-50k','gfm'),('gfm-overlap-tables','cm+tables'),('gfm-overlap-tables','gfm'),('tables-5k','gfm'),('autolinks','gfm'),('tasks','gfm')]
results=[]
for case,preset in cases:
 label=f'{case}-{preset}'
 with (out/f'{label}.process.log').open('w') as log:
  p=subprocess.Popen([str(out/'profile-harness'),str(out/f'{case}.md'),preset,'0','--forever'],stdout=log,stderr=log)
  try:
   time.sleep(.25)
   assert p.poll() is None,'harness exited before profiling'
   r=subprocess.run(['/usr/bin/sample',str(p.pid),'4','-mayDie','-fullPaths','-file',str(out/f'{label}.sample.txt')],capture_output=True,text=True,timeout=20)
   (out/f'{label}.sample.log').write_text(r.stdout+r.stderr)
   assert r.returncode==0,r.stderr
   assert p.poll() is None,'harness exited during profiling'
   results.append({'case':case,'preset':preset,'sample_seconds':4,'profile':f'{label}.sample.txt'})
   print(label,'sample complete',flush=True)
  finally:
   p.terminate()
   try:p.wait(timeout=3)
   except subprocess.TimeoutExpired:p.kill();p.wait()
(out/'profiles.json').write_text(json.dumps(results,indent=2)+'\n')
