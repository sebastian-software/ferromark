# Reproduce the Suspense comparison

Use Python 3.12+, Git, `patch`, and Rust 1.97.1 on an ARM64 Mac for comparable
timing. Other hosts can check correctness but do not reproduce this CPU result.
Cargo's cache must contain the locked dependencies for offline builds.

From the repository root:

```sh
python3 docs/reports/2026-09-13-suspense-inline-code/check-evidence.py
python3 docs/reports/2026-09-13-suspense-inline-code/setup.py /tmp/suspense-replay
cd /tmp/suspense-replay
python3 experiment.py baseline
python3 verify-all.py baseline
python3 verify-precedence.py baseline
```

The proposed source is identified by `candidate_variant` in `metadata.json`. Build
that name and run the same guards before measuring it. For the final source:

```sh
python3 -c 'from experiment import build; assert build("final")'
python3 verify-all.py final
python3 verify-precedence.py final
python3 -c 'from experiment import run; [run("final", rounds=7, ms=50, tag="final/confirm-"+str(i+1), order=i) for i in range(3)]'
python3 - <<'PYTHON'
from experiment import W, run
import json
indices = json.loads((W/'final-followup-selected.json').read_text())
for i in range(3):
    run('final', rounds=7, ms=100, tag='final/followup-'+str(i+1), indices=indices, order=i)
PYTHON
python3 scaling.py final scaling-1
python3 scaling.py final scaling-2
```

The archived `production` name is an intermediate formatted candidate, not the
proposed source. Its native observations are under `native/early-placement/`. The intermediate
`linear-formatted` version is preserved with native observations under
`native/linear-only/`. Other names in `variants.py`
reconstruct the independent probes; their patches and full source hashes are
also retained. The first autolink-code screen used the 93 original controls;
the remaining screens add ten targeted inputs. The `baseline-copy` controls
use a byte-identical copy of `bin/baseline` in separate processes.
The longer exploratory pairs
use `confirmation-selected.json` (75 cases). Final pairs use `selected.json`
(103 cases). No source or input files should change between paired runs.

The exact-output checks include renderer reuse, public block/inline/MDX events,
resource-limit results, all 649 corpus cases and 20,000 additional frozen
HTML/code/link precedence combinations. They preserve existing behavior; they
do not declare every baseline behavior correct according to a specification.
The failing whole-render work test patch and its original output are under
`validation/`; apply that patch to the baseline to reproduce the quadratic work
count without relying on wall-clock timing. The current source contains the
passing version of that test.

For the native Ox comparison, supply a clean checkout at the pinned revision:

```sh
python3 docs/reports/2026-09-13-suspense-inline-code/setup.py /tmp/suspense-native-replay --ox-source /path/to/ox-content
```

First build the chosen Ferromark variant through `experiment.build` in that
workspace, which materializes it in `source/`. The `native/source` symlink uses
that source. Build both `native/ferro/Cargo.toml` and `native/ox/Cargo.toml` with
`cargo build --release --offline --locked`, with ambient `RUSTFLAGS` and
`CARGO_ENCODED_RUSTFLAGS` unset. Copy their `corpus-ferro` and `corpus-ox` binaries
from `target/release` to `native/bin/ferro-release` and `native/bin/ox-release`.
Then run `python3 native/run.py verify` and `python3 native/run.py final-1`
(also `final-2` and `final-3`). Loading, normalization and IPC remain outside the
timed render batches. The primary comparison uses the growing Ox arena.

The broader distribution uses all 638 individual documents and omits the
concatenations and synthetic controls. Reproduce it after the focused runs:

```sh
python3 - <<'PYTHON'
from experiment import W, run
import json
indices = json.loads((W/'corpus-selected.json').read_text())
for i in range(3):
    run('final', rounds=5, ms=20, warm=20, tag='final/corpus-'+str(i+1), indices=indices, order=i)
indices = json.loads((W/'corpus-followup-selected.json').read_text())
for i in range(3):
    run('final', rounds=7, ms=100, tag='final/corpus-followup-'+str(i+1), indices=indices, order=i)
PYTHON
```

The frozen follow-up selection contains every document over 1% slower in all
three recorded broad pairs. A new investigation should also inspect any new
losses in its own repetitions. `CORPUS.md` and `corpus-summary.json` are generated
by `corpus-results.py` from the archived observations.

Run timings serially with no builds, correctness suites or profiler running.
The archive checker validates file hashes, reconstructs every measured source
patch, checks the recorded final guards and recalculates all timing medians and
generated tables. It does not guarantee identical binary hashes or timing on a
different toolchain, build path or machine.
