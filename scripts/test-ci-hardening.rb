#!/usr/bin/env ruby
# frozen_string_literal: true

require 'yaml'

class ContractError < StandardError; end

REPOSITORY_ROOT = File.expand_path('..', __dir__)
CONTRIBUTING_PATH = File.join(REPOSITORY_ROOT, 'CONTRIBUTING.md')
DEPENDENCY_POLICY_PATH = File.join(REPOSITORY_ROOT, 'deny.toml')
CHECKOUT_ACTION = 'actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1'
RUST_TOOLCHAIN_ACTION = 'dtolnay/rust-toolchain@2c7215f132e9ebf062739d9130488b56d53c060c'
INSTALL_ACTION = 'taiki-e/install-action@e67fa11c4b9316fa714ddf0abed07a0c3143b95b'
RUST_CACHE_ACTION = 'Swatinem/rust-cache@42dc69e1aa15d09112580998cf2ef0119e2e91ae'
CODECOV_ACTION = 'codecov/codecov-action@fb8b3582c8e4def4969c97caa2f19720cb33a72f'
CARGO_DENY_ACTION = 'EmbarkStudios/cargo-deny-action@3c6349835b2b7b196a839186cb8b78e02f7b5f25'
RUSTDOC_COMMAND = "RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features --locked"
COVERAGE_COMMAND = 'cargo llvm-cov --all-features --locked --no-report'
COVERAGE_REPORT_PREFIX = 'cargo llvm-cov report --lcov --output-path lcov.info --fail-under-lines '
CODECOV_INPUTS = {
  'files' => 'lcov.info',
  'fail_ci_if_error' => false,
  'token' => '${{ secrets.CODECOV_TOKEN }}'
}.freeze

def fail_contract(message)
  raise ContractError, "CI hardening contract: #{message}"
end

def validate(
  workflow,
  contributing: File.read(CONTRIBUTING_PATH),
  dependency_policy: File.read(DEPENDENCY_POLICY_PATH)
)
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
  expected_actions = [CHECKOUT_ACTION, RUST_TOOLCHAIN_ACTION, INSTALL_ACTION, RUST_CACHE_ACTION, CODECOV_ACTION]
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
  unless commands.length == 2 && commands.first == COVERAGE_COMMAND
    fail_contract('coverage must run cargo llvm-cov for all features with the lockfile')
  end

  report_command = commands.last
  unless report_command.start_with?(COVERAGE_REPORT_PREFIX)
    fail_contract('coverage must report lcov.info and fail under a line coverage floor')
  end

  threshold = report_command.delete_prefix(COVERAGE_REPORT_PREFIX)
  fail_contract('the coverage floor must be a plain percentage') unless threshold.match?(/\A\d+\z/)

  upload = steps.find { |step| step['uses'] == CODECOV_ACTION }.fetch('with')
  fail_contract("coverage must upload lcov.info as #{CODECOV_INPUTS.inspect}") unless upload == CODECOV_INPUTS

  unless contributing.include?("#{threshold}% line coverage")
    fail_contract("CONTRIBUTING.md must document the #{threshold}% line coverage floor")
  end

  dependency_job = jobs['cargo-deny']
  fail_contract('a cargo-deny job must enforce the dependency policy') unless dependency_job
  dependency_steps = dependency_job.fetch('steps')
  dependency_actions = dependency_steps.map { |step| step['uses'] }.compact
  unless dependency_actions == [CHECKOUT_ACTION, CARGO_DENY_ACTION]
    fail_contract("cargo-deny actions must be pinned and ordered as #{[CHECKOUT_ACTION, CARGO_DENY_ACTION].inspect}")
  end

  policy_inputs = dependency_steps.find { |step| step['uses'] == CARGO_DENY_ACTION }.fetch('with')
  unless policy_inputs == { 'command' => 'check', 'arguments' => '' }
    fail_contract('cargo-deny must run every check with the arguments from deny.toml')
  end

  unless dependency_policy.include?('yanked = "deny"')
    fail_contract('deny.toml must reject yanked crates')
  end
end

def deep_copy(value)
  Marshal.load(Marshal.dump(value))
end

def assert_rejected(
  label,
  workflow,
  contributing: File.read(CONTRIBUTING_PATH),
  dependency_policy: File.read(DEPENDENCY_POLICY_PATH)
)
  yield workflow
  validate(workflow, contributing: contributing, dependency_policy: dependency_policy)
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
    step['run'] = 'cargo llvm-cov --locked --no-report'
  end
  assert_rejected('missing coverage upload', deep_copy(workflow)) do |copy|
    copy['jobs']['coverage']['steps'].reject! { |step| step['uses'] == CODECOV_ACTION }
  end
  assert_rejected('mutable coverage upload action', deep_copy(workflow)) do |copy|
    step = copy['jobs']['coverage']['steps'].find { |candidate| candidate['uses'] == CODECOV_ACTION }
    step['uses'] = 'codecov/codecov-action@v7'
  end
  assert_rejected('coverage without a floor', deep_copy(workflow)) do |copy|
    step = copy['jobs']['coverage']['steps'].find do |candidate|
      candidate['run']&.start_with?(COVERAGE_REPORT_PREFIX)
    end
    step['run'] = 'cargo llvm-cov report --lcov --output-path lcov.info'
  end
  assert_rejected(
    'undocumented coverage floor',
    deep_copy(workflow),
    contributing: File.read(CONTRIBUTING_PATH).gsub(/\d+% line coverage/, 'an unstated% line coverage')
  ) { |_copy| }
  assert_rejected('missing dependency policy job', deep_copy(workflow)) do |copy|
    copy['jobs'].delete('cargo-deny')
  end
  assert_rejected('mutable cargo-deny action', deep_copy(workflow)) do |copy|
    step = copy['jobs']['cargo-deny']['steps'].find { |candidate| candidate['uses'] == CARGO_DENY_ACTION }
    step['uses'] = 'EmbarkStudios/cargo-deny-action@v2'
  end
  assert_rejected(
    'yanked crates allowed by the dependency policy',
    deep_copy(workflow),
    dependency_policy: File.read(DEPENDENCY_POLICY_PATH).sub('yanked = "deny"', 'yanked = "warn"')
  ) { |_copy| }
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
