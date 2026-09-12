from experiment import *
import sys
for name in sys.argv[1:]:
 assert build(name) and verify(name)
 subprocess.run(['python3',str(W/'verify-blocks.py'),name],check=True)
 run(name,5,30)
