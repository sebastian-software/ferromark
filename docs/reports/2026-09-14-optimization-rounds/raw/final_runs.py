import subprocess
from pathlib import Path
root=Path('/Users/sebastian/Workspace/ferromark-v2');base=Path('/private/tmp/ferromark-v2-rounds-2')
for suite,modes,window in [('broad',['fresh','reuse','parse','render'],40),('parser-diagnostics',['parse','reuse'],30),('renderer-diagnostics',['reuse','render'],40),('default-autolink',['reuse','render'],40)]:
 command=['python3',str(root/'benchmarks/optimization-rounds/run.py'),str(base/'build-combined'),str(base/f'{suite}-corpus.json'),str(base/f'final-{suite}'),'--rounds','3','--pairs','3','--window-ms',str(window),'--modes',*modes]
 (base/'artifacts'/f'final-{suite}-command.txt').write_text(' '.join(command)+'\n')
 with (base/'artifacts'/f'final-{suite}.log').open('w') as log:
  result=subprocess.run(command,cwd=root,stdout=log,stderr=log)
 print(suite,result.returncode,flush=True)
 if result.returncode:
  print((base/'artifacts'/f'final-{suite}.log').read_text(),flush=True)
  raise SystemExit(result.returncode)
