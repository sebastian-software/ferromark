"""Reconstruct a recorded native Linux round in a fresh directory."""
from pathlib import Path
import argparse,gzip,json,subprocess,shutil
p=argparse.ArgumentParser();p.add_argument('round');p.add_argument('destination',type=Path);a=p.parse_args()
D=Path(__file__).resolve().parent;source=D/'measurements'/a.round;W=a.destination.resolve()
assert source.is_dir(),a.round
assert not W.exists(),'Use a fresh destination'
subprocess.run(['python3',str(D.parent/'2026-09-12-linear-html-escaping/setup.py'),str(W)],check=True)
for name in ['experiment.py','variants.py','regimes.py','selected.json','screen-selected.json','regime-selected.json','attribute-selected.json']:
 shutil.copyfile(source/name,W/name)
shutil.copytree(source/'driver',W/'driver',dirs_exist_ok=True)
(W/'native-sources').mkdir()
for path in (source/'native-sources').glob('*.rs.gz'):(W/'native-sources'/path.stem).write_bytes(gzip.decompress(path.read_bytes()))
(W/'direct.json').write_bytes(gzip.decompress((source/'direct.json.gz').read_bytes()))
# Keep recorded data separate from new observations.
shutil.copyfile(D/'direct.py',W/'direct.py')
subprocess.run(['cargo','fetch','--locked','--manifest-path',str(W/'driver/Cargo.toml')],check=True)
print(W)
