from experiment import run
for index, name in enumerate(['unstable-packed', 'stable-packed', 'stable-tuple']):
    run(name, rounds=5, ms=20, warm=20, tag=name+'/screen', order=index)
