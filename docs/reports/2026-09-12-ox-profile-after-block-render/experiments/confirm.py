from experiment import *
selected=json.loads((W/'confirmation-selected.json').read_text())
for r in range(1,3):
 names=['html-blank-run','escape-short-copy','escape-single-scan','softbreak-guarded']
 if r==2:names.reverse()
 for name in names:run(name,7,50,name+'/confirm-'+str(r),r,indices=selected)
