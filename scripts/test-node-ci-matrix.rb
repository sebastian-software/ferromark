#!/usr/bin/env ruby
# frozen_string_literal: true

require 'yaml'

workflow_path = File.expand_path('../.github/workflows/ci.yml', __dir__)
workflow = YAML.safe_load(File.read(workflow_path), aliases: false)
native = workflow.fetch('jobs').fetch('native')

unless native.fetch('runs-on') == '${{ matrix.os }}'
  abort 'native CI job must run on the operating system selected by the matrix'
end

unless native.fetch('permissions') == { 'contents' => 'read' }
  abort 'native CI job must use contents: read permissions only'
end

expected_targets = [
  { 'os' => 'macos-latest', 'rust_target' => 'aarch64-apple-darwin', 'artifact' => 'darwin-arm64', 'runtime_test' => true },
  { 'os' => 'macos-15-intel', 'rust_target' => 'x86_64-apple-darwin', 'artifact' => 'darwin-x64', 'runtime_test' => true },
  { 'os' => 'ubuntu-24.04-arm', 'rust_target' => 'aarch64-unknown-linux-gnu', 'artifact' => 'linux-arm64-gnu', 'runtime_test' => true },
  { 'os' => 'ubuntu-latest', 'rust_target' => 'x86_64-unknown-linux-musl', 'artifact' => 'linux-x64-musl', 'runtime_test' => false },
  { 'os' => 'ubuntu-latest', 'rust_target' => 'aarch64-unknown-linux-musl', 'artifact' => 'linux-arm64-musl', 'runtime_test' => false },
  { 'os' => 'windows-latest', 'rust_target' => 'x86_64-pc-windows-msvc', 'artifact' => 'win32-x64-msvc', 'runtime_test' => true },
  { 'os' => 'windows-11-arm', 'rust_target' => 'aarch64-pc-windows-msvc', 'artifact' => 'win32-arm64-msvc', 'runtime_test' => true }
]
actual_targets = native.fetch('strategy').fetch('matrix').fetch('include')

unless actual_targets == expected_targets
  abort "native CI matrix must build every non-host platform, including Linux musl"
end

steps = native.fetch('steps')
build_index = steps.index { |step| step['run'] == 'pnpm build:native' }
test_index = steps.index { |step| step['run'] == 'pnpm test' }
verify_index = steps.index do |step|
  step['run'] == 'node ./scripts/verify-platform-artifact.mjs ${{ matrix.artifact }}'
end
zig_index = steps.index do |step|
  step['uses'] == 'mlugg/setup-zig@d1434d08867e3ee9daa34448df10607b98908d29'
end
zigbuild_index = steps.index do |step|
  step['uses'] == 'taiki-e/install-action@e67fa11c4b9316fa714ddf0abed07a0c3143b95b'
end

abort 'native CI job must build the N-API binding' unless build_index
unless steps.fetch(build_index).fetch('env') == { 'FERROMARK_RUST_TARGET' => '${{ matrix.rust_target }}' }
  abort 'native CI job must build the N-API binding for the matrix target'
end
abort 'native CI job must run pnpm test after building the N-API binding' unless test_index && test_index > build_index
unless steps.fetch(test_index).fetch('if') == '${{ matrix.runtime_test }}'
  abort 'native CI job must skip runtime tests only for cross-compiled musl targets'
end
unless verify_index && verify_index > build_index
  abort 'native CI job must verify the generated platform package'
end
unless zig_index && steps.fetch(zig_index).fetch('if') == "${{ contains(matrix.rust_target, '-musl') }}"
  abort 'native CI job must install Zig for musl targets'
end
unless zigbuild_index && steps.fetch(zigbuild_index).fetch('with') == { 'tool' => 'cargo-zigbuild' }
  abort 'native CI job must install cargo-zigbuild for musl targets'
end

puts 'native CI matrix checks passed'
