from pathlib import Path
import json,os,shutil,subprocess
HERE=Path(__file__).resolve().parent
env=os.environ.copy()
for key in list(env):
    if key.startswith('CARGO_PROFILE_') or key in ('CARGO_ENCODED_RUSTFLAGS','RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER','CARGO_TARGET_DIR','LD_PRELOAD','DYLD_INSERT_LIBRARIES'):env.pop(key)
env.update(RUSTFLAGS='-C target-cpu=generic',LC_ALL='C')
commands=[]
for name in ['ferro','ox']:
    dest=HERE/('cache-'+name);dest.mkdir(exist_ok=True)
    for file in ['Cargo.toml','Cargo.lock']:shutil.copyfile(HERE/name/file,dest/file)
    file='src/main.rs' if name=='ferro' else 'main.rs'
    source=(HERE/name/file).read_text().split('// Shared complete-collection protocol',1)[0]
    path=dest/file;path.parent.mkdir(exist_ok=True)
    path.write_text(source+'\n'+(HERE/'cache_worker.rs').read_text())
    subprocess.run(['rustfmt','--edition','2024',str(path)],check=True)
    argv=['cargo','build','--offline','--locked','--release','--manifest-path',str(dest/'Cargo.toml'),'--target-dir',str(HERE/'target-timing')]
    commands.append(argv)
    with (HERE/'cache-build.log').open('a') as log:subprocess.run(argv,cwd=HERE,env=env,stdout=log,stderr=subprocess.STDOUT,check=True)
    binary='ferromark-workflows' if name=='ferro' else 'ox-content-driver'
    out=HERE/(name+'-cache');shutil.copyfile(HERE/'target-timing/release'/binary,out);out.chmod(0o755)
    print('Built',out.name,flush=True)
(HERE/'cache-build-commands.json').write_text(json.dumps(commands,indent=2)+'\n')
