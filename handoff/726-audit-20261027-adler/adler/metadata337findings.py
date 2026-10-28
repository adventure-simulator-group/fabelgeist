import bookkeeping as b
r=b.read(b.ROOT/'evidence/continuation-segovb/ledger.json')['discovery_findings_and_dependencies']
for k in r:
 if 'BASIS' in k or 'BPS' in k or 'ROAD-INFERENCE' in k:
  print(k)
