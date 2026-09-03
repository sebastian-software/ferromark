#!/usr/bin/env ruby
# frozen_string_literal: true

class ContractError < StandardError; end

README_PATH = 'README.md'
MIGRATION_GUIDE = '[0.4–0.7 migration guide](docs/migration-0.4.md)'
REPOSITORY_ROOT = File.expand_path('..', __dir__)
CONTRIBUTING_PATH = File.join(REPOSITORY_ROOT, 'CONTRIBUTING.md')
BENCHMARK_LOCK_PATH = File.join(REPOSITORY_ROOT, 'benchmarks/md4c-comparison/Cargo.lock')
BENCHMARK_BUILD_PATH = File.join(REPOSITORY_ROOT, 'benchmarks/md4c-comparison/build.rs')
PERFORMANCE_PLAN_PATH = File.join(REPOSITORY_ROOT, 'docs/arch/ARCH-PLAN-001-performance-opportunities.md')
PROJECT_STRUCTURE_FILES = %w[highlight.rs events.rs strict.rs].freeze

def fail_contract(message)
  raise ContractError, "README structure contract: #{message}"
end

def section(document, heading)
  start = document.index("## #{heading}\n")
  fail_contract("must contain a #{heading.inspect} section") unless start

  content_start = start + heading.length + 4
  following_heading = document.index(/^## /, content_start)
  document[content_start...(following_heading || document.length)]
end

def locked_benchmark_version(package_name)
  lockfile = File.read(BENCHMARK_LOCK_PATH)
  match = lockfile.match(/^name = "#{Regexp.escape(package_name)}"\nversion = "([^"]+)"$/)
  fail_contract("benchmark lockfile must contain #{package_name}") unless match

  match[1]
end

def pinned_md4c_revision
  build_script = File.read(BENCHMARK_BUILD_PATH)
  revision = build_script[/^const MD4C_REVISION: &str = "([0-9a-f]{40})";$/, 1]
  fail_contract('benchmark build must declare a full MD4C_REVISION') unless revision

  revision
end

