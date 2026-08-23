#!/usr/bin/env ruby
# frozen_string_literal: true

require 'fileutils'
require 'json'
require 'open3'
require 'rubygems/package'
require 'tmpdir'
require 'zlib'

REPOSITORY_ROOT = File.expand_path('..', __dir__)

class ContractError < StandardError; end

def fail_contract(message)
  raise ContractError, "docs.rs metadata contract: #{message}"
end

def cargo_metadata(manifest_path)
  output, status = Open3.capture2e(
    'cargo', 'metadata', '--locked', '--no-deps', '--format-version=1', '--manifest-path', manifest_path,
    chdir: REPOSITORY_ROOT
  )
  fail_contract("cargo metadata failed:\n#{output}") unless status.success?

  JSON.parse(output)
end

def package_for_manifest(metadata, manifest)
  canonical_manifest = File.realpath(manifest)
  package = metadata.fetch('packages').find do |candidate|
    File.realpath(candidate.fetch('manifest_path')) == canonical_manifest
  end
  fail_contract("cargo metadata did not return #{canonical_manifest}") unless package

  package
end

def docs_rs_features(manifest, document: File.read(manifest))
  header = '[package.metadata.docs.rs]'
  start = document.index("#{header}\n")
  fail_contract("#{manifest} must contain #{header}") unless start

  table = document[(start + header.length + 1)..]
  table = table.split(/^\[/, 2).fetch(0)
  match = table.match(/^features\s*=\s*\[(?<features>[^\]]*)\]\s*$/)
  fail_contract("#{manifest} docs.rs metadata must define features") unless match

  match[:features].scan(/"([^"]+)"/).flatten
end

def assert_docs_rs_features(metadata, manifest, document: File.read(manifest))
  features = docs_rs_features(manifest, document: document)
  expected = ['mdx']
  fail_contract("#{manifest} docs.rs features must be #{expected.inspect}, got #{features.inspect}") unless features == expected

  package_features = package_for_manifest(metadata, manifest).fetch('features')
  fail_contract("#{manifest} must declare the mdx feature") unless package_features.key?('mdx')
end

def package_crate(target_directory)
  output, status = Open3.capture2e(
    'cargo', 'package', '--allow-dirty', '--locked', '--offline', '--no-verify', '--target-dir', target_directory,
    chdir: REPOSITORY_ROOT
  )
  fail_contract("cargo package failed:\n#{output}") unless status.success?

  manifest = File.join(REPOSITORY_ROOT, 'Cargo.toml')
  package = package_for_manifest(cargo_metadata(manifest), manifest)
  crate = File.join(target_directory, 'package', "#{package.fetch('name')}-#{package.fetch('version')}.crate")
  fail_contract("cargo package did not create #{crate}") unless File.file?(crate)

  crate
end

def build_mdx_docs(target_directory)
  output, status = Open3.capture2e(
    { 'RUSTDOCFLAGS' => '-D warnings' },
    'cargo', 'doc', '--locked', '--no-deps', '--features', 'mdx', '--target-dir', target_directory,
    chdir: REPOSITORY_ROOT
  )
  fail_contract("MDX documentation build failed:\n#{output}") unless status.success?

  page = File.join(target_directory, 'doc', 'ferromark', 'mdx', 'index.html')
  fail_contract("MDX documentation build did not generate #{page}") unless File.file?(page)
end

def extract_crate(crate, destination)
  Zlib::GzipReader.open(crate) do |gzip|
    Gem::Package::TarReader.new(gzip) do |tar|
      tar.each do |entry|
        next unless entry.file?

        path = File.join(destination, entry.full_name)
        FileUtils.mkdir_p(File.dirname(path))
        File.binwrite(path, entry.read)
      end
    end
  end
end

def validate(repository_root)
  source_manifest = File.join(repository_root, 'Cargo.toml')
  assert_docs_rs_features(cargo_metadata(source_manifest), source_manifest)

  Dir.mktmpdir('ferromark-docs-rs-docs-target.') do |target_directory|
    build_mdx_docs(target_directory)
  end

  Dir.mktmpdir('ferromark-docs-rs-package.') do |destination|
    Dir.mktmpdir('ferromark-docs-rs-target.') do |target_directory|
      extract_crate(package_crate(target_directory), destination)
      manifests = Dir.glob(File.join(destination, '*', 'Cargo.toml'))
      fail_contract("expected one packaged Cargo.toml, got #{manifests.inspect}") unless manifests.length == 1
      packaged_manifest = manifests.fetch(0)
      assert_docs_rs_features(cargo_metadata(packaged_manifest), packaged_manifest)
    end
  end
end

def assert_rejected(label)
  yield
  raise "#{label} mutation unexpectedly passed"
rescue ContractError
  # Expected: this mutation must be rejected by the contract.
end

def self_test(repository_root)
  source_manifest = File.join(repository_root, 'Cargo.toml')
  source_document = File.read(source_manifest)
  metadata = cargo_metadata(source_manifest)
  metadata_without_mdx = JSON.parse(JSON.generate(metadata))
  package_for_manifest(metadata_without_mdx, source_manifest).fetch('features').delete('mdx')

  mutations = {
    'missing docs.rs table' => source_document.sub("[package.metadata.docs.rs]\nfeatures = [\"mdx\"]\n\n", ''),
    'broader docs.rs feature set' => source_document.sub('features = ["mdx"]', 'features = ["mdx", "profiling"]')
  }
  mutations.each do |label, document|
    assert_rejected(label) { assert_docs_rs_features(metadata, source_manifest, document: document) }
  end
  assert_rejected('undeclared mdx feature') { assert_docs_rs_features(metadata_without_mdx, source_manifest) }
end

begin
  if ARGV == ['--self-test']
    validate(REPOSITORY_ROOT)
    self_test(REPOSITORY_ROOT)
  elsif ARGV.empty?
    validate(REPOSITORY_ROOT)
  else
    abort 'usage: test-docs-rs-metadata.rb [--self-test]'
  end
rescue ContractError => error
  abort error.message
end

puts 'docs.rs metadata and packaged manifest checks passed'
