#!/usr/bin/env python3
"""Prepare, measure, validate, and publish the complete manual comparison."""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import math
import os
from pathlib import Path
import platform
import re
import shlex
import shutil
import subprocess
import sys
import tempfile
import time


REPO = Path(__file__).resolve().parents[2]
ECO = REPO / 'benchmarks/markdown-ecosystem'
NATIVE = REPO / 'benchmarks/native-comparison'
REFERENCE = REPO / 'docs/reports/2026-09-21-native-round-4'
CORPUS = REPO / 'docs/reports/2026-09-14-optimization-rounds/broad-corpus.json.gz'
CURRENT = REPO / 'benchmarks/manual-comparison/current.json'

# The established harnesses own semantics, timing, and aggregation.
sys.path.insert(0, str(ECO))
import run as ecosystem  # noqa: E402


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


runner_context = load('runner_context', REPO / 'benchmarks/manual-comparison/runner_context.py')
eco_archive = load('ecosystem_archive', ECO / 'archive-results.py')
native_archive = load('native_archive', NATIVE / 'archive.py')
native = ecosystem.native
PAIRS = (('native', 'markdown-rs'), ('native', 'comrak'), ('native', 'cmark'), ('native', 'cmark-gfm'),
         ('node', 'micromark'), ('node', 'marked'), ('node', 'markdown-it'), ('node', 'remark'),
         ('node', 'showdown'), ('node', 'commonmark'))
LEGACY = ('pulldown-cmark', 'md4c', 'bun', 'ox-content')


def read(folder, name):
    return native_archive.read(folder, name + '.json')


