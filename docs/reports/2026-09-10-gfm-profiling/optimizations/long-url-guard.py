import json, pathlib, re, statistics, subprocess, sys, hashlib
before, after, output = map(pathlib.Path, sys.argv[1:])
root = pathlib.Path('target/gfm-profile')
rows = []
def run(binary, fixture, count):
    result = subprocess.run([str(binary.resolve()), str(fixture), 'gfm', str(count)], capture_output=True, text=True, check=True)
    match = re.search(r'elapsed=([0-9.]+)(ns|µs|ms|s)', result.stderr)
    assert match, result.stderr
    elapsed = float(match[1]) * {'ns':1, 'µs':1000, 'ms':1e6, 's':1e9}[match[2]]
    return elapsed/count, int(re.search(r'out_len=(\d+)', result.stderr)[1])
for length in [32, 256, 4096, 16384]:
    text = ('https://example.org/' + 'a' * length + '\n\n') * 8
    fixture = root / f'url-path-{length}.md'; fixture.write_text(text)
    calibration, _ = run(before, fixture, 100)
    count = max(1, int(250e6 / calibration))
    samples=[]
    for index in range(7):
        pair={}; sizes=[]
        order=[('before',before),('after',after)]
        if index%2: order.reverse()
        for label,binary in order:
            pair[label], size = run(binary, fixture, count); sizes.append(size)
        assert sizes[0]==sizes[1], 'output lengths differ'
        samples.append(pair)
    delta=statistics.median((p['after']/p['before']-1)*100 for p in samples)
    rows.append({'path_length':length,'iterations':count,'input_bytes':len(text),'input_sha256':hashlib.sha256(text.encode()).hexdigest(),'paired_change_percent':delta,'samples':samples})
    print(length, round(delta,2), flush=True)
output.write_text(json.dumps({'recipe':"('https://example.org/' + 'a' * path_length + '\\n\\n') * 8",'before':'original profiling baseline; strikethrough/short-input changes do not target these inputs','policy':'GFM / Untrusted, fresh parser, reused output; seven alternating fixed-iteration windows calibrated to 250 ms; output lengths checked','binary_sha256':{str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [before,after]},'results':rows},indent=2)+'\n')
