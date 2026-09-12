from pathlib import Path
W=Path(__file__).parent
def variant(name,files):
 if name=='baseline':return files
 files['src/escape.rs']=(W/'native-sources'/f'{name}.rs').read_text()
 return files
