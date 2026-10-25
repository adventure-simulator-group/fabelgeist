import json,sys
from pathlib import Path
O=Path(__file__).resolve().parent;R=O.parent.parent
c=json.loads((R/'recovered-sources/catalog.json').read_text())['rows'];s=json.loads((R/'audit/continuation/cumulative-source-scope-parallel-072.json').read_text())['files']
keys={'ranges','read_ranges','read_ranges_as_recorded','material_ranges','effective_material_ranges'}
def rs(x):
 out=[]
 if isinstance(x,dict):
  for k,v in x.items():
   if k in keys and isinstance(v,list):out.extend(t for t in v if isinstance(t,list) and len(t)==2 and all(isinstance(n,int) for n in t))
   out+=rs(v)
 elif isinstance(x,list):
  for v in x:out+=rs(v)
 return out
def ranges(i):
 merged=[]
 for a,b in sorted(rs(s[i])):
  if merged and a<=merged[-1][1]+1:merged[-1][1]=max(b,merged[-1][1])
  else:merged.append([a,b])
 return merged or [[1,c[i]['line_count']]]
for arg in sys.argv[1:]:
 if arg=='--meta':continue
 i=int(arg);print('SOURCE',i,c[i]['source_id'],'EOF',c[i]['line_count'],'RANGES',ranges(i))
 if '--meta' in sys.argv:continue
 lines=Path(c[i]['object']).read_text().splitlines()
 for a,b in ranges(i):
  for n in range(a,min(b,len(lines))+1):print(f'{n}: {lines[n-1]}')
