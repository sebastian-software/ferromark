//! Command-line interface for rendering Markdown as HTML.

use std::error::Error;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use ferromark::{Options, RenderPolicy, to_html_with_options};

const USAGE: &str = "Usage: ferromark [OPTIONS] [INPUT]\n\nRender Markdown to HTML. INPUT is a file path or - for standard input.\n\nOptions:\n  -o, --output FILE       Write HTML to FILE\n      --minimal           Use the minimal Markdown preset\n      --commonmark        Use the CommonMark preset\n      --gfm               Use the GitHub Flavored Markdown preset\n      --trusted           Preserve raw HTML and unrestricted URL schemes\n      --front-matter      Strip leading front matter\n      --no-heading-ids    Do not generate heading ids\n  -h, --help              Print this help text\n  -V, --version           Print the version";

struct Cli {
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    options: Options,
    help: bool,
    version: bool,
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Cli, String> {
    let mut cli = Cli {
        input: None,
        output: None,
        options: Options::default(),
        help: false,
        version: false,
    };
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => cli.help = true,
            "-V" | "--version" => cli.version = true,
            "--minimal" => cli.options = Options::minimal(),
            "--commonmark" => cli.options = Options::commonmark(),
            "--gfm" => cli.options = Options::gfm(),
            "--trusted" => cli.options.render_policy = RenderPolicy::Trusted,
            "--front-matter" => cli.options.front_matter = true,
            "--no-heading-ids" => cli.options.heading_ids = false,
            "-o" | "--output" => {
                cli.output = Some(PathBuf::from(
                    args.next()
                        .ok_or_else(|| format!("{arg} requires a file path"))?,
                ))
            }
            value if value.starts_with('-') && value != "-" => {
                return Err(format!("unknown option: {value}"));
            }
            value => {
                if cli.input.replace(PathBuf::from(value)).is_some() {
                    return Err("only one input path is supported".into());
                }
            }
        }
    }
    Ok(cli)
}

fn run(cli: Cli) -> Result<(), Box<dyn Error>> {
    if cli.help {
        println!("{USAGE}");
        return Ok(());
    }
    if cli.version {
        println!("ferromark {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    let input = match cli.input.as_deref() {
        Some(path) if path != std::path::Path::new("-") => std::fs::read_to_string(path)?,
        _ => {
            let mut input = String::new();
            io::stdin().read_to_string(&mut input)?;
            input
        }
    };
    let html = to_html_with_options(&input, &cli.options);
    match cli.output {
        Some(path) => std::fs::write(path, html)?,
        None => io::stdout().write_all(html.as_bytes())?,
    }
    Ok(())
}

fn main() {
    match parse_args(std::env::args().skip(1))
        .and_then(|cli| run(cli).map_err(|error| error.to_string()))
    {
        Ok(()) => {}
        Err(error) => {
            eprintln!("ferromark: {error}\nTry 'ferromark --help' for usage.");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_options() {
        let cli = parse_args(["--gfm", "--trusted", "-o", "out", "in"].map(String::from)).unwrap();
        assert_eq!(cli.options.render_policy, RenderPolicy::Trusted);
        assert!(cli.options.tables);
        assert_eq!(cli.output.unwrap(), PathBuf::from("out"));
    }
}
