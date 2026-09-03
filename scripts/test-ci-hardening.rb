#!/usr/bin/env ruby
# frozen_string_literal: true

require 'yaml'

class ContractError < StandardError; end

CHECKOUT_ACTION = 'actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1'
RUST_TOOLCHAIN_ACTION = 'dtolnay/rust-toolchain@2c7215f132e9ebf062739d9130488b56d53c060c'
INSTALL_ACTION = 'taiki-e/install-action@e67fa11c4b9316fa714ddf0abed07a0c3143b95b'
RUST_CACHE_ACTION = 'Swatinem/rust-cache@42dc69e1aa15d09112580998cf2ef0119e2e91ae'
RUSTDOC_COMMAND = "RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features --locked"
COVERAGE_COMMAND = 'cargo llvm-cov --all-features --locked'

def fail_contract(message)
  raise ContractError, "CI hardening contract: #{message}"
end

def validate(workflow)
  unless workflow.fetch('permissions') == { 'contents' => 'read' }
    fail_contract('top-level permissions must grant contents: read only')
  end

  jobs = workflow.fetch('jobs')
  rustsec = jobs.fetch('rustsec')
  unless rustsec.fetch('permissions') == { 'contents' => 'read', 'checks' => 'write' }
    fail_contract('rustsec permissions must grant only contents: read and checks: write')
  end

  fmt_commands = jobs.fetch('fmt').fetch('steps').map { |step| step['run'] }.compact
  fail_contract('fmt job must reject rustdoc warnings for all features') unless fmt_commands.include?(RUSTDOC_COMMAND)

  coverage = jobs.fetch('coverage')
  fail_contract('coverage job must run on ubuntu-latest') unless coverage.fetch('runs-on') == 'ubuntu-latest'

  steps = coverage.fetch('steps')
  expected_actions = [CHECKOUT_ACTION, RUST_TOOLCHAIN_ACTION, INSTALL_ACTION, RUST_CACHE_ACTION]
  actual_actions = steps.map { |step| step['uses'] }.compact
  unless actual_actions == expected_actions
    fail_contract("coverage actions must be pinned and ordered as #{expected_actions.inspect}")
  end

  toolchain = steps.find { |step| step['uses'] == RUST_TOOLCHAIN_ACTION }.fetch('with')
  unless toolchain == { 'toolchain' => 'stable', 'components' => 'llvm-tools-preview' }
    fail_contract('coverage must install stable Rust with llvm-tools-preview')
  end

  installer = steps.find { |step| step['uses'] == INSTALL_ACTION }.fetch('with')
  fail_contract('coverage must install cargo-llvm-cov') unless installer == { 'tool' => 'cargo-llvm-cov' }

  commands = steps.map { |step| step['run'] }.compact
  fail_contract('coverage must run cargo llvm-cov for all features with the lockfile') unless commands == [COVERAGE_COMMAND]
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

  assert_rejected('write-scoped top-level token', deep_copy(workflow)) do |copy|
    copy['permissions']['contents'] = 'write'
  end
  assert_rejected('missing rustsec checks permission', deep_copy(workflow)) do |copy|
    copy['jobs']['rustsec']['permissions'].delete('checks')
  end
  assert_rejected('missing rustdoc warning gate', deep_copy(workflow)) do |copy|
    copy['jobs']['fmt']['steps'].reject! { |step| step['run'] == RUSTDOC_COMMAND }
  end
  assert_rejected('mutable coverage action', deep_copy(workflow)) do |copy|
    step = copy['jobs']['coverage']['steps'].find { |candidate| candidate['uses'] == INSTALL_ACTION }
    step['uses'] = 'taiki-e/install-action@v2'
  end
  assert_rejected('coverage without all features', deep_copy(workflow)) do |copy|
    step = copy['jobs']['coverage']['steps'].find { |candidate| candidate['run'] == COVERAGE_COMMAND }
    step['run'] = 'cargo llvm-cov --locked'
  end
end

workflow_path = File.expand_path('../.github/workflows/ci.yml', __dir__)
workflow = YAML.safe_load(File.read(workflow_path), aliases: false)

begin
  if ARGV == ['--self-test']
    self_test(workflow)
  elsif ARGV.empty?
    validate(workflow)
  else
    abort 'usage: test-ci-hardening.rb [--self-test]'
  end
rescue ContractError => error
  abort error.message
end

puts 'CI hardening contract checks passed'
