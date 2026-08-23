#!/usr/bin/env ruby
# frozen_string_literal: true

require 'yaml'

repository_root = File.expand_path('..', __dir__)
contributing = File.read(File.join(repository_root, 'CONTRIBUTING.md'))
cargo_toml = File.read(File.join(repository_root, 'Cargo.toml'))
workflow = YAML.safe_load(File.read(File.join(repository_root, '.github/workflows/ci.yml')), aliases: false)

def fail_contract(message)
  abort "CONTRIBUTING CI contract: #{message}"
end

rust_version = cargo_toml[/^rust-version\s*=\s*"([^"]+)"\s*$/, 1]
fail_contract('Cargo.toml must declare rust-version') unless rust_version

expected_msrv = "The minimum supported Rust version (MSRV) is Rust #{rust_version}."
fail_contract("must state #{expected_msrv.inspect}") unless contributing.include?(expected_msrv)

jobs = workflow.fetch('jobs')
test_job = jobs.fetch('test')
test_command = test_job.fetch('steps').map { |step| step['run'] }.compact.find do |command|
  command.include?('${{ matrix.args }}') && command.start_with?('cargo test ')
end
fail_contract('CI test job must use a matrix command') unless test_command

test_matrix = test_job.fetch('strategy').fetch('matrix').fetch('include')
all_feature_args = test_matrix.map do |entry|
  entry['args'] if entry['args'] == '--all-features'
end.compact
fail_contract('CI test matrix must include --all-features') if all_feature_args.empty?

msrv_entries = test_matrix.select { |entry| entry['rust'] == rust_version }
fail_contract("CI test matrix must test Rust #{rust_version}") if msrv_entries.empty?
msrv_args = msrv_entries.map { |entry| entry['args'] }
unless msrv_args.include?('') && msrv_args.include?('--all-features')
  fail_contract("CI Rust #{rust_version} entries must cover default and --all-features")
end

all_features_test_command = test_command.sub('${{ matrix.args }}', all_feature_args.first)
unless contributing.include?(all_features_test_command)
  fail_contract("must include CI all-features test command #{all_features_test_command.inspect}")
end

clippy_job = jobs.fetch('clippy')
clippy_command = clippy_job.fetch('steps').map { |step| step['run'] }.compact.find do |command|
  command.start_with?('cargo clippy ')
end
fail_contract('CI must define a cargo clippy command') unless clippy_command
fail_contract("must include CI clippy command #{clippy_command.inspect}") unless contributing.include?(clippy_command)

fmt_job = jobs.fetch('fmt')
fmt_command = fmt_job.fetch('steps').map { |step| step['run'] }.compact.find do |command|
  command.start_with?('cargo fmt ')
end
fail_contract('CI must define a cargo fmt command') unless fmt_command
fail_contract("must include CI fmt command #{fmt_command.inspect}") unless contributing.include?(fmt_command)

fail_contract('must link the Node workspace release checks') unless contributing.include?('[releasing guide](docs/releasing.md)')

puts 'CONTRIBUTING CI contract checks passed'
