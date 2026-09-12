from pathlib import Path
import subprocess,json
from variants import variant
W=Path(__file__).parent;out=W/'layout';out.mkdir(exist_ok=True)
files={str(p.relative_to(W/'baseline')):p.read_text() for p in (W/'baseline/src').rglob('*.rs')}
rows=[]
for name in ['baseline','compact-links','compact-all','production']:
 s=variant(name,files.copy())['src/inline/mod.rs'];a=s.index('struct EmitPoint {');b=s.index('\n#[derive(Debug, Clone, Copy)]\nstruct HtmlSpan',a)
 declarations=s[a:b];links=files['src/inline/links.rs'];a=links.index('pub enum AutolinkLiteralKind {');b=links.index('\n}',a)+2
 source='#![allow(dead_code)]\nmod links { #[derive(Debug, Clone, Copy)]\n'+links[a:b]+'\n}\n'+declarations+'\nfn main() {println!("{} {}",std::mem::size_of::<EmitPoint>(),std::mem::size_of::<EmitKind>());}\n'
 (out/f'{name}.rs').write_text(source)
 subprocess.run(['rustc','--edition=2024',str(out/f'{name}.rs'),'-o',str(out/name)],cwd=W,check=True)
 point,kind=map(int,subprocess.check_output([str(out/name)],text=True).split());rows.append(dict(variant=name,emit_point_bytes=point,emit_kind_bytes=kind))
(out/'sizes.json').write_text(json.dumps(rows,indent=2));print(rows)
