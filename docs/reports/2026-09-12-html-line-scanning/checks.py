from pathlib import Path
import subprocess
W=Path(__file__).parent
R=Path.cwd()  # Run from the repository root.
commands={
 'cargo-test':['cargo','test','--locked','--all-features'],
 'cargo-clippy':['cargo','clippy','--all-targets','--all-features','--locked','--','-D','warnings'],
 'byte-search-test':['cargo','test','-p','ferro-byte-search','--locked'],
 'byte-search-clippy':['cargo','clippy','-p','ferro-byte-search','--all-targets','--locked','--','-D','warnings'],
 'fmt':['cargo','fmt','--check'],
 'contracts':['node','--test','./scripts/test-benchmark-ci-contract.mjs','./scripts/test-profiling-scripts.mjs'],
}
(W/'validation').mkdir(exist_ok=True)
for name,cmd in commands.items():
 with (W/'validation'/f'{name}.log').open('w') as out:
  result=subprocess.run(cmd,cwd=R,stdout=out,stderr=subprocess.STDOUT)
 assert result.returncode==0,(name,(W/'validation'/f'{name}.log').read_text()[-3000:])
 print('Passed',name,flush=True)
