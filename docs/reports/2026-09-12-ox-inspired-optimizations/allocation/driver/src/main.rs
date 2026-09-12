use serde::Deserialize;
use serde_json::{json,Value};
use std::{hint::black_box,io::{self,BufRead,Write},time::{Duration,Instant}};
#[derive(Deserialize)]
struct Case {case:String,input:String,flags:u32,reuse:bool}
fn options(flags:u32)->ferro::Options {
 let mut o=ferro::Options::commonmark();o.tables=flags&1!=0;o.strikethrough=flags&2!=0;o.task_lists=flags&4!=0;o.heading_ids=flags&8!=0;
 o.render_policy=if flags&16!=0 {ferro::RenderPolicy::Untrusted} else {ferro::RenderPolicy::Trusted};
 if flags&32!=0{o.footnotes=true;o.inline_footnotes=true;o.math=true;o.callouts=true;o.autolink_literals=true;o.highlight=true;o.superscript=true;o.subscript=true;o.definition_lists=true;}
 o
}
fn main(){
 let args:Vec<_>=std::env::args().collect();let cases:Vec<Case>=serde_json::from_slice(&std::fs::read(&args[1]).unwrap()).unwrap();
 let opts:Vec<_>=cases.iter().map(|c|options(c.flags)).collect();let mut reuse:Vec<_>=opts.iter().cloned().map(ferro::Renderer::with_options).collect();
 let mut out=io::BufWriter::new(io::stdout().lock());
 for line in io::stdin().lock().lines(){let q:Value=serde_json::from_str(&line.unwrap()).unwrap();let i=q["index"].as_u64().unwrap() as usize;let c=&cases[i];let o=&opts[i];
  let row=match q["op"].as_str().unwrap(){
   "verify" if c.flags&256!=0=>{let r=ferro::mdx::render_with_options(&c.input,o);json!({"body":r.body,"esm":r.esm,"front_matter":r.front_matter})},
   "verify"=>{let r=ferro::parse_with_options(&c.input,o);let fresh=ferro::to_html_with_options(&c.input,o);let mut transitions=vec![];
    for s in [&c.input[..],"# Other\n\n- [x] New", "", &c.input[..]] {let actual=reuse[i].render(s);let expected=ferro::to_html_with_options(s,o);assert_eq!(actual,expected);transitions.push(actual);}
    json!({"html":r.html,"fresh":fresh,"limits":format!("{:?}",r.resource_limits),"headings":format!("{:?}",r.headings),"transitions":transitions})},
   #[cfg(feature="allocations")]
   "alloc"=>{
    allocation::reset();
    for _ in 0..10 {if c.flags&256!=0{drop(black_box(ferro::mdx::render_with_options(black_box(&c.input),o)));}else if c.reuse{drop(black_box(reuse[i].render(black_box(&c.input))));}else{drop(black_box(ferro::to_html_with_options(black_box(&c.input),o)));}}
    let (calls,bytes)=allocation::snapshot();json!({"case":c.case,"calls":calls/10,"requested_bytes":bytes/10})
   },
   "window"=>{let ms=q["ms"].as_u64().unwrap();let start=Instant::now();let mut n=0;loop{for _ in 0..16{if c.flags&256!=0{drop(black_box(ferro::mdx::render_with_options(black_box(&c.input),o)));}else if c.reuse{drop(black_box(reuse[i].render(black_box(&c.input))));}else{drop(black_box(ferro::to_html_with_options(black_box(&c.input),o)));}}n+=16;if start.elapsed()>=Duration::from_millis(ms){break;}}
    json!({"case":c.case,"count":n,"elapsed_ns":start.elapsed().as_nanos() as u64})},
   _=>panic!("unknown op"),
  };writeln!(out,"{row}").unwrap();out.flush().unwrap();
 }
}

#[cfg(feature = "allocations")]
mod allocation {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
    static CALLS: AtomicU64 = AtomicU64::new(0);
    static BYTES: AtomicU64 = AtomicU64::new(0);
    pub struct Counting;
    unsafe impl GlobalAlloc for Counting {
        unsafe fn alloc(&self, l: Layout) -> *mut u8 {
            CALLS.fetch_add(1, Relaxed);
            BYTES.fetch_add(l.size() as u64, Relaxed);
            unsafe { System.alloc(l) }
        }
        unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
            CALLS.fetch_add(1, Relaxed);
            BYTES.fetch_add(l.size() as u64, Relaxed);
            unsafe { System.alloc_zeroed(l) }
        }
        unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
            CALLS.fetch_add(1, Relaxed);
            BYTES.fetch_add(n as u64, Relaxed);
            unsafe { System.realloc(p, l, n) }
        }
        unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
            unsafe { System.dealloc(p, l) }
        }
    }
    pub fn reset() {
        CALLS.store(0, Relaxed);
        BYTES.store(0, Relaxed);
    }
    pub fn snapshot() -> (u64, u64) {
        (CALLS.load(Relaxed), BYTES.load(Relaxed))
    }
}
#[cfg(feature = "allocations")]
#[global_allocator]
static GLOBAL: allocation::Counting = allocation::Counting;