def validate(
  document,
  contributing: File.read(CONTRIBUTING_PATH),
  performance_plan: File.read(PERFORMANCE_PLAN_PATH)
)
  headings = document.scan(/^## (.+)$/).flatten
  fail_contract('must not repeat top-level headings') unless headings.uniq.length == headings.length

  headings.each do |heading|
    fail_contract("#{heading.inspect} must not be empty") if section(document, heading).strip.empty?
  end

  cli_start = document.index("## CLI\n")
  configuration_start = document.index("## Markdown configuration\n")
  unless cli_start && configuration_start && cli_start < configuration_start
    fail_contract('CLI must precede Markdown configuration so each section owns its content')
  end

  cli = section(document, 'CLI')
  configuration = section(document, 'Markdown configuration')
  fail_contract('CLI must document installation') unless cli.include?('cargo install ferromark')
  fail_contract('CLI must document trusted mode') unless cli.include?('--trusted')
  fail_contract('Markdown configuration must describe presets') unless configuration.include?('Options::minimal()')
  unless configuration.include?('`Options` is non-exhaustive')
    fail_contract('Markdown configuration must document Options construction')
  end
  fail_contract('README must preserve the migration guide link') unless document.include?(MIGRATION_GUIDE)

  benchmarks = section(document, 'Benchmarks')
  unless benchmarks.include?('These rankings are Apple Silicon results only')
    fail_contract('Benchmarks must scope published rankings to Apple Silicon')
  end
  unless benchmarks.include?('has not been re-measured') && benchmarks.include?('x86-64')
    fail_contract('Benchmarks must disclose the missing x86-64 comparison')
  end
  %w[pulldown-cmark comrak].each do |package_name|
    locked_version = locked_benchmark_version(package_name)
    unless benchmarks.match?(/#{Regexp.escape(package_name)}\s+#{Regexp.escape(locked_version)}/)
      fail_contract("Benchmarks must state locked #{package_name} #{locked_version}")
    end
  end

  md4c_revision = pinned_md4c_revision
  short_revision = md4c_revision[0, 7]
  unless benchmarks.include?("md4c @ #{short_revision}")
    fail_contract("Benchmarks must state pinned md4c revision #{short_revision}")
  end
  unless benchmarks.include?("checkout --detach #{short_revision}") &&
         contributing.include?("checkout --detach #{short_revision}")
    fail_contract("README and CONTRIBUTING must check out pinned md4c revision #{short_revision}")
  end
  [benchmarks, contributing].each do |benchmark_instructions|
    unless benchmark_instructions.include?('cargo bench --locked') &&
           benchmark_instructions.include?('--manifest-path benchmarks/md4c-comparison/Cargo.toml')
      fail_contract('README and CONTRIBUTING must use the locked isolated benchmark manifest')
    end
  end
  if performance_plan.include?('PERF_ATTEMPTS.md')
    fail_contract('Performance plan must not reference the removed PERF_ATTEMPTS.md')
  end
  comparison_commands = performance_plan.scan(/`([^`\n]*cargo bench[^`\n]*comparison[^`\n]*)`/).flatten
  fail_contract('Performance plan must document the comparison benchmark command') if comparison_commands.empty?
  unless comparison_commands.all? do |command|
           command.include?('MD4C_DIR=/path/to/md4c cargo bench --locked') &&
             command.include?('--manifest-path benchmarks/md4c-comparison/Cargo.toml')
         end
    fail_contract('Every performance-plan comparison command must use the locked isolated benchmark')
  end
  unless document.include?('baseline SSE2 (x86-64)')
    fail_contract('README must describe the x86-64 inline SIMD path')
  end

  project_structure = section(document, 'Project structure')
  PROJECT_STRUCTURE_FILES.each do |filename|
    unless project_structure.include?(filename)
      fail_contract("Project structure must include #{filename}")
    end
  end
end

def assert_rejected(label)
  yield
  raise "#{label} mutation unexpectedly passed"
rescue ContractError
  # Expected: the contract must reject this mutation.
end

def self_test(document)
  validate(document)

  assert_rejected('empty Markdown configuration') do
    validate(
      document.sub(
        "## Markdown configuration\n\nStart from",
        "## Markdown configuration\n\n## Configuration details\n\nStart from"
      )
    )
  end
  assert_rejected('misowned Markdown configuration content') do
    validate(document.sub("## Markdown configuration\n", ''))
  end
  assert_rejected('CLI trusted guidance') do
    validate(document.gsub('--trusted', '--safe'))
  end
  assert_rejected('migration guide link') do
    validate(document.sub(MIGRATION_GUIDE, 'migration guide'))
  end
  assert_rejected('Apple Silicon benchmark scope') do
    validate(document.sub('These rankings are Apple Silicon results only', 'These rankings apply everywhere'))
  end
  assert_rejected('x86-64 benchmark caveat') do
    validate(document.sub('has not been re-measured', 'has been re-measured'))
  end
  assert_rejected('locked comrak version') do
    locked_version = locked_benchmark_version('comrak')
    validate(document.sub(/comrak\s+#{Regexp.escape(locked_version)}/, 'comrak 0.0.0'))
  end
  assert_rejected('pinned md4c contributor checkout') do
    revision = pinned_md4c_revision[0, 7]
    contributing = File.read(CONTRIBUTING_PATH)
    validate(document, contributing: contributing.sub("checkout --detach #{revision}", 'checkout --detach main'))
  end
  assert_rejected('removed performance evidence file') do
    performance_plan = File.read(PERFORMANCE_PLAN_PATH)
    validate(document, performance_plan: "References PERF_ATTEMPTS.md\n#{performance_plan}")
  end
  assert_rejected('isolated performance benchmark command') do
    performance_plan = File.read(PERFORMANCE_PLAN_PATH)
    validate(
      document,
      performance_plan: performance_plan.gsub('MD4C_DIR=/path/to/md4c cargo bench --locked', 'cargo bench')
    )
  end
  PROJECT_STRUCTURE_FILES.each do |filename|
    assert_rejected("project structure #{filename}") do
      validate(document.sub(filename, 'omitted-source-file.rs'))
    end
  end
end

document = File.read(File.join(REPOSITORY_ROOT, README_PATH))

begin
  if ARGV == ['--self-test']
    self_test(document)
  elsif ARGV.empty?
    validate(document)
  else
    abort 'usage: test-readme-structure-contract.rb [--self-test]'
  end
rescue ContractError => error
  abort error.message
end

puts 'README structure contract checks passed'
