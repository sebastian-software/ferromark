use std::hint::black_box;
use std::io::{self, BufRead, Write};
#[cfg(not(feature = "heap"))]
use std::time::{Duration, Instant};

use serde::Deserialize;
use serde_json::{Value, json};

mod adapters;

#[cfg(feature = "heap")]
mod heap;
#[cfg(feature = "heap")]
#[global_allocator]
static GLOBAL: heap::Counting = heap::Counting;

#[derive(Deserialize)]
struct Document {
    id: String,
    input: String,
}

#[derive(Deserialize)]
struct Corpus {
    previews: Vec<Document>,
    guides: Vec<Document>,
    documentation: Vec<Document>,
}

struct Session {
    variant: String,
    ferro: ferromark::Options,
    comrak: comrak::Options<'static>,
    pulldown: pulldown_cmark::Options,
    renderer: Option<ferromark::Renderer>,
}

impl Session {
    fn new(variant: &str) -> Self {
        assert!(matches!(
            variant,
            "preview-fresh"
                | "preview-reuse"
                | "preview-pulldown"
                | "preview-comrak"
                | "guide-metadata"
                | "guide-pulldown"
                | "guide-comrak"
                | "ferromark-stream"
                | "ferromark-retain"
                | "pulldown-stream"
                | "pulldown-retain"
                | "comrak-stream"
                | "comrak-retain"
        ));
        let mut ferro = ferromark::Options::commonmark();
        ferro.render_policy = ferromark::RenderPolicy::Trusted;
        ferro.tables = true;
        ferro.strikethrough = true;
        ferro.task_lists = true;
        let mut comrak = comrak::Options::default();
        comrak.extension.table = true;
        comrak.extension.strikethrough = true;
        comrak.extension.tasklist = true;
        comrak.render.r#unsafe = true;
        let mut pulldown = pulldown_cmark::Options::ENABLE_TABLES
            | pulldown_cmark::Options::ENABLE_STRIKETHROUGH
            | pulldown_cmark::Options::ENABLE_TASKLISTS;
        if variant.starts_with("preview-") || variant.starts_with("guide-") {
            // Application adapters enforce the explicit URL allowlist before
            // rendering. Escape raw HTML while letting that policy decide URLs.
            comrak.render.escape = true;
            comrak.extension.alerts = true;
            pulldown |= pulldown_cmark::Options::ENABLE_GFM;
        }
        Self {
            variant: variant.to_owned(),
            ferro,
            comrak,
            pulldown,
            renderer: (variant == "preview-reuse").then(ferromark::Renderer::new),
        }
    }

