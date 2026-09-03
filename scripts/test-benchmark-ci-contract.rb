#!/usr/bin/env ruby
# frozen_string_literal: true

require 'yaml'

class ContractError < StandardError; end

WORKFLOW_PATH = File.expand_path('../.github/workflows/benchmarks.yml', __dir__)
CHECKOUT_ACTION = 'actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1'
RUST_TOOLCHAIN_ACTION = 'dtolnay/rust-toolchain@2c7215f132e9ebf062739d9130488b56d53c060c'
RUST_CACHE_ACTION = 'Swatinem/rust-cache@42dc69e1aa15d09112580998cf2ef0119e2e91ae'
CACHE_RESTORE_ACTION = 'actions/cache/restore@0057852bfaa89a56745cba8c7296529d2fc39830'
BENCHMARK_ACTION = 'benchmark-action/github-action-benchmark@52576c92bccf6ac60c8223ec7eb2565637cae9ba'
CACHE_SAVE_ACTION = 'actions/cache/save@0057852bfaa89a56745cba8c7296529d2fc39830'
CACHE_KEY = 'ferromark-benchmark-${{ runner.os }}-main-${{ github.sha }}'

def fail_contract(message)
  raise ContractError, "Benchmark CI contract: #{message}"
end

def triggers(workflow)
  # Psych follows YAML 1.1 and parses an unquoted `on` key as boolean true.
  workflow['on'] || workflow[true] || fail_contract('workflow triggers are missing')
end

def validate(workflow)
  unless workflow.fetch('permissions') == { 'contents' => 'read' }
    fail_contract('workflow permissions must grant contents: read only')
  end

  events = triggers(workflow)
  push = events.fetch('push')
  pull_request = events.fetch('pull_request')
  fail_contract('push must target main') unless push.fetch('branches') == ['main']
  fail_contract('push must run for src changes') unless push.fetch('paths').include?('src/**')
  unless pull_request.fetch('paths').include?('src/**')
    fail_contract('pull requests must run benchmarks for src changes')
  end
  fail_contract('manual benchmark runs must remain available') unless events.key?('workflow_dispatch')

  job = workflow.fetch('jobs').fetch('benchmark')
  fail_contract('benchmark job must use ubuntu-latest') unless job.fetch('runs-on') == 'ubuntu-latest'

  steps = job.fetch('steps')
  actions = steps.map { |step| step['uses'] }.compact
  expected_actions = [
    CHECKOUT_ACTION,
    RUST_TOOLCHAIN_ACTION,
    RUST_CACHE_ACTION,
    CACHE_RESTORE_ACTION,
    BENCHMARK_ACTION,
    CACHE_SAVE_ACTION
  ]
  fail_contract('workflow actions must remain pinned and ordered') unless actions == expected_actions

  restore = steps.find { |step| step['uses'] == CACHE_RESTORE_ACTION }
  restore_inputs = restore.fetch('with')
  fail_contract('benchmark history cache key changed') unless restore_inputs.fetch('key') == CACHE_KEY
  unless restore_inputs.fetch('restore-keys').include?('ferromark-benchmark-${{ runner.os }}-main-')
    fail_contract('pull requests must restore the latest main benchmark history')
  end

  command = steps.find { |step| step['name'] == 'Run representative benchmarks' }.fetch('run')
  required_fragments = [
    'set -o pipefail',
    'cargo bench --locked --bench parsing',
    '--output-format bencher',
    '--warm-up-time 1',
    '--measurement-time 3',
    '--sample-size 30'
  ]
  required_fragments.each do |fragment|
    fail_contract("benchmark command must include #{fragment}") unless command.include?(fragment)
  end

  compare = steps.find { |step| step['uses'] == BENCHMARK_ACTION }.fetch('with')
  expected_compare = {
    'name' => 'ferromark parser',
    'tool' => 'cargo',
    'output-file-path' => 'benchmark-output.txt',
    'external-data-json-path' => '.benchmark-cache/benchmark-data.json',
    'save-data-file' => "${{ github.ref == 'refs/heads/main' && github.event_name != 'pull_request' }}",
    'alert-threshold' => '120%',
    'fail-threshold' => '120%',
    'fail-on-alert' => "${{ github.event_name == 'pull_request' }}",
    'summary-always' => true
  }
  fail_contract('comparison must fail pull requests at a 20% regression') unless compare == expected_compare

  save = steps.find { |step| step['uses'] == CACHE_SAVE_ACTION }
  expected_condition = "github.ref == 'refs/heads/main' && github.event_name != 'pull_request' && steps.benchmark-history.outputs.cache-hit != 'true'"
  unless save.fetch('if') == expected_condition
    fail_contract('benchmark history must only be saved outside pull requests')
  end
  fail_contract('saved benchmark history must reuse the restore key') unless save.dig('with', 'key') == CACHE_KEY
end

def deep_copy(value)
  Marshal.load(Marshal.dump(value))
end

def assert_rejected(label, workflow)
  yield workflow
  validate(workflow)
  raise "#{label} mutation unexpectedly passed"
rescue ContractError
  # Expected: this mutation must be rejected by the contract.
end

def self_test(workflow)
  validate(workflow)

  assert_rejected('write-scoped token', deep_copy(workflow)) do |copy|
    copy['permissions']['contents'] = 'write'
  end
  assert_rejected('missing source trigger', deep_copy(workflow)) do |copy|
    triggers(copy).fetch('pull_request').fetch('paths').delete('src/**')
  end
  assert_rejected('mutable benchmark action', deep_copy(workflow)) do |copy|
    step = copy.fetch('jobs').fetch('benchmark').fetch('steps').find { |candidate| candidate['uses'] == BENCHMARK_ACTION }
    step['uses'] = 'benchmark-action/github-action-benchmark@v1'
  end
  assert_rejected('relaxed regression threshold', deep_copy(workflow)) do |copy|
    step = copy.fetch('jobs').fetch('benchmark').fetch('steps').find { |candidate| candidate['uses'] == BENCHMARK_ACTION }
    step.fetch('with')['fail-threshold'] = '150%'
  end
  assert_rejected('PR cache write', deep_copy(workflow)) do |copy|
    step = copy.fetch('jobs').fetch('benchmark').fetch('steps').find { |candidate| candidate['uses'] == CACHE_SAVE_ACTION }
    step['if'] = 'always()'
  end
end

workflow = YAML.safe_load(File.read(WORKFLOW_PATH), aliases: false)

begin
  if ARGV == ['--self-test']
    self_test(workflow)
  elsif ARGV.empty?
    validate(workflow)
  else
    abort 'usage: test-benchmark-ci-contract.rb [--self-test]'
  end
rescue ContractError => error
  abort error.message
end

puts 'benchmark CI contract checks passed'
