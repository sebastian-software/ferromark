#!/usr/bin/env ruby
# frozen_string_literal: true

require 'json'
require 'yaml'

ROOT = File.expand_path('..', __dir__)
class ContractError < StandardError; end

def validate(package, platforms, lockfile, config)
  pins = package.fetch('optionalDependencies')
  locked = lockfile.fetch('importers').fetch('ferromark').fetch('optionalDependencies')
  updaters = config.fetch('packages').fetch('.').fetch('extra-files')
  raise ContractError, 'native platform packages and optional dependencies differ' unless pins.keys.sort == platforms.keys.sort

  platforms.each do |name, platform|
    version = package.fetch('version')
    unless platform.fetch('version') == version && pins.fetch(name) == version
      raise ContractError, "#{name}: native package and dependency must match #{version}"
    end
    entry = locked.fetch(name)
    unless entry.fetch('specifier') == version
      raise ContractError, "#{name}: pnpm lockfile specifier must match #{version}"
    end
    unless entry.fetch('version') == "link:npm/#{name.delete_prefix('ferromark-')}"
      raise ContractError, "#{name}: lockfile must resolve to its local workspace package"
    end
    expected = {
      'type' => 'yaml',
      'path' => 'node/pnpm-lock.yaml',
      'jsonpath' => "$.importers.ferromark.optionalDependencies['#{name}'].specifier"
    }
    unless updaters.include?(expected)
      raise ContractError, "#{name}: release-please must update its pnpm lockfile specifier"
    end
  end
end

def assert_rejected(label, inputs)
  copy = Marshal.load(Marshal.dump(inputs))
  yield(*copy)
  validate(*copy)
  abort "#{label}: invalid release configuration was accepted"
rescue ContractError
  # Expected: detect version drift without running any installs or builds.
end

package = JSON.parse(File.read(File.join(ROOT, 'node/ferromark/package.json')))
platforms = Dir.glob(File.join(ROOT, 'node/ferromark/npm/*/package.json')).to_h do |path|
  platform = JSON.parse(File.read(path))
  [platform.fetch('name'), platform]
end
lockfile = YAML.safe_load(File.read(File.join(ROOT, 'node/pnpm-lock.yaml')), aliases: false)
config = JSON.parse(File.read(File.join(ROOT, 'release-please-config.json')))
inputs = [package, platforms, lockfile, config]

begin
  validate(*inputs)
  if ARGV == ['--self-test']
    name = platforms.keys.first
    assert_rejected('outdated lockfile', inputs) do |_, _, lock, _|
      lock['importers']['ferromark']['optionalDependencies'][name]['specifier'] = '0.0.0'
    end
    assert_rejected('mismatched native package', inputs) do |_, native, _, _|
      native[name]['version'] = '0.0.0'
    end
    assert_rejected('mismatched dependency pin', inputs) do |pkg, _, _, _|
      pkg['optionalDependencies'][name] = '0.0.0'
    end
    assert_rejected('registry resolution', inputs) do |_, _, lock, _|
      lock['importers']['ferromark']['optionalDependencies'][name]['version'] = '0.0.0'
    end
    assert_rejected('missing release updater', inputs) do |_, _, _, cfg|
      cfg['packages']['.']['extra-files'].reject! { |entry| entry.is_a?(Hash) && entry['type'] == 'yaml' }
    end
  elsif !ARGV.empty?
    abort 'usage: test-release-version-sync.rb [--self-test]'
  end
rescue ContractError => error
  abort "Release version sync: #{error.message}"
end

puts "release version sync checks passed (#{platforms.length} native packages)"
