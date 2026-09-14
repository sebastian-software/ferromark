import subprocess,time
from pathlib import Path
base=Path('/private/tmp/ferromark-v2-rounds-2');root=Path('/Users/sebastian/Workspace/ferromark-v2')
start=time.monotonic()
while not (base/'final-default-autolink/summary.json').exists():
 if time.monotonic()-start>600:raise SystemExit('final suite did not finish')
 time.sleep(1)
pattern='^(comment-review|comment-ack|comment-unicode|typescript-handbook-compiler-options|wiki-chess-article-body|wiki-volcano-article-body)$'
for name,build,pairs in [('long-followup','build-combined',5),('same-binary-control','same-binary-control-build',3)]:
 command=['python3',str(root/'benchmarks/optimization-rounds/run.py'),str(base/build),str(base/'corpus.json'),str(base/name),'--rounds','3','--pairs',str(pairs),'--window-ms','100','--modes','fresh','reuse','render','--filter',pattern]
 (base/'artifacts'/f'{name}-command.txt').write_text(' '.join(command)+'\n')
 with (base/'artifacts'/f'{name}.log').open('w') as log:
  result=subprocess.run(command,cwd=root,stdout=log,stderr=log)
 print(name,result.returncode,flush=True)
 if result.returncode:raise SystemExit(result.returncode)
