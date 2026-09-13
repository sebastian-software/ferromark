"""Build diagnostic workers without editing either production implementation."""
from pathlib import Path
import hashlib, json, os, shutil, subprocess

HERE = Path(__file__).resolve().parent
ROOT = Path(os.environ.get('FERROMARK_ROOT', '/Users/sebastian/Workspace/.codex/15d0/ferromark')).resolve()
FIELD = ROOT / 'benchmarks/workflows/field'
OX = ROOT / 'benchmarks/native-pipeline-comparison/engines/ox-content'
FERRO = '''mod adapter {
    use std::cell::RefCell;
    pub struct Renderer {
        options: ferromark::Options,
        session: Option<RefCell<ferromark::Renderer>>,
    }
    impl Renderer {
        pub fn new(flags: u32) -> Self {
            assert_eq!(flags, 7);
            let mode = std::env::args().nth(1).unwrap();
            let options = ferromark::options!(ferromark::Options::commonmark();
                render_policy: ferromark::RenderPolicy::Trusted,
                tables: true, strikethrough: true, task_lists: true,
                heading_ids: mode.ends_with("-ids"));
            let session = mode.contains("reuse").then(|| RefCell::new(ferromark::Renderer::with_options(options.clone())));
            Self { options, session }
        }
        pub fn render(&self, input: &str) -> String {
            match &self.session {
                Some(renderer) => renderer.borrow_mut().render(input),
                None => ferromark::to_html_with_options(input, &self.options),
            }
        }
        pub fn options(&self) -> String { format!("{:?}; reusable session: {}", self.options, self.session.is_some()) }
    }
}
'''

def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()

env = os.environ.copy()
for key in list(env):
    if key.startswith('CARGO_PROFILE_') or key in ('RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER','CARGO_TARGET_DIR','LD_PRELOAD','DYLD_INSERT_LIBRARIES'):
        env.pop(key)
env['LC_ALL'] = 'C'
metadata = {'source_revision': subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
            'rustc': subprocess.check_output(['rustc','-Vv'],text=True),
            'source_inputs': {}, 'commands': [], 'binaries': {}}
shared = (FIELD/'worker.rs').read_text()
counts = '''#[cfg(feature = "counters")]
            "counts" => {
                ferromark::profiling::reset();
                let output_bytes = run();
                serde_json::json!({"output_bytes": output_bytes, "counters": format!("{:?}", ferromark::profiling::snapshot())})
            }
            '''
for name, origin in [('ferro', FIELD.parent), ('ox', OX)]:
    dest = HERE/name; dest.mkdir(exist_ok=True)
    manifest = (origin/'Cargo.toml').read_text()
    manifest = manifest.replace('path = "../.."', 'path = '+json.dumps(str(ROOT)))
    if name=='ferro': manifest=manifest.replace('heap = []', 'heap = []\ncounters = ["ferromark/profiling"]')
    (dest/'Cargo.toml').write_text(manifest)
    shutil.copyfile(origin/'Cargo.lock',dest/'Cargo.lock')
    if name=='ferro':
        (dest/'src').mkdir(exist_ok=True)
        path=dest/'src/main.rs'; code=FERRO
        worker=shared.replace('"time" => {',counts+'"time" => {')
    else:
        path=dest/'main.rs'; worker=shared
        code=(OX/'main.rs').read_text().split('include!("../worker.rs");')[0]
        code=code.replace('pub const NAME: &str = "ox-content";', '')
        code=code.replace('html: RefCell<HtmlRenderer>,', 'html: RefCell<HtmlRenderer>, fresh: bool,')
        code=code.replace('                html,', '                html, fresh: std::env::args().nth(1).unwrap() == "ox-fresh",')
        code=code.replace('self.html.borrow_mut().render(&document)', 'if self.fresh { HtmlRenderer::with_options(self.html_options.clone()).render(&document) } else { self.html.borrow_mut().render(&document) }')
    path.write_text(code+'\n'+worker)
    subprocess.run(['rustfmt','--edition','2024',str(path)],check=True)
    for profile in (['timing','sample','counters'] if name=='ferro' else ['timing','sample']):
        flags='-C target-cpu=generic'
        if profile=='sample': flags+=' -C debuginfo=2 -C force-frame-pointers=yes'
        argv=['cargo','build','--offline','--release','--locked','--manifest-path',str(dest/'Cargo.toml'),'--target-dir',str(HERE/('target-'+profile))]
        if profile=='counters': argv+=['--features','counters']
        metadata['commands'].append({'argv':argv,'cwd':str(HERE),'RUSTFLAGS':flags})
        with (HERE/'build.log').open('a') as log:
            subprocess.run(argv,cwd=HERE,env={**env,'RUSTFLAGS':flags},stdout=log,stderr=subprocess.STDOUT,check=True)
        binary='ferromark-workflows' if name=='ferro' else 'ox-content-driver'
        out=HERE/(name+'-'+profile)
        shutil.copyfile(HERE/('target-'+profile)/'release'/binary,out)
        out.chmod(0o755)
        metadata['binaries'][out.name]=sha(out)
        print('Built',out.name,flush=True)
for base in [ROOT/'src',ROOT/'crates']:
    for path in sorted(base.rglob('*.rs')): metadata['source_inputs'][str(path.relative_to(ROOT))]=sha(path)
for name in ['Cargo.toml','Cargo.lock','benchmarks/workflows/corpus.json']:
    metadata['source_inputs'][name]=sha(ROOT/name)
corpus=json.loads((ROOT/'benchmarks/workflows/corpus.json').read_text())
for i,doc in enumerate(corpus['documentation']): corpus['doc-'+str(i)]=[doc]
(HERE/'corpus.json').write_text(json.dumps(corpus,ensure_ascii=False,indent=2)+'\n')
(HERE/'build.json').write_text(json.dumps(metadata,indent=2)+'\n')
