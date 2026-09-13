from experiment import *
import sys
for name in sys.argv[1:]:
 assert verify(name)
 for f in ['timing','blocks','extra','extended','fences']:
  subprocess.run([sys.executable,str(W/('verify-'+f+'.py')),name],check=True)
