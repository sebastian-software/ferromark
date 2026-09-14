#!/usr/bin/env python3
"""Archive an experiment workspace without duplicating every decoded input."""
import argparse
import gzip
import hashlib
import json
from pathlib import Path
import shutil

p=argparse.ArgumentParser(description=__doc__)
p.add_argument('experiments',type=Path)
p.add_argument('report',type=Path)
a=p.parse_args()
(a.report/'raw').mkdir(parents=True,exist_ok=True)
(a.report/'patches').mkdir(exist_ok=True)
for source in sorted((a.experiments/'artifacts').iterdir()):
 if source.suffix in ('.patch','.diff'):
  target=a.report/'patches'/source.name
  if source.suffix=='.diff': target=target.with_suffix('.failure.md')
  shutil.copyfile(source,target)
 elif source.suffix in ('.log','.md','.json','.txt') and 'disassembly' not in source.name:
  shutil.copyfile(source,a.report/'raw'/source.name)
for directory in sorted(a.experiments.iterdir()):
 if not directory.is_dir(): continue
 if not ((directory/'summary.json').exists() or (directory/'allocations.json').exists() or (directory/'usage.json').exists()): continue
 out=a.report/'raw'/directory.name;out.mkdir(exist_ok=True)
 for source in directory.iterdir():
  if source.suffix not in ('.json','.csv','.rs') or source.name=='corpus.json':continue
  if source.name in ('verification.json','samples.json','arena-capacities.json'):
   (out/(source.name+'.gz')).write_bytes(gzip.compress(source.read_bytes(),mtime=0))
  else:shutil.copyfile(source,out/source.name)
for filename in ('corpus.json','final-corpus.json','table-corpus.json','broad-corpus.json','default-autolink-corpus.json','parser-diagnostics-corpus.json','renderer-diagnostics-corpus.json'):
 source=a.experiments/filename
 if source.exists():(a.report/(filename+'.gz')).write_bytes(gzip.compress(source.read_bytes(),mtime=0))
for filename in ('build_lto_control.py','audit_text.py','screen.py','final_runs.py','followups.py'):
 source=a.experiments/filename
 if source.exists():shutil.copyfile(source,a.report/'raw'/filename)
for directory in ('marker','marker-a','marker-c','structure','structure-rank','renderer','url-simd'):
 source=a.experiments/directory/'EXPERIMENT.md'
 if source.exists():(a.report/'raw'/(directory+'-ledger.md')).write_text(source.read_text().rstrip()+'\n')
for label, source in (("worker.rs",a.experiments/'build-combined/worker.rs'),
                      ("original-worker.rs",a.experiments/'build-probe-no-lto/baseline/src/main.rs')):
 if source.exists():shutil.copyfile(source,a.report/'raw'/label)
manifest={str(f.relative_to(a.report)):hashlib.sha256(f.read_bytes()).hexdigest()
          for f in sorted(a.report.rglob('*')) if f.is_file() and f.name not in ('SHA256SUMS.json','README.md')}
(a.report/'SHA256SUMS.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(f'{len(manifest)} archived files')
