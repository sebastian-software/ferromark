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
  { 'os' => 'macos-latest', 'rust_target' => 'aarch64-apple-darwin' },
  { 'os' => 'macos-15-intel', 'rust_target' => 'x86_64-apple-darwin' },
  { 'os' => 'ubuntu-24.04-arm', 'rust_target' => 'aarch64-unknown-linux-gnu' },
  { 'os' => 'windows-latest', 'rust_target' => 'x86_64-pc-windows-msvc' },
  { 'os' => 'windows-11-arm', 'rust_target' => 'aarch64-pc-windows-msvc' }
]
actual_targets = native.fetch('strategy').fetch('matrix').fetch('include')

unless actual_targets == expected_targets
  abort "native CI matrix must execute the supported macOS, Linux arm64, and Windows targets"
end

steps = native.fetch('steps')
build_index = steps.index { |step| step['run'] == 'pnpm build:native' }
test_index = steps.index { |step| step['run'] == 'pnpm test' }

abort 'native CI job must build the N-API binding' unless build_index
unless steps.fetch(build_index).fetch('env') == { 'FERROMARK_RUST_TARGET' => '${{ matrix.rust_target }}' }
  abort 'native CI job must build the N-API binding for the matrix target'
end
abort 'native CI job must run pnpm test after building the N-API binding' unless test_index && test_index > build_index

puts 'native CI matrix checks passed'
