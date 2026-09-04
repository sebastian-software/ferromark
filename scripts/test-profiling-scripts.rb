#!/usr/bin/env ruby
# frozen_string_literal: true

require 'fileutils'
require 'open3'
require 'tmpdir'
require 'timeout'

ROOT = File.expand_path('..', __dir__)
SIMPLE = File.read(File.join(__dir__, 'profile_simple.sh'))
COMMONMARK = File.read(File.join(__dir__, 'profile_commonmark50k.sh'))
SHARED = File.read(File.join(__dir__, 'profile_common.sh'))

def process_alive?(pid)
  Process.kill(0, Integer(pid))
  true
rescue Errno::ESRCH, Errno::EPERM
  false
end

def fail_contract(message)
  abort "Profiling script contract: #{message}"
end

def require_text(document, text)
  fail_contract("missing #{text.inspect}") unless document.include?(text)
end

require_text(SIMPLE, 'profile_build_harness')
require_text(SIMPLE, 'non-pgo')
require_text(COMMONMARK, 'profile_build_harness')
require_text(COMMONMARK, 'profile_build_comparison')
require_text(SHARED, '--message-format=json-render-diagnostics')
require_text(SHARED, 'json.loads')
require_text(SHARED, 'MD4C_DIR')
require_text(SHARED, 'trap cleanup_profile_child EXIT')
require_text(SHARED, "trap 'exit 130' INT")
require_text(SHARED, "trap 'exit 143' TERM")
require_text(SHARED, "trap 'exit 129' HUP")
require_text(SHARED, 'Failed to build ferromark profile_harness.')
require_text(SHARED, 'Failed to build the isolated cross-parser comparison bench.')
fail_contract('simple script still builds the removed root comparison bench') if SIMPLE.include?('cargo bench --bench comparison')
fail_contract('scripts must resolve their repository from their own location') unless SIMPLE.include?('BASH_SOURCE') && COMMONMARK.include?('BASH_SOURCE')

