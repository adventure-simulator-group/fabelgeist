import bookkeeping as b
r=b.read(b.ROOT/'evidence/continuation-segovb/ledger.json')['discovery_findings_and_dependencies']
for k in r:
 if 'TERRAIN' in k or 'GEOGRAPH' in k or 'POLYGON' in k or 'ERROR' in k:
  print(k, str(r[k].get('classification',''))[:180])
print(r['ADLER-L11-PICKUP-ANCHOR-FRAME'])
