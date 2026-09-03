#!/usr/bin/env ruby
# frozen_string_literal: true

require 'fileutils'
require 'open3'
require 'tmpdir'

class ContractError < StandardError; end

GUIDE_PATH = 'docs/migration-0.4.md'
EXAMPLES = %w[
  profile-extended
  inline-parser-argument
  options-clone
  fenced-code-pattern
  parse-result-headings
].freeze

def fail_contract(message)
  raise ContractError, "migration guide contract: #{message}"
end

def extract_example(document, name)
  marker = "<!-- migration-example: #{name} -->"
  start = document.index(marker)
  fail_contract("must contain #{marker}") unless start

  block = document[(start + marker.length)..]
  match = block.match(/\A[ \t\r\n]*```rust\r?\n(?<code>.*?)^```[ \t]*\r?$/m)
  fail_contract("#{name} must be followed by a Rust code block") unless match

  match[:code]
end

def assert_contains(document, needle, message)
  fail_contract(message) unless document.include?(needle)
end

def validate_document(document, readme, cargo_toml, node_package, node_workspace, changelog)
  assert_contains(document, '# Migrating from ferromark 0.3 to 0.7', 'must name its supported upgrade range')
  [
    '## Before you start',
    '## 0.4: replace `Profile`',
    '## 0.5: update configurable inline parsing',
    '## 0.6: clone options and make integration matches forward-compatible',
    '## 0.7: raise runtime prerequisites',
    '## Validate the completed upgrade'
  ].each { |heading| assert_contains(document, heading, "must contain #{heading}") }

  assert_contains(readme, "[0.4–0.7 migration guide](#{GUIDE_PATH})", 'README must link the 0.4–0.7 guide')
  assert_contains(document, 'Options::from(Profile)', 'must explain the removed Profile conversion')
  assert_contains(document, 'inline_footnotes: bool', 'must name the inserted inline-parser argument')
  assert_contains(document, 'InlineFootnote(Range)', 'must name the added inline event')
  assert_contains(document, 'Options` no longer implements `Copy`', 'must explain the Options Copy removal')
  assert_contains(document, '`#[non_exhaustive]`', 'must explain the fenced-code forward-compatibility contract')
  assert_contains(document, '`headings`', 'must explain the ParseResult metadata field')
  [
    'remove Profile and Options::from(Profile)',
    'a positional parse_with_options argument',
    'Options no longer implements Copy',
    'FencedCodeBlock gained the meta field and is now non_exhaustive',
    'ferromark now requires Rust 1.88 or newer',
    'ferromark npm package now requires Node.js 22 or newer'
  ].each { |change| assert_contains(changelog, change, "CHANGELOG must record #{change.inspect}") }

  rust_version = cargo_toml[/^rust-version\s*=\s*"([^"]+)"\s*$/, 1]
  fail_contract('Cargo.toml must declare rust-version') unless rust_version
  assert_contains(document, "requires Rust #{rust_version} or newer", 'must state Cargo.toml Rust requirement')

  node_version = node_package[/"node"\s*:\s*">=([0-9]+(?:\.[0-9]+){0,2})"/, 1]
  fail_contract('node package must declare a minimum Node version') unless node_version
  assert_contains(document, "Node.js #{node_version} or newer", 'must state npm package Node requirement')

  workspace_node = node_workspace[/"node"\s*:\s*">=([0-9]+(?:\.[0-9]+){0,2})"/, 1]
  workspace_pnpm = node_workspace[/"packageManager"\s*:\s*"pnpm@([^"]+)"/, 1]
  fail_contract('node workspace must declare a minimum Node version') unless workspace_node
  fail_contract('node workspace must pin pnpm') unless workspace_pnpm
  assert_contains(document, "pnpm #{workspace_pnpm}", 'must state the node workspace pnpm version')
  assert_contains(document, "Node.js #{workspace_node} or newer", 'must state the node workspace Node requirement')

  EXAMPLES.each { |name| extract_example(document, name) }
  assert_contains(
    extract_example(document, 'inline-parser-argument'),
    'false, // inline_footnotes',
    'inline parser example must preserve the new argument position'
  )
  assert_contains(
    extract_example(document, 'fenced-code-pattern'),
    '..',
    'fenced-code example must ignore future fields'
  )
