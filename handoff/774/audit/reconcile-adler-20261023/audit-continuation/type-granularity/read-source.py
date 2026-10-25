import json,pathlib,sys,hashlib
O=pathlib.Path(__file__).resolve().parent;R=O.parent.parent
cat=json.loads((R/'recovered-sources/catalog.json').read_text())['rows'];scope=json.loads((R/'audit/continuation/cumulative-source-scope-parallel-072.json').read_text())['files']
overlay_rows=[json.loads(l) for l in (R/'audit/continuation/owner-range-extensions.jsonl').read_text().splitlines() if l]
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
 out=[]
 for a,b in sorted(rs(scope[i])+[v for q in overlay_rows if q.get('source_id')==cat[i]['source_id'] and q.get('sha256')==cat[i].get('expected_sha256') for v in q.get('material_ranges',[])]):
  if out and a<=out[-1][1]+1:out[-1][1]=max(out[-1][1],b)
  else:out.append([a,b])
 return out or [[1,cat[i]['line_count']]]
i=int(sys.argv[1]);x=cat[i];raw=pathlib.Path(x['object']).read_bytes();sha=hashlib.sha256(raw).hexdigest();assert sha==(x.get('expected_sha256') or x.get('verified_object_sha256'))
print('SOURCE',i,x['source_id'],'SHA',sha,'EOF',x['line_count'],'SCOPED',ranges(i))
if len(sys.argv)>2:
 a,b=map(int,sys.argv[2:4]);body=raw.decode().splitlines()
 allowed={k for l,u in ranges(i) for k in range(l,min(u,len(body))+1)}
 for k in range(a,min(b,len(body))+1):
  if k in allowed:print(f'{k}: {body[k-1]}')
