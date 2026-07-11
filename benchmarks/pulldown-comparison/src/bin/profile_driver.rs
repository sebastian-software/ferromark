use std::{
    env,
    error::Error,
    fs,
    hint::black_box,
    path::PathBuf,
    process::ExitCode,
    str::FromStr,
    time::{Duration, Instant},
};

use ferromark_pulldown_comparison::{
    Corpus, CountingAllocator, MeasurementWindow, ParserKind, RunConfig, RunMetadata,
    RunMeasurement, render_ferromark_config_into, render_pulldown_config_into,
};

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

struct Arguments {
    parser: ParserKind,
    configuration: RunConfig,
    corpus: Corpus,
    limit: Limit,
    warmup_iterations: u64,
    json_path: Option<PathBuf>,
}

enum Limit {
    Iterations(u64),
    Duration(Duration),
}

struct Measurement {
    iterations: u64,
    elapsed: Duration,
    output_bytes: usize,
    output_capacity: usize,
    allocations: ferromark_pulldown_comparison::AllocationSnapshot,
    pipeline: Option<serde_json::Value>,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let Some(arguments) = parse_arguments()? else {
        return Ok(());
    };
    if !arguments.configuration.supports(arguments.parser) {
        return Err(format!(
            "configuration `{}` is not supported by `{}`",
            arguments.configuration, arguments.parser
        )
        .into());
    }

    let corpus = arguments.corpus.materialize();
    let measurement = match arguments.parser {
        ParserKind::Ferromark => measure_ferromark(&arguments, corpus.input()),
        ParserKind::PulldownCmark => measure_pulldown(&arguments, corpus.input())?,
    };
    let metadata = RunMetadata::new(
        arguments.parser,
        arguments.configuration,
        corpus.corpus(),
        RunMeasurement {
            input_bytes: corpus.input().len(),
            output_bytes: measurement.output_bytes,
            iterations: measurement.iterations,
            elapsed_ns: measurement.elapsed.as_nanos(),
            output_capacity: measurement.output_capacity,
            allocations: measurement.allocations,
            pipeline: measurement.pipeline,
        },
    );
    let json = serde_json::to_string_pretty(&metadata)?;
    if let Some(path) = arguments.json_path {
        fs::write(path, format!("{json}\n"))?;
    }
    println!("{json}");
    Ok(())
}

fn measure_ferromark(arguments: &Arguments, input: &str) -> Measurement {
    let mut output = Vec::with_capacity(input.len() + input.len() / 4);
    for _ in 0..arguments.warmup_iterations {
        render_ferromark_config_into(input, arguments.configuration, &mut output);
    }

    reset_pipeline();
    let window = MeasurementWindow::start();
    let start = Instant::now();
    let iterations = run_loop(&arguments.limit, || {
        render_ferromark_config_into(black_box(input), arguments.configuration, &mut output);
        black_box(output.len());
    });
    let elapsed = start.elapsed();
    let allocations = window.finish();
    let pipeline = pipeline_snapshot();
    Measurement {
        iterations,
        elapsed,
        output_bytes: output.len(),
        output_capacity: output.capacity(),
        allocations,
        pipeline,
    }
}

fn measure_pulldown(arguments: &Arguments, input: &str) -> Result<Measurement, Box<dyn Error>> {
    let mut output = String::with_capacity(input.len() + input.len() / 4);
    for _ in 0..arguments.warmup_iterations {
        render_pulldown_config_into(input, arguments.configuration, &mut output)?;
    }

    reset_pipeline();
    let window = MeasurementWindow::start();
    let start = Instant::now();
    let mut render_error = None;
    let iterations = run_loop(&arguments.limit, || {
        if let Err(error) =
            render_pulldown_config_into(black_box(input), arguments.configuration, &mut output)
        {
            render_error = Some(error);
        }
        black_box(output.len());
    });
    let elapsed = start.elapsed();
    let allocations = window.finish();
    if let Some(error) = render_error {
        return Err(error.into());
    }
    Ok(Measurement {
        iterations,
        elapsed,
        output_bytes: output.len(),
        output_capacity: output.capacity(),
        allocations,
        pipeline: None,
    })
}

#[cfg(feature = "profiling")]
fn reset_pipeline() {
    ferromark::profiling::reset();
}

#[cfg(not(feature = "profiling"))]
fn reset_pipeline() {}

