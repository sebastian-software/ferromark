from pathlib import Path
import json
W=Path(__file__).parent
cases=json.loads((W/'cases.json').read_text())[:694]; selected=[i for i in json.loads((W/'selected.json').read_text()) if i<694]
for name,s in [
 ('root-short-lines','<div>\n'+'  <p>x & text</p>\n'*4096+'\n'),
 ('root-long-lines','<div>\n'+('x'*4096+'\n')*16+'\n'),
 ('separate-blocks',('<div>\ntext\n</div>\n\n')*2048),
 ('comment-long-lines','<!--\n'+('x'*4096+'\n')*16+'-->\n'),
 ('script-long-lines','<script>\n'+('x'*4096+'\n')*16+'</script>\n'),
 ('script-candidates','<script>\n'+(' <other> '*256+'\n')*32+'</script>\n'),
 ('comment-short-lines','<!--\n'+'text\n'*8192+'-->\n'),
 ('processing-long-lines','<?pi\n'+('x'*4096+'\n')*16+'?>\n'),
 ('cdata-long-lines','<![CDATA[\n'+('x'*4096+'\n')*16+']]>\n'),
 ('declaration-long-lines','<!DOCTYPE\n'+('x'*4096+'\n')*16+'>\n'),
 ('container-html','> <div>\n'+'>  <p>x & text</p>\n'*4096+'>\n'),
 ('custom-tags',('<custom data-a="x">\ntext\n</custom>\n\n')*2048),
]:
 selected.append(len(cases));cases.append(dict(case='html/'+name,input=s,flags=0,reuse=False))
(W/'cases.json').write_text(json.dumps(cases));(W/'selected.json').write_text(json.dumps(selected))
