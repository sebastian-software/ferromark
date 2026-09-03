#!/usr/bin/env ruby
# frozen_string_literal: true

TYPE_PATH = 'node/ferromark/index.d.mts'
README_PATH = 'node/ferromark/README.md'
RUST_PATH = 'src/lib.rs'
NATIVE_PATH = 'node/native/src/lib.rs'
JAVASCRIPT_PATH = 'node/ferromark/index.mjs'

class ContractFailure < StandardError; end

def fail_contract(message)
  raise ContractFailure, message
end

def source_block(source, pattern, description)
  block = source[pattern, 1]
  fail_contract("cannot read #{description}") unless block
  block
end

def rust_options(rust)
  struct_body = source_block(rust, /pub struct Options \{(.*?)^\}/m, 'Rust Options fields')
  default_body = rust[/impl Default for Options \{\s*fn default\(\) -> Self \{\s*Self \{(.*?)\n        \}\n    \}/m, 1]
  fail_contract('cannot read Rust Options defaults') unless default_body
  fields = struct_body.scan(/^    pub (\w+): ([^,]+),$/).to_h
  defaults = default_body.scan(/^            (\w+): (.+),$/).to_h
  fail_contract('Rust Options must declare at least one field') if fields.empty?
  fail_contract("Rust Options fields without defaults: #{(fields.keys - defaults.keys).join(', ')}") unless (fields.keys - defaults.keys).empty?
  fail_contract("Rust defaults without fields: #{(defaults.keys - fields.keys).join(', ')}") unless (defaults.keys - fields.keys).empty?
  fields.each_key { |field| fields[field] = defaults.fetch(field) }
end

def camel_case(field)
  field.split('_').each_with_index.map { |part, index| index.zero? ? part : part.capitalize }.join
end

def documented_default(value)
  case value
  when 'true' then 'on'
  when 'false' then 'off'
  when 'None' then 'unset'
  when 'RenderPolicy::Untrusted' then "`'untrusted'`"
  else fail_contract("unsupported Rust Options default #{value.inspect}")
  end
end

def node_option_docs(types)
  body = source_block(types, /export interface Options \{(.*?)^\}/m, 'Node Options declaration')
  body.scan(/\/\*\*(.*?)\*\/\s*(\w+)\?:/m).map { |documentation, field| [field, documentation] }.to_h
end

def native_option_fields(native)
  body = source_block(native, /pub struct Options \{(.*?)^\}/m, 'native Options fields')
  body.scan(/^    pub (\w+): /).to_h { |(field)| [field, true] }
end

def javascript_option_fields(javascript)
  body = source_block(
    javascript,
    /const optionKeys = new Set\(\[(.*?)^\]\)/m,
    'JavaScript option-key set'
  )
  body.scan(/^  '([^']+)',$/).flatten
end

def list_with_and(items)
  return items.first if items.length == 1
  return items.join(' and ') if items.length == 2

  "#{items[0...-1].join(', ')}, and #{items.last}"
end

def validate(types, readme, rust, native, javascript)
  rust_fields = rust_options(rust)
  node_docs = node_option_docs(types)
  native_fields = native_option_fields(native)
  javascript_fields = javascript_option_fields(javascript)
  expected_node_fields = rust_fields.keys.to_h { |field| [camel_case(field), field] }

  fail_contract("Node Options field mismatch: expected #{expected_node_fields.keys.join(', ')}") unless node_docs.keys.sort == expected_node_fields.keys.sort
  fail_contract("native Options field mismatch: expected #{rust_fields.keys.join(', ')}") unless native_fields.keys.sort == rust_fields.keys.sort
  fail_contract("JavaScript option-key mismatch: expected #{expected_node_fields.keys.join(', ')}") unless javascript_fields.sort == expected_node_fields.keys.sort

  rust_fields.each do |rust_field, default_value|
    node_field = camel_case(rust_field)
    documentation = node_docs.fetch(node_field)
    default = documented_default(default_value)
    fail_contract("#{node_field} TSDoc must state default #{default}") unless documentation.include?("Default: #{default}")
    fail_contract("native mapping missing #{rust_field}") unless native.include?("options.#{rust_field}")
  end

  fail_contract('README must link Options declaration') unless readme.include?('[`Options`](./index.d.mts)')
  enabled = rust_fields.select { |_field, default| default == 'true' }.keys.map { |field| "`#{camel_case(field)}`" }
  policy = rust_fields.select { |_field, default| default == 'RenderPolicy::Untrusted' }.keys.map { |field| "`#{camel_case(field)}`" }
  unset = rust_fields.select { |_field, default| default == 'None' }.keys.map { |field| "`#{camel_case(field)}`" }
  fail_contract('README summary requires one untrusted policy default') unless policy.length == 1
  fail_contract('README summary requires one unset option default') unless unset.length == 1
  expected_defaults = "Defaults on: #{list_with_and(enabled)}. All other boolean syntax extensions default off; #{policy.first} defaults to `'untrusted'` and #{unset.first} is unset."
  fail_contract('README must state source-derived defaults') unless readme.include?(expected_defaults)
  fail_contract('README must document table constraints') unless readme.include?('require `tables`')
end

types = File.read(TYPE_PATH); readme = File.read(README_PATH); rust = File.read(RUST_PATH); native = File.read(NATIVE_PATH); javascript = File.read(JAVASCRIPT_PATH)
validate(types, readme, rust, native, javascript)
if ARGV == ['--self-test']
  def rejected
    yield
  rescue ContractFailure
    return
  else
    fail_contract('mutation unexpectedly passed')
  end
  rejected do
    validate(types.sub('Default: on', 'Default: off'), readme, rust, native, javascript)
  end
  rejected do
    validate(types, readme.gsub('[`Options`](./index.d.mts)', 'Options'), rust, native, javascript)
  end
  future_field = rust.sub(
    '    pub link_base_path: Option<Box<str>>,',
    "    pub link_base_path: Option<Box<str>>,\n    pub future_option: bool,"
  ).sub(
    '            link_base_path: None,',
    "            link_base_path: None,\n            future_option: false,"
  )
  rejected do
    validate(types, readme, future_field, native, javascript)
  end
  future_with_docs = types.sub(
    "  /**\n   * Prefix internal absolute link destinations",
    "  /** Future option. Default: off. */\n  futureOption?: boolean\n  /**\n   * Prefix internal absolute link destinations"
  )
  rejected do
    validate(future_with_docs, readme, future_field, native, javascript)
  end
  rejected do
    validate(types, readme, rust, native, javascript.sub("  'linkBasePath',\n", ''))
  end
elsif !ARGV.empty?
  fail_contract('usage: test-node-options-docs-contract.rb [--self-test]')
end
puts 'node options docs contract checks passed'
