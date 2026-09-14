//! Direct UTF-8 Markdown -> HTML. No subprocess, JSON, I/O, or JS in timed calls.
use std::hint::black_box;
use std::io::{self, BufRead, Write};
use std::time::{Duration, Instant};

#[global_allocator]
static ALLOC: bun_alloc::Mimalloc = bun_alloc::Mimalloc;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Fresh,
    Reuse,
}

trait Engine {
    fn verify_input(&self, _source: &str) {}
    fn render<T>(&mut self, source: &str, consume: impl FnOnce(&[u8]) -> T) -> T;
}

// Reuse the native lifecycle adapter, but choose each engine's own options.
mod ox {
    pub use ox_content_allocator::Allocator;
    pub use ox_content_parser::{Parser, ParserOptions};
    pub use ox_content_renderer::{HtmlRenderer, HtmlRendererOptions};
}
macro_rules! arena_engine {
    ($module:ident, $api:ident, $options:ident) => {
        mod $module {
            use super::{Engine, Mode, $api as api};
            pub struct Renderer {
                options: api::ParserOptions,
                html_options: api::HtmlRendererOptions,
                arena: api::Allocator,
                renderer: api::HtmlRenderer,
                mode: Mode,
            }
            impl Renderer {
                pub fn new(gfm: bool, mode: Mode, max_source: usize) -> Self {
                    let (options, html_options) = super::$options(gfm);
                    Self {
                        renderer: api::HtmlRenderer::with_options(html_options.clone()),
                        arena: api::Allocator::for_source_len(max_source),
                        options,
                        html_options,
                        mode,
                    }
                }
            }
            impl Engine for Renderer {
                fn render<T>(&mut self, source: &str, consume: impl FnOnce(&[u8]) -> T) -> T {
                    if self.mode == Mode::Fresh {
                        let arena = api::Allocator::for_source_len(source.len());
                        let doc = api::Parser::with_options(&arena, source, self.options.clone())
                            .parse()
                            .expect("parse");
                        let mut renderer =
                            api::HtmlRenderer::with_options(self.html_options.clone());
                        let html = renderer.render(&doc);
                        consume(html.as_bytes())
                    } else {
                        let result = {
                            let doc = api::Parser::with_options(
                                &self.arena,
                                source,
                                self.options.clone(),
                            )
                            .parse()
                            .expect("parse");
                            consume(self.renderer.render_borrowed(&doc).as_bytes())
                        };
                        self.arena.reset();
                        result
                    }
                }
            }
        }
    };
}
use ferromark_v2 as v2;
fn ox_options(gfm: bool) -> (ox::ParserOptions, ox::HtmlRendererOptions) {
    (
        ox::ParserOptions {
            tables: gfm,
            strikethrough: gfm,
            task_lists: gfm,
            ..ox::ParserOptions::default()
        },
        ox::HtmlRendererOptions {
            autolink_urls: false,
            autolink_target_blank: false,
            link_target_blank: false,
            ..ox::HtmlRendererOptions::new()
        },
    )
}
fn v2_options(gfm: bool) -> (v2::ParserOptions, v2::HtmlRendererOptions) {
    (
        v2::ParserOptions {
            tables: gfm,
            strikethrough: gfm,
            task_lists: gfm,
            ..v2::ParserOptions::commonmark()
        },
        // The shared lane deliberately excludes GFM tag filtering and bare
        // autolinks. CommonMark rendering also disables IDs, callouts, TOCs,
        // and code-fence metadata; those product defaults are not common work.
        v2::HtmlRendererOptions::commonmark(),
    )
}
arena_engine!(ox_engine, ox, ox_options);
arena_engine!(v2_engine, v2, v2_options);

