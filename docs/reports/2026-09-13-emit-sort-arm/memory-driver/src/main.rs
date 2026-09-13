use serde::Deserialize;
use serde_json::{json,Value};
use std::{hint::black_box,io::{self,BufRead,Write}};
#[derive(Deserialize)]
struct Case {case:String,input:String,flags:u32,reuse:bool}
fn options(flags:u32)->ferro::Options {
 let mut o=ferro::Options::commonmark();o.tables=flags&1!=0;o.strikethrough=flags&2!=0;o.task_lists=flags&4!=0;o.heading_ids=flags&8!=0;
 o.render_policy=if flags&16!=0 {ferro::RenderPolicy::Untrusted} else {ferro::RenderPolicy::Trusted};
 if flags&64!=0{o.footnotes=true;o.autolink_literals=true;}
 if flags&32!=0{o.footnotes=true;o.inline_footnotes=true;o.math=true;o.callouts=true;o.autolink_literals=true;o.highlight=true;o.superscript=true;o.subscript=true;o.definition_lists=true;}
 o.line_comments=flags&512!=0;o.allow_html=flags&1024==0;o.disallowed_raw_html=flags&2048!=0;
 o
}

mod heap;
#[global_allocator]
static GLOBAL: heap::Counting = heap::Counting;
fn main() {
 heap::self_test();
 let args:Vec<_>=std::env::args().collect();
 let cases:Vec<Case>=serde_json::from_slice(&std::fs::read(&args[1]).unwrap()).unwrap();
 let mut out=io::BufWriter::new(io::stdout().lock());
 for line in io::stdin().lock().lines() {
  let q:Value=serde_json::from_str(&line.unwrap()).unwrap();
  let i=q["index"].as_u64().unwrap() as usize; let c=&cases[i]; let o=options(c.flags);
  assert!(!c.reuse && c.flags&256==0);
  let render=|| ferro::to_html_with_options(black_box(&c.input), &o);
  drop(black_box(render()));
  let mut samples=Vec::with_capacity(5);
  for _ in 0..5 {
   let base=heap::begin();let html=black_box(render());let stats=heap::end(base);
   assert_eq!(stats.live_after_render, html.capacity() as u64);
   drop(html);assert_eq!(heap::remaining(base),0);samples.push(stats);
  }
  assert!(samples.windows(2).all(|p|p[0]==p[1]));
  let html=render();
  writeln!(out,"{}",json!({"case":c.case,"index":i,"input_bytes":c.input.len(),"html":html,"samples":samples})).unwrap();out.flush().unwrap();
 }
}
