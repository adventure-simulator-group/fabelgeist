#!/usr/bin/env python3
import json,pathlib,sys
p=pathlib.Path(__file__).resolve().parent
name,lo,hi=sys.argv[1],int(sys.argv[2]),int(sys.argv[3])
caps=[]
for f in sorted(p.glob("SOURCE-CAPTURES*.json")):
 d=json.loads(f.read_text());caps.extend(d if isinstance(d,list) else [d])
rows=[x for x in caps if x['capture']==name];assert len(rows)==1;c=rows[0]
regs=[json.loads(f.read_text()) for f in p.glob("SOURCE*REGISTRATION.json")]
assert any(any(x['path']==c['path'] and x['range'][0]<=lo<=hi<=x['range'][1] for x in r.get('sources',[])) for r in regs)
lines=(p/name).read_text().splitlines();assert 1<=lo<=hi<=len(lines)
out=''.join(f'{i:4d} {lines[i-1]}\n' for i in range(lo,hi+1))
with (p/f'DISPLAY-{name[7:10]}-{lo}-{hi}.txt').open('x') as f:f.write(out)
print(out,end='')
