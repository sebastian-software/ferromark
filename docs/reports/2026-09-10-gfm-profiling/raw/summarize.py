import json, pathlib, statistics, hashlib, re, subprocess
root=pathlib.Path.cwd(); raw=root/'target/gfm-profile'; out=root/'docs/reports/2026-09-10-gfm-profiling'
out.mkdir(exist_ok=True)
times=json.loads((raw/'timings.json').read_text()); counts=json.loads((raw/'counts.json').read_text()); eq=json.loads((raw/'output-equivalence.json').read_text())
sha=lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
meta={'baseline':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'date':'2026-09-10','hardware':'Apple M1 Pro','os':'macOS 26.6.2 (25G83)','rustc':'1.97.1 (8bab26f4f 2026-07-14)','llvm':'22.1.6','target':'aarch64-apple-darwin','profile':'release-debug; opt-level=3, fat LTO, one codegen unit; target-cpu=apple-m1, +neon','allocator':'System; allocation counters only in a separate profiling-feature build','policy':'Untrusted','timing':'Five >=75 ms windows; 16 warmup renders per window; clock checked every 16 renders; rotated/reversed preset order; ns per render; uninstrumented build','lane':'buffer: fresh parser state, retained output allocation','allocation_unit':'alloc + realloc calls and cumulatively requested bytes per warmed Renderer render; not peak/live memory','source_hashes':{str(p.relative_to(root)):sha(p) for p in [root/'Cargo.lock',raw/'measurement-probe.rs',root/'examples/profile_harness.rs']},'local_raw_artifacts':'target/gfm-profile (ignored; not a durable repository artifact)'}
fixtures=[]
for name in dict.fromkeys(x['case'] for x in times):
 p=raw/(name+'.md'); s=p.read_text(); unit=None; repeat=1
 if name not in ['commonmark-50k','tables-5k']:
  repeat={'plain':128,'gfm-overlap-tables':80,'autolinks':96,'tasks':96,'mixed-gfm':48}[name]; unit=s[:len(s)//repeat]; assert unit*repeat==s
 fixtures.append({'case':name,'bytes':len(p.read_bytes()),'sha256':sha(p),**({'unit':unit,'repeat':repeat} if unit else {'path':f'benches/fixtures/{name}.md'}),**next({k:v for k,v in r.items() if k!='case'} for r in eq if r['case']==name)})
rows=[]
for case in [f['case'] for f in fixtures]:
 for preset in dict.fromkeys(x['preset'] for x in times if x['round']==0):
  xs=sorted((x for x in times if x['case']==case and x['preset']==preset and x['lane']=='buffer'),key=lambda x:x['round']); assert len(xs)==5
  ns=[round(x['ns'],3) for x in xs]
  rows.append({'case':case,'preset':preset,'ns':ns,'median_ns':round(statistics.median(ns),3),'same_as_commonmark':xs[0]['same_as_commonmark']})
selected_counts=[r for r in counts if r['lane']=='renderer' and r['case'] in ['gfm-overlap-tables','tables-5k','tasks'] and r['preset'] in ['commonmark','tables','strike','gfm']]
profiles=[]
needles={'split_table_cells':'split_table_cells','CellState':'CellState::add_text','parse_with_options_in_document':'InlineParser::parse_with_options_in_document','render_block_event':'RenderContext::render_block_event','render_inline_content':'render_inline_content','url_escape_link_destination_raw':'url_escape_link_destination_raw','has_underscore_in_last_two_segments':'has_underscore_in_last_two_segments'}
for desc in json.loads((raw/'profiles.json').read_text()):
 p=raw/desc['profile']; s=p.read_text(); n=int(re.search(r'Call graph:\n\s+(\d+) Thread_',s)[1]); flat=s.split('Sort by top of stack, same collapsed (when >= 5):')[1].split('Binary Images:')[0]; symbols={}
 for line in flat.splitlines():
  match=re.search(r'\s+(\d+)\s*$',line)
  if match:
   for needle,label in needles.items():
    if needle in line: symbols[label]=int(match[1])
 profiles.append({**desc,'sha256':sha(p),'main_thread_samples':n,'selected_top_of_stack_samples':symbols})
summary={'method':meta,'fixtures':fixtures,'buffer_timings':rows,'warmed_renderer_counts':selected_counts,'cpu_samples':profiles}
# One measured row per line keeps the evidence small and reviewable.
with (out/'summary.json').open('w') as f:
 f.write('{\n  "method": '+json.dumps(meta,ensure_ascii=False)+',\n')
 for i,(key,values) in enumerate(list(summary.items())[1:]):
  f.write('  '+json.dumps(key)+': [\n'+',\n'.join('    '+json.dumps(v,ensure_ascii=False) for v in values)+'\n  ]'+(',' if i<3 else '')+'\n')
 f.write('}\n')
# Validate serialization and all medians before using the report.
assert json.loads((out/'summary.json').read_text())==summary
print('Evidence bytes:',(out/'summary.json').stat().st_size)
