import bookkeeping as b,json
from verify_l11_evidence import all_ranges
r=b.read(b.PUBLISHED/'audit05/original-scope-072.json')['files']
for i in range(2066,2078):
 x=r[i];print(json.dumps(dict(index=i,source_id=x['source_id'],sha256=x['sha256'],ranges=sorted(set(tuple(z) for z in all_ranges(x))),state=x.get('state'))))
for i,x in enumerate(r):
 if x['snapshot']=='pr-working' and any(q in x['path'] for q in ['/startup.rs','/mission/receipt.rs','/mission/enrollment.rs','/openings/']):
  print('linkedmetadata',i,x['source_id'],x['sha256'],sorted(set(tuple(z) for z in all_ranges(x))))
