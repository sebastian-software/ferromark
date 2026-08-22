#!/usr/bin/env ruby
# frozen_string_literal: true

require "find"
require "psych"

def mapping_pairs(mapping)
  mapping.children.each_slice(2)
end

def scalar_key?(node, name)
  node.is_a?(Psych::Nodes::Scalar) && node.value == name
end

def add_uses_entry(key, value, path, entries)
  entries << [path, key.start_line + 1, value.is_a?(Psych::Nodes::Scalar) ? value.value : nil]
end

def scan_step(step, path, entries)
  return unless step.is_a?(Psych::Nodes::Mapping)

  mapping_pairs(step).each do |key, value|
    add_uses_entry(key, value, path, entries) if scalar_key?(key, "uses")
  end
end

def scan_job(job, path, entries)
  return unless job.is_a?(Psych::Nodes::Mapping)

  mapping_pairs(job).each do |key, value|
    if scalar_key?(key, "uses")
      add_uses_entry(key, value, path, entries)
    elsif scalar_key?(key, "steps") && value.is_a?(Psych::Nodes::Sequence)
      value.children.each { |step| scan_step(step, path, entries) }
    end
  end
end

def scan_document(document, path, entries)
  root = document.children.first
  return unless root.is_a?(Psych::Nodes::Mapping)

  mapping_pairs(root).each do |key, value|
    next unless scalar_key?(key, "jobs") && value.is_a?(Psych::Nodes::Mapping)

    mapping_pairs(value).each { |_job_name, job| scan_job(job, path, entries) }
  end
end

def scan_file(path, entries)
  stream = Psych.parse_stream(File.read(path), filename: path)
  stream.children.each { |document| scan_document(document, path, entries) }
rescue Psych::Exception => error
  warn "Failed to parse workflow file #{path}: #{error.message}"
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

invalid_entries = entries.reject do |_path, _line, value|
  value&.start_with?("./", "docker://") || value&.match?(/\A[^[:space:]@]+@[0-9a-fA-F]{40}\z/)
end

unless invalid_entries.empty?
  warn "Workflow actions must use full 40-character commit SHAs:"
  invalid_entries.each do |path, line, value|
    warn "#{path}:#{line}: uses: #{value}"
  end
  exit 1
end
