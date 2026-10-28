import bookkeeping as b
from verify_l11_evidence import all_ranges
r=b.read(b.PUBLISHED/'audit05/original-scope-072.json')['files']
for i in range(2100,2125):
 x=r[i]
 print(i,x['source_id'],x['sha256'],x.get('scope_kind'),sorted(set(tuple(v) for v in all_ranges(x))))
