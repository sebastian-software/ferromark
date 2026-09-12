use serde::Deserialize;
use serde_json::{json,Value};
use std::{hint::black_box,io::{self,BufRead,Write},time::{Duration,Instant}};
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
fn main(){
 let args:Vec<_>=std::env::args().collect();let cases:Vec<Case>=serde_json::from_slice(&std::fs::read(&args[1]).unwrap()).unwrap();
 let opts:Vec<_>=cases.iter().map(|c|options(c.flags)).collect();let mut reuse:Vec<_>=opts.iter().cloned().map(ferro::Renderer::with_options).collect();
 let mut out=io::BufWriter::new(io::stdout().lock());
 for line in io::stdin().lock().lines(){let q:Value=serde_json::from_str(&line.unwrap()).unwrap();let i=q["index"].as_u64().unwrap() as usize;let c=&cases[i];let o=&opts[i];
  let row=match q["op"].as_str().unwrap(){
   "events"=>{let mut p=ferro::InlineParser::new();let mut events=vec![];
    p.parse_with_options(c.input.as_bytes(),None,o.render_policy==ferro::RenderPolicy::Trusted,o.strikethrough,o.highlight,o.superscript,o.subscript,o.autolink_literals,o.math,o.inline_footnotes,None,&mut events);
    let normal=format!("{events:?}");events.clear();p.parse_mdx(c.input.as_bytes(),None,&mut events);
    json!({"html":ferro::to_html_with_options(&c.input,o),"events":normal,"mdx_events":format!("{events:?}")})},
   "blocks"=>{let mut p=ferro::BlockParser::new_with_options(c.input.as_bytes(),o.clone());let mut events=vec![];p.parse(&mut events);
    let r=ferro::parse_with_options(&c.input,o);let mdx=ferro::mdx::render_with_options(&c.input,o);
    json!({"events":format!("{events:?}"),"html":r.html,"headings":format!("{:?}",r.headings),"limits":format!("{:?}",r.resource_limits),"segments":format!("{:?}",ferro::mdx::segment_spanned(&c.input)),"mdx_body":mdx.body,"mdx_esm":mdx.esm,"mdx_front_matter":mdx.front_matter})},
   "html"=>json!({"html":ferro::to_html_with_options(&c.input,o)}),
   "verify" if c.flags&256!=0=>{let r=ferro::mdx::render_with_options(&c.input,o);json!({"body":r.body,"esm":r.esm,"front_matter":r.front_matter})},
   "verify"=>{let r=ferro::parse_with_options(&c.input,o);let fresh=ferro::to_html_with_options(&c.input,o);let mut transitions=vec![];
    for s in [&c.input[..],"# Other\n\n- [x] New", "", &c.input[..]] {let actual=reuse[i].render(s);let expected=ferro::to_html_with_options(s,o);assert_eq!(actual,expected);transitions.push(actual);}
    json!({"html":r.html,"fresh":fresh,"limits":format!("{:?}",r.resource_limits),"headings":format!("{:?}",r.headings),"transitions":transitions})},
   "window"=>{let ms=q["ms"].as_u64().unwrap();let start=Instant::now();let mut n=0;loop{for _ in 0..16{if c.flags&256!=0{drop(black_box(ferro::mdx::render_with_options(black_box(&c.input),o)));}else if c.reuse{drop(black_box(reuse[i].render(black_box(&c.input))));}else{drop(black_box(ferro::to_html_with_options(black_box(&c.input),o)));}}n+=16;if start.elapsed()>=Duration::from_millis(ms){break;}}
    json!({"case":c.case,"count":n,"elapsed_ns":start.elapsed().as_nanos() as u64})},
   _=>panic!("unknown op"),
  };writeln!(out,"{row}").unwrap();out.flush().unwrap();
 }
}
