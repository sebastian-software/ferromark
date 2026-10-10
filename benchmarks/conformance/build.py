#!/usr/bin/env python3
"""Build untimed spec adapters with the comparison campaign's existing pins."""
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tomllib

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
NATIVE = REPO / 'benchmarks/native-comparison'
ECO = REPO / 'benchmarks/markdown-ecosystem'


def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


pins = module('conformance_native_pins', NATIVE / 'prepare.py')
eco_pins = module('conformance_ecosystem_pins', ECO / 'prepare-native.py')


def registry(path):
    return {(p['name'], p['version'], p['source'], p.get('checksum'))
            for p in tomllib.loads(path.read_text())['package'] if 'source' in p}


def prepare(root, inputs):
    root.mkdir(parents=True, exist_ok=False)
    commands = []
    env = dict(os.environ)
    for key in list(env):
        if key in ('RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'CARGO_TARGET_DIR', 'CARGO_BUILD_TARGET') or key.startswith('CARGO_PROFILE_'):
            env.pop(key)
    env['CARGO_BUILD_JOBS'] = '4'
    env['RUSTFLAGS'] = '-C target-cpu=generic'
    def run(command, cwd=REPO, override=None):
        command = list(map(str, command))
        commands.append(command)
        subprocess.run(command, cwd=cwd, env=override or env, check=True)
    cache = root / 'cache'
    run([sys.executable, NATIVE / 'restore.py', cache])
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=REPO, text=True).strip()
    options = ['--bun-source', cache / 'bun', '--bun-native-cache', cache / 'native',
               '--md4c-source', cache / 'md4c', '--ox-archive', cache / 'ox.tar.gz',
               '--ferromark-v1-source', REPO, '--ferromark-v2-source', REPO,
               '--ferromark-v2-revision', revision, '--worker', HERE / 'native-worker.rs']
    fetch = root / 'fetch'
    run([sys.executable, NATIVE / 'prepare.py', fetch, *options, '--lockfile', NATIVE / 'Cargo.lock'])
    seed = fetch / 'bun/Cargo.lock'
    original = registry(seed)
    # Only the current local Ferromark package entry may change. Registry pins
    # remain exactly those used by the performance campaign.
    run(['cargo', '+' + pins.BUN_TOOLCHAIN, 'fetch', '--manifest-path', fetch / 'bun/Cargo.toml'])
    if registry(seed) != original:
        raise ValueError('Native registry resolution drift')
    run(['cargo', '+' + pins.BUN_TOOLCHAIN, 'fetch', '--locked', '--manifest-path', fetch / 'bun/Cargo.toml'])
    native = root / 'native'
    run([sys.executable, NATIVE / 'prepare.py', native, *options, '--lockfile', seed, '--compile'])
    native_info = json.loads((native / 'build.json').read_text())
    # prepare.py describes its timing adapter by default. Record the worker
    # actually copied and compiled for this untimed build instead.
    native_info['adapter_sha256']['worker.rs'] = pins.sha(HERE / 'native-worker.rs')
    if pins.sha(native / 'bun/comparison-worker/worker.rs') != native_info['adapter_sha256']['worker.rs']:
        raise ValueError('Native adapter copy differs')
    (native / 'build.json').write_text(json.dumps(native_info, indent=2) + '\n')
    shutil.rmtree(fetch)
    rust = root / 'rust'
    rust.mkdir()
    manifest = (ECO / 'native/Cargo.toml').read_text().replace('path = "../../.."', 'path = ' + json.dumps(str(REPO)))
    manifest = manifest.replace('path = "../worker.rs"', 'path = ' + json.dumps(str(HERE / 'ecosystem-worker.rs')))
    (rust / 'Cargo.toml').write_text(manifest)
    shutil.copyfile(ECO / 'native/Cargo.lock', rust / 'Cargo.lock')
    original = registry(rust / 'Cargo.lock')
    run(['cargo', 'fetch', '--manifest-path', rust / 'Cargo.toml'])
    if registry(rust / 'Cargo.lock') != original:
        raise ValueError('Ecosystem registry resolution drift')
    run(['cargo', 'build', '--release', '--locked', '--manifest-path', rust / 'Cargo.toml'])
    gold = root / 'goldmark'
    gold.mkdir()
    for name in ('go.mod', 'go.sum'):
        shutil.copyfile(ECO / 'goldmark' / name, gold / name)
    shutil.copyfile(HERE / 'goldmark-worker.go', gold / 'main.go')
    go_env = {key: value for key, value in env.items() if key not in ('GOFLAGS', 'GOOS', 'GOARCH', 'GOGC', 'GOMEMLIMIT', 'GOMAXPROCS', 'GOEXPERIMENT') and not key.startswith(('GOAMD', 'GOARM', 'GOPPC', 'GOMIPS', 'GOWASM'))}
    go_env.update(CGO_ENABLED='0', GOTOOLCHAIN='local', GOWORK='off')
    go_version = subprocess.check_output(['go', 'version'], text=True, env=go_env).strip()
    if go_version.split()[2] != 'go1.27.1':
        raise ValueError('Use Go 1.27.1 from the performance campaign')
    run(['go', '-C', gold, 'build', '-mod=readonly', '-trimpath', '-o', root / 'goldmark-worker', '.'], override=go_env)
    binaries = {'native': Path(native_info['binary']), 'rust': rust / 'target/release/markdown-ecosystem-worker', 'goldmark': root / 'goldmark-worker'}
    for engine, (url, commit, _) in eco_pins.PINS.items():
        source, build = root / (engine + '-source'), root / (engine + '-build')
        run(['git', 'clone', '--quiet', url, source])
        run(['git', '-C', source, 'checkout', '--quiet', commit])
        run(['cmake', '-S', source, '-B', build, '-DCMAKE_BUILD_TYPE=Release', '-DCMAKE_POLICY_VERSION_MINIMUM=3.5', '-DCMARK_TESTS=OFF', '-DCMARK_SHARED=OFF', '-DCMARK_STATIC=ON'])
        run(['cmake', '--build', build, '--parallel', '2'])
        binary = root / (engine + '-worker')
        command = ['clang', '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror', '-I' + str(source / 'src'), '-I' + str(build / 'src'), HERE / 'cmark-worker.c']
        if engine == 'cmark-gfm':
            command += ['-DGFM', '-I' + str(source / 'extensions'), '-I' + str(build / 'extensions'), build / 'extensions/libcmark-gfm-extensions.a', build / 'src/libcmark-gfm.a']
        else:
            command += [build / 'src/libcmark.a']
        run([*command, '-o', binary])
        binaries[engine] = binary
    run(['npm', 'ci', '--ignore-scripts', '--prefix', ECO])
    # Darwin ARM crypto dependencies require the target's default CPU features.
    # The addon uses its ordinary build recipe; generic flags apply only to
    # the external native/Rust comparison workers. No timing is performed.
    addon_env = dict(env)
    addon_env.pop('RUSTFLAGS', None)
    run(['cargo', 'build', '-p', 'ferromark-node', '--profile', 'release-node', '--locked'], override=addon_env)
    target = subprocess.check_output(['node', '--input-type=module', '-e', 'import {benchmarkTarget} from "./benchmarks/markdown-ecosystem/benchmark-target.mjs"; console.log(benchmarkTarget(process.platform,process.arch));'], cwd=REPO, text=True).strip()
    lib = 'libferromark_node.dylib' if sys.platform == 'darwin' else ('ferromark_node.dll' if sys.platform == 'win32' else 'libferromark_node.so')
    addon = REPO / 'node/ferromark' / ('ferromark.' + target + '.node')
    shutil.copyfile(REPO / 'target/release-node' / lib, addon)
    metadata = {'ferromark_version': tomllib.loads((REPO / 'Cargo.toml').read_text())['package']['version'], 'inputs': inputs, 'revision': revision, 'binaries': {key: {'path': str(path), 'sha256': pins.sha(path)} for key, path in binaries.items()},
                'addon': {'path': str(addon), 'sha256': pins.sha(addon), 'rustflags': 'target defaults; no PGO'}, 'native': native_info,
                'ecosystem_registry': sorted(registry(rust / 'Cargo.lock')), 'go': go_version,
                'commands': commands, 'node': subprocess.check_output(['node', '--version'], text=True).strip()}
    (root / 'build.json').write_text(json.dumps(metadata, indent=2) + '\n')
    return metadata