#[cfg(feature = "profiling")]
fn pipeline_snapshot() -> Option<serde_json::Value> {
    let counters = ferromark::profiling::snapshot();
    Some(serde_json::json!({
        "documents": counters.documents,
        "block_events": counters.block_events,
        "block_text_events": counters.block_text_events,
        "block_container_events": counters.block_container_events,
        "block_table_events": counters.block_table_events,
        "block_code_events": counters.block_code_events,
        "max_block_event_capacity": counters.max_block_event_capacity,
        "inline_parses": counters.inline_parses,
        "inline_input_bytes": counters.inline_input_bytes,
        "inline_events": counters.inline_events,
        "inline_text_events": counters.inline_text_events,
        "inline_link_events": counters.inline_link_events,
        "inline_html_events": counters.inline_html_events,
        "inline_marks": counters.inline_marks,
        "inline_emit_points": counters.inline_emit_points,
        "inline_fast_paths": counters.inline_fast_paths,
        "max_inline_event_capacity": counters.max_inline_event_capacity,
        "max_mark_capacity": counters.max_mark_capacity,
        "max_emit_point_capacity": counters.max_emit_point_capacity,
        "paragraph_copied_bytes": counters.paragraph_copied_bytes,
    }))
}

#[cfg(not(feature = "profiling"))]
fn pipeline_snapshot() -> Option<serde_json::Value> {
    None
}

fn run_loop(limit: &Limit, mut iteration: impl FnMut()) -> u64 {
    match limit {
        Limit::Iterations(iterations) => {
            for _ in 0..*iterations {
                iteration();
            }
            *iterations
        }
        Limit::Duration(duration) => {
            let deadline = Instant::now() + *duration;
            let mut iterations = 0;
            while Instant::now() < deadline {
                iteration();
                iterations += 1;
            }
            iterations
        }
    }
}

fn parse_arguments() -> Result<Option<Arguments>, Box<dyn Error>> {
    let mut parser = ParserKind::Ferromark;
    let mut configuration = RunConfig::CommonMark;
    let mut corpus = Corpus::CommonMark50K;
    let mut limit = Limit::Duration(Duration::from_secs(30));
    let mut warmup_iterations = 10;
    let mut json_path = None;
    let mut arguments = env::args().skip(1);

    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--parser" => parser = ParserKind::from_str(&next_value(&mut arguments, "--parser")?)?,
            "--config" => {
                configuration = RunConfig::from_str(&next_value(&mut arguments, "--config")?)?;
            }
            "--corpus" => corpus = Corpus::from_str(&next_value(&mut arguments, "--corpus")?)?,
            "--iterations" => {
                limit = Limit::Iterations(next_value(&mut arguments, "--iterations")?.parse()?);
            }
            "--seconds" => {
                limit = Limit::Duration(Duration::from_secs(
                    next_value(&mut arguments, "--seconds")?.parse()?,
                ));
            }
            "--warmup-iterations" => {
                warmup_iterations = next_value(&mut arguments, "--warmup-iterations")?.parse()?;
            }
            "--json" => json_path = Some(PathBuf::from(next_value(&mut arguments, "--json")?)),
            "--list" => {
                print_selectors();
                return Ok(None);
            }
            "--help" | "-h" => {
                print_help();
                return Ok(None);
            }
            _ => return Err(format!("unknown argument `{argument}`").into()),
        }
    }

    Ok(Some(Arguments {
        parser,
        configuration,
        corpus,
        limit,
        warmup_iterations,
        json_path,
    }))
}

fn next_value(
    arguments: &mut impl Iterator<Item = String>,
    option: &str,
) -> Result<String, Box<dyn Error>> {
    arguments
        .next()
        .ok_or_else(|| format!("missing value for `{option}`").into())
}

fn print_selectors() {
    println!("parsers: ferromark, pulldown-cmark");
    println!(
        "configurations: {}",
        RunConfig::ALL.map(RunConfig::as_str).join(", ")
    );
    let corpus_names = Corpus::ALL
        .into_iter()
        .chain([Corpus::Extended])
        .map(Corpus::as_str)
        .collect::<Vec<_>>()
        .join(", ");
    println!("corpora: {corpus_names}");
}

fn print_help() {
    println!(
        "profile_driver [--parser NAME] [--config NAME] [--corpus NAME] \
         [--seconds N | --iterations N] [--warmup-iterations N] [--json PATH]"
    );
}
