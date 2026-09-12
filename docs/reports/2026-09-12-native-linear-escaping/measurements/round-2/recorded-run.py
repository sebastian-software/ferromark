"""Native Linux escape integration screen; temporary experiment branch only."""
from pathlib import Path
import json,subprocess,os,sys,hashlib,gzip,shutil,statistics
R=Path(__file__).resolve().parents[1];D=R/'docs/reports/2026-09-12-linear-html-escaping';W=Path(os.environ['RUNNER_TEMP'])/'escape-linux'
subprocess.run([sys.executable,str(D/'setup.py'),str(W)],check=True)
# Preserve the ordinary driver operations and add a separate direct API control.
p=W/'driver/src/main.rs';s=p.read_text();needle='   _=>panic!("unknown op"),'
extra='''   "escape"=>{let mut escaped=Vec::with_capacity(c.input.len());if c.flags&1!=0 {ferro::escape_attr_into(&mut escaped,c.input.as_bytes());}else{ferro::escape_text_into(&mut escaped,c.input.as_bytes());}json!({"output":escaped})},
   "window-escape"|"window-escape-single"=>{let ms=q["ms"].as_u64().unwrap();let batch=if q["op"]=="window-escape-single"{1}else{16};let start=Instant::now();let mut n=0;loop{for _ in 0..batch{let mut escaped=Vec::with_capacity(c.input.len()*if c.flags&2!=0{2}else{1});if c.flags&1!=0{ferro::escape_attr_into(&mut escaped,black_box(c.input.as_bytes()));}else{ferro::escape_text_into(&mut escaped,black_box(c.input.as_bytes()));}drop(black_box(escaped));}n+=batch;if start.elapsed()>=Duration::from_millis(ms){break;}}json!({"case":c.case,"count":n,"elapsed_ns":start.elapsed().as_nanos() as u64})},
'''
assert needle in s;p.write_text(s.replace(needle,extra+needle).replace('c.input.len()*if c.flags&2!=0{2}else{1}', 'c.input.len()*if c.flags&2!=0{2}else{1}+if c.flags&4!=0{16}else{0}'))
# Each variant starts from the measured complete implementation. ARM keeps that
# implementation verbatim; only the non-NEON continuation changes.
s=(W/'linear-source/src/escape.rs').read_text();begin=s.index('    let prefix_len =',s.index('fn first_escape<const'));end=s.index('\n}',begin);arm=s[begin:end]
common='''        let common = memchr3(b'<', b'>', b'&', input);
        #[cfg(test)] tests::record_long_search(input.len(), common);
        let limit = common.unwrap_or(input.len());
        if ATTR { memchr2(b'"', b'\\\'', &input[..limit]).or(common) } else { memchr(b'"', &input[..limit]).or(common) }
'''
bodies={
 'threshold-1024':'''        if input.len() <= SHORT_SCAN_MAX { return first_escape_in_set::<ATTR>(input); }
        if input.len() > 1024 { return first_escape_long::<ATTR>(input); }
'''+common,
 'threshold-8192':'''        if input.len() <= SHORT_SCAN_MAX { return first_escape_in_set::<ATTR>(input); }
        if input.len() > 8192 { return first_escape_long::<ATTR>(input); }
'''+common,
 'no-prefix-256':'''        if input.len() <= SHORT_SCAN_MAX { return first_escape_in_set::<ATTR>(input); }
        first_escape_long::<ATTR>(input)
''',
 'no-prefix-1024':'''        if input.len() <= SHORT_SCAN_MAX { return first_escape_in_set::<ATTR>(input); }
        first_escape_long::<ATTR>(input)
''',
 'prefix-8192':arm,
}
sources={'current':s}
for name,body in bodies.items():
 if name=='prefix-8192':candidate=s
 else:candidate=s[:begin]+'    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]\n    {\n'+arm+'\n    }\n    #[cfg(not(all(target_arch = "aarch64", target_feature = "neon")))]\n    {\n'+body+'    }'+s[end:]
 width=8192 if name=='prefix-8192' else 1024 if name=='no-prefix-1024' else 256
 if width!=256:candidate=candidate.replace('let mut width = SHORT_SCAN_MAX * 2;',f'let mut width = if cfg!(all(target_arch = "aarch64", target_feature = "neon")) {{ SHORT_SCAN_MAX * 2 }} else {{ {width} }};')
 sources[name]=candidate
