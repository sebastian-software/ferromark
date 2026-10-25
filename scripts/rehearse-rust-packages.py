#!/usr/bin/env python3
"""Build the Cargo archive, compile its own targets and verify an isolated consumer."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tomllib

ROOT = Path(__file__).resolve().parents[1]


def run(args, log):
    with log.open('w') as stream:
        try:
            subprocess.run(args, cwd=ROOT, stdout=stream, stderr=subprocess.STDOUT, check=True)
        except subprocess.CalledProcessError:
            stream.flush()
            print(log.read_text()[-12000:], file=sys.stderr)
            raise


def patched_workspace(output, root_manifest):
    """Resolve the unpublished core patch in a disposable workspace.

    A global crates.io patch also replaces Ferrigura's registry Ferromark and
    necessarily changes that graph. Re-lock only this source mirror, never the
    checkout or the unpatched core archive. Keep all registry versions pinned
    with `cargo update --workspace`, then package with `--locked` as usual.
    """
    stage = output / 'source'
    stage.mkdir()
    for name in ['Cargo.toml', 'Cargo.lock']:
        shutil.copyfile(ROOT / name, stage / name)
    for name in ['src', 'tests', 'benches', 'examples', 'docs', 'benchmarks',
                 'README.md', 'LICENSE', 'LICENSE-MIT', 'UPSTREAM.md']:
        (stage / name).symlink_to(ROOT / name, target_is_directory=(ROOT / name).is_dir())
    for member in root_manifest['workspace']['members']:
        target = stage / member
        target.mkdir(parents=True)
        shutil.copyfile(ROOT / member / 'Cargo.toml', target / 'Cargo.toml')
        for source in (ROOT / member).iterdir():
            if source.name not in ['Cargo.toml', 'target']:
                (target / source.name).symlink_to(source, target_is_directory=source.is_dir())
    patch = f'patch.crates-io.ferromark.path={json.dumps(str(stage))}'
    run(['cargo', 'update', '--workspace', '--offline', '--manifest-path', str(stage / 'Cargo.toml'),
         '--config', patch], output / 'patched-workspace.log')
    return stage, patch


def packaged_targets_command(package, patched_core=None):
    """Compiles every target of the unpacked archive, not only its library.

    `tests/`, `benches/` and `examples/` ship, so a consumer running
    `cargo test` on the published crate compiles files the `include`
    allow-list has to cover: an `include_str!` reaching outside the package
    fails there and nowhere else. The isolated consumer below links the
    library alone and never sees it.

    `--offline` is enough because the dev-dependencies resolve from the same
    lockfile `cargo fetch --locked` populated. The command runs from the
    repository root through `--manifest-path`, so `rust-toolchain.toml` still
    selects the pinned toolchain for a package unpacked outside the checkout.
    """
    command = ['cargo', 'check', '--all-targets', '--locked', '--offline',
               '--manifest-path', str(package / 'Cargo.toml')]
    if patched_core is not None:
        command.extend([
            '--config',
            f'patch.crates-io.ferromark.path={json.dumps(str(patched_core))}',
        ])
    return command


def rehearsal_result(version, artifacts):
    """What this rehearsal proved. Every step above raises on failure."""
    return dict(version=version, packages=artifacts, packaged_consumer='passed',
        packaged_targets='passed',
        method='Full cargo package --locked in dependency order, including packaged builds; an '
               'isolated consumer links both unpacked archives, and each unpacked archive compiles '
               'its own test, bench and example targets.',
        limits='No registry upload or publishing credentials were tested.')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path, help='new directory; existing outputs are never overwritten')
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir()
    root = tomllib.loads((ROOT / 'Cargo.toml').read_text())
    workspace = root['workspace']
    # `ferromark` is the repository root package, which is what Release Please's
    # `rust` strategy requires. The optional transform crate is the only
    # published member; the native binding stays private.
    names = [root['package']['name']]
    version = root['package']['version']
    assert names == ['ferromark']
    for member in workspace['members']:
        manifest = tomllib.loads((ROOT / member / 'Cargo.toml').read_text())
        member_package = manifest['package']
        assert member_package['version'] == version, f'{member}: lockstep version'
        if member_package.get('publish') is not False:
            names.append(member_package['name'])
        if member_package['name'] == 'ferromark-transforms':
            dependency = manifest['dependencies']['ferromark']
            assert dependency['version'] == f'={version}', f'{member}: exact core version'
            assert dependency['path'] == '..', f'{member}: local development path'
    assert names == ['ferromark', 'ferromark-transforms']
    stage, patch = patched_workspace(output, root)
    for name in names:
        command = [
            'cargo', 'package', '-p', name, '--locked', '--offline', '--allow-dirty',
            '--target-dir', str(output / 'target'),
        ]
        if name == 'ferromark-transforms':
            command.extend([
                '--manifest-path', str(stage / 'Cargo.toml'),
                '--config',
                patch,
            ])
        run(command, output / f'{name}-package.log')
    unpacked = output / 'unpacked'
    unpacked.mkdir()
    artifacts = []
    for name in names:
        archive = output / 'target/package' / f'{name}-{version}.crate'
        with tarfile.open(archive) as tar:
            tar.extractall(unpacked, filter='data')
        package = unpacked / f'{name}-{version}'
        assert (package / 'README.md').is_file(), f'{name}: README'
        manifest = tomllib.loads((package / 'Cargo.toml').read_text())
        assert manifest['package']['version'] == version
        assert manifest['package']['repository'] == workspace['package']['repository']
        assert manifest['package']['license'] == 'MIT'
        assert (package / 'src/lib.rs').is_file()
        if name == 'ferromark':
            assert (package / 'LICENSE').read_bytes() == (ROOT / 'LICENSE').read_bytes(), f'{name}: upstream MIT notice'
            assert (package / 'LICENSE-MIT').is_file(), f'{name}: MIT license text'
            assert (package / 'UPSTREAM.md').is_file(), f'{name}: attribution'
        for section in ('dependencies', 'dev-dependencies', 'build-dependencies'):
            for dependency, spec in manifest.get(section, {}).items():
                assert not dependency.startswith('ferromark_'), f'{name}: internal crate dependency'
                if isinstance(spec, dict):
                    assert 'path' not in spec, f'{name}: leaked workspace path'
        if name == 'ferromark-transforms':
            core = manifest['dependencies']['ferromark']
            assert core['version'] == f'={version}', 'transforms: exact core dependency'
            assert 'path' not in core, 'transforms: published dependency must use crates.io'
        artifacts.append(dict(name=name, version=version, archive=str(archive.relative_to(output)),
            bytes=archive.stat().st_size, sha256=hashlib.sha256(archive.read_bytes()).hexdigest()))
    consumer = output / 'consumer'
    (consumer / 'src').mkdir(parents=True)
    core_path = json.dumps(str(unpacked / f'ferromark-{version}'))
    transforms_path = json.dumps(str(unpacked / f'ferromark-transforms-{version}'))
    (consumer / 'Cargo.toml').write_text(f'''[workspace]
[package]
name = "ferromark-transforms-packaged-consumer"
version = "0.0.0"
edition = "2024"
publish = false
[dependencies]
ferromark = {{ path = {core_path}, version = "={version}" }}
ferromark-transforms = {{ path = {transforms_path}, version = "={version}" }}
[patch.crates-io]
ferromark = {{ path = {core_path} }}
''')
    (consumer / 'src/main.rs').write_text('''use std::error::Error;
use ferromark::ast::{Document, Node};
use ferromark::{Allocator, HtmlRenderer, parse};
use ferromark_transforms::{BoxError, TransformContext, TransformPass, TransformPipeline};

struct AddPeriod;
impl TransformPass for AddPeriod {
    fn name(&self) -> &'static str { "add-period" }
    fn apply<'a>(&mut self, document: &mut Document<'a>, context: &TransformContext<'a>) -> Result<(), BoxError> {
        for node in &mut document.children {
            if let Node::Paragraph(paragraph) = node {
                if let Some(Node::Text(text)) = paragraph.children.last_mut() {
                    let value = format!("{}.", text.value);
                    context.replace_text_value(text, &value);
                }
            }
        }
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let source = "Hello";
    let arena = Allocator::new();
    let mut document = parse(&arena, source)?;
    let renderer_options = ferromark::HtmlRendererOptions::new();
    let context = TransformContext::new(&arena, source, &renderer_options);
    let mut pipeline = TransformPipeline::new();
    pipeline.add(AddPeriod);
    pipeline.run(&mut document, &context)?;
    let mut renderer = HtmlRenderer::new();
    assert_eq!(renderer.render(&document), "<p>Hello.</p>\\n");
    Ok(())
}
''')
    shutil.copyfile(ROOT / 'Cargo.lock', consumer / 'Cargo.lock')
    run(['cargo', 'run', '--offline', '--manifest-path', str(consumer / 'Cargo.toml')], output / 'consumer.log')
    original = tomllib.loads((ROOT / 'Cargo.lock').read_text())['package']
    resolved = tomllib.loads((consumer / 'Cargo.lock').read_text())['package']
    for package in resolved:
        if 'source' in package:
            assert any(all(candidate.get(key) == package.get(key) for key in ('name', 'version', 'source', 'checksum'))
                       for candidate in original), f'Unexpected registry dependency: {package["name"]}'
        elif package['name'] != 'ferromark-transforms-packaged-consumer':
            assert package['name'] in names and package['version'] == version
    for name in names:
        core = unpacked / f'ferromark-{version}' if name == 'ferromark-transforms' else None
        run(packaged_targets_command(unpacked / f'{name}-{version}', core),
            output / f'{name}-targets.log')
    result = rehearsal_result(version, artifacts)
    (output / 'results.json').write_text(json.dumps(result, indent=2) + '\n')
    print(f'Verified ferromark and ferromark-transforms archives, targets and isolated consumer: {output}')


if __name__ == '__main__':
    main()
