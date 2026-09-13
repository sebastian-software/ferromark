from pathlib import Path
import subprocess,os,shutil,sys
W=Path(__file__).parent
profile=sys.argv[1] if len(sys.argv)>1 else 'release'
env=dict(os.environ);env.pop('RUSTFLAGS',None);env.pop('CARGO_ENCODED_RUSTFLAGS',None)
if profile=='sample':env['RUSTFLAGS']='-C force-frame-pointers=yes'
(W/'bin').mkdir(exist_ok=True)
for engine in ['ferro','ox']:
 d=W/engine
 # Archived lockfiles include the final harness package names.
 with (W/f'build-{engine}-{profile}.log').open('w') as out:
  subprocess.run(['cargo','build','--offline','--locked','--profile',profile,'--manifest-path',str(d/'Cargo.toml')],cwd=W,env=env,stdout=out,stderr=subprocess.STDOUT,check=True)
 shutil.copy2(d/'target'/profile/f'corpus-{engine}',W/'bin'/f'{engine}-{profile}')

 if profile=='sample':
  dsym=d/'target/sample'/f'corpus-{engine}.dSYM'
  if dsym.exists():shutil.copytree(dsym,W/'bin'/f'{engine}-sample.dSYM',dirs_exist_ok=True)
 print(engine,profile,'built',flush=True)
