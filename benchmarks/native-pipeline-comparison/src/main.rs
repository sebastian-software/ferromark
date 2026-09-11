//! Persistent native workers. IPC, configuration, and output inspection are untimed.
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    hint::black_box,
    io::{self, BufRead, Write},
    time::{Duration, Instant},
};

#[derive(Deserialize)]
struct Case {
    case: String,
    input: String,
    flags: u32,
}

struct Renderer {
    ferro: ferromark::Options,
    satteri: satteri_pulldown_cmark::Options,
}

impl Renderer {
    fn new(flags: u32) -> Self {
        assert!(flags <= 7);
        let ferro = ferromark::options!(ferromark::Options::commonmark();
            render_policy: ferromark::RenderPolicy::Trusted,
            tables: flags & 1 != 0,
            strikethrough: flags & 2 != 0,
            task_lists: flags & 4 != 0,
        );
        use satteri_pulldown_cmark::Options as S;
        let mut satteri = S::empty();
        satteri.set(S::ENABLE_TABLES, flags & 1 != 0);
        satteri.set(S::ENABLE_STRIKETHROUGH, flags & 2 != 0);
        satteri.set(S::ENABLE_TASKLISTS, flags & 4 != 0);
        Self { ferro, satteri }
    }

    fn render(&self, engine: &str, input: &str) -> String {
        match engine {
            "ferromark" => ferromark::to_html_with_options(input, &self.ferro),
            "satteri" => {
                // The public high-level markdown_to_html uses these same two
                // stages with DEFAULT_OPTIONS. Select explicit syntax flags,
                // retaining position tracking and the normal HTML fast/fallback path.
                let (arena, errors) = satteri_pulldown_cmark::parse(input, self.satteri);
                debug_assert!(errors.is_empty());
                satteri_ast::mdast_to_html(&arena)
            }
            _ => panic!("unknown engine"),
        }
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let engine = &args[1];
    assert!(engine == "ferromark" || engine == "satteri");
    let cases: Vec<Case> = serde_json::from_slice(&std::fs::read(&args[2]).unwrap()).unwrap();
    let renderers: Vec<_> = cases.iter().map(|c| Renderer::new(c.flags)).collect();
    let mut out = io::BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        let request: Value = serde_json::from_str(&line.unwrap()).unwrap();
        let index = request["index"].as_u64().unwrap() as usize;
        let case = &cases[index];
        let renderer = &renderers[index];
        let result = match request["op"].as_str().unwrap() {
            "verify" => json!({"case":case.case,"engine":engine,"flags":case.flags,
                "bytes":case.input.len(),"html":renderer.render(engine,&case.input),
                "options":if engine == "ferromark" {format!("{:?}",renderer.ferro)}
                    else {format!("{:?}; mdast_to_html default conversion; positions retained",renderer.satteri)}}),
            "window" => {
                let ms = request["ms"].as_u64().unwrap();
                assert!(ms > 0);
                let start = Instant::now();
                let mut count = 0u64;
                loop {
                    for _ in 0..16 {
                        drop(black_box(renderer.render(engine, black_box(&case.input))));
                    }
                    count += 16;
                    if start.elapsed() >= Duration::from_millis(ms) {
                        break;
                    }
                }
                let elapsed = start.elapsed().as_nanos() as u64;
                json!({"case":case.case,"engine":engine,"count":count,"elapsed_ns":elapsed,
                    "ns_per_render":elapsed as f64/count as f64})
            }
            "mdx" => {
                if engine == "satteri" {
                    match satteri::compile_mdx(&case.input, &satteri_mdxjs::Options::default()) {
                        Ok(code) => {
                            json!({"case":case.case,"engine":engine,"stage":"MDX to JavaScript","code":code})
                        }
                        Err(error) => {
                            json!({"case":case.case,"engine":engine,"stage":"MDX to JavaScript","error":error})
                        }
                    }
                } else {
                    let code = ferromark::mdx::render_with_options(&case.input, &renderer.ferro)
                        .to_component("Content")
                        .unwrap();
                    json!({"case":case.case,"engine":engine,"stage":"segmentation and JSX module emission","code":code})
                }
            }
            _ => panic!("unknown operation"),
        };
        writeln!(out, "{result}").unwrap();
        out.flush().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROBE: &str = "| A |\n| --- |\n| B |\n\n~~old~~\n\n- [x] done\n- [ ] pending\n\nwww.example.com\n\n<script>x</script>\n";

    #[test]
    fn native_stages_match_the_high_level_satteri_api() {
        for input in ["", PROBE, "# Hello\n\nText &amp; **strong**."] {
            let (arena, _) =
                satteri_pulldown_cmark::parse(input, satteri_pulldown_cmark::DEFAULT_OPTIONS);
            assert_eq!(
                satteri_ast::mdast_to_html(&arena),
                satteri::markdown_to_html(input)
            );
        }
    }

    #[test]
    fn independent_flags_and_trusted_rendering_are_preserved() {
        for flags in 0..=7 {
            let renderer = Renderer::new(flags);
            for engine in ["ferromark", "satteri"] {
                let html = renderer.render(engine, PROBE);
                assert_eq!(html.contains("<table>"), flags & 1 != 0);
                assert_eq!(html.contains("<del>old</del>"), flags & 2 != 0);
                assert_eq!(
                    html.matches("type=\"checkbox\"").count(),
                    if flags & 4 != 0 { 2 } else { 0 }
                );
                assert_eq!(html.matches("checked").count(), usize::from(flags & 4 != 0));
                assert!(!html.contains("href="));
                assert!(html.contains("<script>x</script>"));
            }
        }
    }

    #[test]
    fn mdx_compilation_is_not_segmentation() {
        let input = "export const = ;\n\n# Heading\n";
        assert!(satteri::compile_mdx(input, &satteri_mdxjs::Options::default()).is_err());
        assert!(
            ferromark::mdx::render(input)
                .to_component("Content")
                .is_ok()
        );
    }
}
