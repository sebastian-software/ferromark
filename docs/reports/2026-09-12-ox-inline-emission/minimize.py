from experiment import W,Worker
import json
rows=json.loads((W/'code-two/extra-verification.json').read_text())['failures'];out=[]
def observe(source,flags):
 case=W/'minimize-case.json';case.write_text(json.dumps([dict(case='minimized',input=source,flags=flags,reuse=False)]))
 result={}
 for name in ['baseline','code-two']:
  p=Worker(W/'bin'/name,case);result[name]=p.call('events',0);p.close()
 return result
for row in [rows[0],rows[1]]:
 s=row['case']['input'];flags=row['case']['flags'];changed=True
 for size in [16,8,4,2,1]:
  changed=True
  while changed:
   changed=False
   for i in range(len(s)):
    candidate=s[:i]+s[i+size:];r=observe(candidate,flags)
    if r['baseline']!=r['code-two']:s=candidate;changed=True;break
 r=observe(s,flags);out.append(dict(input=s,flags=flags,**r));print(json.dumps(out[-1],ensure_ascii=False),flush=True)
(W/'code-two/minimized.json').write_text(json.dumps(out,indent=2)+'\n')
