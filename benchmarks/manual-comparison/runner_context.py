#!/usr/bin/env python3
"""Record and validate the two Blacksmith comparison profiles before work starts."""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import platform
import re
import subprocess

GIB = 1024 ** 3
PROFILES = {
    'macos-arm64': {'label': 'blacksmith-6vcpu-macos-26', 'system': 'Darwin',
                    'architecture': 'arm64', 'cpus': 6, 'memory_gib': 24, 'os_version': '26', 'cpu': 'Apple M4 Pro (Virtual)'},
    'linux-x86-64': {'label': 'blacksmith-4vcpu-ubuntu-2404', 'system': 'Linux',
                     'architecture': 'x86_64', 'cpus': 4, 'memory_gib': 16, 'os_version': '24.04'},
}


def probe(args):
    return subprocess.check_output(args, text=True).strip()


def observe():
    system = platform.system()
    if system == 'Darwin':
        memory = int(probe(['sysctl', '-n', 'hw.memsize']))
        cpu = probe(['sysctl', '-n', 'machdep.cpu.brand_string'])
        version = probe(['sw_vers', '-productVersion'])
        translated = subprocess.run(['sysctl', '-in', 'sysctl.proc_translated'], capture_output=True, text=True)
        if translated.stdout.strip() == '1':
            raise ValueError('Rosetta cannot be used for this runner profile.')
    elif system == 'Linux':
        memory = next(int(line.split()[1]) * 1024 for line in Path('/proc/meminfo').read_text().splitlines()
                      if line.startswith('MemTotal:'))
        cpu = next(line.split(':', 1)[1].strip() for line in Path('/proc/cpuinfo').read_text().splitlines()
                   if line.startswith('model name'))
        version = platform.freedesktop_os_release().get('VERSION_ID', '')
        if platform.freedesktop_os_release().get('ID') != 'ubuntu':
            raise ValueError('The Linux comparison profile requires Ubuntu.')
    else:
        raise ValueError('These runner profiles require macOS or Linux.')
    return {'system': system, 'architecture': platform.machine(), 'cpu': cpu,
            'cpus': os.cpu_count(), 'available_cpus': len(os.sched_getaffinity(0)) if hasattr(os, 'sched_getaffinity') else os.cpu_count(),
            'memory_bytes': memory, 'os_version': version, 'kernel': platform.release()}


def validate(context):
    if context.get('schema') != 1 or context.get('provider') != 'Blacksmith':
        raise ValueError('Unknown runner context.')
    expected = PROFILES.get(context.get('profile'))
    if expected is None or context.get('runner_label') != expected['label']:
        raise ValueError('Unknown or mismatched runner profile.')
    actual = context['observed']
    for key in ('system', 'architecture', 'cpus'):
        if actual[key] != expected[key]:
            raise ValueError(f'Runner {key}: expected {expected[key]}, found {actual[key]}.')
    if actual['available_cpus'] != expected['cpus']:
        raise ValueError('CPU affinity does not match the requested runner size.')
    # Linux reports usable RAM after kernel reservations. Allow up to 5% below
    # the advertised size, and 256 MiB above; reject larger automatic upgrades.
    memory = expected['memory_gib'] * GIB
    if not memory * 0.95 <= actual['memory_bytes'] <= memory + GIB / 4:
        raise ValueError('Runner memory does not match the requested size.')
    version = actual['os_version'].split('.')[0] if expected['system'] == 'Darwin' else actual['os_version']
    if version != expected['os_version']:
        raise ValueError('Runner OS version does not match the pinned image.')
    if expected.get('cpu') and actual['cpu'] != expected['cpu']:
        raise ValueError('Runner CPU model does not match the selected profile.')
    if not re.fullmatch(r'https://github\.com/sebastian-software/ferromark/actions/runs/[0-9]+', context['run_url']):
        raise ValueError('Runner context must identify a Ferromark run in sebastian-software.')
    return expected


def assert_host(context):
    validate(context)
    if observe() != context['observed']:
        raise ValueError('Runner resources or OS changed after the context was captured.')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('profile', choices=PROFILES)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    try:
        if os.environ.get('GITHUB_REPOSITORY') != 'sebastian-software/ferromark':
            raise ValueError('Run these jobs in sebastian-software/ferromark.')
        context = {'schema': 1, 'provider': 'Blacksmith', 'profile': args.profile,
                   'runner_label': PROFILES[args.profile]['label'], 'observed': observe(),
                   'run_url': os.environ['BENCH_RUN_URL'], 'run_attempt': os.environ['GITHUB_RUN_ATTEMPT'],
                   'image_os': os.environ.get('ImageOS', 'unavailable'), 'image_version': os.environ.get('ImageVersion', 'unavailable')}
        # Preserve observations even if a provider upgrade violates the profile.
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(context, indent=2) + '\n')
        validate(context)
        print(json.dumps(context, indent=2))
    except (ValueError, KeyError, OSError, subprocess.CalledProcessError) as error:
        parser.exit(1, f'Runner profile rejected: {error}\n')


if __name__ == '__main__':
    main()