def write(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')


def git(*args, cwd=REPO):
    return subprocess.check_output(['git', *args], cwd=cwd, text=True).strip()


def execute(args, cwd, log, env=None):
    print('$ ' + shlex.join(map(str, args)), flush=True)
    with log.open('a') as stream:
        stream.write('$ ' + shlex.join(map(str, args)) + '\n')
        stream.flush()
        subprocess.run(list(map(str, args)), cwd=cwd, env=env, stdout=stream,
                       stderr=subprocess.STDOUT, check=True)


def platform_details(system, machine):
    """Canonical platform keys, independent of Python/Node architecture spelling."""
    architectures = {'arm64': 'arm64', 'aarch64': 'arm64', 'x86_64': 'x86-64', 'amd64': 'x86-64', 'x64': 'x86-64', 'x86-64': 'x86-64'}
    systems = {'Darwin': ('macos', 'macOS'), 'Linux': ('linux', 'Linux')}
    architecture = architectures.get(machine.lower())
    if system not in systems or architecture is None:
        raise ValueError(f'The native harness supports macOS/Linux on arm64 and x86-64, found {system}/{machine}.')
    slug, label = systems[system]
    return {'system': system, 'architecture': architecture, 'platform': f'{slug}-{architecture}',
            'platform_label': f'{label} {architecture}'}


def addon_library(system):
    return 'libferromark_node.dylib' if system == 'Darwin' else 'libferromark_node.so'


def host_description(details):
    if details['system'] == 'Darwin':
        cpu = subprocess.check_output(['sysctl', '-n', 'machdep.cpu.brand_string'], text=True).strip()
        memory = int(subprocess.check_output(['sysctl', '-n', 'hw.memsize'], text=True)) // (1024 ** 3)
        os_version = subprocess.check_output(['sw_vers', '-productVersion'], text=True).strip()
        return f'{cpu}, {memory} GB RAM, macOS {os_version}'
    cpuinfo = Path('/proc/cpuinfo').read_text()
    cpu = next((line.split(':', 1)[1].strip() for line in cpuinfo.splitlines()
                if line.lower().startswith(('model name', 'hardware'))), details['architecture'])
    memory_kb = next(int(line.split()[1]) for line in Path('/proc/meminfo').read_text().splitlines() if line.startswith('MemTotal:'))
    return f'{cpu}, {memory_kb / (1024 ** 2):.1f} GB RAM, Linux {platform.release()}'


def preflight():
    details = platform_details(platform.system(), platform.machine())
    required = ('git', 'curl', 'clang', 'clang++', 'cmake', 'cargo', 'rustup', 'node', 'npm')
    if details['system'] == 'Darwin':
        required += ('caffeinate',)
        translated = subprocess.run(['sysctl', '-in', 'sysctl.proc_translated'], capture_output=True, text=True)
        if translated.stdout.strip() == '1':
            raise ValueError('Use native executables outside Rosetta so recorded hardware and process architectures agree.')
    missing = [name for name in required if shutil.which(name) is None]
    if missing:
        raise ValueError('Missing tools: ' + ', '.join(missing) + '. See benchmarks/manual-comparison/README.md.')
    if os.environ.get('NODE_OPTIONS'):
        raise ValueError('Unset NODE_OPTIONS so Node uses its default runtime configuration.')
    version = subprocess.check_output(['node', '--version'], text=True).strip()
    if not version.startswith('v24.'):
        raise ValueError('Use Node.js 24 for this comparison, found ' + version)
    probe = "import { localTarget } from './benchmarks/markdown-ecosystem/benchmark-target.mjs'; console.log(JSON.stringify({system: process.platform === 'darwin' ? 'Darwin' : process.platform === 'linux' ? 'Linux' : process.platform, architecture: process.arch, target: localTarget()}));"
    node = json.loads(subprocess.check_output(['node', '--input-type=module', '-e', probe], cwd=REPO, text=True))
    assert platform_details(node['system'], node['architecture']) == details, 'Python and Node process architectures differ'
    if os.environ.get('RUSTUP_TOOLCHAIN'):
        raise ValueError('Unset RUSTUP_TOOLCHAIN so the committed toolchain pin selects the compiler.')
    rustc = subprocess.check_output(['rustc', '-vV'], cwd=REPO, text=True)
    triple = native_archive.prepare.host_triple(rustc)
    assert triple.split('-', 1)[0] in (('aarch64',) if details['architecture'] == 'arm64' else ('x86_64',)), 'Rust host architecture differs'
    if git('status', '--porcelain'):
        raise ValueError('Commit the measured changes first, or use a clean worktree; no files were changed.')
    git('cat-file', '-e', native_archive.prepare.FERROMARK_V1_REVISION + '^{commit}')
    return {**details, 'machine': host_description(details), 'hostname': platform.node(),
            'host_platform': platform.platform(), 'addon_file': 'ferromark.' + node['target'] + '.node'}


def prepare(output, context_path=None):
    host = preflight()
    context = json.loads(context_path.read_text()) if context_path else None
    if context is not None:
        runner_context.assert_host(context)
    if output == REPO or REPO in output.parents:
        raise ValueError('Choose an output directory outside the checkout.')
    output.mkdir(parents=True, exist_ok=False)
    revision = git('rev-parse', 'HEAD')
    source = output / 'source'
    logs = output / 'logs'
    logs.mkdir()
    env = os.environ.copy()
    # Prevent ambient compiler settings from changing the declared profiles.
    for key in list(env):
        if key in ('RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'CARGO_TARGET_DIR', 'CARGO_BUILD_TARGET') or key.startswith('CARGO_PROFILE_'):
            env.pop(key)
    env['CARGO_BUILD_JOBS'] = str(min(os.cpu_count() or 1, 4))
    env['PYTHONDONTWRITEBYTECODE'] = '1'
    log = logs / 'prepare.log'
    execute(['git', 'clone', '--no-hardlinks', '--no-checkout', REPO, source], REPO, log, env)
    execute(['git', 'checkout', '--detach', revision], source, log, env)
    python = sys.executable
    execute(['rustup', 'toolchain', 'install', native_archive.prepare.BUN_TOOLCHAIN, '--profile', 'minimal'], source, log, env)
    for lane in ('native-comparison', 'markdown-ecosystem'):
        execute([python, '-m', 'unittest', 'discover', '-s', 'benchmarks/' + lane, '-p', 'test_*.py'], source, logs / (lane + '-tests.log'), env)
    cache = output / 'cache'
    (cache / 'native').mkdir(parents=True)
    shutil.copyfile(source / REFERENCE.relative_to(REPO) / 'restore.py', cache / 'restore.py')
    execute([python, cache / 'restore.py'], source, log, env)
    options = ['--bun-source', cache / 'bun', '--bun-native-cache', cache / 'native',
               '--md4c-source', cache / 'md4c', '--ox-archive', cache / 'ox.tar.gz',
               '--ferromark-v1-source', source, '--ferromark-v2-source', source,
               '--ferromark-v2-revision', revision, '--worker', source / NATIVE.relative_to(REPO) / 'worker.rs',
               '--bun-lock', source / REFERENCE.relative_to(REPO) / 'Cargo.lock']
    runner = source / NATIVE.relative_to(REPO) / 'prepare.py'
    execute([python, runner, output / 'fetch-workspace', *options], source, log, env)
    execute(['cargo', '+' + native_archive.prepare.BUN_TOOLCHAIN, 'fetch', '--manifest-path', output / 'fetch-workspace/bun/Cargo.toml'], source, log, env)
    shutil.rmtree(output / 'fetch-workspace')
    execute([python, runner, output / 'native-build', *options, '--compile'], source, logs / 'native-build.log', env)
    execute([python, source / NATIVE.relative_to(REPO) / 'audit_sources.py', output / 'native-build', output / 'source-audit.json',
             '--v1', source, '--v2', source, '--bun', cache / 'bun', '--md4c', cache / 'md4c', '--ox-archive', cache / 'ox.tar.gz'], source, log, env)
    execute([python, source / ECO.relative_to(REPO) / 'prepare-native.py', output / 'ecosystem-build'], source, logs / 'ecosystem-build.log', env)
    execute(['npm', 'ci', '--ignore-scripts', '--prefix', 'benchmarks/markdown-ecosystem'], source, log, env)
    execute(['cargo', 'build', '-p', 'ferromark-node', '--profile', 'release-node', '--locked'], source, logs / 'node-build.log', env)
    addon = source / 'node/ferromark' / host['addon_file']
    shutil.copyfile(source / 'target/release-node' / addon_library(host['system']), addon)
    execute(['node', '--test', source / ECO.relative_to(REPO) / 'test-benchmark-loader.mjs'], source, logs / 'node-loader-tests.log', {**env, 'FERROMARK_BENCH_TEST_ADDON': str(addon)})
    write(output / 'prepared.json', {'schema': 2, 'revision': revision, **host, 'node': subprocess.check_output(['node', '--version'], text=True).strip(),
                                   'addon_sha256': native.sha(addon), 'corpus_sha256': native.sha(CORPUS),
                                   **({'runner_context': context} if context else {})})
    (output / 'host.txt').write_text(host['machine'] + '\n')
    print('Prepared. Measure with: ./scripts/benchmark-comparison measure ' + shlex.quote(str(output)), flush=True)


def measure(output, cooldown, verify_only=False):
    host = preflight()
    prepared = read(output, 'prepared')
    context = prepared.get('runner_context')
    if context is not None:
        runner_context.assert_host(context)
    assert prepared['schema'] == 2 and prepared['revision'] == git('rev-parse', 'HEAD'), 'checkout revision changed since preparation'
    assert all(host[key] == prepared[key] for key in host), 'run on the same host used for preparation'
    source = output / 'source'
    assert git('rev-parse', 'HEAD', cwd=source) == prepared['revision'] and not git('status', '--porcelain', cwd=source)
    assert native.sha(source / 'node/ferromark' / prepared['addon_file']) == prepared['addon_sha256']
    assert subprocess.check_output(['node', '--version'], text=True).strip() == prepared['node'], 'Node runtime changed'
    evidence = output / 'evidence'
    evidence.mkdir(exist_ok=True)
    logs = output / 'logs'
    python = sys.executable
    native_command = [python, source / NATIVE.relative_to(REPO) / 'run.py', output / 'native-build/build.json', source / CORPUS.relative_to(REPO)]
    pair_commands = [(track + '-' + engine, [python, source / ECO.relative_to(REPO) / 'run.py', track]) for track, engine in PAIRS]

    def pair_command(name, base, destination, verify=False):
        track, engine = name.split('-', 1)
        args = [*base, destination, '--competitor', engine]
        if track == 'native':
            args += ['--binary', output / 'ecosystem-build/worker', '--build-metadata', output / 'ecosystem-build/build.json']
        return args + (['--verify-only'] if verify else [])

    def fresh_attempt(path):
        # Keep interrupted attempts for diagnosis; never accept them as completed runs.
        if path.exists():
            path.rename(path.with_name(path.name + '-interrupted-' + str(time.time_ns())))
        return path

    # All option/output guards finish before the first timed lane.
    verify_root = fresh_attempt(output / 'verify')
    verify_root.mkdir()
    execute([*native_command, verify_root / 'native', '--verify-only'], source, logs / 'verify.log')
    for name, base in pair_commands:
        execute(pair_command(name, base, verify_root / name, True), source, logs / 'verify.log')
    if verify_only:
        print('All native and Node.js option/output guards passed; no timing or publication occurred.')
        return
    native_results = output / 'native-results'
    if not (native_results / 'summary.json').exists():
        print(f'Cooling down for {cooldown} seconds before native timing.', flush=True)
        time.sleep(cooldown)
        execute([*native_command, fresh_attempt(native_results)], source, logs / 'native-measure.log')
    execute([python, source / NATIVE.relative_to(REPO) / 'report.py', native_results], source, logs / 'native-report.log')
    native_evidence = evidence / 'native'
    if native_evidence.exists() and not (native_evidence / 'SHA256SUMS').exists():
        native_evidence.rename(output / ('native-archive-interrupted-' + str(time.time_ns())))
    if not native_evidence.exists():
        execute([python, source / NATIVE.relative_to(REPO) / 'archive.py', evidence,
                 '--name', 'native', '--results', native_results, '--build-dir', output / 'native-build',
                 '--seed-lock', source / REFERENCE.relative_to(REPO) / 'Cargo.lock', '--reference', source / REFERENCE.relative_to(REPO),
                 '--restore-dir', output / 'cache', '--host-file', output / 'host.txt', '--host-summary', prepared['machine'],
                 '--origin', (f"The [Blacksmith benchmark run]({context['run_url']}) used an isolated committed checkout."
                              if context else 'A manual benchmark run used an isolated committed checkout.'),
                 '--host-kind', 'managed-runner' if context else 'local',
                 '--verify-only-passed', '--harness-revision', prepared['revision'], '--build-log', logs / 'native-build.log',
                 '--source-audit', output / 'source-audit.json', '--tests-log', logs / 'native-comparison-tests.log'], source, logs / 'native-archive.log')
    for name, base in pair_commands:
        destination = evidence / name
        if destination.exists():
            try:
                eco_archive.validate(destination, prepared['revision'])
            except (AssertionError, KeyError, OSError, json.JSONDecodeError):
                destination.rename(output / (name + '-archive-interrupted-' + str(time.time_ns())))
            else:
                print('Already complete: ' + name, flush=True)
                continue
        raw = output / ('results-' + name)
        if not (raw / 'summary.json').exists():
            print(f'Cooling down for {cooldown} seconds before {name}.', flush=True)
            time.sleep(cooldown)
            execute(pair_command(name, base, fresh_attempt(raw)), source, logs / (name + '.log'))
        shutil.copyfile(logs / ('ecosystem-build.log' if name.startswith('native-') else 'node-build.log'), raw / 'build.log')
        eco_archive.retain(raw, destination, prepared['revision'])
    if context is not None:
        runner_context.assert_host(context)
        shutil.copyfile(source / 'benchmarks/manual-comparison/runner_context.py', evidence / 'runner-context.py')
    shutil.copyfile(output / 'prepared.json', evidence / 'suite.json')
    shutil.copyfile(output / 'host.txt', evidence / 'host.txt')
    shutil.copyfile(source / 'benchmarks/manual-comparison/cli.py', evidence / 'manual-cli.py')
    shutil.copyfile(source / 'scripts/benchmark-comparison', evidence / 'benchmark-comparison')
    verify_evidence(evidence)
    write(evidence / 'figures.json', {'figures': figures(evidence, 'LOCAL')})
    checksums(evidence)
    print('Complete. Publish with: ./scripts/benchmark-comparison publish ' + shlex.quote(str(output)), flush=True)


def checksums(folder, check=False):
    files = sorted(path for path in folder.rglob('*') if path.is_file() and path.name != 'SHA256SUMS')
    assert not any(path.is_symlink() for path in folder.rglob('*')), 'symlinks are not retained evidence'
    text = ''.join(f'{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.relative_to(folder)}\n' for path in files)
    if check:
        assert (folder / 'SHA256SUMS').read_text() == text, 'evidence checksum drift'
    else:
        (folder / 'SHA256SUMS').write_text(text)


def committed(revision, path):
    return subprocess.check_output(['git', 'show', revision + ':' + path], cwd=REPO)


def verify_platform(suite, config, build=None):
    details = platform_details(suite['system'], suite['architecture'])
    assert all(suite[key] == value for key, value in details.items()), 'inconsistent recorded platform'
    assert config['host_before']['platform'] == config['host_after']['platform'] == suite['host_platform'], 'mixed host platforms'
    if build is not None:
        assert platform_details(build['platform']['system'], build['platform']['machine']) == details, 'build target differs from recorded platform'
    return details


def verify_evidence(folder):
    suite = read(folder, 'suite')
    assert suite['schema'] == 2
    revision = suite['revision']
    assert re.fullmatch(r'[0-9a-f]{40}', revision), 'full source revision required'
    assert suite['corpus_sha256'] == native.sha(CORPUS)
    context = suite.get('runner_context')
    if context is not None:
        runner_context.validate(context)
        assert context['profile'] == suite['platform'], 'runner profile differs from recorded platform'
        assert (folder / 'runner-context.py').read_bytes() == committed(revision, 'benchmarks/manual-comparison/runner_context.py')
        provenance = json.loads((folder / 'native/checks/commands.json').read_text())
        assert provenance['host_kind'] == 'managed-runner' and context['run_url'] in provenance['workflow_origin']
    details = platform_details(suite['system'], suite['architecture'])
    assert all(suite[key] == value for key, value in details.items()), 'inconsistent recorded platform'
    for track, engine in PAIRS:
        config, _ = eco_archive.validate(folder / (track + '-' + engine), revision)
        assert config['engines'] == ['v2', engine] and config['track'] == track
        assert 'host_after' in config, 'incomplete run'
        verify_platform(suite, config)
        if track == 'node':
            assert config['native_addon_sha256'] == {'node/ferromark/' + suite['addon_file']: suite['addon_sha256']}
            assert config['runtime'] == suite['node']
    directory = folder / 'native'
    config, build = read(directory, 'run'), read(directory, 'build')
    corpus, outputs, rows = read(directory, 'corpus'), read(directory, 'verification'), read(directory, 'samples')
    frozen = native.read_json(CORPUS)
    assert corpus['cases'] == frozen['cases'] and config['corpus_sha256'] == suite['corpus_sha256']
    assert (config['rounds'], config['samples'], config['window_ms'], config['warmup_ms']) == (3, 6, 40, 60)
    assert config['case_filter'] is None and config['corpus_case_count'] == 57 and 'host_after' in config
    verify_platform(suite, config, build)
    assert tuple(config['engines']) == native.ENGINES and tuple(config['modes']) == native.MODES
    assert build['engines']['ferromark_v2']['revision'] == revision
    assert native.sha(directory / 'Cargo.lock') == build['lock_sha256']
    expected_jobs = [dict(name=c['name'], profile=c['profile'], members=[c['name']]) for c in frozen['cases']]
    for profile in ('commonmark', 'gfm'):
        members = [c['name'] for c in frozen['cases'] if c['profile'] == profile]
        if members:
            expected_jobs.append(dict(name='rotating-' + profile, profile=profile, members=members))
    assert config['jobs'] == expected_jobs
    assert set(outputs) == {case['name'] for case in frozen['cases']}
    for item in outputs.values():
        assert set(item['outputs']) == set(native.ENGINES)
        assert item['versus_v2'] == {e: native.classify(item['outputs']['v2'], item['outputs'][e]) for e in native.ENGINES}
    native_archive.audit_windows(config, rows, outputs)
    assert read(directory, 'summary') == native.summaries(rows, config['jobs']), 'native aggregate drift'
    for key, path in [('runner_sha256', 'run.py'), ('verifier_sha256', 'verify.py')]:
        assert hashlib.sha256(committed(revision, 'benchmarks/native-comparison/' + path)).hexdigest() == config[key]
    for path, digest in build['adapter_sha256'].items():
        assert hashlib.sha256(committed(revision, 'benchmarks/native-comparison/' + path)).hexdigest() == digest
    audit = read(directory, 'source-audit')
    assert audit['v2']['revision'] == revision and audit['v2']['sha256']
    source_paths = git('ls-tree', '-r', '--name-only', revision, '--', 'src', 'crates', 'Cargo.toml', 'Cargo.lock').splitlines()
    assert set(audit['v2']['sha256']) == set(source_paths), 'incomplete native core source audit'
    assert audit['v2']['checked_files'] == len(source_paths)
    for path, digest in audit['v2']['sha256'].items():
        assert hashlib.sha256(committed(revision, path)).hexdigest() == digest
    for value in [*audit.values(), *audit['native_dependencies'].values()]:
        assert not value.get('mismatches', [])
    return suite


def figures(folder, report):
    suite = read(folder, 'suite')
    catalog = json.loads((REPO / 'homepage/app/data/benchmark-projects.json').read_text())
    values = {}
    for track, engine in PAIRS:
        result = read(folder / (track + '-' + engine), 'summary')
        config = read(folder / (track + '-' + engine), 'run')
        values[engine] = (result['agreeing_documents'], result['v2_relative_throughput'], config['profile_scope'], track + '-' + engine, config['host_before']['time_utc'][:10])
    directory = folder / 'native'
    six, five, scores = native_archive.main_scores(read(directory, 'summary'), read(directory, 'verification'))
    for engine in LEGACY:
        group, members = ('six', six) if engine == 'ox-content' else ('five', five)
        speed = {mode: 1 / scores[group, mode][engine] for mode in native.MODES}
        values[engine] = (len(members), speed, 'Frozen profiles; shared ' + group + '-engine equivalent-HTML set', 'native', read(directory, 'run')['host_before']['time_utc'][:10])
    assert set(values) == {project['id'] for project in catalog}, 'every homepage project must be measured'
    result = []
    for project in catalog:
        count, speed, scope, lane, measured = values[project['id']]
        assert count > 0 and all(isinstance(v, (int, float)) and math.isfinite(v) and v > 0 for v in speed.values())
        result.append({**{key: project[key] for key in ('id', 'label', 'runtime')}, 'platform': suite['platform'],
                       'platformLabel': suite['platform_label'], 'machine': suite['machine'], 'measured': measured,
                       'revision': suite['revision'][:8], 'report': report + '/' + lane, 'profileScope': scope,
                       'documents': count, 'corpusDocuments': 57, 'fresh': speed['fresh'], 'reuse': speed['reuse']})
    return result


def content(values):
    groups = {}
    for value in values:
        groups.setdefault(value['platform'], []).append(value)
    return ''.join(platform_content(group) for group in groups.values())


def platform_content(values):
    text = f"\n## Manual comparison — {values[0]['platformLabel']}\n\n"
    text += 'All 14 homepage libraries were remeasured on one host from the same committed\nsource revision. Factors mean Ferromark throughput relative to the library.\nNative and Node.js calls have separate build and allocation contracts.\n\n'
    text += '| Runtime | Project | Equivalent documents | Fresh | Reuse |\n| --- | --- | ---: | ---: | ---: |\n'
    for value in values:
        text += f"| {value['runtime']} | {value['label']} | {value['documents']}/57 | {value['fresh']:.2f}× | {value['reuse']:.2f}× |\n"
    first = values[0]
    root = str(Path(first['report']).parent)
    dates = sorted({value['measured'] for value in values})
    text += f"\nMachine: {first['machine']}. Measurement dates: {', '.join(dates)}, source `{first['revision']}`.\n\n"
    text += 'Each pair uses three process rounds, six alternating 40 ms windows and\n60 ms warmup. Scores use equal-document geometric means of the per-engine\nmedian of round medians. Only equivalent HTML contributes. The original\nsix-engine harness retains its shared five/six-engine agreement sets, shared\nmimalloc, pinned Bun toolchain and rotating-batch diagnostics. The four other\nnative pairs use system malloc and the stable Rust toolchain; Node uses the\nrelease-node addon without PGO and includes public binding costs and GC.\ncmark and commonmark.js use CommonMark only in both engines on all inputs.\nThese sets and build contracts do not support a shared-set ranking.\n\n'
    text += 'Builds and downloads finish before verification and timing. Lanes run\nsequentially with a cooldown; host observations and per-round ranges remain\nin the raw evidence. Background system activity can still add noise. These\nresults describe this machine and corpus, with no significance claim. Other platforms\nand earlier reports remain separate.\n\n'
    text += f'[Raw evidence](https://github.com/sebastian-software/ferromark/tree/main/{root}), [manual workflow](https://github.com/sebastian-software/ferromark/blob/main/benchmarks/manual-comparison/README.md).\n'
    return text


def active():
    pointer = json.loads(CURRENT.read_text())
    values = []
    for platform_id, name in sorted(pointer['reports'].items()):
        path = report_path(name)
        checksums(path, check=True)
        suite = verify_evidence(path)
        assert suite['platform'] == platform_id, 'selected report belongs to another platform'
        values.extend(figures(path, str(path.relative_to(REPO))))
    return values


def select_report(pointer, platform_id, name):
    """Replace only the measured platform's selected report."""
    return {'reports': {**pointer['reports'], platform_id: name}}


def report_path(name):
    assert re.fullmatch(r'[0-9]{4}-[0-9]{2}-[0-9]{2}-[a-z0-9][a-z0-9-]*', name), 'use a dated lowercase report name'
    path = REPO / 'docs/reports' / name
    assert path.resolve().parent == (REPO / 'docs/reports').resolve() and not path.is_symlink()
    return path


def publish(output, name, check=False):
    evidence = output / 'evidence'
    checksums(evidence, check=True)
    verify_evidence(evidence)
    if check:
        print('Validated all 14 comparisons; no files changed.')
        return
    suite = read(evidence, 'suite')
    name = name or read(evidence / 'native', 'run')['host_before']['time_utc'][:10] + '-manual-' + suite['platform'] + '-' + suite['revision'][:8]
    target = report_path(name)
    if target.exists():
        raise ValueError('Report already exists; choose a new name. Historical reports are immutable.')
    with tempfile.TemporaryDirectory(prefix='.manual-comparison-', dir=target.parent) as temp:
        staged = Path(temp) / 'report'
        shutil.copytree(evidence, staged)
        values = figures(staged, str(target.relative_to(REPO)))
        write(staged / 'figures.json', {'figures': values})
        (staged / 'README.md').write_text('# Manual comparison\n' + content(values) + '\n[SHA256SUMS](SHA256SUMS) covers all retained evidence.\n')
        checksums(staged)
        staged.rename(target)
    write(CURRENT, select_report(json.loads(CURRENT.read_text()), suite['platform'], name))
    for script in ('publish_values.py', 'publish.py'):
        subprocess.run([sys.executable, ECO / script], cwd=REPO, check=True)
    print('Published local report and homepage data: ' + str(target))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    commands.add_parser('doctor', help='check the host, prerequisites, and clean committed checkout')
    for command in ('prepare', 'run', 'measure', 'verify', 'publish'):
        sub = commands.add_parser(command)
        sub.add_argument('output', type=lambda value: Path(value).expanduser().resolve())
        if command in ('prepare', 'run'):
            sub.add_argument('--runner-context', type=Path, help='retained, validated managed-runner metadata')
        if command in ('run', 'measure'):
            sub.add_argument('--cooldown-seconds', type=int, default=60)
        if command == 'publish':
            sub.add_argument('--name', help='new immutable report name, e.g. 2026-10-01-linux-x86-64')
            sub.add_argument('--check', action='store_true', help='validate the complete evidence without changing files')
    args = parser.parse_args()
    if sys.flags.optimize:
        parser.error('Python optimization disables evidence checks; use Python without -O.')
    if hasattr(args, 'cooldown_seconds') and not 0 <= args.cooldown_seconds <= 600:
        parser.error('cooldown must be between 0 and 600 seconds')
    try:
        if args.command == 'doctor':
            print(json.dumps(preflight(), indent=2))
        elif args.command == 'publish':
            publish(args.output, args.name, args.check)
        elif args.command == 'prepare':
            prepare(args.output, args.runner_context)
        else:
            if args.command == 'run':
                prepare(args.output, args.runner_context)
            awake = subprocess.Popen(['caffeinate', '-i', '-w', str(os.getpid())]) if platform.system() == 'Darwin' else None
            try:
                measure(args.output, getattr(args, 'cooldown_seconds', 0), verify_only=args.command == 'verify')
            finally:
                if awake is not None:
                    awake.terminate()
                    awake.wait()
    except (ValueError, AssertionError, KeyError, OSError, subprocess.CalledProcessError) as error:
        parser.exit(1, f'Benchmark stopped: {error}\nExisting logs and partial attempts are preserved.\n')


if __name__ == '__main__':
    main()
