#!/usr/bin/env python3
"""Build the frozen baseline and retained source, verify, then time three pairs."""
from pathlib import Path
import shutil,subprocess
import experiment
W=Path(__file__).parent
for name,source in [('baseline',W/'baseline/src'),('candidate',W/'retained/src')]:
 shutil.copytree(source,W/'source/src',dirs_exist_ok=True)
 for path in (W/'source/src').rglob('*.rs'):path.touch()
 with (W/(name+'-build.log')).open('w') as out:subprocess.run(['cargo','build','--release','--locked','--manifest-path','driver/Cargo.toml'],cwd=W,stdout=out,stderr=subprocess.STDOUT,check=True)
 (W/'bin').mkdir(exist_ok=True);shutil.copy2(W/'driver/target/release/ox-experiment-driver',W/'bin'/name)
 if name=='baseline':experiment.baseline()
for i in range(3):assert experiment.run('candidate',rounds=9,ms=75,tag=f'run-{i+1}',order=i)
