from pathlib import Path
import json,random,itertools,hashlib
W=Path(__file__).parent
p=W/'driver/src/main.rs';s=p.read_text()
key='   "html"=>'
new='''   "events"=>{let mut p=ferro::InlineParser::new();let mut events=vec![];
    p.parse_with_options(c.input.as_bytes(),None,o.render_policy==ferro::RenderPolicy::Trusted,o.strikethrough,o.highlight,o.superscript,o.subscript,o.autolink_literals,o.math,o.inline_footnotes,None,&mut events);
    let normal=format!("{events:?}");events.clear();p.parse_mdx(c.input.as_bytes(),None,&mut events);
    json!({"html":ferro::to_html_with_options(&c.input,o),"events":normal,"mdx_events":format!("{events:?}")})},
'''
assert s.count(key)==1;s=s.replace(key,new+key);p.write_text(s)
cases=[]
def add(s):
 for flags in [15,47,31]:cases.append(dict(case=f'inline-guard/{len(cases)}',input=s,flags=flags,reuse=False))
atoms=['`a`','`` a ``','` a\\n b `','` `','``a`b``','*a*','**a**','***a***','~~a~~','[a](/b "title")','![a](/b)','[a][ref]','<a x="y">','<!-- x -->','<http://a.test>','\\*','\\\n','  \n','\n','~a~','^a^','==a==','$a$','^[a]','<X>{value}</X>','&amp;','é']
for a,b in itertools.product(atoms,repeat=2):add(a+b);add(a+'\n'+b)
rng=random.Random(0xF308)
fragments=atoms+['[',']','(',')','`','``','***','__','\\','<','>','!',' ','\t','\r\n','plain','[ref]: /url "title"\n\n']
for _ in range(4000):add(''.join(rng.choices(fragments,k=rng.randrange(1,20))))
(W/'extra.json').write_text(json.dumps(cases));print('Extra cases',len(cases))