# Round two retains the best small-input threshold and tests cached cursors for
# whole large writes. The original single probe remains linear in its own input.
base=sources['current']
legacy="""        if input.len() <= SHORT_SCAN_MAX { return first_escape_in_set::<ATTR>(input); }
"""+common
first=base.index('    let prefix_len =',base.index('fn first_escape<const'));last=base.index('\n}',first)
cached=base[:first]+'    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]\n    {\n'+arm+'\n    }\n    #[cfg(not(all(target_arch = "aarch64", target_feature = "neon")))]\n    {\n'+legacy+'    }'+base[last:]
cached=cached.replace('#[inline(never)]\nfn first_escape_long','#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]\n#[inline(never)]\nfn first_escape_long')
for attr in [False,True]:
 fname='escape_full_into' if attr else 'escape_text_into'
 at=cached.index('    let mut start = 0usize;',cached.index('pub fn '+fname+'('))
 cached=cached[:at]+f'    #[cfg(not(all(target_arch = "aarch64", target_feature = "neon")))]\n    if input.len() > 8192 {{ escape_cached::<{str(attr).lower()}>(out, input); return; }}\n'+cached[at:]
helper=r"""
#[cfg(not(all(target_arch = "aarch64", target_feature = "neon")))]
#[inline(never)]
fn escape_cached<const ATTR: bool>(out: &mut Vec<u8>, input: &[u8]) {
    let mut start = 0;
    let mut common = memchr3(b'<', b'>', b'&', input);
    #[cfg(test)] tests::record_long_search(input.len(), common);
    let mut quote = if ATTR { memchr2(b'"', b'\'', input) } else { memchr(b'"', input) };
    #[cfg(test)] tests::record_long_search(input.len(), quote);
    loop {
        let at = common.unwrap_or(input.len()).min(quote.unwrap_or(input.len()));
        out.extend_from_slice(&input[start..at]);
        if at == input.len() { break; }
        if ATTR { push_attr_escape(out, input[at]); } else { push_text_escape(out, input[at]); }
        start = at + 1;
        if common == Some(at) {
            common = memchr3(b'<', b'>', b'&', &input[start..]);
            #[cfg(test)] tests::record_long_search(input.len() - start, common);
            common = common.map(|i| start + i);
        } else {
            quote = if ATTR { memchr2(b'"', b'\'', &input[start..]) } else { memchr(b'"', &input[start..]) };
            #[cfg(test)] tests::record_long_search(input.len() - start, quote);
            quote = quote.map(|i| start + i);
        }
    }
}

"""
at=cached.index('/// URL percent-encode');cached=cached[:at]+helper+cached[at:]
forced=cached
for fname in ['escape_text_into','escape_full_into']:
 forced=forced.replace('#[inline]\npub fn '+fname,'#[cfg_attr(all(target_arch = "aarch64", target_feature = "neon"), inline)]\n#[cfg_attr(not(all(target_arch = "aarch64", target_feature = "neon")), inline(always))]\npub fn '+fname)
sources={'threshold-8192':sources['threshold-8192'],'bulk-cached-8192':cached,'bulk-inline-8192':forced}
for name in sources:sources[name]=sources[name].replace('for repeats in [128, 256]', 'for repeats in [1024, 2048]')

oracle_test='    #[test]\n    fn large_writes_match_scalar_escape_oracle() {\n        let alphabet = b"ab<>&\\"\'\\0\\x80\\xff";\n        for len in [8191, 8192, 8193, 16385] {\n            let input: Vec<u8> = (0..len).map(|i| alphabet[(i * 17 + i / 31) % alphabet.len()]).collect();\n            for attr in [false, true] {\n                let mut expected = Vec::new();\n                for &byte in &input {\n                    match byte {\n                        b\'<\' => expected.extend_from_slice(b"&lt;"),\n                        b\'>\' => expected.extend_from_slice(b"&gt;"),\n                        b\'&\' => expected.extend_from_slice(b"&amp;"),\n                        b\'"\' => expected.extend_from_slice(b"&quot;"),\n                        b\'\\\'\' if attr => expected.extend_from_slice(b"&#39;"),\n                        _ => expected.push(byte),\n                    }\n                }\n                let mut actual = Vec::new();\n                if attr { escape_attr_into(&mut actual, &input); } else { escape_text_into(&mut actual, &input); }\n                assert_eq!(actual, expected, "len={len}, attr={attr}");\n            }\n        }\n    }\n\n'
for name in sources:
 sources[name]=sources[name].replace("    #[test]\n    fn quoted_code_search_work_grows_linearly", oracle_test+"    #[test]\n    fn quoted_code_search_work_grows_linearly")
snap=W/'native-sources';snap.mkdir()
for name,s in sources.items():(snap/f'{name}.rs').write_text(s)
(W/'variants.py').write_text('''from pathlib import Path
W=Path(__file__).parent
def variant(name,files):
 if name=='baseline':return files
 files['src/escape.rs']=(W/'native-sources'/f'{name}.rs').read_text()
 return files
''')
# Direct controls reproduce CI plain/html cases, with additional scale/boundary
# controls. Large quote-only input uses single-render windows.
direct=[]
def add(name,text,flags=0):direct.append(dict(case='direct-escape/'+name,input=text,flags=flags,reuse=False))
add('ci-plain', 'Hello, this is plain text without any special characters. '*100)
add('ci-html-heavy', "<script>alert('xss')</script> & more <tags> here! "*100,2)
for n in [128,512,1024,4096,8192,65536]:
 add(f'plain-{n}','x'*n);add(f'late-quote-{n}','x'*(n-1)+'"')
