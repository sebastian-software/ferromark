from pathlib import Path
import shutil,tomllib,json,importlib.util,gzip
ROOT=Path('/Users/sebastian/Workspace/.codex/15d0/ferromark');W=Path(__file__).parent
for name in ['baseline','source']:
 p=W/name;p.mkdir(exist_ok=True);shutil.copytree(ROOT/'src',p/'src',dirs_exist_ok=True)
 p.joinpath('Cargo.toml').write_text('''[package]
name="ferromark"
version="0.9.0"
edition="2024"
[workspace]
[lib]
path="src/lib.rs"
[features]
default=[]
mdx=[]
profiling=[]
[dependencies]
memchr="=2.8.3"
smallvec="=1.15.2"
html-escape={version="=0.2.14",default-features=false}
rustc-hash="=2.1.3"
unicode-ident="=1.0.24"
''')
(W/'driver/Cargo.toml').write_text('''[package]
name="ox-experiment-driver"
version="0.0.0"
edition="2024"
[workspace]
[dependencies]
ferro={package="ferromark",path="../source",features=["mdx"]}
serde={version="1",features=["derive"]}
serde_json="1"
[profile.release]
opt-level=3
lto="fat"
codegen-units=1
panic="abort"
''')
sample=(ROOT/'docs/reports/2026-09-12-ox-content-audit/harness/sample.md').read_text()
def c(name,s,flags=8,reuse=False):return dict(case=name,input=s,flags=flags,reuse=reuse)
cases=[c('ox-large-cm','\n\n'.join([sample]*100)),c('ox-large-tables','\n\n'.join([sample]*100),9),c('ox-huge-tables','\n\n'.join([sample]*2150),9),c('short','Hello **world**!\n'),c('short-reuse','Hello **world**!\n',8,True),c('plain',('A plain paragraph with some ordinary words and no formatting.\n\n')*100),c('multiline',('A plain paragraph with some ordinary words\nand more text on a second line\n\n')*100),c('inline',('A **bold** paragraph with *emphasis*, `code` and [a link](https://example.com).\n\n')*100),c('lists',('- First item\n- Second **bold** item\n  - Nested item\n\n')*100),c('code',('```javascript\nfunction hello() { return 1 < 2 && true; }\n```\n\n')*100),c('tables',('| Header 1 | Header 2 |\n|---|---|\n| Cell 1 | Cell 2 |\n| Cell 3 | Cell 4 |\n\n')*100,9),c('headings',('# Heading 1\n\n## Heading 2\n\n### Heading 3\n\n')*100),c('unique-headings','\n\n'.join(f'# Heading {n}' for n in range(300))),c('long-prose','a'*65536),c('late-marker','a'*65536+' *b*'),c('long-code','```text\n'+('a'*100+'\n')*700+'```\n'),c('escape-dense',('```\n<&> \'" & text <>&\n```\n\n')*100)]
spec=importlib.util.spec_from_file_location('old',ROOT/'docs/reports/2026-09-11-native-hotspot-optimization/experiment.py');mod=importlib.util.module_from_spec(spec);spec.loader.exec_module(mod)
for name,(flags,s) in mod.inputs().items():cases.append(c(name,s,flags))
(W/'cases.json').write_text(json.dumps(cases))
checks=cases.copy()
for x in json.loads((ROOT/'tests/spec.json').read_text()):
 for flags in [0,8,255]:checks.append(c(f'spec/{x["example"]}/{flags}',x['markdown'],flags))
# Rendering must retain generated suffix collisions, UTF-8 and all buffer lifecycles.
for text in ['# foo\n# foo\n# foo-1\n# foo\n# foo-2','## Ä Ö 日本語 **Title** &amp;\n\nnext','\r\n# A\r\n\r\none\r\ntwo\r\n','> a\n> b\n\n- x\n  y\n','a  \nb\n\nc\\\nd','a\n\n[x]: /url\n\n[x]\n', '\n'.join('# ' + 'x'*n for n in range(150)),('*a '*20000+' b*'*20000),'> '*300+'deep',('`'*6000+'x'+'`'*6000)]:
 for flags in [0,8,255]:checks.append(c('edge',text,flags))
(W/'verify.json').write_text(json.dumps(checks))
print(len(cases),'timing cases;',len(checks),'verification cases')
