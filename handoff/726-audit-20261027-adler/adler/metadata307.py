import bookkeeping as b,json
from verify_l11_evidence import all_ranges
r=b.read(b.PUBLISHED/'audit05/original-scope-072.json')['files']
for i in range(1989,2003):
 x=r[i];print(json.dumps(dict(index=i,source_id=x['source_id'],sha256=x['sha256'],ranges=sorted(set(tuple(z) for z in all_ranges(x))),state=x.get('state'),scope_kind=x.get('scope_kind'))))
