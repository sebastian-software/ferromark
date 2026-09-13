from pathlib import Path
import os,subprocess,shutil,json,hashlib,difflib
W=Path(__file__).parent
D=W/'attribution';D.mkdir(exist_ok=True);(D/'source').mkdir(exist_ok=True)
shutil.copyfile(W/'source/Cargo.toml',D/'source/Cargo.toml');shutil.copytree(W/'source/src',D/'source/src',dirs_exist_ok=True)
changes={'src/inline/marks.rs':['fn collect_marks_impl<'], 'src/inline/mod.rs':['    fn emit_events('], 'src/inline/links.rs':['pub fn find_autolinks_into(']}
patch=''
for f,needles in changes.items():
 p=D/'source'/f;base=p.read_text();s=base
 for n in needles:
  assert s.count(n)==1;s=s.replace(n,('    #[inline(never)]\n' if n.startswith('    ') else '#[inline(never)]\n')+n)
 p.write_text(s);patch+=''.join(difflib.unified_diff(base.splitlines(True),s.splitlines(True),fromfile='a/'+f,tofile='b/'+f))
(D/'patch.diff').write_text(patch)
(D/'ferro').mkdir(exist_ok=True)
for f in ['Cargo.toml','Cargo.lock']:shutil.copyfile(W/'ferro'/f,D/'ferro'/f)
shutil.copytree(W/'ferro/src',D/'ferro/src',dirs_exist_ok=True)
env=dict(os.environ);env.pop('CARGO_ENCODED_RUSTFLAGS',None);env['RUSTFLAGS']='-C force-frame-pointers=yes'
with (D/'build.log').open('w') as log:subprocess.run(['cargo','build','--offline','--locked','--profile','sample','--manifest-path',str(D/'ferro/Cargo.toml')],cwd=D,env=env,stdout=log,stderr=subprocess.STDOUT,check=True)
shutil.copy2(D/'ferro/target/sample/corpus-ferro',W/'bin/attribution-sample')
dsym=D/'ferro/target/sample/corpus-ferro.dSYM'
if dsym.exists():shutil.copytree(dsym,W/'bin/attribution-sample.dSYM',dirs_exist_ok=True)
(D/'source.json').write_text(json.dumps({str(p.relative_to(D/'source')):hashlib.sha256(p.read_bytes()).hexdigest() for p in (D/'source/src').rglob('*.rs')},indent=2)+'\n')
print('Attribution build ready',flush=True)
