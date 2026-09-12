"""Sample the same native executables after timing has finished."""
from pathlib import Path
import sys,subprocess,json,shutil
W=Path(sys.argv[1]);D=W/'native-profile';D.mkdir(exist_ok=True)
with (D/'setup.log').open('w') as log:
 r=subprocess.run(['sudo','apt-get','update','-qq'],stdout=log,stderr=subprocess.STDOUT)
 if r.returncode==0:r=subprocess.run(['sudo','apt-get','install','-y','linux-tools-generic'],stdout=log,stderr=subprocess.STDOUT)
 subprocess.run(['sudo','sysctl','kernel.perf_event_paranoid=-1'],stdout=log,stderr=subprocess.STDOUT)
perf=next(iter(sorted(Path('/usr/lib/linux-tools').glob('*/perf'),reverse=True)),None)
status={'perf':str(perf) if perf else None,'runs':[]}
cases=json.loads((W/'cases.json').read_text())
for name in ['baseline','masked-direct','probe-attr-outlined']:
 for command,label in [(['nm','-SC','--size-sort',str(W/'bin'/name)],'symbols'),(['objdump','-dC',str(W/'bin'/name)],'assembly')]:
  with (D/(name+'-'+label+'.txt')).open('w') as out:subprocess.run(command,stdout=out,stderr=subprocess.STDOUT)
 if perf:
  for label,file,index in [('attribute-plain','attributes.json',8),('multiline','cases.json',6),('html-short','cases.json',next(i for i,c in enumerate(cases) if c['case']=='html/root-short-lines'))]:
   stem=name+'-'+label;data=D/(stem+'.data')
   with (D/(stem+'-record.log')).open('w') as log:
    result=subprocess.run([str(perf),'record','-e','cpu-clock:u','-F','997','--no-buildid','-o',str(data),'--',str(W/'bin'/name),str(W/file)],input=json.dumps(dict(op='window',index=index,ms=5000))+'\n',text=True,stdout=log,stderr=subprocess.STDOUT)
   status['runs'].append(dict(candidate=name,case=label,returncode=result.returncode))
   if result.returncode==0:
    with (D/(stem+'-report.txt')).open('w') as out:subprocess.run([str(perf),'report','--stdio','--no-children','--percent-limit','0.5','--sort','symbol','-i',str(data)],stdout=out,stderr=subprocess.STDOUT)
    with (D/(stem+'-annotate.txt')).open('w') as out:subprocess.run([str(perf),'annotate','--stdio','--percent-limit','0.5','-i',str(data)],stdout=out,stderr=subprocess.STDOUT)
(D/'status.json').write_text(json.dumps(status,indent=2)+'\n')
print('PROFILE',status,flush=True)
