from pathlib import Path
import gzip
W=Path(__file__).parent

def variant(name, files):
    if name != 'baseline':
        files['src/inline/mod.rs'] = gzip.decompress((W/'snapshots'/(name+'.rs.gz')).read_bytes()).decode()
    return files
