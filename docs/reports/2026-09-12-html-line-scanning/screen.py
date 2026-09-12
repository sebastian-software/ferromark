from experiment import *
import sys
for name in sys.argv[1:]:
 assert build(name) and verify(name)
 subprocess.run([sys.executable,str(W/'verify-blocks.py'),name],check=True)
 assert not json.loads((W/name/'block-verification.json').read_text())['failures']
 run(name)