Dir.mktmpdir('ferromark-profiling-contract.') do |directory|
  fake_bin = File.join(directory, 'fake-profile-harness')
  fake_cargo = File.join(directory, 'cargo')
  fake_sample = File.join(directory, 'sample')
  cargo_cwd = File.join(directory, 'cargo.cwd')
  cargo_args = File.join(directory, 'cargo.args')

  File.write(
    fake_bin,
    <<~'SH'
      #!/bin/sh
      printf '%s\n' "$$" >"$FAKE_CHILD_PID"
      if [ "${FAKE_HARNESS_STATUS:-0}" -ne 0 ]; then
        exit "$FAKE_HARNESS_STATUS"
      fi
      exec sleep 1000
    SH
  )
  File.write(
    fake_cargo,
    <<~'SH'
      #!/bin/sh
      pwd >"$FAKE_CARGO_CWD"
      printf 'args=%s\n' "$*" >"$FAKE_CARGO_ARGS"
      printf 'target=%s\n' "$CARGO_TARGET_DIR" >>"$FAKE_CARGO_ARGS"
      printf 'encoded=%s\n' "$CARGO_ENCODED_RUSTFLAGS" >>"$FAKE_CARGO_ARGS"
      printf 'rustflags=%s\n' "$RUSTFLAGS" >>"$FAKE_CARGO_ARGS"
      printf 'md4c=%s\n' "$MD4C_DIR" >>"$FAKE_CARGO_ARGS"
      if [ "${FAKE_CARGO_STATUS:-0}" -ne 0 ]; then
        echo 'fake cargo diagnostic' >&2
        exit "$FAKE_CARGO_STATUS"
      fi
      printf '{"reason":"compiler-artifact","target":{"name":"%s","kind":["%s"]},"executable":"%s"}\n' "$FAKE_PROFILE_TARGET" "$FAKE_PROFILE_KIND" "$FAKE_PROFILE_BIN"
      printf '{"reason":"compiler-artifact","target":{"name":"unrelated","kind":["example"]},"executable":"/bin/false"}\n'
    SH
  )
  File.write(
    fake_sample,
    <<~'SH'
      #!/bin/sh
      for arg; do output="$arg"; done
      : >"$output"
      if [ "${FAKE_SAMPLE_SLEEP:-0}" -ne 0 ]; then
        printf '%s\n' "$$" >"$FAKE_SAMPLE_PID"
        exec sleep "$FAKE_SAMPLE_SLEEP"
      fi
      if [ "${FAKE_SAMPLE_STATUS:-0}" -ne 0 ]; then
        exit "$FAKE_SAMPLE_STATUS"
      fi
      exit 0
    SH
  )
  [fake_bin, fake_cargo, fake_sample].each { |path| FileUtils.chmod('+x', path) }

  env = ENV.to_h.merge(
    'PATH' => "#{directory}:#{ENV.fetch('PATH')}",
    'FAKE_PROFILE_BIN' => fake_bin,
    'FAKE_PROFILE_TARGET' => 'profile_harness',
    'FAKE_PROFILE_KIND' => 'example',
    'FAKE_CARGO_CWD' => cargo_cwd,
    'FAKE_CARGO_ARGS' => cargo_args,
    'FAKE_CHILD_PID' => File.join(directory, 'child.pid'),
    'FAKE_SAMPLE_PID' => File.join(directory, 'sample.pid'),
    'PROFILE_OUTPUT_DIR' => directory,
    'CARGO_TARGET_DIR' => 'relative-target'
  )

  output, error, status = Open3.capture3(
    env,
    File.join(__dir__, 'profile_simple.sh'),
    'non-pgo', '0.1', '1',
    chdir: directory
  )
  fail_contract("ferromark-only smoke failed: #{output}\n#{error}") unless status.success?
  expected_cwd = ROOT
  fail_contract('build did not run from the repository root') unless File.read(cargo_cwd).chomp == expected_cwd
  expected_target_dir = File.join(File.realpath(directory), 'relative-target')
  fail_contract('custom CARGO_TARGET_DIR was not forwarded') unless File.read(cargo_args).include?(expected_target_dir)
  fail_contract('sample output was not created') unless File.file?(File.join(directory, 'ferromark-simple.sample.txt'))
  cargo_log = File.read(cargo_args)
  fail_contract('non-PGO mode did not clear Cargo encoded flags') unless cargo_log.include?("encoded=\n")
  fail_contract('non-PGO mode did not clear RUSTFLAGS') unless cargo_log.include?("rustflags=\n")

  output, error, status = Open3.capture3(
    env.merge('FAKE_CARGO_STATUS' => '42'),
    File.join(__dir__, 'profile_simple.sh'),
    'non-pgo', '0.1', '1',
    chdir: directory
  )
  fail_contract('cargo build failure was swallowed') if status.success?
  require_text(error, 'Failed to build ferromark profile_harness')
  require_text(error, 'fake cargo diagnostic')

  output, error, status = Open3.capture3(
    env.merge('FAKE_SAMPLE_STATUS' => '7'),
    File.join(__dir__, 'profile_simple.sh'),
    'non-pgo', '0.1', '1',
    chdir: directory
  )
  fail_contract('sample failure was swallowed') if status.success?
  require_text(error, 'sample failed for PID')
  fail_contract('sample failure left the benchmark child running') if process_alive?(File.read(File.join(directory, 'child.pid')))

  output, error, status = Open3.capture3(
    env.merge('FAKE_HARNESS_STATUS' => '23'),
    File.join(__dir__, 'profile_simple.sh'),
    'non-pgo', '0.1', '1',
    chdir: directory
  )
  fail_contract('early benchmark exit was not rejected') if status.success?
  require_text(error, 'Profiling child exited before sampling')
  fail_contract('early benchmark exit left the child running') if process_alive?(File.read(File.join(directory, 'child.pid')))

  signal_env = env.merge('FAKE_SAMPLE_SLEEP' => '1000')
  stdin, stdout, stderr, wait_thread = Open3.popen3(
    signal_env,
    File.join(__dir__, 'profile_simple.sh'),
    'non-pgo', '10', '20',
    chdir: directory
  )
  stdin.close
  begin
    Timeout.timeout(5) do
      sleep 0.05 until File.file?(File.join(directory, 'sample.pid'))
    end
    Process.kill('TERM', wait_thread.pid)
    status = Timeout.timeout(5) { wait_thread.value }
    fail_contract('SIGTERM was incorrectly reported as success') if status.success?
  rescue Timeout::Error
    Process.kill('KILL', wait_thread.pid) rescue Errno::ESRCH
    wait_thread.value
    fail_contract('SIGTERM did not stop the profiling script')
  ensure
    stdout.close
    stderr.close
  end
  begin
    Timeout.timeout(5) do
      while process_alive?(File.read(File.join(directory, 'child.pid'))) || process_alive?(File.read(File.join(directory, 'sample.pid')))
        sleep 0.05
      end
    end
  rescue Timeout::Error
    fail_contract('SIGTERM left a benchmark or sample child running')
  end

  output, error, status = Open3.capture3(
    env,
    File.join(__dir__, 'profile_simple.sh'),
    'non-pgo', '0', '1',
    chdir: directory
  )
  fail_contract('zero sample duration was not rejected') if status.success?
  require_text(error, 'sample_seconds must be a positive number')

  output, error, status = Open3.capture3(
    env,
    File.join(__dir__, 'profile_simple.sh'),
    'non-pgo', '2', '1',
    chdir: directory
  )
  fail_contract('sample budget overflow was not rejected') if status.success?
  require_text(error, 'must not exceed measurement_seconds')

  profdata = File.join(directory, 'profile data.profdata')
  File.write(profdata, 'profile')
  output, error, status = Open3.capture3(
    env.merge('PGO_PROFDATA' => 'profile data.profdata'),
    File.join(__dir__, 'profile_simple.sh'),
    'pgo', '0.1', '1',
    chdir: directory
  )
  fail_contract("PGO smoke failed: #{output}\n#{error}") unless status.success?
  cargo_log = File.read(cargo_args)
  encoded_flags = cargo_log.lines.find { |line| line.start_with?('encoded=') }.to_s.chomp.delete_prefix('encoded=')
  expected_profile_flag = "-Cprofile-use=#{File.realpath(profdata)}"
  fail_contract('PGO flags were not encoded for Cargo') unless encoded_flags.include?(expected_profile_flag) && encoded_flags.include?("\u001f")
  fail_contract('PGO mode did not clear the legacy RUSTFLAGS channel') unless cargo_log.include?("rustflags=\n")

  output, error, status = Open3.capture3(
    env.reject { |key, _| key == 'PGO_PROFDATA' },
    File.join(__dir__, 'profile_simple.sh'),
    'pgo', '0.1', '1',
    chdir: directory
  )
  fail_contract('missing PGO data was not rejected') if status.success?
  require_text(error, 'PGO mode requires PGO_PROFDATA')

  output, error, status = Open3.capture3(
    env.reject { |key, _| key == 'MD4C_DIR' },
    File.join(__dir__, 'profile_commonmark50k.sh'),
    '5k', 'pulldown-cmark', '0.1', '1', 'non-pgo',
    chdir: directory
  )
  fail_contract('cross-parser profiling without MD4C_DIR was not rejected') if status.success?
  require_text(error, 'Cross-parser profiling requires MD4C_DIR')

  output, error, status = Open3.capture3(
    env.merge(
      'MD4C_DIR' => '.',
      'FAKE_PROFILE_TARGET' => 'comparison',
      'FAKE_PROFILE_KIND' => 'bench'
    ),
    File.join(__dir__, 'profile_commonmark50k.sh'),
    '5k', 'pulldown-cmark', '0.1', '1', 'non-pgo',
    chdir: directory
  )
  fail_contract("cross-parser smoke failed: #{output}\n#{error}") unless status.success?
  cargo_log = File.read(cargo_args)
  require_text(cargo_log, 'benchmarks/md4c-comparison/Cargo.toml')
  fail_contract('relative MD4C_DIR was not canonicalized before changing cwd') unless cargo_log.include?("md4c=#{File.realpath(directory)}\n")
  fail_contract('cross-parser smoke did not create sample output') unless File.file?(File.join(directory, 'ferromark-commonmark5k-pulldown-cmark-non-pgo.sample.txt'))
end

puts 'profiling script contract checks passed'
