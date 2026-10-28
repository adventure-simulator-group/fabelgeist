import bookkeeping as b,json
from verify_l11_evidence import all_ranges
r=b.read(b.PUBLISHED/'audit05/original-scope-072.json')['files']
for i in range(1977,1989):
 x=r[i];print(json.dumps(dict(index=i,source_id=x['source_id'],sha256=x['sha256'],ranges=list(all_ranges(x)),state=x.get('state'),scope_kind=x.get('scope_kind'))))
ledger=b.read(b.ROOT/'evidence/continuation-segovb/ledger.json')
print('codec_keys',[k for k,v in ledger['discovery_findings_and_dependencies'].items() if any(w in (k+' '+str(v)).lower() for w in ['internally tagged','internallytagged','terrainlandformrecipe','scenevista'])])

