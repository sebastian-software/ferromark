//! Untimed spec profiles for the pinned native comparison build.
use std::io::{self, BufRead, Write};

#[global_allocator]
static ALLOC: bun_alloc::Mimalloc = bun_alloc::Mimalloc;

fn render(engine: &str, gfm: bool, source: &str) -> Vec<u8> {
    match engine {
        "v2" => {
            let arena = ferromark_v2::Allocator::for_source_len(source.len());
            let parser = if gfm {
                ferromark_v2::ParserOptions::gfm_spec()
            } else {
                ferromark_v2::ParserOptions::commonmark()
            };
            let options = if gfm {
                ferromark_v2::HtmlRendererOptions::gfm_spec()
            } else {
                ferromark_v2::HtmlRendererOptions::commonmark()
            };
            let doc = ferromark_v2::Parser::with_options(&arena, source, parser)
                .parse()
                .expect("parse");
            ferromark_v2::HtmlRenderer::with_options(options)
                .render(&doc)
                .into_bytes()
        }
        "ox-content" => {
            use ox_content_parser::{Parser, ParserOptions};
            use ox_content_renderer::{HtmlRenderer, HtmlRendererOptions};
            let arena = ox_content_allocator::Allocator::for_source_len(source.len());
            let parser = ParserOptions {
                tables: gfm,
                strikethrough: gfm,
                task_lists: gfm,
                autolinks: gfm,
                ..ParserOptions::default()
            };
            let options = HtmlRendererOptions {
                autolink_urls: false,
                autolink_target_blank: false,
                link_target_blank: false,
                disallow_raw_html: gfm,
                ..HtmlRendererOptions::new()
            };
            let doc = Parser::with_options(&arena, source, parser)
                .parse()
                .expect("parse");
            HtmlRenderer::with_options(options)
                .render(&doc)
                .into_bytes()
        }
        "bun" => {
            let mut options = bun_md::root::Options::default();
            for (name, _, setter) in bun_md::root::Options::BOOL_FIELD_SETTERS {
                setter(
                    &mut options,
                    gfm && matches!(
                        *name,
                        "tables"
                            | "strikethrough"
                            | "tasklists"
                            | "permissive_autolinks"
                            | "tag_filter"
                    ),
                );
            }
            bun_md::root::render_to_html_with_options(source.as_bytes(), options)
                .expect("Bun render")
                .as_ref()
                .to_vec()
        }
        "pulldown-cmark" => {
            let options = if gfm {
                pulldown_cmark::Options::ENABLE_TABLES
                    | pulldown_cmark::Options::ENABLE_STRIKETHROUGH
                    | pulldown_cmark::Options::ENABLE_TASKLISTS
            } else {
                pulldown_cmark::Options::empty()
            };
            let mut out = String::new();
            pulldown_cmark::html::push_html(
                &mut out,
                pulldown_cmark::Parser::new_ext(source, options),
            );
            out.into_bytes()
        }
        "md4c" => {
            let mut out = Vec::new();
            // Pinned md4c.h: tables, strike, tasks, permissive URL/email/www links.
            let flags = if gfm {
                0x0100 | 0x0200 | 0x0800 | 0x0004 | 0x0008 | 0x0400
            } else {
                0
            };
            let rc = unsafe {
                md_html(
                    source.as_ptr(),
                    source.len().try_into().expect("length"),
                    md4c_output,
                    (&mut out as *mut Vec<u8>).cast(),
                    flags,
                    0,
                )
            };
            assert_eq!(rc, 0, "md4c render");
            out
        }
        _ => panic!("unknown engine"),
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
    if len != 0 {
        // The synchronous callback receives the live unique output vector and span.
        unsafe {
            (&mut *userdata.cast::<Vec<u8>>())
                .extend_from_slice(std::slice::from_raw_parts(data, len as usize));
        }
    }
}
fn main() {
    bun_core::StackCheck::configure_thread();
    let args: Vec<String> = std::env::args().skip(1).collect();
    assert!(args.len() >= 4 && args[2] == "fresh");
    assert!(matches!(args[1].as_str(), "commonmark" | "gfm"));
    let inputs: Vec<String> = args[3..]
        .iter()
        .map(|p| std::fs::read_to_string(p).expect("input"))
        .collect();
    let mut out = io::BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        match line.expect("command").as_str() {
            "quit" => break,
            "verify" => {
                for (index, source) in inputs.iter().enumerate() {
                    let html = render(&args[0], args[1] == "gfm", source);
                    write!(out, "html {index} ").unwrap();
                    for byte in html {
                        write!(out, "{byte:02x}").unwrap();
                    }
                    writeln!(out).unwrap();
                    out.flush().unwrap();
                }
                writeln!(out, "done").unwrap();
                out.flush().unwrap();
            }
            _ => panic!("unknown command"),
        }
    }
}
