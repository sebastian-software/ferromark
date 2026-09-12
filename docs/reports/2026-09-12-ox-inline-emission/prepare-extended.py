from pathlib import Path
import random,json
W=Path(__file__).parent;r=random.Random(0x493308)
atoms=['`a`','`` a ``','`a\n b`','` `','``a`b``','*a*','**a**','***a***','~~a~~','[a](/b "title")','![a](/b)','[a][ref]','[ref]: /url "title"\n\n','[^n]: a\n\n','[^n]','<a x="y">','<!-- x -->','<http://a.test>','\\*','\\\n','  \n','\n','~a~','^a^','==a==','$a$','^[a]','<X>{value}</X>','&amp;','é','[',']','(',')','`','``','***','__','\\','<','>','!',' ','\t','\r\n','plain']
cases=[]
for i in range(20000):
 s=''.join(r.choices(atoms,k=r.randrange(1,40)))
 for flags in [15,47,31]:cases.append(dict(case=f'extended/{len(cases)}',input=s,flags=flags,reuse=False))
(W/'extended.json').write_text(json.dumps(cases));print('Prepared',len(cases))
s=(W/'verify-extra.py').read_text().replace('extra.json','extended.json').replace('extra-hashes.json','extended-hashes.json').replace('extra-verification.json','extended-verification.json')
(W/'verify-extended.py').write_text(s)
