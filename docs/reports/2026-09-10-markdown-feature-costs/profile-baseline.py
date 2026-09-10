import pathlib,subprocess,time,json
root=pathlib.Path('target/feature-costs'); cases=json.loads((root/'catalog.json').read_text()); records=[]
for case_id,preset,name in [('lifecycle/presets/tiny','commonmark','tiny-commonmark'),('lifecycle/presets/tiny','default','tiny-default'),('lifecycle/presets/light-1k','commonmark','light-commonmark'),('core/emphasis/medium','commonmark','emphasis-commonmark')]:
    case=next(c for c in cases if c['id']==case_id); variant=case['variants'][-1] if case['group']=='core' else case['variants'][0]
    fixture=root/(name+'.md');fixture.write_text(variant['input']);profile=root/(name+'.sample.txt')
    with (root/(name+'.process.log')).open('w') as log:
        child=subprocess.Popen(['target/gfm-profile/profile-url',str(fixture),preset,'0','--forever'],stdout=log,stderr=log)
        try:
            time.sleep(.25);assert child.poll() is None
            result=subprocess.run(['/usr/bin/sample',str(child.pid),'4','-mayDie','-fullPaths','-file',str(profile)],capture_output=True,text=True,check=True)
            assert child.poll() is None
        finally:
            child.terminate()
            try: child.wait(timeout=3)
            except subprocess.TimeoutExpired: child.kill();child.wait()
    records.append({'case':case_id,'preset':preset,'profile':profile.name,'seconds':4})
    print(name,flush=True)
(root/'profiles.json').write_text(json.dumps(records,indent=2)+'\n')
