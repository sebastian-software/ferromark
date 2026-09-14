import subprocess
from pathlib import Path
root=Path('/Users/sebastian/Workspace/ferromark-v2'); base=Path('/private/tmp/ferromark-v2-rounds-2')
control='comment-table|comment-ack|wiki-chess-article-body|typescript-handbook-typescript-5-0'
for variant, pattern, modes in [
 ('marker-a','scanner-', ['parse','reuse']),
 ('marker-b','scanner-', ['parse','reuse']),
 ('structure','table-', ['parse','reuse']),
 ('renderer','autolink-', ['reuse','render']),
 ('url-simd','autolink-', ['reuse','render']),
 ('probe-new-worker','scan-|wiki-volcano|wiki-chess|typescript-handbook', ['reuse','render']),
]:
 command=['python3',str(root/'benchmarks/optimization-rounds/run.py'), str(base/f'build-{variant}'),str(base/'corpus.json'),str(base/f'screen-{variant}'),'--rounds','1','--pairs','3','--window-ms','20','--modes',*modes,'--filter',f'{pattern}|^({control})$']
 (base/'artifacts'/f'screen-{variant}-command.txt').write_text(' '.join(command)+'\n')
 with (base/'artifacts'/f'screen-{variant}.log').open('w') as log:
  result=subprocess.run(command,cwd=root,stdout=log,stderr=log)
 print(variant,result.returncode,flush=True)
 if result.returncode:
  print((base/'artifacts'/f'screen-{variant}.log').read_text(),flush=True)