for n in [4096,65536]:
 add(f'quotes-{n}',('let s = "text";\n'*((n+14)//15))[:n])
 add(f'attr-quotes-{n}',("xx'\""*((n+3)//4))[:n],1)
(W/'direct.json').write_text(json.dumps(direct))
for n in [512,8192,65536]:add(f'late-quote-reserved-{n}','x'*(n-1)+'"',4)
add('html-heavy-65536', ("<script>alert('xss')</script> & more <tags> here! "*2000)[:65536],2)
(W/'direct.json').write_text(json.dumps(direct))
meta=dict(revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=R,text=True).strip(),rustc=subprocess.check_output(['rustc','-Vv'],text=True),cpu=subprocess.check_output(['lscpu'],text=True),driver_sha256=hashlib.sha256((W/'driver/src/main.rs').read_bytes()).hexdigest())
(W/'native-metadata.json').write_text(json.dumps(meta,indent=2))
subprocess.run(['cargo','fetch','--locked','--manifest-path',str(W/'driver/Cargo.toml')],check=True)
sys.path.insert(0,str(W));from experiment import *
assert build('baseline') and verify('baseline')
for name in sources:assert build(name) and verify(name)
shutil.copy2(W/'bin/baseline',W/'bin/aa-control');(W/'aa-control').mkdir(exist_ok=True)
subprocess.run([sys.executable,str(W/'verify-all.py'),'baseline','bulk-cached-8192','bulk-inline-8192'],check=True)
meta['binary_sha256']={name:hashlib.sha256((W/'bin'/name).read_bytes()).hexdigest() for name in ['baseline','aa-control']+list(sources)}
(W/'native-metadata.json').write_text(json.dumps(meta,indent=2))
with (W/'native-escape-tests.log').open('w') as log:
 result=subprocess.run(['cargo','test','--manifest-path',str(W/'source/Cargo.toml'),'--offline','--lib','escape::tests'],stdout=log,stderr=subprocess.STDOUT)
if result.returncode:print((W/'native-escape-tests.log').read_text(),flush=True)
assert result.returncode==0
selected=json.loads((W/'screen-selected.json').read_text())+[10,41,691,697,699,701,705]
def direct_run(name,pair):
 dest=W/name/f'direct-{pair}';dest.mkdir(parents=True,exist_ok=True);workers={n:Worker(W/'bin'/n,W/'direct.json') for n in ['baseline',name]};raw=[];summary=[]
 for i,c in enumerate(direct):
  actual=[w.call('escape',i) for w in workers.values()];assert actual[0]==actual[1],(name,i)
  op='window-escape-single' if len(c['input'])>=65536 and 'quotes-' in c['case'] else 'window-escape'
  vals={n:[] for n in workers}
  for w in workers.values():w.call(op,i,40)
  for r in range(5):
   for n in (list(workers) if (r+pair)%2==0 else list(reversed(workers))):
    x=workers[n].call(op,i,50);x.update(engine=n,round=r,index=i,operation=op);raw.append(x);vals[n].append(x['elapsed_ns']/x['count'])
  a,b=[statistics.median(vals[n]) for n in workers];summary.append(dict(index=i,case=c['case'],baseline_ns=a,candidate_ns=b,change_pct=(b/a-1)*100))
 for w in workers.values():w.close()
 (dest/'windows.jsonl.gz').write_bytes(gzip.compress(('\n'.join(json.dumps(x) for x in raw)+'\n').encode(),mtime=0));(dest/'summary.json').write_text(json.dumps(summary,indent=2))
 print(name,'direct',pair,' '.join(f"{r['case']}={r['change_pct']:+.1f}%" for r in summary),flush=True)
for pair in [0,1]:
 for name in (['aa-control']+list(sources) if pair==0 else list(reversed(sources))+['aa-control']):
  run(name,rounds=5,ms=50,warm=40,tag=f'{name}/screen-{pair+1}',order=pair,indices=selected)
  direct_run(name,pair+1)
# Save the complete reviewable experiment; no target directories or executables.
archive=Path(os.environ['RUNNER_TEMP'])/'escape-linux-evidence';archive.mkdir()
for p in W.rglob('*'):
 if not p.is_file() or any(x in p.parts for x in ['target','bin','__pycache__']):continue
 if p.suffix=='.rs' and 'src' in p.parts and p.relative_to(W).parts[0] not in ['driver']:continue
 dest=archive/p.relative_to(W);dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,dest)
print('EVIDENCE',archive,flush=True)
