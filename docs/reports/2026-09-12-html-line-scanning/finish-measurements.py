from experiment import *
import sys
assert build('production') and verify('production')
for script in ['verify-timing.py','verify-blocks.py','verify-extra.py','verify-extended.py']:
 subprocess.run([sys.executable,str(W/script),'production'],check=True)
for record in ['timing-verification.json','block-verification.json','extra-verification.json','extended-verification.json']:
 assert not json.loads((W/'production'/record).read_text())['failures']
for r in range(3):run('production',9,75,'production/confirm-'+str(r+1),r)
subprocess.run([sys.executable,str(W/'memory.py'),'baseline','production'],check=True)
subprocess.run([sys.executable,str(W/'prepare-corpus.py')],check=True)
subprocess.run([sys.executable,str(W/'corpus/run.py'),'verify'],cwd=W/'corpus',check=True)
for n in range(1,4):subprocess.run([sys.executable,str(W/'corpus/run.py'),'final-'+str(n)],cwd=W/'corpus',check=True)
