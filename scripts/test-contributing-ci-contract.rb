#!/usr/bin/env ruby
# frozen_string_literal: true

require 'fileutils'
require 'tmpdir'
require 'yaml'

class ContractError < StandardError; end

def fail_contract(message)
  raise ContractError, "CONTRIBUTING CI contract: #{message}"
end

def markdown_section(document, heading)
  start = document.index("## #{heading}\n")
  fail_contract("must contain a #{heading.inspect} section") unless start

  content_start = start + heading.length + 4
  next_heading = document.index(/^## /, content_start)
  document[content_start...(next_heading || document.length)]
end

def fenced_bash_commands(section, heading)
  match = section.match(/```bash\n(?<commands>.*?)\n```/m)
  fail_contract("#{heading.inspect} must contain a bash code block") unless match

  match[:commands].lines.map(&:strip).reject(&:empty?)
end

def ci_command(job, prefix)
  command = job.fetch('steps').map { |step| step['run'] }.compact.find do |candidate|
    candidate.start_with?(prefix)
  end
  fail_contract("CI must define a #{prefix.strip} command") unless command

  command
end

def validate(repository_root)
  contributing = File.read(File.join(repository_root, 'CONTRIBUTING.md'))
  cargo_toml = File.read(File.join(repository_root, 'Cargo.toml'))
  workflow = YAML.safe_load(File.read(File.join(repository_root, '.github/workflows/ci.yml')), aliases: false)

  rust_version = cargo_toml[/^rust-version\s*=\s*"([^"]+)"\s*$/, 1]
  fail_contract('Cargo.toml must declare rust-version') unless rust_version

  getting_started = markdown_section(contributing, 'Getting started')
  expected_msrv = "The minimum supported Rust version (MSRV) is Rust #{rust_version}."
  unless getting_started.include?(expected_msrv)
    fail_contract("Getting started must state #{expected_msrv.inspect}")
  end
  unless getting_started.include?('Run the [required local checks](#required-local-checks) below.')
    fail_contract('Getting started must point contributors to Required local checks')
  end
  bootstrap_commands = fenced_bash_commands(getting_started, 'Getting started')
  if bootstrap_commands.any? { |command| command.start_with?('cargo ') }
    fail_contract('Getting started must not duplicate cargo commands from Required local checks')
  end

  jobs = workflow.fetch('jobs')
  test_job = jobs.fetch('test')
  test_command = ci_command(test_job, 'cargo test ')
  fail_contract('CI test job must use a matrix command') unless test_command.include?('${{ matrix.args }}')

  test_matrix = test_job.fetch('strategy').fetch('matrix').fetch('include')
  all_feature_args = test_matrix.map { |entry| entry['args'] if entry['args'] == '--all-features' }.compact
  fail_contract('CI test matrix must include --all-features') if all_feature_args.empty?

  msrv_entries = test_matrix.select { |entry| entry['rust'] == rust_version }
  fail_contract("CI test matrix must test Rust #{rust_version}") if msrv_entries.empty?
  msrv_args = msrv_entries.map { |entry| entry['args'] }
  unless msrv_args.include?('') && msrv_args.include?('--all-features')
    fail_contract("CI Rust #{rust_version} entries must cover default and --all-features")
  end

  expected_commands = [
    test_command.sub('${{ matrix.args }}', all_feature_args.first),
    ci_command(jobs.fetch('clippy'), 'cargo clippy '),
    ci_command(jobs.fetch('fmt'), 'cargo fmt ')
  ]
  required_checks = markdown_section(contributing, 'Required local checks')
  actual_commands = fenced_bash_commands(required_checks, 'Required local checks')
  unless actual_commands == expected_commands
    fail_contract(
      "Required local checks commands must exactly match CI: expected #{expected_commands.inspect}, got #{actual_commands.inspect}"
    )
  end
  unless required_checks.include?('[releasing guide](docs/releasing.md)')
    fail_contract('Required local checks must link the Node workspace release checks')
  end
