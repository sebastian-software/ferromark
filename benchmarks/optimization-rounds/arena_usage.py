#!/usr/bin/env python3
"""Measure occupied chunk bytes, including padding, separately from reserved capacity."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

p=argparse.ArgumentParser(description=__doc__)
p.add_argument('allocation_build',type=Path)
p.add_argument('corpus',type=Path)
p.add_argument('output',type=Path)
a=p.parse_args()
a.output.mkdir(parents=True,exist_ok=False)
metadata=json.loads((a.allocation_build/'build.json').read_text())
worker=Path(__file__).with_name('arena_usage.rs').read_bytes()
records={}
for name,info in metadata['engines'].items():
 root=Path(info['binary']).parents[2]
 (root/'src/bin').mkdir(exist_ok=True)
 (root/'src/bin/arena-usage.rs').write_bytes(worker)
 env=os.environ.copy();env.pop('CARGO_ENCODED_RUSTFLAGS',None)
 env['RUSTFLAGS']='-C target-cpu=generic';env['CARGO_TARGET_DIR']=str(root/'target')
 with (a.output/f'{name}-build.log').open('w') as log:
  subprocess.run(['cargo','+1.95','build','--release','--offline','--locked','--bin','arena-usage'],cwd=root,env=env,stdout=log,stderr=log,check=True)
 binary=root/'target/release/arena-usage'
 records[name]={'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'cases':{}}
 for case in json.loads(a.corpus.read_text())['cases']:
  assert case['profile']=='gfm'
  path=a.output/(case['name']+'.md');path.write_text(case['input'])
  length,capacity,used=map(int,subprocess.check_output([binary,path],text=True).split())
  assert length==case['byte_count']
  records[name]['cases'][case['name']]={'input_bytes':length,'arena_capacity_bytes':capacity,'arena_occupied_bytes_including_padding':used}
(a.output/'usage.json').write_text(json.dumps({'worker_sha256':hashlib.sha256(worker).hexdigest(),'build':metadata,'measurements':records},indent=2)+'\n')
print(a.output/'usage.json')
