import bookkeeping as b,json
r=b.read(b.PUBLISHED/'audit05/original-scope-072.json')['files']
for i in range(1977,1989):print(json.dumps(dict(index=i,**r[i])))
ledger=b.read(b.ROOT/'evidence/continuation-segovb/ledger.json')
for k,v in ledger['discovery_findings_and_dependencies'].items():
 if any(w in (k+' '+str(v)).lower() for w in ['internally tagged','internallytagged','postcard','codec']):
  print(k,json.dumps(v)[:1800])