end

def with_fixture(source_root)
  Dir.mktmpdir('ferromark-contributing-contract.') do |fixture_root|
    FileUtils.mkdir_p(File.join(fixture_root, '.github/workflows'))
    %w[CONTRIBUTING.md Cargo.toml].each do |path|
      FileUtils.cp(File.join(source_root, path), File.join(fixture_root, path))
    end
    FileUtils.cp(
      File.join(source_root, '.github/workflows/ci.yml'),
      File.join(fixture_root, '.github/workflows/ci.yml')
    )
    yield fixture_root
  end
end

def mutate_section_command(document, heading, original, replacement)
  section_start = document.index("## #{heading}\n")
  fail "test fixture is missing #{heading.inspect}" unless section_start

  prefix = document[0...section_start]
  section = document[section_start..]
  changed = section.sub!(original, replacement)
  fail "test fixture is missing #{original.inspect}" unless changed

  prefix + section
end

def assert_rejected(label)
  yield
  raise "#{label} mutation unexpectedly passed"
rescue ContractError
  # Expected: this mutation must be rejected by the contract.
end

def self_test(source_root)
  validate(source_root)

  source_document = File.read(File.join(source_root, 'CONTRIBUTING.md'))
  required_commands = fenced_bash_commands(
    markdown_section(source_document, 'Required local checks'),
    'Required local checks'
  )
  required_test = required_commands.find { |command| command.start_with?('cargo test ') }
  required_clippy = required_commands.find { |command| command.start_with?('cargo clippy ') }
  required_fmt = required_commands.find { |command| command.start_with?('cargo fmt ') }
  fail 'test fixture is missing a required cargo test command' unless required_test
  fail 'test fixture is missing a required cargo clippy command' unless required_clippy
  fail 'test fixture is missing a required cargo fmt command' unless required_fmt

  rust_version = File.read(File.join(source_root, 'Cargo.toml'))[/^rust-version\s*=\s*"([^"]+)"\s*$/, 1]
  fail 'test fixture is missing Cargo.toml rust-version' unless rust_version
  required_msrv = "The minimum supported Rust version (MSRV) is Rust #{rust_version}."

  mutations = {
    'required all-features test' => [required_test, 'cargo test --locked'],
    'required clippy command' => [required_clippy, 'cargo clippy'],
    'required fmt command' => [required_fmt, 'cargo fmt'],
    'required MSRV statement' => [required_msrv, 'The minimum supported Rust version (MSRV) is Rust unknown.']
  }

  mutations.each do |label, (original, replacement)|
    with_fixture(source_root) do |fixture_root|
      path = File.join(fixture_root, 'CONTRIBUTING.md')
      document = File.read(path)
      if label == 'required MSRV statement'
        document.sub!(original, replacement) || raise("test fixture is missing #{original.inspect}")
      else
        document = mutate_section_command(document, 'Required local checks', original, replacement)
      end
      File.write(path, document)
      assert_rejected(label) { validate(fixture_root) }
    end
  end

  # The bootstrap section deliberately contains no cargo command, leaving
  # Required local checks as the single source for CI-equivalent commands.
  with_fixture(source_root) do |fixture_root|
    path = File.join(fixture_root, 'CONTRIBUTING.md')
    document = mutate_section_command(
      File.read(path),
      'Getting started',
      'cd ferromark',
      "cd ferromark\ncargo test --locked"
    )
    File.write(path, document)
    assert_rejected('bootstrap cargo command') { validate(fixture_root) }
  end
end

repository_root = File.expand_path('..', __dir__)

begin
  if ARGV == ['--self-test']
    self_test(repository_root)
  elsif ARGV.empty?
    validate(repository_root)
  else
    abort 'usage: test-contributing-ci-contract.rb [--self-test]'
  end
rescue ContractError => error
  abort error.message
end

puts 'CONTRIBUTING CI contract checks passed'
