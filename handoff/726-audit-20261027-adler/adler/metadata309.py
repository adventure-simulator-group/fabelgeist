import bookkeeping as b,json
l=b.read(b.ROOT/'evidence/continuation-segovb/ledger.json')
print([k for k in l['discovery_findings_and_dependencies'] if any(w in k for w in ['SCENE','PROVISION','MATERIAL','IMPORT','ERROR'])])
from verify_l11_evidence import all_ranges
r=b.read(b.PUBLISHED/'audit05/original-scope-072.json')['files']
for i in range(2003,2013):
 x=r[i];print(json.dumps(dict(index=i,source_id=x['source_id'],sha256=x['sha256'],ranges=sorted(set(tuple(z) for z in all_ranges(x))),state=x.get('state'))))
