#!/usr/bin/env python3
"""Build native ecosystem adapters from pinned upstream commits, with system malloc."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
PINS = {
    'cmark': ('https://github.com/commonmark/cmark.git', 'eec0eeba6d31189fd828314576494566d539b1e3', '0.31.2'),
    'cmark-gfm': ('https://github.com/github/cmark-gfm.git', '587a12bb54d95ac37241377e6ddc93ea0e45439b', '0.29.0.gfm.13'),
}


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('output', type=Path)
    args = p.parse_args()
    root = args.output.resolve()
    root.mkdir(parents=True, exist_ok=False)
    commands = []
    def run(command, env=None):
        commands.append(command)
        subprocess.run(command, check=True, env=env, cwd=REPO)
    flags = '-C target-cpu=generic'
    rust_env = dict(os.environ, RUSTFLAGS=flags)
    run(['cargo', 'build', '--release', '--locked', '--manifest-path', str(HERE / 'native/Cargo.toml')], rust_env)
    rust = HERE / 'native/target/release/markdown-ecosystem-worker'
    binaries = {'rust': {'path': str(rust), 'sha256': sha(rust)}}
    for engine, (url, revision, version) in PINS.items():
        source, build = root / (engine + '-source'), root / (engine + '-build')
        run(['git', 'clone', '--quiet', url, str(source)])
        run(['git', '-C', str(source), 'checkout', '--quiet', revision])
        run(['cmake', '-S', str(source), '-B', str(build), '-DCMAKE_BUILD_TYPE=Release',
             '-DCMAKE_POLICY_VERSION_MINIMUM=3.5', '-DCMAKE_C_FLAGS_RELEASE=-O3', '-DCMARK_TESTS=OFF', '-DCMARK_SHARED=OFF', '-DCMARK_STATIC=ON'])
        run(['cmake', '--build', str(build), '--parallel', '2'])
        worker = root / (engine + '-worker')
        command = ['clang', '-std=c11', '-O3', '-Wall', '-Wextra', '-Werror', '-I' + str(source / 'src'),
                   '-I' + str(build / 'src'), str(HERE / 'cmark-worker.c')]
        if engine == 'cmark-gfm':
            command += ['-DGFM', '-I' + str(source / 'extensions'), '-I' + str(build / 'extensions'),
                        str(build / 'extensions/libcmark-gfm-extensions.a'), str(build / 'src/libcmark-gfm.a')]
        else:
            command += [str(build / 'src/libcmark.a')]
        command += ['-o', str(worker)]
        run(command)
        binaries[engine] = {'path': str(worker), 'sha256': sha(worker), 'revision': revision, 'version': version}
    dispatcher = root / 'worker'
    dispatcher.write_text('#!/usr/bin/env python3\nimport os, sys\n' +
                          'binaries = ' + repr({k: v['path'] for k, v in binaries.items()}) + '\n' +
                          "binary = binaries.get(sys.argv[1], binaries['rust'])\n" +
                          'os.execv(binary, [binary, *sys.argv[1:]])\n')
    dispatcher.chmod(0o755)
    metadata = {'binaries': binaries, 'sources': PINS, 'commands': commands,
                'rustflags': flags, 'clang': subprocess.check_output(['clang', '--version'], text=True),
                'rust': subprocess.check_output(['rustc', '-vV'], text=True),
                'adapter_sha256': {name: sha(HERE / name) for name in ('worker.rs', 'cmark-worker.c', 'prepare-native.py')},
                'dispatcher_sha256': sha(dispatcher), 'allocator': 'system malloc; no PGO'}
    (root / 'build.json').write_text(json.dumps(metadata, indent=2) + '\n')
    print(dispatcher)


if __name__ == '__main__':
    main()
