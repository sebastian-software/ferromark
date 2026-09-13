from experiment import W,run
import json,shutil
shutil.copy2(W/'bin/baseline', W/'bin/baseline-copy')
indices=json.loads((W/'followup-selected.json').read_text())
run('baseline-copy',rounds=7,ms=60,warm=40,tag='aa-control',indices=indices)
for iteration in range(3):
    for index,name in enumerate(['unstable-packed','stable-packed','stable-tuple']):
        run(name,rounds=7,ms=60,warm=40,tag=name+'/confirm-'+str(iteration+1),order=iteration+index,indices=indices)
