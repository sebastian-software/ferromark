//! Temporary measurement probe. No parser algorithms are changed.
use ferromark::{Options, RenderPolicy, Renderer};
use std::{hint::black_box, time::{Duration, Instant}};

#[cfg(feature = "profiling")]
mod allocation {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed};
    pub static ENABLED: AtomicBool = AtomicBool::new(false);
    pub static CALLS: AtomicUsize = AtomicUsize::new(0);
    pub static BYTES: AtomicUsize = AtomicUsize::new(0);
    pub struct Counter;
    unsafe impl GlobalAlloc for Counter {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            if ENABLED.load(Relaxed) { CALLS.fetch_add(1,Relaxed); BYTES.fetch_add(layout.size(),Relaxed); }
            unsafe { System.alloc(layout) }
        }
        unsafe fn realloc(&self, p: *mut u8, old: Layout, size: usize) -> *mut u8 {
            if ENABLED.load(Relaxed) { CALLS.fetch_add(1,Relaxed); BYTES.fetch_add(size,Relaxed); }
            unsafe { System.realloc(p,old,size) }
        }
        unsafe fn dealloc(&self, p: *mut u8, layout: Layout) { unsafe { System.dealloc(p,layout) } }
    }
}
#[cfg(feature = "profiling")]
#[global_allocator]
static ALLOCATOR: allocation::Counter = allocation::Counter;

fn options(name: &str) -> Options {
    let mut o=Options::commonmark();
    match name {
        "commonmark" => {},
        "tables" => o.tables=true,
        "autolinks" => o.autolink_literals=true,
        "strike" => o.strikethrough=true,
        "tasks" => o.task_lists=true,
        "filter" => o.disallowed_raw_html=true,
        "overlap" => {o.tables=true;o.strikethrough=true;o.task_lists=true;},
        "gfm" => o=Options::gfm(),
        "gfm-no-autolinks" => {o=Options::gfm();o.autolink_literals=false;},
        "gfm-no-tables" => {o=Options::gfm();o.tables=false;},
        _ => panic!("unknown preset"),
    }
    o
}
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode=args.get(1).map_or("measure",String::as_str);
    let corpus=corpora();
    if mode=="fixtures" {
        for (name,input) in &corpus {std::fs::write(format!("target/gfm-profile/{name}.md"),input).unwrap();}
        return;
    }
    let presets=["commonmark","tables","autolinks","strike","tasks","filter","overlap","gfm","gfm-no-autolinks","gfm-no-tables"];
    let mut results=Vec::new();
    let rounds=if mode=="measure" {5} else {1};
    for round in 0..rounds {
      for (case,input) in &corpus {
        let baseline=ferromark::to_html_with_options(input,&Options::commonmark());
        let mut order:Vec<_>=(0..presets.len()).collect();
        order.rotate_left(round % presets.len());
        if round % 2 == 1 {order.reverse();}
        for index in order {
          let preset=presets[index];
          let opts=options(preset);
          let expected=ferromark::to_html_with_options(input,&opts);
          for lane in ["owned","buffer","renderer"] {
            let mut renderer=Renderer::with_options(opts.clone());
            let mut output=Vec::with_capacity(input.len()*2);
            renderer.render_into(input,&mut output);
            assert_eq!(output,expected.as_bytes());
            ferromark::to_html_into_with_options(input,&mut output,&opts);
            assert_eq!(output,expected.as_bytes());
            let mut render=|| {
              match lane {
                "owned" => { black_box(ferromark::to_html_with_options(black_box(input),black_box(&opts))); },
                "buffer" => {ferromark::to_html_into_with_options(black_box(input),&mut output,black_box(&opts));black_box(&output);},
                _ => {renderer.render_into(black_box(input),&mut output);black_box(&output);},
              }
            };
            for _ in 0..16 {render();}
            if mode=="measure" {
              let start=Instant::now();let mut n=0u64;
              loop {for _ in 0..16 {render();} n+=16;if start.elapsed()>=Duration::from_millis(75){break;}}
              results.push(serde_json::json!({"case":case,"preset":preset,"lane":lane,"round":round,"input_bytes":input.len(),"output_bytes":expected.len(),"same_as_commonmark":expected==baseline,"ns":start.elapsed().as_secs_f64()*1e9/n as f64}));
            } else {
              #[cfg(feature="profiling")]
              {
                use std::sync::atomic::Ordering::Relaxed;
                ferromark::profiling::reset();
                allocation::CALLS.store(0,Relaxed);allocation::BYTES.store(0,Relaxed);allocation::ENABLED.store(true,Relaxed);
                render();
                allocation::ENABLED.store(false,Relaxed);
                let snap=ferromark::profiling::snapshot();
                results.push(serde_json::json!({"case":case,"preset":preset,"lane":lane,"calls":allocation::CALLS.load(Relaxed),"requested_bytes":allocation::BYTES.load(Relaxed),"inline_parses":snap.inline_parses,"inline_bytes":snap.inline_input_bytes,"fast_paths":snap.inline_fast_paths,"marks":snap.inline_marks,"inline_events":snap.inline_events,"block_events":snap.block_events,"table_events":snap.block_table_events,"paragraph_copied_bytes":snap.paragraph_copied_bytes}));
              }
              #[cfg(not(feature="profiling"))]
              panic!("counts require --features profiling");
            }
          }
        }
      }
      eprintln!("round {}/{} complete",round+1,rounds);
    }
    println!("{}",serde_json::to_string_pretty(&results).unwrap());
    // Keep the trust policy explicit in this probe: every preset is untrusted.
    assert_eq!(options("gfm").render_policy,RenderPolicy::Untrusted);
}
fn corpora()->Vec<(&'static str,String)> {
    vec![
      ("plain","Ordinary words with no special syntax and enough text for several vector loads.\n\n".repeat(128)),
      ("commonmark-50k",include_str!("../benches/fixtures/commonmark-50k.md").into()),
      ("gfm-overlap-tables","| Item | State |\n| --- | --- |\n| **table** | ~~pending~~ done |\n\n".repeat(80)),
      ("tables-5k",include_str!("../benches/fixtures/tables-5k.md").into()),
      ("autolinks","Visit https://example.com/guide and www.example.org or mail person@example.com.\n\n".repeat(96)),
      ("tasks","- [x] First task\n- [ ] Second **task**\n- Ordinary item\n\n".repeat(96)),
      ("mixed-gfm",("## Notes\n\nVisit https://example.com/guide and [the guide](/guide).\n\n| Item | State |\n| --- | --- |\n| **table** | ~~pending~~ done |\n\n- [x] First task\n- [ ] Second task\n\n").repeat(48)),
    ]
}
