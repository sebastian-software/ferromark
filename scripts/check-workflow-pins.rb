#!/usr/bin/env ruby
# frozen_string_literal: true

require "find"
require "psych"

class WorkflowScanError < StandardError; end

def ensure_acyclic!(node, ancestors = {})
  return unless node.is_a?(Hash) || node.is_a?(Array)

  object_id = node.object_id
  raise WorkflowScanError, "cyclic YAML aliases are not supported" if ancestors[object_id]

  ancestors[object_id] = true
  if node.is_a?(Hash)
    node.each { |key, value| ensure_acyclic!(key, ancestors); ensure_acyclic!(value, ancestors) }
  else
    node.each { |value| ensure_acyclic!(value, ancestors) }
  end
ensure
  ancestors.delete(object_id) if object_id
end

def add_uses_entry(value, path, entries)
  entries << [path, value.is_a?(String) ? value : nil]
end

def scan_step(step, path, entries)
  return unless step.is_a?(Hash)

  add_uses_entry(step["uses"], path, entries) if step.key?("uses")
end

def scan_job(job, path, entries)
  return unless job.is_a?(Hash)

  add_uses_entry(job["uses"], path, entries) if job.key?("uses")
  job["steps"].each { |step| scan_step(step, path, entries) } if job["steps"].is_a?(Array)
end

def scan_document(document, path, entries)
  return unless document.is_a?(Hash) && document["jobs"].is_a?(Hash)

  document["jobs"].each_value { |job| scan_job(job, path, entries) }
end

def scan_file(path, entries)
  source = File.read(path)
  stream = Psych.parse_stream(source, filename: path)
  raise WorkflowScanError, "multiple YAML documents are not supported" unless stream.children.length == 1

  document = Psych.safe_load(source, filename: path, aliases: true)
  ensure_acyclic!(document)
  scan_document(document, path, entries)
rescue Psych::Exception => error
  warn "Failed to parse workflow file #{path}: #{error.message}"
  exit 2
rescue WorkflowScanError, SystemStackError => error
  warn "Failed to scan workflow file #{path}: #{error.message}"
  exit 2
rescue SystemCallError => error
  warn "Failed to read workflow file #{path}: #{error.message}"
  exit 2
end

workflow_directory = ARGV.fetch(0)
entries = []

begin
  Find.find(workflow_directory) do |path|
    next unless File.file?(path) && path.match?(/\.ya?ml\z/)

    scan_file(path, entries)
  end
rescue SystemCallError => error
  warn "Failed to scan workflow files: #{error.message}"
  exit 2
end

invalid_entries = entries.reject do |_path, value|
  value&.start_with?("./", "docker://") ||
    value&.match?(/\A\$\/[^[:space:]@]+\z/) ||
    value&.match?(/\A[^[:space:]@]+@[0-9a-fA-F]{40}\z/)
end

unless invalid_entries.empty?
  warn "Workflow actions must use full 40-character commit SHAs:"
  invalid_entries.each do |path, value|
    warn "#{path}: uses: #{value || "uses value is not a scalar"}"
  end
  exit 1
end