    fn documents<'a>(&self, corpus: &'a Corpus) -> &'a [Document] {
        if self.variant.starts_with("preview-") {
            &corpus.previews
        } else if self.variant.starts_with("guide-") {
            &corpus.guides
        } else {
            &corpus.documentation
        }
    }

    fn html(&mut self, input: &str) -> String {
        let input = black_box(input);
        if self.variant == "preview-fresh" {
            ferromark::to_html(input)
        } else if self.variant == "preview-reuse" {
            self.renderer.as_mut().unwrap().render(input)
        } else if self.variant == "preview-pulldown" {
            adapters::pulldown(input, self.pulldown, false).html
        } else if self.variant == "preview-comrak" {
            adapters::comrak(input, &self.comrak, false).html
        } else if self.variant.starts_with("ferromark-") {
            ferromark::to_html_with_options(input, &self.ferro)
        } else if self.variant.starts_with("pulldown-") {
            let mut out = String::new();
            pulldown_cmark::html::push_html(
                &mut out,
                pulldown_cmark::Parser::new_ext(input, self.pulldown),
            );
            out
        } else {
            comrak::markdown_to_html(input, &self.comrak)
        }
    }

    fn metadata<'a>(&self, input: &'a str) -> ferromark::ParseResult<'a> {
        match self.variant.as_str() {
            "guide-metadata" => ferromark::parse(input),
            "guide-pulldown" => adapters::pulldown(input, self.pulldown, true),
            "guide-comrak" => adapters::comrak(input, &self.comrak, true),
            _ => panic!("not a metadata variant"),
        }
    }

    fn run(&mut self, corpus: &Corpus) -> usize {
        let retain = self.variant.ends_with("-retain");
        let mut kept = Vec::new();
        let mut bytes = 0;
        for document in self.documents(corpus) {
            if self.variant.starts_with("guide-") {
                let result = self.metadata(black_box(&document.input));
                bytes += result.html.len();
                black_box(&result);
                drop(result);
            } else {
                let html = black_box(self.html(&document.input));
                bytes += html.len();
                if retain {
                    kept.push(html);
                }
            }
        }
        black_box(&kept);
        drop(kept);
        black_box(bytes)
    }

    fn verify(&mut self, corpus: &Corpus) -> Value {
        let mut outputs = Vec::new();
        for document in self.documents(corpus) {
            let (html, metadata) = if self.variant.starts_with("guide-") {
                let result = self.metadata(&document.input);
                assert!(result.resource_limits.is_empty(), "{}", document.id);
                assert!(result.front_matter.is_some());
                assert!(!result.headings.is_empty());
                let headings: Vec<_> = result
                    .headings
                    .iter()
                    .map(|h| json!({"level": h.level, "id": h.id, "text": h.text}))
                    .collect();
                (
                    result.html,
                    json!({"front_matter": result.front_matter, "headings": headings}),
                )
            } else {
                // Check the production resource report outside the measured HTML API.
                if self.variant.starts_with("ferromark-")
                    || matches!(self.variant.as_str(), "preview-fresh" | "preview-reuse")
                {
                    let options = if self.variant.starts_with("preview-") {
                        ferromark::Options::default()
                    } else {
                        self.ferro.clone()
                    };
                    assert!(
                        ferromark::parse_with_options(&document.input, &options)
                            .resource_limits
                            .is_empty(),
                        "{}",
                        document.id
                    );
                }
                (self.html(&document.input), Value::Null)
            };
            outputs.push(json!({"id": document.id, "html": html, "metadata": metadata}));
        }
        let mut default_options = ferromark::Options::default();
        default_options.front_matter = self.variant.starts_with("guide-");
        let adapter = matches!(
            self.variant.as_str(),
            "preview-pulldown" | "preview-comrak" | "guide-pulldown" | "guide-comrak"
        );
        json!({"variant": self.variant, "outputs": outputs,
        "effective_options": {
            "ferromark": format!("{:?}", if self.variant.starts_with("preview-") || self.variant.starts_with("guide-") { default_options } else { self.ferro.clone() }),
            "metadata_front_matter": self.variant.starts_with("guide-"),
            "application_adapter": adapter.then(|| json!({
                "url_policy": "relative URLs or http/https/mailto/ftp/geo/irc/ircs/matrix/sms/tel/xmpp; all raw HTML escaped",
                "heading_slugs": "Comrak Anchorizer via public events/heading hook; generated IDs and metadata must match corpus reference",
                "callouts": true,
            })),
            "comrak": format!("{:?}", self.comrak),
            "pulldown": format!("{:?}", self.pulldown),
            "owned_html": true,
            "retain_all_outputs": self.variant.ends_with("-retain"),
        }})
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    #[cfg(feature = "heap")]
    if args.get(1).map(String::as_str) == Some("--self-test") {
        heap::self_test();
        println!("allocator accounting self-test passed");
        return;
    }
    let corpus: Corpus = serde_json::from_slice(&std::fs::read(&args[1]).unwrap()).unwrap();
    let variant = &args[2];
    let mut session = Session::new(variant);
    let mut out = io::BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        let request: Value = serde_json::from_str(&line.unwrap()).unwrap();
        let result = match request["action"].as_str().unwrap() {
            "verify" => session.verify(&corpus),
            #[cfg(not(feature = "heap"))]
            "time" => {
                let duration = Duration::from_millis(request["milliseconds"].as_u64().unwrap());
                let start = Instant::now();
                let mut iterations = 0;
                let mut output_bytes = 0;
                loop {
                    for _ in 0..4 {
                        output_bytes += session.run(&corpus) as u64;
                        iterations += 1;
                    }
                    if start.elapsed() >= duration {
                        break;
                    }
                }
                let elapsed_ns = start.elapsed().as_nanos() as u64;
                json!({"elapsed_ns": elapsed_ns, "iterations": iterations,
                    "output_bytes": output_bytes, "ns_per_workload": elapsed_ns as f64 / iterations as f64})
            }
            #[cfg(feature = "heap")]
            "memory" => {
                // A fresh session for each observation, infrastructure and input outside scope.
                let base = heap::begin();
                let mut measured = Session::new(variant);
                let cold_bytes = measured.run(&corpus);
                let cold = heap::end(base);
                heap::begin_at(base);
                let output_bytes = measured.run(&corpus);
                let warm = heap::end(base);
                assert_eq!(cold_bytes, output_bytes);
                drop(measured);
                assert_eq!(heap::remaining(base), 0, "session leaked allocations");
                json!({"cold": cold, "warm": warm, "output_bytes": output_bytes, "live_after_drop": 0})
            }
            _ => panic!("unknown action"),
        };
        writeln!(out, "{result}").unwrap();
        out.flush().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn previews_escape_user_html_and_enforce_the_same_link_and_image_policy() {
        let input = "<div onclick=\"bad()\">visible source</div>\n\nInline <script>bad()</script>.\n\n[x](javas&#99;ript:bad) ![alt](data:text/html,bad) [custom](custom:bad)\n\n[docs](https://example.org) [phone](tel:123) [relative](/docs)\n\n> [!NOTE]\n> Keep this callout.\n";
        for variant in [
            "preview-fresh",
            "preview-reuse",
            "preview-pulldown",
            "preview-comrak",
        ] {
            let html = Session::new(variant).html(input);
            for forbidden in [
                "<div onclick=",
                "<script>",
                "href=\"javascript:",
                "src=\"data:",
                "href=\"custom:",
            ] {
                assert!(!html.contains(forbidden), "{variant}: {html}");
            }
            for required in [
                "&lt;div",
                "visible source",
                "&lt;script&gt;",
                "href=\"https://example.org\"",
                "href=\"tel:123\"",
                "href=\"/docs\"",
                "markdown-alert-note",
                "Keep this callout.",
                "alt=\"alt\"",
            ] {
                assert!(html.contains(required), "{variant}: {html}");
            }
        }
    }

    #[test]
    fn metadata_includes_raw_front_matter_plain_heading_text_and_working_duplicate_ids() {
        let input = "---\ntitle: Example\n---\n# Title **bold**\n\n## Repeat\n\n## Repeat\n\n```md\n# Not a heading\n```\n";
        let reference = ferromark::parse(input);
        assert_eq!(reference.headings.len(), 3);
        for variant in ["guide-metadata", "guide-pulldown", "guide-comrak"] {
            let result = Session::new(variant).metadata(input);
            assert_eq!(result.front_matter, Some("title: Example\n"), "{variant}");
            assert_eq!(result.headings, reference.headings, "{variant}");
            for heading in result.headings {
                let id = heading.id.unwrap();
                assert!(
                    result
                        .html
                        .contains(&format!("<h{} id=\"{id}\">", heading.level)),
                    "{variant}: {id}"
                );
            }
        }
    }
}
