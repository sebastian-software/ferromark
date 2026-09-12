from pathlib import Path
W=Path(__file__).parent
def variant(name,files):
 if name == 'baseline':return files
 root=W/('production-source' if name == 'production' else 'variants/'+name)
 assert root.exists(), name
 for p in root.rglob('*.rs'):files[str(p.relative_to(root))]=p.read_text()
 return files
