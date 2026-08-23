#!/usr/bin/env ruby
# frozen_string_literal: true

TYPE_PATH = 'node/ferromark/index.d.mts'
README_PATH = 'node/ferromark/README.md'
RUST_PATH = 'src/lib.rs'
NATIVE_PATH = 'node/native/src/lib.rs'

FIELDS = {
  'renderPolicy' => ['render_policy', "`'untrusted'`"],
  'allowHtml' => ['allow_html', 'on'], 'allowLinkRefs' => ['allow_link_refs', 'on'],
  'tables' => ['tables', 'on'], 'mergedTableCells' => ['merged_table_cells', 'off'],
  'tableColumnWidths' => ['table_column_widths', 'off'], 'strikethrough' => ['strikethrough', 'on'],
  'highlight' => ['highlight', 'off'], 'superscript' => ['superscript', 'off'],
  'subscript' => ['subscript', 'off'], 'taskLists' => ['task_lists', 'on'],
  'autolinkLiterals' => ['autolink_literals', 'off'], 'disallowedRawHtml' => ['disallowed_raw_html', 'on'],
  'footnotes' => ['footnotes', 'off'], 'inlineFootnotes' => ['inline_footnotes', 'off'],
  'frontMatter' => ['front_matter', 'off'], 'headingIds' => ['heading_ids', 'on'],
  'math' => ['math', 'off'], 'callouts' => ['callouts', 'on'], 'definitionLists' => ['definition_lists', 'off'],
  'lineComments' => ['line_comments', 'off'], 'indentedCodeBlocks' => ['indented_code_blocks', 'on'],
  'linkBasePath' => ['link_base_path', 'unset']
}.freeze

def fail_contract(message)
  raise message
end

def validate(types, readme, rust, native)
  FIELDS.each do |js, (rs, default)|
    field = types[/\/\*\*(.*?)\*\/\s*#{Regexp.escape(js)}\?:/m, 1]
    fail_contract("#{js} needs TSDoc") unless field
    fail_contract("#{js} TSDoc must state default #{default}") unless field.include?("Default: #{default}")
    fail_contract("Rust Options missing #{rs}") unless rust.include?("pub #{rs}:")
    fail_contract("native mapping missing #{rs}") unless native.include?("pub #{rs}:") && native.include?("options.#{rs}")
  end
  fail_contract('README must link Options declaration') unless readme.include?('[`Options`](./index.d.mts)')
  fail_contract('README must state untrusted default') unless readme.include?("defaults to `'untrusted'")
  fail_contract('README must document table constraints') unless readme.include?('require `tables`')
end

types = File.read(TYPE_PATH); readme = File.read(README_PATH); rust = File.read(RUST_PATH); native = File.read(NATIVE_PATH)
validate(types, readme, rust, native)
if ARGV == ['--self-test']
  def rejected
    yield
    raise 'mutation unexpectedly passed'
  rescue RuntimeError
  end
  rejected do
    validate(types.sub('Default: on.', 'Default: off.'), readme, rust, native)
  end
  rejected do
    validate(types, readme.sub('[`Options`](./index.d.mts)', 'Options'), rust, native)
  end
elsif !ARGV.empty?
  fail_contract('usage: test-node-options-docs-contract.rb [--self-test]')
end
puts 'node options docs contract checks passed'
