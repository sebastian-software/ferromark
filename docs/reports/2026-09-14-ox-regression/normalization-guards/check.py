"""Replay the expected semantic failure of the diagnostic normalization bypass."""
import argparse
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
spec = importlib.util.spec_from_file_location('diagnostic', ROOT / 'benchmarks/ox-regression/run.py')
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('binary', type=Path)
    p.add_argument('output', type=Path)
    args = p.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    rows = []
    for name, source in [('nul', 'a\0b\n'), ('bom', '\ufeffplain\n')]:
        path = args.output / (name + '.md')
        path.write_text(source)
        outputs = {}
        for label, env in [('control', {}), ('skip', {'FERROMARK_DIAGNOSTIC_SKIP_NORMALIZATION': '1'})]:
            worker = runner.Worker(args.binary, 'v2', 'commonmark', 'parse', [path], env)
            try:
                outputs[label] = worker.details()[0]
            finally:
                worker.close()
        assert outputs['control']['html'] != outputs['skip']['html']
        assert outputs['control']['ast_debug'] != outputs['skip']['ast_debug']
        rows.append(dict(name=name, source=source, outputs=outputs, expected_semantic_failure=True))
    (args.output / 'guards.json').write_text(json.dumps(rows, indent=2))
    print('Both NUL/BOM fixtures reject the bypass as expected.')


if __name__ == '__main__':
    main()
