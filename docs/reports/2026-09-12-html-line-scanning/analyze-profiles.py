from pathlib import Path
import json,gzip,re,subprocess,collections
W=Path(__file__).parent
DEMANGLER=W/'demangle/target/release/sample-demangle'
def parse(path):
 s=gzip.decompress(path.read_bytes()).decode();part=s.split('Call graph:\n',1)[1].split('\nTotal number in stack',1)[0]
 rows=[]
 for line in part.splitlines():
  m=re.match(r'^([ +!:|]*)(\d+) (.*)$',line)
  if m:
   rest=m[3];sym=rest.split('  (in ',1)[0];loc=re.search(r'\]\s+([^ ]+:\d+)\s*$',rest)
   rows.append(dict(depth=len(m[1]),count=int(m[2]),raw=sym,line=loc[1] if loc else ''))
 names=subprocess.check_output([str(DEMANGLER)],input='\n'.join(r['raw'] for r in rows)+'\n',text=True).splitlines()
 stack=[];nodes=[]
 for row,name in zip(rows,names):
  while stack and nodes[stack[-1]]['depth']>=row['depth']:stack.pop()
  row['name']=name;row['self']=row['count'];row['ancestors']=stack.copy()
  if stack:nodes[stack[-1]]['self']-=row['count']
  nodes.append(row);stack.append(len(nodes)-1)
 assert all(n['self']>=0 for n in nodes),[(n['name'],n['self']) for n in nodes if n['self']<0]
 total=sum(n['self'] for n in nodes);self_counts=collections.Counter();lines=collections.Counter();inclusive=collections.Counter();stage=collections.Counter();alloc=0;heading=0;sorting=0
 for i,n in enumerate(nodes):
  self_counts[n['name']]+=n['self'];lines[(n['name'],n['line'])]+=n['self']
  chain=[nodes[j]['name'] for j in n['ancestors']]+[n['name']]
  for name in set(chain):inclusive[name]+=n['self']
  joined=';'.join(chain)
  if re.search(r'heading|slug',joined,re.I):heading+=n['self']
  if any('sort' in name and 'EmitPoint' in name for name in chain):sorting+=n['self']
  if 'ferromark::block::' in joined:label='visible block frames'
  elif 'ferromark::inline::' in joined:label='visible inline frames'
  elif 'ox_content_parser::' in joined:label='visible parser frames (Ox)'
  elif 'ox_content_renderer::' in joined:label='visible renderer frames (Ox)'
  elif 'ferromark::' in joined:label='remaining Ferromark pipeline/render'
  else:label='harness/runtime'
  stage[label]+=n['self']
  if any(re.search(r'(malloc|realloc|(?:^|_)free(?:$|_)|madvise|mmap|munmap|raw_vec.*grow|alloc::alloc::)',name) for name in chain):alloc+=n['self']
 def ranked(c):return [dict(symbol=k,samples=v,pct=round(v/total*100,2)) for k,v in c.most_common(25) if v]
 return dict(file=path.name,total_samples=total,stages={k:dict(samples=v,pct=round(v/total*100,2)) for k,v in stage.items()},allocation_stack_samples=alloc,allocation_stack_pct=round(alloc/total*100,2),heading_stack_pct=round(heading/total*100,2),emit_sort_stack_pct=round(sorting/total*100,2),self=ranked(self_counts),inclusive=ranked(inclusive),lines=[dict(symbol=k[0],line=k[1],samples=v,pct=round(v/total*100,2)) for k,v in lines.most_common(30)])
rows=[parse(p) for p in sorted((W/'profiles').glob('*.txt.gz'))]
(W/'profile-summary.json').write_text(json.dumps(rows,indent=2)+'\n')
for r in rows:
 print(r['file'],r['total_samples'],'samples',r['stages'],'allocator',r['allocation_stack_pct'])
 for x in r['self'][:6]:print(' ',x['pct'],x['symbol'])
