#!/usr/bin/env ruby
# frozen_string_literal: true

class ContractError < StandardError; end

README_PATH = 'README.md'
MIGRATION_GUIDE = '[0.4–0.7 migration guide](docs/migration-0.4.md)'

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

def validate(document)
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
  unless document.include?('baseline SSE2 (x86-64)')
    fail_contract('README must describe the x86-64 inline SIMD path')
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
end

document = File.read(File.expand_path("../#{README_PATH}", __dir__))

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
