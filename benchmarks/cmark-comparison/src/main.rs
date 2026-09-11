//! Two native libraries per process; no CLI startup or serialization in the timer.
use serde_json::{Value, json};
use std::{
    ffi::{CStr, c_char},
    hint::black_box,
    ptr::NonNull,
    sync::Once,
    time::{Duration, Instant},
};

const COMPETITOR: &str = if cfg!(feature = "cmark-gfm") {
    "cmark-gfm"
} else {
    "cmark"
};
static INIT: Once = Once::new();

unsafe extern "C" {
    fn bench_cmark_init() -> i32;
    fn bench_cmark_render(input: *const u8, len: usize, flags: u32) -> *mut c_char;
    fn bench_cmark_free(html: *mut c_char);
}

struct COutput(NonNull<c_char>);
impl Drop for COutput {
    fn drop(&mut self) {
        // The bridge returns an owned buffer from cmark's default allocator.
        unsafe { bench_cmark_free(self.0.as_ptr()) };
    }
}
enum Output {
    Rust(String),
    C(COutput),
}
impl Output {
    fn text(&self) -> &str {
        match self {
            Self::Rust(s) => s,
            // cmark returns a live, NUL-terminated UTF-8 buffer. Inspection is
            // outside timing; rendering never copies it into a Rust buffer.
            Self::C(out) => unsafe { CStr::from_ptr(out.0.as_ptr()) }.to_str().unwrap(),
        }
    }
}

struct Renderers {
    flags: u32,
    ferro: ferromark::Options,
}
impl Renderers {
    fn new(flags: u32) -> Self {
        assert!(flags <= 7, "unsupported configuration bits");
        assert!(
            flags == 0 || cfg!(feature = "cmark-gfm"),
            "cmark only supports CommonMark"
        );
        INIT.call_once(|| {
            // Register immutable extension descriptors once, before any timing.
            assert_eq!(unsafe { bench_cmark_init() }, 1);
        });
        let ferro = ferromark::options!(ferromark::Options::commonmark();
            render_policy: ferromark::RenderPolicy::Trusted,
            tables: flags & 1 != 0,
            strikethrough: flags & 2 != 0,
            task_lists: flags & 4 != 0,
        );
        Self { flags, ferro }
    }

