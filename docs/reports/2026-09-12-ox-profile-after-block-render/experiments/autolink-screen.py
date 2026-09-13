import sys
from experiment import *
for name in ['autolink-prefilter']:
 if not build(name) or not verify(name):continue
 ok=True
 for script in ['verify-timing.py','verify-blocks.py','verify-extra.py','verify-extended.py','verify-fences.py']:
  p=subprocess.run([sys.executable,str(W/script),name]);ok=ok and p.returncode==0
 if ok:run(name,5,30)
 else:print('Rejected incorrect variant',name,flush=True)

if (W/"autolink-prefilter/summary.json").exists():
 for r in range(1,3):run("autolink-prefilter",7,50,"autolink-prefilter/confirm-"+str(r),r,indices=json.loads((W/"confirmation-selected.json").read_text()))
