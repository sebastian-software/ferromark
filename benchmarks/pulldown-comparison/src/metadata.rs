use std::{env, process::Command};

use serde::Serialize;

use crate::{AllocationSnapshot, Corpus, ParserKind, RunConfig};

/// Build and machine metadata retained with every profiling result.
#[derive(Debug, Serialize)]
pub struct EnvironmentMetadata {
    /// Current git commit, if available.
    pub git_commit: String,
    /// Whether tracked or untracked files differ from the commit.
    pub git_dirty: bool,
    /// Complete `rustc -Vv` output.
    pub rustc: String,
    /// Rust compilation target architecture and operating system.
    pub target: String,
    /// Effective `RUSTFLAGS` visible to the runner.
    pub rustflags: String,
    /// Explicit CPU/compiler mode selected by the orchestration script.
    pub cpu_mode: String,
    /// Operating system description.
    pub operating_system: String,
}

impl EnvironmentMetadata {
    /// Collect environment metadata outside the measured execution window.
    pub fn collect() -> Self {
        Self {
            git_commit: command_output("git", &["rev-parse", "HEAD"]),
            git_dirty: !command_output("git", &["status", "--porcelain"]).is_empty(),
            rustc: command_output("rustc", &["-Vv"]),
            target: format!("{}-{}", env::consts::ARCH, env::consts::OS),
            rustflags: env::var("RUSTFLAGS").unwrap_or_default(),
            cpu_mode: env::var("FERROMARK_CPU_MODE").unwrap_or_else(|_| "unspecified".into()),
            operating_system: command_output("uname", &["-a"]),
        }
    }
}

/// Machine-readable summary for one diagnostic run.
#[derive(Debug, Serialize)]
pub struct RunMetadata {
    /// Build and machine context.
    pub environment: EnvironmentMetadata,
    /// Selected parser.
    pub parser: String,
    /// Selected feature and trust configuration.
    pub configuration: String,
    /// Stable corpus identifier.
    pub corpus: String,
    /// Markdown input size.
    pub input_bytes: usize,
    /// Produced output size from the last iteration.
    pub output_bytes: usize,
    /// Number of measured iterations.
    pub iterations: u64,
    /// Measured wall-clock duration in nanoseconds.
    pub elapsed_ns: u128,
    /// Retained reusable output capacity.
    pub output_capacity: usize,
    /// Allocation activity inside the measured render loop.
    pub allocations: AllocationSnapshot,
    /// Feature-gated Ferromark pipeline counters, when enabled.
    pub pipeline: Option<serde_json::Value>,
}

/// Values produced by the measured render loop.
pub struct RunMeasurement {
    /// Markdown input size.
    pub input_bytes: usize,
    /// Produced output size from the last iteration.
    pub output_bytes: usize,
    /// Number of measured iterations.
    pub iterations: u64,
    /// Measured wall-clock duration in nanoseconds.
    pub elapsed_ns: u128,
    /// Retained reusable output capacity.
    pub output_capacity: usize,
    /// Allocation activity inside the measured render loop.
    pub allocations: AllocationSnapshot,
    /// Feature-gated Ferromark pipeline counters, when enabled.
    pub pipeline: Option<serde_json::Value>,
}

impl RunMetadata {
    /// Build a summary after the measurement window has closed.
    pub fn new(
        parser: ParserKind,
        configuration: RunConfig,
        corpus: Corpus,
        measurement: RunMeasurement,
    ) -> Self {
        Self {
            environment: EnvironmentMetadata::collect(),
            parser: parser.to_string(),
            configuration: configuration.to_string(),
            corpus: corpus.to_string(),
            input_bytes: measurement.input_bytes,
            output_bytes: measurement.output_bytes,
            iterations: measurement.iterations,
            elapsed_ns: measurement.elapsed_ns,
            output_capacity: measurement.output_capacity,
            allocations: measurement.allocations,
            pipeline: measurement.pipeline,
        }
    }
}

fn command_output(program: &str, arguments: &[&str]) -> String {
    Command::new(program)
        .args(arguments)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_metadata_should_use_stable_selector_names() {
        let metadata = RunMetadata::new(
            ParserKind::Ferromark,
            RunConfig::CommonMark,
            Corpus::CommonMark5K,
            RunMeasurement {
                input_bytes: 10,
                output_bytes: 12,
                iterations: 2,
                elapsed_ns: 100,
                output_capacity: 32,
                allocations: AllocationSnapshot::default(),
                pipeline: None,
            },
        );

        assert_eq!(metadata.configuration, "commonmark");
    }
}
