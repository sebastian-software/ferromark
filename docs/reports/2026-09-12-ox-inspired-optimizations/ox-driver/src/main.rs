use ox_content_allocator::Allocator;
use ox_content_parser::{Parser,ParserOptions};
use ox_content_renderer::{HtmlRenderer,HtmlRendererOptions};
use serde::Deserialize;
use serde_json::{json,Value};
use std::{hint::black_box,io::{self,BufRead,Write},time::{Duration,Instant}};
mod normalize;
#[derive(Deserialize)]
struct Case {case:String,input:String,flags:u32,reuse:bool}
fn main(){
 let args:Vec<_>=std::env::args().collect();let cases:Vec<Case>=serde_json::from_slice(&std::fs::read(&args[1]).unwrap()).unwrap();
 let opts:Vec<_>=cases.iter().map(|c|(ParserOptions{tables:c.flags&1!=0,strikethrough:c.flags&2!=0,task_lists:c.flags&4!=0,..Default::default()},HtmlRendererOptions{autolink_urls:false,autolink_target_blank:false,link_target_blank:false,..Default::default()})).collect();
 let mut reuse:Vec<_>=opts.iter().map(|(_,h)|HtmlRenderer::with_options(h.clone())).collect();
 let mut out=io::BufWriter::new(io::stdout().lock());
 for line in io::stdin().lock().lines(){let q:Value=serde_json::from_str(&line.unwrap()).unwrap();let i=q["index"].as_u64().unwrap() as usize;let c=&cases[i];let (p,h)=&opts[i];
 let mut render=||{let s=black_box(&c.input);let arena=Allocator::for_source_len(s.len());let doc=Parser::with_options(&arena,s,p.clone()).parse().unwrap();if c.reuse{reuse[i].render(&doc)}else{HtmlRenderer::with_options(h.clone()).render(&doc)}};
 let row=match q["op"].as_str().unwrap(){
 "verify"=>{let html=render();json!({"html":html,"normalized_equal":normalize::normalize_html(&html)==normalize::normalize_html(q["html"].as_str().unwrap())})},
 "window"=>{let ms=q["ms"].as_u64().unwrap();let start=Instant::now();let mut n=0;loop{for _ in 0..16{drop(black_box(render()));}n+=16;if start.elapsed()>=Duration::from_millis(ms){break;}}json!({"case":c.case,"count":n,"elapsed_ns":start.elapsed().as_nanos() as u64})},_=>panic!("unknown op")};
 writeln!(out,"{row}").unwrap();out.flush().unwrap();
 }
}
