# Initial setup for this recorded experiment. Requires the archived baseline
# source checked out at ROOT and the prepared native publication build.
# Use a separate checkout when replaying; never overwrite production changes.
from pathlib import Path
import json,shutil,tarfile,hashlib
root=Path('/Users/sebastian/Workspace/ferromark');e=root/'docs/reports/2026-09-11-native-hotspot-optimization';w=root/'target/native-hotspot';w.mkdir(exist_ok=True)
bun=Path('/private/tmp/ferromark-bun-publication-20260911');package=bun/'ferromark-native-probe';package.mkdir(exist_ok=True)
expected=json.loads((e/'baseline-source.json').read_text())
assert all(hashlib.sha256((root/name).read_bytes()).hexdigest()==digest for name,digest in expected.items()), 'Restore the archived baseline in an isolated checkout before setup'
base=w/'baseline';base.mkdir();shutil.copytree(root/'src',base/'src');shutil.copyfile(root/'Cargo.toml',base/'Cargo.toml')
with tarfile.open(e/'baseline.tar.gz','w:gz') as tf:
 for p in base.iterdir():tf.add(p,arcname=p.name)
(e/'baseline-source.json').write_text(json.dumps({str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in (root/'src').rglob('*.rs')},indent=2)+'\n')
source=w/'source';shutil.copytree(base/'src',source/'src')
(source/'Cargo.toml').write_text('''[package]
name="ferromark"
version="0.8.0"
edition="2024"
[workspace]
[dependencies]
memchr="2.7"
smallvec="1.13"
html-escape={version="0.2",default-features=false}
rustc-hash="2.0"
unicode-ident="1.0"
[features]
default=[]
mdx=[]
profiling=[]
''')
manifest=(bun/'ferromark-comparison/Cargo.toml').read_text().replace('ferromark-bun-comparison','ferromark-native-probe').replace(str(root)+'"',str(source)+'"')
(package/'Cargo.toml').write_text(manifest);shutil.copyfile(bun/'ferromark-comparison/build.rs',package/'build.rs')
driver=(root/'benchmarks/bun-comparison/driver.rs').read_text()
marker='    if mode == "selftest" {'
probe='''    if mode == "probe" {
        let flags: u32 = args[2].parse().unwrap();
        let parser: usize = args[3].parse().unwrap();
        let ms: u64 = args[4].parse().unwrap();
        let r = Renderers::new(flags);
        let mut input = String::new();
        std::io::stdin().read_to_string(&mut input).unwrap();
        for _ in 0..32 { black_box(r.render(parser, black_box(&input))); }
        let start = Instant::now();
        let mut iterations = 0u64;
        while start.elapsed() < Duration::from_millis(ms) {
            for _ in 0..16 { black_box(r.render(parser, black_box(&input))); }
            iterations += 16;
        }
        println!("{}", json!({"iterations":iterations,"elapsed_ns":start.elapsed().as_nanos()}));
        return;
    }
'''
assert marker in driver;driver=driver.replace(marker,probe+marker)
(package/'driver.rs').write_text(driver);(e/'probe.rs').write_text(driver)
workspace=(bun/'Cargo.toml').read_text();start=workspace.index('members = [');end=workspace.index(']',start)+1
(e/'original-bun-workspace.toml').write_text(workspace);shutil.copyfile(bun/'Cargo.lock',e/'Cargo.lock')
members=[name for name in json.loads(workspace[start+10:end]) if name!='ferromark-comparison'];members.append('ferromark-native-probe');workspace=workspace[:start]+'members = '+json.dumps(members)+workspace[end:];(bun/'Cargo.toml').write_text(workspace)
shutil.copy2(bun/'target/release/ferromark-bun-comparison',w/'publication-baseline')
print('Isolated native probe prepared')