struct V1 {
    options: ferromark_v1::Options,
    renderer: ferromark_v1::Renderer,
    out: Vec<u8>,
    mode: Mode,
}
impl V1 {
    fn new(gfm: bool, mode: Mode) -> Self {
        let mut options = ferromark_v1::Options::commonmark();
        options.render_policy = ferromark_v1::RenderPolicy::Trusted;
        options.tables = gfm;
        options.strikethrough = gfm;
        options.task_lists = gfm;
        options.heading_ids = false;
        options.callouts = false;
        options.disallowed_raw_html = false;
        Self {
            renderer: ferromark_v1::Renderer::with_options(options.clone()),
            options,
            out: Vec::new(),
            mode,
        }
    }
}
impl Engine for V1 {
    fn verify_input(&self, source: &str) {
        let parsed = ferromark_v1::parse_with_options(source, &self.options);
        assert!(
            parsed.resource_limits.is_empty(),
            "V1 resource fallback: {:?}",
            parsed.resource_limits
        );
    }
    fn render<T>(&mut self, source: &str, consume: impl FnOnce(&[u8]) -> T) -> T {
        if self.mode == Mode::Fresh {
            let html = ferromark_v1::to_html_with_options(source, &self.options);
            consume(html.as_bytes())
        } else {
            self.renderer.render_into(source, &mut self.out);
            consume(&self.out)
        }
    }
}
struct Bun {
    options: bun_md::root::Options,
}
impl Bun {
    fn new(gfm: bool) -> Self {
        let mut options = bun_md::root::Options::default();
        for (name, _, setter) in bun_md::root::Options::BOOL_FIELD_SETTERS {
            setter(&mut options, gfm && matches!(*name, "tables" | "strikethrough" | "tasklists"));
        }
        Self { options }
    }
}
impl Engine for Bun {
    fn render<T>(&mut self, source: &str, consume: impl FnOnce(&[u8]) -> T) -> T {
        let html = bun_md::root::render_to_html_with_options(source.as_bytes(), self.options)
            .expect("Bun render");
        consume(html.as_ref())
    }
}
struct Pulldown {
    options: pulldown_cmark::Options,
    out: String,
    mode: Mode,
}
impl Pulldown {
    fn new(gfm: bool, mode: Mode) -> Self {
        let mut options = pulldown_cmark::Options::empty();
        options.set(pulldown_cmark::Options::ENABLE_TABLES, gfm);
        options.set(pulldown_cmark::Options::ENABLE_STRIKETHROUGH, gfm);
        options.set(pulldown_cmark::Options::ENABLE_TASKLISTS, gfm);
        Self { options, out: String::new(), mode }
    }
}
impl Engine for Pulldown {
    fn render<T>(&mut self, source: &str, consume: impl FnOnce(&[u8]) -> T) -> T {
        if self.mode == Mode::Fresh {
            let mut out = String::new();
            pulldown_cmark::html::push_html(
                &mut out,
                pulldown_cmark::Parser::new_ext(source, self.options),
            );
            consume(out.as_bytes())
        } else {
            self.out.clear();
            pulldown_cmark::html::push_html(
                &mut self.out,
                pulldown_cmark::Parser::new_ext(source, self.options),
            );
            consume(self.out.as_bytes())
        }
    }
}
unsafe extern "C" {
    fn md_html(
        input: *const u8,
        len: u32,
        output: extern "C" fn(*const u8, u32, *mut std::ffi::c_void),
        userdata: *mut std::ffi::c_void,
        parser_flags: u32,
        renderer_flags: u32,
    ) -> i32;
}
extern "C" fn md4c_output(data: *const u8, len: u32, userdata: *mut std::ffi::c_void) {
    if len == 0 {
        return;
    }
    // md_html calls synchronously with a live byte span and our unique output Vec.
    unsafe {
        (&mut *userdata.cast::<Vec<u8>>())
            .extend_from_slice(std::slice::from_raw_parts(data, len as usize));
    }
}
fn md4c_render(source: &str, flags: u32, out: &mut Vec<u8>) {
    out.clear();
    let rc = unsafe {
        md_html(
            source.as_ptr(),
            source.len().try_into().expect("md4c input fits u32"),
            md4c_output,
            (out as *mut Vec<u8>).cast(),
            flags,
            0,
        )
    };
    assert_eq!(rc, 0, "md4c render");
}
struct Md4c {
    flags: u32,
    out: Vec<u8>,
    mode: Mode,
}
impl Md4c {
    fn new(gfm: bool, mode: Mode) -> Self {
        // Pinned md4c.h: TABLES=0x100, STRIKETHROUGH=0x200, TASKLISTS=0x800.
        Self { flags: if gfm { 0x0100 | 0x0200 | 0x0800 } else { 0 }, out: Vec::new(), mode }
    }
}
impl Engine for Md4c {
    fn render<T>(&mut self, source: &str, consume: impl FnOnce(&[u8]) -> T) -> T {
        if self.mode == Mode::Fresh {
            let mut out = Vec::new();
            md4c_render(source, self.flags, &mut out);
            consume(&out)
        } else {
            md4c_render(source, self.flags, &mut self.out);
            consume(&self.out)
        }
    }
}
fn serve(mut engine: impl Engine, inputs: &[String]) {
    let mut stdout = io::BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        let line = line.expect("command");
        if line == "verify" {
            for (index, source) in inputs.iter().enumerate() {
                engine.verify_input(source);
                let html = engine.render(source, |bytes| bytes.to_vec());
                write!(stdout, "html {index} ").unwrap();
                for byte in html {
                    write!(stdout, "{byte:02x}").unwrap();
                }
                writeln!(stdout).unwrap();
            }
            writeln!(stdout, "done").unwrap();
        } else if let Some(ns) = line.strip_prefix("bench ") {
            let budget = Duration::from_nanos(ns.parse().expect("budget"));
            let start = Instant::now();
            let mut iterations = 0u64;
            let mut checksum = 0usize;
            loop {
                for _ in 0..32 {
                    for source in inputs {
                        checksum = checksum.wrapping_add(
                            engine.render(black_box(source), |html| black_box(html).len()),
                        );
                    }
                }
                iterations += 32;
                if start.elapsed() >= budget {
                    break;
                }
            }
            writeln!(
                stdout,
                "timing {iterations} {} {}",
                start.elapsed().as_nanos(),
                black_box(checksum)
            )
            .unwrap();
        } else if line == "quit" {
            break;
        } else {
            panic!("unknown command");
        }
        stdout.flush().unwrap();
    }
}
fn native_main() {
    bun_core::StackCheck::configure_thread();
    let args: Vec<String> = std::env::args().skip(1).collect();
    assert!(args.len() >= 4, "worker ENGINE PROFILE MODE INPUT...");
    let gfm = match args[1].as_str() {
        "commonmark" => false,
        "gfm" | "gfm-shared" => true,
        _ => panic!("profile"),
    };
    let mode = match args[2].as_str() {
        "fresh" => Mode::Fresh,
        "reuse" => Mode::Reuse,
        _ => panic!("mode"),
    };
    let inputs: Vec<String> =
        args[3..].iter().map(|p| std::fs::read_to_string(p).expect("input")).collect();
    let max_source = inputs.iter().map(String::len).max().unwrap();
    match args[0].as_str() {
        "v2" => serve(v2_engine::Renderer::new(gfm, mode, max_source), &inputs),
        "ox-content" => serve(ox_engine::Renderer::new(gfm, mode, max_source), &inputs),
        "v1" => serve(V1::new(gfm, mode), &inputs),
        "bun" => serve(Bun::new(gfm), &inputs),
        "pulldown-cmark" => serve(Pulldown::new(gfm, mode), &inputs),
        "md4c" => serve(Md4c::new(gfm, mode), &inputs),
        _ => panic!("engine"),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Stage {
    Init,
    Parse,
    Render,
}

// Keep the six-engine fresh/reuse adapter above unchanged. These additional
// entry points isolate parser construction (including its reference prepass),
// complete parsing, and rendering a prebuilt AST under the same allocator.
macro_rules! staged_engine {
    ($name:ident, $api:ident, $options:ident) => {
        mod $name {
            use super::*;
            use $api as api;

            pub fn run(profile: &str, stage: Stage, sources: &[String]) {
                let gfm = matches!(profile, "gfm" | "gfm-shared");
                let (options, html_options) = $options(gfm);
                let max_source = sources.iter().map(String::len).max().unwrap();
                let mut arena = api::Allocator::for_source_len(max_source);
                let render_arena = api::Allocator::for_source_len(max_source);
                let documents = if stage == Stage::Render {
                    sources.iter().map(|source| {
                        api::Parser::with_options(&render_arena, source, options.clone())
                            .parse().expect("prebuilt document")
                    }).collect::<Vec<_>>()
                } else {
                    Vec::new()
                };
                let mut renderer = api::HtmlRenderer::with_options(html_options);
                let mut stdout = io::BufWriter::new(io::stdout().lock());
                for line in io::stdin().lock().lines() {
                    let line = line.expect("command");
                    if line == "verify" || line == "details" {
                        for (index, source) in sources.iter().enumerate() {
                            let document = api::Parser::with_options(&arena, source, options.clone())
                                .parse().expect("verify document");
                            let html = renderer.render_borrowed(&document);
                            if line == "details" {
                                let details = serde_json::json!({
                                    "html": html,
                                    "ast_debug": format!("{document:?}"),
                                    "children": document.children.len(),
                                    "parser_size": std::mem::size_of::<api::Parser<'_>>(),
                                    "parser_options_size": std::mem::size_of::<api::ParserOptions>(),
                                    "renderer_size": std::mem::size_of::<api::HtmlRenderer>(),
                                    "document_size": std::mem::size_of_val(&document),
                                    "node_size": document.children.first().map(std::mem::size_of_val),
                                    "arena_capacity": arena.allocated_bytes(),
                                    "metric": match stage {
                                        Stage::Init => source.len(),
                                        Stage::Parse => document.children.len(),
                                        Stage::Render => html.len(),
                                    },
                                });
                                writeln!(stdout, "{details}").unwrap();
                            } else {
                                write!(stdout, "html {index} ").unwrap();
                                for byte in html.bytes() {
                                    write!(stdout, "{byte:02x}").unwrap();
                                }
                                writeln!(stdout).unwrap();
                            }
                        }
                        arena.reset();
                        writeln!(stdout, "done").unwrap();
                    } else if let Some(ns) = line.strip_prefix("bench ") {
                        let budget = Duration::from_nanos(ns.parse().expect("budget"));
                        let start = Instant::now();
                        let mut iterations = 0u64;
                        let mut checksum = 0usize;
                        loop {
                            for _ in 0..32 {
                                for (index, source) in sources.iter().enumerate() {
                                    checksum = checksum.wrapping_add(match stage {
                                        Stage::Init => {
                                            let parser = api::Parser::with_options(
                                                &arena, black_box(source), options.clone());
                                            black_box(&parser);
                                            drop(parser);
                                            arena.reset();
                                            black_box(source.len())
                                        }
                                        Stage::Parse => {
                                            let children = {
                                                let document = api::Parser::with_options(
                                                    &arena, black_box(source), options.clone())
                                                    .parse().expect("parse");
                                                black_box(&document);
                                                black_box(document.children.len())
                                            };
                                            arena.reset();
                                            children
                                        }
                                        Stage::Render => {
                                            black_box(renderer.render_borrowed(&documents[index])).len()
                                        }
                                    });
                                }
                            }
                            iterations += 32;
                            if start.elapsed() >= budget { break; }
                        }
                        writeln!(stdout, "timing {iterations} {} {}",
                            start.elapsed().as_nanos(), black_box(checksum)).unwrap();
                    } else if line == "quit" {
                        break;
                    } else {
                        panic!("unknown command");
                    }
                    stdout.flush().unwrap();
                }
            }
        }
    };
}
staged_engine!(staged_v2, v2, v2_options);
staged_engine!(staged_ox, ox, ox_options);

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let stage = match args.get(2).map(String::as_str) {
        Some("init") => Stage::Init,
        Some("parse") => Stage::Parse,
        Some("render") => Stage::Render,
        _ => return native_main(),
    };
    assert!(matches!(args[1].as_str(), "commonmark" | "gfm" | "gfm-shared"));
    let sources: Vec<String> =
        args[3..].iter().map(|p| std::fs::read_to_string(p).unwrap()).collect();
    match args[0].as_str() {
        "v2" => staged_v2::run(&args[1], stage, &sources),
        "ox-content" => staged_ox::run(&args[1], stage, &sources),
        _ => panic!("staged measurements require v2 or ox-content"),
    }
}