end

def compile_examples(repository_root, document)
  examples = EXAMPLES.map { |name| [name, extract_example(document, name)] }

  Dir.mktmpdir('ferromark-migration-guide.') do |directory|
    File.write(File.join(directory, 'Cargo.toml'), <<~TOML)
      [package]
      name = "ferromark-migration-guide-contract"
      version = "0.0.0"
      edition = "2024"

      [dependencies]
      ferromark = { path = #{repository_root.inspect}, features = ["mdx"] }
    TOML
    FileUtils.mkdir_p(File.join(directory, 'src'))
    functions = examples.map do |name, code|
      "fn #{name.tr('-', '_')}() {\n#{code.lines.map { |line| "  #{line}" }.join}\n}\n"
    end.join("\n")
    File.write(File.join(directory, 'src/main.rs'), "#![allow(dead_code)]\n\n#{functions}\nfn main() {}\n")

    stdout, stderr, status = Open3.capture3(
      'cargo', 'check', '--quiet', '--manifest-path', File.join(directory, 'Cargo.toml')
    )
    next if status.success?

    fail_contract("documented Rust examples must compile:\n#{stdout}#{stderr}")
  end
end

def validate(repository_root)
  guide = File.read(File.join(repository_root, GUIDE_PATH))
  readme = File.read(File.join(repository_root, 'README.md'))
  cargo_toml = File.read(File.join(repository_root, 'Cargo.toml'))
  node_package = File.read(File.join(repository_root, 'node/ferromark/package.json'))
  node_workspace = File.read(File.join(repository_root, 'node/package.json'))
  changelog = File.read(File.join(repository_root, 'CHANGELOG.md'))
  validate_document(guide, readme, cargo_toml, node_package, node_workspace, changelog)
  compile_examples(repository_root, guide)
end

def assert_rejected(label)
  yield
  raise "#{label} mutation unexpectedly passed"
rescue ContractError
  # Expected: this mutation must be rejected by the contract.
end

def self_test(repository_root)
  validate(repository_root)

  guide = File.read(File.join(repository_root, GUIDE_PATH))
  readme = File.read(File.join(repository_root, 'README.md'))
  cargo_toml = File.read(File.join(repository_root, 'Cargo.toml'))
  node_package = File.read(File.join(repository_root, 'node/ferromark/package.json'))
  node_workspace = File.read(File.join(repository_root, 'node/package.json'))
  changelog = File.read(File.join(repository_root, 'CHANGELOG.md'))

  assert_rejected('README guide link') do
    validate_document(
      guide,
      readme.sub("[0.4–0.7 migration guide](#{GUIDE_PATH})", 'migration guide'),
      cargo_toml,
      node_package,
      node_workspace,
      changelog
    )
  end
  assert_rejected('inline parser argument') do
    validate_document(guide.sub('false, // inline_footnotes', 'None, // footnote_store'), readme, cargo_toml, node_package, node_workspace, changelog)
  end
  assert_rejected('fenced-code fallback pattern') do
    validate_document(guide.sub("            ..\n", ''), readme, cargo_toml, node_package, node_workspace, changelog)
  end
  assert_rejected('Rust version') do
    validate_document(guide.sub('requires Rust 1.88 or newer', 'requires Rust 1.85 or newer'), readme, cargo_toml, node_package, node_workspace, changelog)
  end
  assert_rejected('Node version') do
    validate_document(guide.sub('Node.js 22.12.0 or newer', 'Node.js 20.0.0 or newer'), readme, cargo_toml, node_package, node_workspace, changelog)
  end
end

repository_root = File.expand_path('..', __dir__)

begin
  if ARGV == ['--self-test']
    self_test(repository_root)
  elsif ARGV.empty?
    validate(repository_root)
  else
    abort 'usage: test-migration-guide-contract.rb [--self-test]'
  end
rescue ContractError => error
  abort error.message
end

puts 'Migration guide contract checks passed'
