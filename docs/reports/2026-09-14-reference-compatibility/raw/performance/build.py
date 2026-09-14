"""Reuse the verified first-correction binary and freeze the second correction."""
from pathlib import Path
import hashlib, importlib.util, json, subprocess
ROOT=Path('/Users/sebastian/Workspace/ferromark-v2')
BUILD=Path('/private/tmp/ferromark-v2-correctness-round2/performance/build')
PREVIOUS=Path('/private/tmp/ferromark-v2-correctness-fixes/performance/build-01')
spec=importlib.util.spec_from_file_location('prepare',ROOT/'benchmarks/optimization-rounds/prepare.py')
helper=importlib.util.module_from_spec(spec);spec.loader.exec_module(helper)
helper.BASELINE_REVISION='4a1e55f190e16fad60314a1c91183123710d8bba'
old=json.loads((PREVIOUS/'build.json').read_text())
baseline=old['engines']['candidate']
verification=helper.verify_baseline(Path(baseline['frozen_source']))
assert helper.sha256(Path(baseline['binary']))==baseline['binary_sha256']
assert helper.tree_sha(Path(baseline['frozen_source']))==baseline['frozen_source_sha256']
worker=helper.WORKER.read_bytes(); worker_sha=hashlib.sha256(worker).hexdigest()
assert worker_sha==old['worker_sha256']
assert old['rustc']==subprocess.check_output(['rustc','+1.95','-vV'],text=True)
BUILD.mkdir(parents=True,exist_ok=False);(BUILD/'worker.rs').write_bytes(worker)
candidate=helper.build_engine('candidate',ROOT,BUILD,worker,worker_sha,'fat',None)
metadata=dict(schema=2,rustc=old['rustc'],baseline_revision_expected=helper.BASELINE_REVISION,
 baseline_verification=verification,worker_sha256=worker_sha,worker_source=str(helper.WORKER),lto='fat',
 frozen_lock_sha256=helper.sha256(ROOT/'Cargo.lock'),
 baseline_reuse_note='Prior candidate binary verified against first-correction commit 4a1e55f; same production core as starting c4bb223.',
 engines=dict(baseline=baseline|{'cache':'reused'},candidate=candidate))
(BUILD/'build.json').write_text(json.dumps(metadata,indent=2)+'\n')
print(BUILD/'build.json')
