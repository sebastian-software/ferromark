import sys
from experiment import *
assert build('baseline') and verify('baseline')
for script in ['verify-timing.py','verify-blocks.py','verify-extra.py','verify-extended.py','verify-fences.py']:
 subprocess.run([sys.executable,str(W/script),'baseline'],check=True)
for name in ['html-blank-run','softbreak-ranges','escape-short-copy','escape-single-scan']:
 if not build(name) or not verify(name):continue
 for script in ['verify-timing.py','verify-blocks.py','verify-extra.py','verify-extended.py','verify-fences.py']:
  subprocess.run([sys.executable,str(W/script),name],check=True)
 run(name,5,30)
