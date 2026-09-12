"""Diagnostic prototypes only. Never writes production source."""
from pathlib import Path
import shutil,subprocess,json,os,hashlib,difflib,statistics,gzip
from run import W,Worker
base=(W/'source/src/block/parser.rs').read_text()
old_advance='''        while !self.cursor.is_eof() && !self.cursor.at(b'\\n') {
            parser_cursor_bump!(self.cursor);
        }
        self.cursor.offset()'''
new_advance='''        let remaining = self.cursor.remaining_slice();
        let distance = memchr::memchr(b'\\n', remaining).unwrap_or(remaining.len());
        parser_cursor_advance!(self.cursor, distance);
        self.cursor.offset()'''
old_peek='''        let end = slice
            .iter()
            .position(|&b| b == b'\\n')
            .unwrap_or(slice.len());'''
new_peek='''        let end = memchr::memchr(b'\\n', slice).unwrap_or(slice.len());'''
assert base.count(old_advance)==1 and base.count(old_peek)==1
variants={'advance':base.replace(old_advance,new_advance),'peek':base.replace(old_peek,new_peek),'both':base.replace(old_advance,new_advance).replace(old_peek,new_peek)}
shutil.copytree(W/'source',W/'probe-source',dirs_exist_ok=True)
for p in (W/'probe-source/src').rglob('*.rs'):p.touch()
(W/'probes').mkdir(exist_ok=True)
shutil.copytree(W/'ferro/src',W/'probe-driver/src',dirs_exist_ok=True)
(W/'probe-driver/Cargo.toml').write_text((W/'ferro/Cargo.toml').read_text().replace('path="../source"','path="../probe-source"'))
shutil.copyfile(W/'ferro/Cargo.lock',W/'probe-driver/Cargo.lock')
class Probe(Worker):
 def __init__(self,name):
  self.engine=name
  self.p=subprocess.Popen([str(W/'bin'/f'probe-{name}'),str(W/'cases.json')],stdin=subprocess.PIPE,stdout=subprocess.PIPE,text=True)
# Exact HTML, including heading IDs and all retained known mismatches.
reference=[];w=Worker('ferro');cases=json.loads((W/'case-manifest.json').read_text())
for i in range(len(cases)):reference.append(hashlib.sha256(w.call('verify',i)['html'].encode()).hexdigest())
w.close();(W/'probes/baseline-html-sha256.json').write_text(json.dumps(reference))
env=dict(os.environ);env.pop('RUSTFLAGS',None);env.pop('CARGO_ENCODED_RUSTFLAGS',None)
for name,source in variants.items():
 (W/'probe-source/src/block/parser.rs').write_text(source)
 (W/'probes'/f'{name}.patch').write_text(''.join(difflib.unified_diff(base.splitlines(True),source.splitlines(True),fromfile='a/src/block/parser.rs',tofile='b/src/block/parser.rs')))
 with (W/'probes'/f'{name}-build.log').open('w') as log:
  subprocess.run(['cargo','build','--release','--offline','--locked','--manifest-path',str(W/'probe-driver/Cargo.toml')],cwd=W,env=env,stdout=log,stderr=subprocess.STDOUT,check=True)
 shutil.copy2(W/'probe-driver/target/release/corpus-ferro',W/'bin'/f'probe-{name}')
 w=Probe(name);hashes=[]
 for i,h in enumerate(reference):
  actual=hashlib.sha256(w.call('verify',i)['html'].encode()).hexdigest();assert actual==h,(name,i);hashes.append(actual)
 w.close();(W/'probes'/f'{name}-verification.json').write_text(json.dumps(dict(exact_cases=len(hashes),sha256=hashlib.sha256(json.dumps(hashes).encode()).hexdigest())))
 print('Built and verified',name,len(hashes),'cases',flush=True)
# No compilation or profiling overlaps these timing windows.
selected=json.loads((W/'selected.json').read_text());workers={'baseline':Worker('ferro')}|{n:Probe(n) for n in variants}
for run in [1,2]:
 raw=[];summary=[]
 for i in selected:
  values={n:[] for n in workers}
  for w in workers.values():w.call('window',i,25)
  for r in range(7):
   names=list(workers);names=names[r%4:]+names[:r%4]
   for n in names:
    row=workers[n].call('window',i,40);row.update(index=i,engine=n,round=r);raw.append(row);values[n].append(row['elapsed_ns']/row['count'])
  med={n:statistics.median(v) for n,v in values.items()};summary.append(dict(index=i,case=cases[i]['case'],median_ns=med,change_pct={n:(v/med['baseline']-1)*100 for n,v in med.items()}))
 (W/'probes'/f'run-{run}-raw.jsonl.gz').write_bytes(gzip.compress(('\n'.join(json.dumps(r) for r in raw)+'\n').encode(),mtime=0));(W/'probes'/f'run-{run}-summary.json').write_text(json.dumps(summary,indent=2)+'\n')
 print('Probe timing pass',run,'complete',flush=True)
for w in workers.values():w.close()
# Restore the baseline copy so no prototype is accidentally used later.
(W/'probe-source/src/block/parser.rs').write_text(base)