    fn render(&self, index: usize, input: &str) -> Output {
        if index == 0 {
            Output::Rust(ferromark::to_html_with_options(input, &self.ferro))
        } else {
            assert_eq!(index, 1);
            // The borrowed input lives through this synchronous call. The
            // result has unique ownership and is freed by COutput::drop.
            let ptr = unsafe { bench_cmark_render(input.as_ptr(), input.len(), self.flags) };
            Output::C(COutput(NonNull::new(ptr).expect("cmark rendering failed")))
        }
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let mode = args.get(1).expect("verify|bench <cases.json>");
    assert!(mode == "verify" || mode == "bench");
    let cases: Value = serde_json::from_slice(&std::fs::read(&args[2]).unwrap()).unwrap();
    let samples = args.get(3).map_or(80, |x| x.parse::<usize>().unwrap());
    let window_ms = args.get(4).map_or(63, |x| x.parse::<u64>().unwrap());
    let warmup_ms = args.get(5).map_or(3000, |x| x.parse::<u64>().unwrap());
    let rotation = args.get(6).map_or(0, |x| x.parse::<usize>().unwrap());
    assert!(samples > 0 && window_ms > 0);
    for case in cases.as_array().expect("case array") {
        let flags: u32 = case["flags"].as_u64().unwrap().try_into().unwrap();
        let renderers = Renderers::new(flags);
        let input = case["input"].as_str().unwrap();
        let outputs: Vec<_> = (0..2).map(|p| renderers.render(p, input)).collect();
        let sizes: Vec<_> = outputs.iter().map(|out| out.text().len()).collect();
        if mode == "verify" {
            println!(
                "{}",
                json!({"case":case["case"], "flags":flags, "bytes":input.len(),
                    "outputs":{"ferromark":outputs[0].text(), COMPETITOR:outputs[1].text()},
                    "ferromark_options":format!("{:?}", renderers.ferro),
                    "cmark_options":{"unsafe_html":true,"extensions":extension_names(flags)},
                })
            );
            continue;
        }
        drop(outputs);
        for p in 0..2 {
            let start = Instant::now();
            while start.elapsed() < Duration::from_millis(warmup_ms) {
                drop(black_box(renderers.render(p, black_box(input))));
            }
        }
        let mut times = [Vec::new(), Vec::new()];
        for rep in 0..samples {
            for offset in 0..2 {
                let p = (rep + rotation + offset) % 2;
                let start = Instant::now();
                let mut count = 0u64;
                loop {
                    for _ in 0..16 {
                        drop(black_box(renderers.render(p, black_box(input))));
                    }
                    count += 16;
                    if start.elapsed() >= Duration::from_millis(window_ms) {
                        break;
                    }
                }
                times[p].push(start.elapsed().as_secs_f64() * 1e9 / count as f64);
            }
        }
        for (p, name) in ["ferromark", COMPETITOR].iter().enumerate() {
            println!(
                "{}",
                json!({"case":case["case"],"parser":name,"bytes":input.len(),
                "output_bytes":sizes[p],"ns_per_render":times[p]})
            );
        }
    }
}

fn extension_names(flags: u32) -> Vec<&'static str> {
    ["table", "strikethrough", "tasklist"]
        .into_iter()
        .enumerate()
        .filter_map(|(i, name)| (flags & (1 << i) != 0).then_some(name))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmark(flags: u32, input: &str) -> String {
        Renderers::new(flags).render(1, input).text().to_owned()
    }

    #[test]
    fn core_renders_owned_html_and_preserves_trusted_html_and_urls() {
        for _ in 0..20 {
            assert_eq!(
                cmark(0, "# Hi\n\n<b>raw</b> [link](javascript:alert)\n"),
                "<h1>Hi</h1>\n<p><b>raw</b> <a href=\"javascript:alert\">link</a></p>\n"
            );
            assert_eq!(cmark(0, ""), "");
            assert_eq!(cmark(0, "a\0b\n"), "<p>a\u{fffd}b</p>\n");
        }
    }

    #[test]
    fn core_does_not_enable_extensions() {
        let out = cmark(
            0,
            "- [x] done\n\n~~old~~ www.example.com\n\n| A |\n| - |\n| B |\n",
        );
        assert!(!out.contains("checkbox") && !out.contains("<del>") && !out.contains("<table>"));
        assert!(out.contains("www.example.com") && !out.contains("href="));
    }

    #[cfg(feature = "cmark-gfm")]
    #[test]
    fn each_gfm_switch_changes_only_its_requested_feature() {
        let input = "| A |\n| --- |\n| B |\n\n~~old~~\n\n- [x] done\n- [ ] pending\n\nwww.example.com\n\n<script>x</script>\n";
        for flags in 0..=7 {
            let out = cmark(flags, input);
            assert_eq!(out.contains("<table>"), flags & 1 != 0);
            assert_eq!(out.contains("<del>old</del>"), flags & 2 != 0);
            assert_eq!(
                out.matches("type=\"checkbox\"").count(),
                if flags & 4 != 0 { 2 } else { 0 }
            );
            assert_eq!(out.matches("checked").count(), usize::from(flags & 4 != 0));
            assert!(!out.contains("href="), "autolinks must stay disabled");
            assert!(
                out.contains("<script>x</script>"),
                "tag filtering must stay disabled"
            );
        }
    }

    #[cfg(not(feature = "cmark-gfm"))]
    #[test]
    #[should_panic(expected = "cmark only supports CommonMark")]
    fn unsupported_features_are_rejected_instead_of_silently_timed() {
        Renderers::new(1);
    }
}
