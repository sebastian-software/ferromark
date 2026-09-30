"""Local execution smoke only: shortened samples cannot be published."""
import importlib.util
import json
from pathlib import Path
import subprocess
import sys

root = Path(sys.argv[1]).resolve()
source = root / 'source'
spec = importlib.util.spec_from_file_location('suite', source / 'benchmarks/manual-comparison/cli.py')
suite = importlib.util.module_from_spec(spec)
spec.loader.exec_module(suite)
out = root / 'diagnostic'
out.mkdir()
corpus = source / 'docs/reports/2026-09-14-optimization-rounds/broad-corpus.json.gz'
windows = ['--rounds', '1', '--samples', '1', '--window-ms', '10', '--warmup-ms', '10']
commands = [(['native-shared'], [sys.executable, source / 'benchmarks/native-comparison/run.py', root / 'native-build/build.json', corpus, out / 'native-shared', *windows])]
for track, engine in suite.PAIRS:
    name = track + '-' + engine
    command = [sys.executable, source / 'benchmarks/markdown-ecosystem/run.py', track, out / name, '--competitor', engine, *windows]
    if track == 'native':
        command += ['--binary', root / 'ecosystem-build/worker', '--build-metadata', root / 'ecosystem-build/build.json']
    commands.append(([name], command))
records = []
for (name,), command in commands:
    print('Diagnostic timing: ' + name, flush=True)
    with (out / (name + '.log')).open('w') as stream:
        subprocess.run(list(map(str, command)), cwd=source, stdout=stream, stderr=subprocess.STDOUT, check=True)
    lane = out / name
    config = json.loads((lane / 'run.json').read_text())
    outputs = json.loads((lane / 'verification.json').read_text())
    rows = json.loads((lane / 'samples.json').read_text())
    assert (config['rounds'], config['samples'], config['window_ms'], config['warmup_ms']) == (1, 1, 10, 10)
    assert len(outputs) == 57
    if name == 'native-shared':
        suite.native_archive.audit_windows(config, rows, outputs)
        assert len(rows) == 118
    else:
        assert len(rows) == 114
        for row in rows:
            for engine in config['engines']:
                timing = row[engine]
                count = suite.ecosystem.output_units(outputs[row['case']]['outputs'][engine], config['track'])
                assert timing['iterations'] > 0 and timing['elapsed_ns'] >= 10_000_000
                assert timing['checksum'] == timing['iterations'] * count
        # Prove a short run is rejected by the real publication validator.
        try:
            suite.eco_archive.validate(lane, config['git_head'])
        except AssertionError:
            pass
        else:
            raise AssertionError('short diagnostic unexpectedly accepted for publication')
    records.append({'lane': name, 'engines': config['engines'], 'verified_documents': len(outputs), 'timing_rows': len(rows), 'command': list(map(str, command)), 'status': 'passed'})
(out / 'readiness.json').write_text(json.dumps({'publishable': False, 'revision': suite.git('rev-parse', 'HEAD', cwd=source), 'lanes': records}, indent=2) + '\n')
print('All diagnostic windows and checksums passed; short runs are not publishable.', flush=True)
